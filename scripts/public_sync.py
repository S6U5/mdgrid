#!/usr/bin/env python3
"""手元のリポ(履歴を公開しない)と公開リポのあいだで、変更を書き出す・取り込む。

手順と考え方は docs/public-sync.md。設定と印は手元の .git/ の中だけに置く:
  git の設定 publicsync.*(remote・伏せる手元の名前)と ref refs/public-sync/*。

  status                 公開リポを取ってきて、両方向の取りこぼしを出す
  import [<sha>...]      公開リポの main の未反映のコミットを cherry-pick -x で今のブランチへ
  export <名前> -m <文>  手元の main(--from で、main を含むブランチ)を伏せ字にして、
                         公開リポの export/<名前> へ push する(先に import を済ませる)
  init --local <rev> --github <rev>   2つの印を置く(最初の1回)
  install-hook           公開リポへ手元の履歴を push しないための pre-push のフックを置く
  pre-push               フックから呼ばれる

標準ライブラリと git・gh だけを使う。
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

REF_GITHUB = "refs/public-sync/github"
REF_LOCAL = "refs/public-sync/local"
REF_EXPORTS = "refs/public-sync/exports/"
EXPORT_PREFIX = "export/"

UUID = r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
# 承認の証拠(decidespec の approval-evidence)の会話の記録の場所
EVIDENCE_RE = re.compile(r"\b(?:claude|codex):" + UUID + r"(?:/" + UUID + r")?")
EVIDENCE_GREP = r"(claude|codex):[0-9a-f]{8}-[0-9a-f]{4}-"
PATH_END = r"[^\s`'\"()\[\]<>{},、。（）「」]*"

MASK_EVIDENCE = "〔伏せ字: 会話の記録の場所〕"
MASK_PATH = "〔伏せ字: 手元のパス〕"
MASK_WORD = "〔伏せ字: 手元の名前〕"


class Fail(Exception):
    pass


def git(*args, input=None, env=None, check=True):
    p = subprocess.run(
        ["git", *args],
        input=input,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and p.returncode != 0:
        raise Fail("git %s: %s" % (" ".join(args), p.stderr.decode(errors="replace").strip()))
    return p


def out(*args, **kw):
    return git(*args, **kw).stdout.decode().strip()


def rev(name):
    p = git("rev-parse", "--verify", "--quiet", name + "^{commit}", check=False)
    return p.stdout.decode().strip() if p.returncode == 0 else None


def config_all(key):
    p = git("config", "--get-all", key, check=False)
    return [v for v in p.stdout.decode().splitlines() if v]


class Settings:
    def __init__(self):
        self.remote = (config_all("publicsync.remote") or ["github"])[-1]
        self.branch = (config_all("publicsync.localBranch") or ["main"])[-1]
        self.paths = config_all("publicsync.redactPath")
        self.words = config_all("publicsync.redactWord")
        self.forbid = config_all("publicsync.forbid")
        url = git("remote", "get-url", self.remote, check=False).stdout.decode().strip()
        if not url:
            raise Fail(
                "remote %s がありません。git remote add %s https://github.com/<owner>/<repo>.git"
                % (self.remote, self.remote)
            )
        self.url = url
        m = re.search(r"github\.com[:/]([^/]+/[^/]+?)(?:\.git)?/?$", url)
        self.repo = m.group(1) if m else None
        self.main = "refs/remotes/%s/main" % self.remote

    def literals(self):
        return self.paths + self.words + self.forbid


# --- 伏せ字 ---------------------------------------------------------------


def redact_text(s, st):
    s = EVIDENCE_RE.sub(MASK_EVIDENCE, s)
    for prefix in st.paths:
        s = re.sub(re.escape(prefix) + PATH_END, MASK_PATH, s)
    for word in st.words:
        s = s.replace(word, MASK_WORD)
    return s


_tree_cache = {}


def redacted_tree(commit, st):
    """commit の木の、伏せ字にした版の木の id。"""
    if commit in _tree_cache:
        return _tree_cache[commit]
    pats = ["-e", "claude:", "-e", "codex:"]
    for lit in st.paths + st.words:
        pats += ["-e", lit]
    p = git("grep", "-l", "-z", "-I", "-F", *pats, commit, "--", check=False)
    paths = [x.split(":", 1)[1] for x in p.stdout.decode().split("\0") if x]
    updates = []
    for path in paths:
        blob = git("cat-file", "blob", "%s:%s" % (commit, path)).stdout
        try:
            text = blob.decode("utf-8")
        except UnicodeDecodeError:
            continue
        new = redact_text(text, st)
        if new == text:
            continue
        sha = out("hash-object", "-w", "--stdin", input=new.encode("utf-8"))
        mode = out("ls-tree", commit, "--", path).split()[0]
        updates.append("%s %s\t%s" % (mode, sha, path))
    if not updates:
        tree = out("rev-parse", commit + "^{tree}")
    else:
        with tempfile.TemporaryDirectory() as d:
            env = dict(os.environ, GIT_INDEX_FILE=os.path.join(d, "index"))
            git("read-tree", commit, env=env)
            git("update-index", "--index-info", input=("\n".join(updates) + "\n").encode(), env=env)
            tree = out("write-tree", env=env)
    _tree_cache[commit] = tree
    return tree


def leaks(treeish, st):
    """木の中の、外に出さない文字列の行。"""
    hits = []
    p = git("grep", "-n", "-I", "-E", "-e", EVIDENCE_GREP, treeish, "--", check=False)
    hits += p.stdout.decode().splitlines()
    lits = st.literals()
    if lits:
        args = []
        for lit in lits:
            args += ["-e", lit]
        p = git("grep", "-n", "-I", "-F", *args, treeish, "--", check=False)
        hits += p.stdout.decode().splitlines()
    return hits


# --- 公開リポの側の印 -------------------------------------------------------


def fetch(st):
    print("公開リポを取ってくる: %s" % st.remote, file=sys.stderr)
    git("fetch", "--quiet", "--prune", st.remote)
    if not rev(st.main):
        raise Fail("%s がありません" % st.main)


_pr_cache = {}


def pr_of(sha, st):
    """公開リポのコミットを入れた PR(number・title・head)。分からなければ None。"""
    if sha in _pr_cache:
        return _pr_cache[sha]
    info = None
    if st.repo:
        p = subprocess.run(
            ["gh", "api", "repos/%s/commits/%s/pulls" % (st.repo, sha)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if p.returncode == 0:
            for pr in json.loads(p.stdout.decode() or "[]"):
                if pr.get("merged_at") or pr.get("merge_commit_sha") == sha:
                    info = {
                        "number": pr["number"],
                        "title": pr["title"],
                        "head": pr["head"]["ref"],
                    }
                    break
    _pr_cache[sha] = info
    return info


def imported(sha, st):
    p = git(
        "log", st.branch, "--format=%H", "-1", "-F",
        "--grep", "cherry picked from commit %s" % sha,
        check=False,
    )
    return bool(p.stdout.strip())


def classify(sha, st):
    """('export', 書き出しのブランチ) / ('imported', None) / ('pending', None)。"""
    pr = pr_of(sha, st)
    if pr and pr["head"].startswith(EXPORT_PREFIX):
        return "export", pr["head"]
    if imported(sha, st):
        return "imported", None
    return "pending", None


def pending_github(st):
    cur = rev(REF_GITHUB)
    if not cur:
        raise Fail("印 %s がありません。最初に init を使う(docs/public-sync.md)" % REF_GITHUB)
    shas = out("rev-list", "--reverse", "--first-parent", "%s..%s" % (cur, st.main)).split()
    return shas


def advance(st):
    """反映済みと分かる所まで、2つの印を進める。残りの公開リポのコミットを返す。"""
    shas = pending_github(st)
    for i, sha in enumerate(shas):
        kind, head = classify(sha, st)
        if kind == "pending":
            return shas[i:]
        if kind == "export":
            ref = REF_EXPORTS + head[len(EXPORT_PREFIX):]
            local = rev(ref)
            if local:
                git("update-ref", REF_LOCAL, local)
                git("update-ref", "-d", ref)
            else:
                print(
                    "注意: %s は書き出しのブランチ %s から入ったが、手元に書き出しの印がない。手元の基準は動かさない"
                    % (sha[:9], head),
                    file=sys.stderr,
                )
        git("update-ref", REF_GITHUB, sha)
    return []


def describe(sha, st):
    subject = out("log", "-1", "--format=%s", sha)
    pr = pr_of(sha, st)
    where = "#%d %s" % (pr["number"], pr["head"]) if pr else "PR なし"
    return "%s %s(%s)" % (sha[:9], subject, where)


# --- コマンド ------------------------------------------------------------


def cmd_status(a, st):
    fetch(st)
    rest = advance(st)
    print("== 公開リポの main で、まだ手元に取り込んでいないもの: %d" % len(rest))
    for sha in rest:
        print("  " + describe(sha, st))
    local = rev(REF_LOCAL)
    print("== 手元の %s で、まだ書き出していないコミット(%s..%s)" % (st.branch, local[:9], st.branch))
    commits = out("log", "--format=%h %s", "%s..%s" % (local, st.branch)).splitlines()
    for line in commits[:30]:
        print("  " + line[:150])
    if len(commits) > 30:
        print("  ほか %d" % (len(commits) - 30))
    if not commits:
        print("  なし")
    exports = out("for-each-ref", "--format=%(refname)", REF_EXPORTS).splitlines()
    if exports:
        print("== マージ待ちの書き出し")
        for ref in exports:
            print("  " + EXPORT_PREFIX + ref[len(REF_EXPORTS):])
    stat = out("diff", "--shortstat", st.main, redacted_tree(rev(st.branch), st))
    print("== 今書き出すと公開リポの main から変わるもの")
    print("  " + stat + "(ファイルの一覧は export <名前> --dry-run)" if stat else "  なし(揃っている)")
    return 0


def cmd_export(a, st):
    name = a.name[len(EXPORT_PREFIX):] if a.name.startswith(EXPORT_PREFIX) else a.name
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*(/[A-Za-z0-9._-]+)*", name):
        raise Fail("書き出しの名前は英数字・. _ - / だけにする: %s" % name)
    branch = EXPORT_PREFIX + name
    source = rev(a.source)
    if not source:
        raise Fail("%s がありません" % a.source)
    fetch(st)
    rest = advance(st)
    if rest:
        raise Fail(
            "公開リポにまだ取り込んでいない変更がある。書き出すと消えるので、先に import して %s へ入れる:\n  "
            % st.branch
            + "\n  ".join(describe(sha, st) for sha in rest)
        )
    if git("merge-base", "--is-ancestor", st.branch, source, check=False).returncode != 0:
        raise Fail("%s に %s の取り込みが入っていない。先に %s をマージする" % (a.source, st.branch, st.branch))
    remote_branch = "refs/remotes/%s/%s" % (st.remote, branch)
    ours = rev(remote_branch) or rev(st.main)
    # 書き出す木 = 伏せ字にした手元の木。取り込み済みなので、公開リポの側だけの変更は無い
    tree = redacted_tree(source, st)
    if tree == out("rev-parse", ours + "^{tree}"):
        print("書き出すものがない(%s と同じ)" % (branch if rev(remote_branch) else "公開リポの main"))
        return 0
    bad = leaks(tree, st)
    if bad:
        raise Fail(
            "外に出さない文字列が残っている。伏せ字の設定(publicsync.*)か元のファイルを直す:\n  "
            + "\n  ".join(bad[:30])
        )
    print(out("diff", "--stat", ours, tree))
    if a.dry_run:
        print("試しだけ。木 %s(見る: git diff %s %s)" % (tree, ours[:9], tree[:9]))
        return 0
    if not a.message:
        raise Fail("-m でコミットの文を渡す")
    msg = "\n\n".join(a.message) + "\n"
    commit = out("commit-tree", tree, "-p", ours, input=msg.encode())
    print("push: %s → %s %s" % (commit[:9], st.remote, branch))
    p = subprocess.run(["git", "push", st.remote, "%s:refs/heads/%s" % (commit, branch)])
    if p.returncode != 0:
        raise Fail("push できなかった")
    git("update-ref", REF_EXPORTS + name, source)
    if a.pr:
        if not st.repo:
            raise Fail("remote の URL から GitHub のリポが分からない")
        p = subprocess.run(
            ["gh", "pr", "list", "-R", st.repo, "--head", branch, "--state", "open", "--json", "url"],
            stdout=subprocess.PIPE,
        )
        found = json.loads(p.stdout.decode() or "[]")
        if found:
            print("PR はある: %s" % found[0]["url"])
        else:
            title = a.title or a.message[0]
            args = ["gh", "pr", "create", "-R", st.repo, "--base", "main", "--head", branch, "--title", title]
            args += ["--body-file", a.body_file] if a.body_file else ["--body", "\n\n".join(a.message[1:])]
            if subprocess.run(args).returncode != 0:
                raise Fail("PR を作れなかった")
    print("マージは人が公開リポで行う。マージのあとの status で、手元の基準が進む。")
    return 0


def cmd_import(a, st):
    fetch(st)
    rest = advance(st)
    shas = [rev(s) for s in a.commits] if a.commits else rest
    if not shas:
        print("取り込むものがない")
        return 0
    for sha in shas:
        if not sha:
            raise Fail("コミットが見つからない")
        if imported(sha, st):
            print("取り込み済み: " + describe(sha, st))
            continue
        print("取り込む: " + describe(sha, st))
        parents = out("rev-list", "--parents", "-n1", sha).split()[1:]
        args = ["cherry-pick", "-x"] + (["-m", "1"] if len(parents) > 1 else []) + [sha]
        p = subprocess.run(["git", *args])
        if p.returncode != 0:
            print(
                "\nぶつかった。直して git cherry-pick --continue のあと、もう一度 import する。"
                "specs の生成の区画と test-locks.json は手元の側を取り、check.py --write で作り直す"
                "(docs/public-sync.md)。",
                file=sys.stderr,
            )
            return 1
    print("取り込んだ。今のブランチが %s でなければ、%s へマージしたあとで status が印を進める。" % (st.branch, st.branch))
    advance(st)
    return 0


def cmd_init(a, st):
    local, github = rev(a.local), rev(a.github)
    if not local or not github:
        raise Fail("--local と --github にコミットを渡す")
    git("update-ref", REF_LOCAL, local)
    git("update-ref", REF_GITHUB, github)
    print("印を置いた: local=%s github=%s" % (local[:9], github[:9]))
    return 0


HOOK = """#!/bin/sh
# public-sync: 公開リポへ手元の履歴を push しない(docs/public-sync.md)
top=$(git rev-parse --show-toplevel)
exec python3 "$top/scripts/public_sync.py" pre-push "$@"
"""


def cmd_install_hook(a, st):
    hooks = out("rev-parse", "--git-path", "hooks")
    path = os.path.join(hooks, "pre-push")
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            if "public-sync" not in f.read():
                raise Fail("%s が別の中身で既にある。手で合わせる" % path)
    with open(path, "w", encoding="utf-8") as f:
        f.write(HOOK)
    os.chmod(path, 0o755)
    print("置いた: " + path)
    return 0


def cmd_pre_push(a, st):
    if a.remote != st.remote and a.url != st.url:
        return 0
    roots = out("rev-list", "--max-parents=0", st.branch).split()
    for line in sys.stdin.read().splitlines():
        parts = line.split()
        if len(parts) != 4 or set(parts[1]) == {"0"}:
            continue
        sha = parts[1]
        for root in roots:
            if git("merge-base", "--is-ancestor", root, sha, check=False).returncode == 0:
                print(
                    "public-sync: %s は手元の履歴を含むので、公開リポへ push しない。"
                    "python3 scripts/public_sync.py export を使う(docs/public-sync.md)" % parts[0],
                    file=sys.stderr,
                )
                return 1
        bad = leaks(sha, st)
        if bad:
            print("public-sync: 外に出さない文字列がある:\n  " + "\n  ".join(bad[:30]), file=sys.stderr)
            return 1
    return 0


def main(argv):
    ap = argparse.ArgumentParser(description="手元のリポと公開リポのあいだの書き出しと取り込み")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status")
    p = sub.add_parser("import")
    p.add_argument("commits", nargs="*")
    p = sub.add_parser("export")
    p.add_argument("name")
    p.add_argument("-m", "--message", action="append", default=[])
    p.add_argument("--from", dest="source", default=None)
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--pr", action="store_true")
    p.add_argument("--title")
    p.add_argument("--body-file")
    p = sub.add_parser("init")
    p.add_argument("--local", required=True)
    p.add_argument("--github", required=True)
    sub.add_parser("install-hook")
    p = sub.add_parser("pre-push")
    p.add_argument("remote")
    p.add_argument("url")
    a = ap.parse_args(argv)
    try:
        st = Settings()
        if a.cmd == "export" and a.source is None:
            a.source = st.branch
        return {
            "status": cmd_status,
            "import": cmd_import,
            "export": cmd_export,
            "init": cmd_init,
            "install-hook": cmd_install_hook,
            "pre-push": cmd_pre_push,
        }[a.cmd](a, st)
    except Fail as e:
        print("public-sync: %s" % e, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
