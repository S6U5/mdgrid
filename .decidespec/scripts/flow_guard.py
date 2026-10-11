#!/usr/bin/env python3
"""sdd-flow の門番(Claude Code の PreToolUse フック)。

decidespec の形で仕様を持つリポ(specs/README.md に decidespec の生成区画がある)で、コードのファイル
(specs/ の外の、Markdown でないファイル)を書こうとしたとき、実装が進行中の変更の記録
(specs/_changes/*.md で status: active、フェーズの表の実装が「進行中」、それより前がすべて済か対象外)が
無ければ、書き込みを止める(終了コード 2、理由は標準エラー)。それ以外は何もしない(終了コード 0)。
記録は check.py と同じ規則(E23・E24)で形と順序を確かめ、崩れた記録では門を開けない。
Bash は、コマンドの中の書き込み先(> >> の先、tee・sed -i・cp・mv・touch・rm の対象)を拾って同じ判定をする。
git apply・patch や字句を読めないコマンド(閉じない引用符など)のように書き込み先が読めないものは、記録が無ければ止める。拾えない書き方(スクリプトの中の書き込みなど)は通してしまう。
--session-start: decidespec の形のリポで起動したとき、流れの案内を標準出力に出す(SessionStart フック)。
decidespec の形でないリポ、リポの外のファイルには手を出さない。依存なし。
"""
from __future__ import annotations

import json
import re
import shlex
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check  # noqa: E402  同じフォルダの check.py(記録の形と順序の規則を共有する)

MARK = "<!-- decidespec:begin -->"
PHASES = ["要求", "不明点", "仕様化", "設計", "タスク", "実装", "照合"]


def find_root(path):
    for d in [path] + list(path.parents):
        readme = d / "specs" / "README.md"
        if readme.is_file():
            try:
                if MARK in readme.read_text(encoding="utf-8"):
                    return d
            except OSError:
                return None
            return None
        if (d / ".git").exists():
            return None
    return None


def front(text):
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    fm = {}
    if m:
        for line in m.group(1).splitlines():
            if ":" in line:
                k, v = line.split(":", 1)
                fm[k.strip()] = v.strip()
    return fm, text[m.end():] if m else text


def implementing_one(p, fm, body, done_ok=False):
    """記録 p が、実装が進行中(done_ok なら済も)で、形の崩れていないものなら p を返す。"""
    if fm.get("status") not in (("active", "done") if done_ok else ("active",)):
        return None
    sec = re.search(r"(?m)^##\s+フェーズ[^\n]*\n(.*?)(?=^##\s|\Z)", body, re.S)
    if not sec:
        return None
    states = {}
    for line in sec.group(1).splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) >= 2 and cells[0] in PHASES:
            states[cells[0]] = "対象外" if cells[1].startswith("対象外") else cells[1]
    if states.get("実装") in (("進行中", "済") if done_ok else ("進行中",)) and all(
            states.get(n) in ("済", "対象外") for n in PHASES[:PHASES.index("実装")]):
        errs, _ = check.check_change(p.name, fm, body, {}, check.datetime.date.today(), 10 ** 6)
        if not [e for e in errs if e.startswith(("E23", "E24"))]:
            return p
    return None


def implementing(root, done_ok=False):
    """実装が進行中の記録(done_ok なら、実装が済の active / done の記録も)を返す。形の崩れた記録は数えない。"""
    for p in sorted((root / "specs" / "_changes").glob("*.md")):
        try:
            fm, body = front(p.read_text(encoding="utf-8"))
        except OSError:
            continue
        if implementing_one(p, fm, body, done_ok):
            return p
    return None


WRITE_CMDS = {"tee", "touch", "rm", "truncate", "unlink", "rmdir"}
MOVE_CMDS = {"cp", "mv", "install", "ln"}
REDIRECTS = {">", ">>", "&>", "&>>", ">|"}
SEPARATORS = {";", "&&", "||", "|", "&", "\n", "(", ")", "((", "))"}


def _mask_arith(line):
    """(( … )) と $(( … )) の中を空白で塗る(中の << はシフトで、ヒアドキュメントではない)。
    引用符の中の (( は算術の始まりとしない(_premark と同じ)。"""
    out, i, depth, quote = list(line), 0, None, None
    while i < len(line):
        two = line[i:i + 2]
        if depth is None:
            c = line[i]
            if quote == "'":
                quote = None if c == "'" else quote
            elif c == "\\":
                i += 2
                continue
            elif c in "'\"" and quote in (None, c):
                quote = None if quote else c
            elif quote is None and two == "((":
                depth, i = 0, i + 2
                continue
        else:
            if two == "))" and depth == 0:
                depth, i = None, i + 2
                continue
            if line[i] == "(":
                depth += 1
            elif line[i] == ")":
                depth -= 1
            out[i] = " "
        i += 1
    return "".join(out)


def strip_heredocs(command):
    """ヒアドキュメントの本文(<<WORD の次の行から WORD の行まで)を取り除く。本文は書き込み先ではない。"""
    lines, out, i = command.split("\n"), [], 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        words = re.findall(r"(?<!<)<<-?(?!<)\s*['\"]?([A-Za-z_][\w]*)['\"]?", _mask_arith(line))
        i += 1
        for w in words:
            while i < len(lines) and lines[i].strip() != w:
                i += 1
            i += 1
    return "\n".join(out)


# 字句の前に置き換える語(引用符の外で)。\> \< は逃がした比較の語、(( と )) は算術の括りの語
MARKS = {"\\>": "_DSX_GT_", "\\<": "_DSX_LT_", "((": "_DSX_OPEN2_", "))": "_DSX_CLOSE2_"}


def _premark(text):
    r"""左から読み、\ はすぐ後の1文字と組にして消費する(\\ は1組)。奇数個の \ の直後の > < だけを語にする。
    引用符の外の (( と )) は、空白で区切った1つの語にする。単引用符の中はそのまま。"""
    out, i, quote = [], 0, None
    while i < len(text):
        c = text[i]
        if quote == "'":
            out.append(c)
            quote = None if c == "'" else quote
            i += 1
            continue
        if c == "\\" and i + 1 < len(text):
            pair = text[i:i + 2]
            out.append(MARKS.get(pair, pair) if pair[1] in "<>" else pair)
            i += 2
            continue
        if c in "'\"" and quote in (None, c):
            quote = None if quote else c
        elif quote is None and text[i:i + 2] in ("((", "))"):
            out.append(" %s " % MARKS[text[i:i + 2]])
            i += 2
            continue
        out.append(c)
        i += 1
    return "".join(out)


def tokens(command):
    text = _premark(strip_heredocs(command))
    # ( と ) は1文字ずつの語にする(`(( 1 ))>f` の > を拾うため、句読点の組にしない)
    lex = shlex.shlex(text, posix=True, punctuation_chars=";&|<>")
    lex.whitespace = " \t"
    lex.commenters = ""
    lex.wordchars += "[]"  # [[ と ]] を1つの語にする
    try:
        toks = list(lex)
    except ValueError:
        return None  # 字句を読めない(閉じない引用符など)。書き込み先を読めないものとして扱う
    back = {"_DSX_OPEN2_": "((", "_DSX_CLOSE2_": "))"}
    out = []
    for t in toks:
        if t in back:
            out.append(back[t])
            continue
        out.append(t.replace("_DSX_GT_", "\\>").replace("_DSX_LT_", "\\<"))
    return out


# 語の頭に来ても後の [[ や (( をコマンドの位置のままにする予約語
KEYWORD_HEADS = {"!", "if", "then", "elif", "else", "while", "until", "do", "{", "time"}
COMPARE_END = {"[[": "]]", "((": "))"}


def _substitution(toks, j):
    """toks[j] がコマンド置換・プロセス置換(`$(`・バッククォート・`<(`・`>(`)を含むか。"""
    u = toks[j]
    if "`" in u or "$(" in u:
        return True
    return u == "(" and j > 0 and toks[j - 1] in ("$", "<", ">")


def _compare_end(toks, i, cmd):
    """toks[i] が [[ … ]] か (( … ))(算術展開 $(( … )) を含む)の始まりなら、閉じる語の位置を返す。
    中の > < は比較で、リダイレクトではない。コマンドの位置に無い・閉じていない・中にコマンド置換か
    プロセス置換があるときは None(今までどおり読む。置換の中の書き込みを見逃さないため)。"""
    t = toks[i]
    if t not in COMPARE_END:
        return None
    expansion = t == "((" and i > 0 and toks[i - 1].endswith("$")
    if not expansion and any(w not in KEYWORD_HEADS for w in cmd):
        return None
    depth = 0
    for j in range(i + 1, len(toks)):
        u = toks[j]
        if _substitution(toks, j):
            return None
        if t == "((":
            if u == "(":
                depth += 1
            elif u == "((":
                depth += 2
            elif u == ")":
                if not depth:
                    return None
                depth -= 1
            elif u == "))":
                if depth == 0:
                    return j
                if depth == 1:
                    return None
                depth -= 2
        elif u == "]]":
            return j
        elif u in (";", "\n"):
            return None
    return None


def bash_targets(command, cwd):
    """Bash のコマンドから書き込み先を拾う(引用符の中とヒアドキュメントの本文は見ない)。
    書き込み先が読めない書き込み(git apply・patch・字句を読めないコマンド)は None を入れる。cd は後のパスの基準を変える。"""
    out, base, cmd = [], cwd, []
    toks = tokens(command)
    if toks is None:
        return [None]
    toks = toks + [";"]
    i = 0
    while i < len(toks):
        t = toks[i]
        end = _compare_end(toks, i, cmd)
        if end is not None:
            if i == 0 or not toks[i - 1].endswith("$"):
                cmd.append(t)  # [[ と (( は単純なコマンドの名前として扱う(書き込みはしない)
            i = end + 1
            continue
        if t in REDIRECTS and i + 1 < len(toks):
            target = toks[i + 1]
            if target not in ("/dev/null",) and target not in SEPARATORS:
                out.append(_join(base, target))
            i += 2
            continue
        if t in (">&", "<", "<<", "<<<", "<&"):
            i += 2
            continue
        if t in SEPARATORS:
            base = _one(cmd, base, out)
            cmd = []
        elif re.match(r"^\d+$", t) and i + 1 < len(toks) and toks[i + 1] in REDIRECTS | {">&"}:
            pass  # 1> 2>> などの fd の番号
        else:
            cmd.append(t)
        i += 1
    return out


def _join(base, a):
    """base の下のパス。~ で始まればホームから(絶対パスならそのまま)。"""
    return base / Path(a).expanduser()


def _one(words, base, out):
    """単純なコマンド1つ。書き込み先を out に足し、cd なら新しい基準を返す。"""
    while words and re.match(r"^[A-Za-z_]\w*=", words[0]):
        words = words[1:]
    if not words:
        return base
    name, rest = Path(words[0]).name, words[1:]
    args = [w for w in rest if not w.startswith("-")]
    if name == "git" and args[:1] == ["mv"]:
        return out.extend(_join(base, a) for a in args[1:]) or base
    if name == "dd":
        out += [_join(base, w[3:]) for w in rest if w.startswith("of=")]
        return base
    if name == "cd":
        return _join(base, args[0]) if args else Path.home()
    if name in WRITE_CMDS:
        out += [_join(base, a) for a in args]
    elif name in MOVE_CMDS:
        # mv は移す元も消える。cp・install・ln は書き込み先(最後)だけ
        out += [_join(base, a) for a in (args if name == "mv" else args[-1:])]
    elif name == "sed" and any(w == "-i" or w.startswith(("-i", "--in-place")) for w in rest):
        files, j, script_seen = [], 0, any(w in ("-e", "--expression", "-f") for w in rest)
        while j < len(rest):
            w = rest[j]
            if w == "-i" and j + 1 < len(rest) and rest[j + 1] == "":
                j += 2  # BSD の -i '' の空の拡張子
                continue
            if w in ("-e", "--expression", "-f"):
                j += 2
                continue
            if not w.startswith("-"):
                if script_seen:
                    files.append(w)
                else:
                    script_seen = True  # 最初の引数は sed のスクリプト
            j += 1
        out += [_join(base, f) for f in files]
    elif (name == "patch" and "--dry-run" not in rest) or \
            (name == "git" and args[:1] == ["apply"] and "--check" not in rest):
        out.append(None)
    return base


def is_code(root, path, after=False):
    """記録の要るコードか。specs/ と Markdown のほか、spec-init がコピーした道具(.decidespec/)と入口の置き場
    (.claude/skills/decidespec-*/・.agents/skills/decidespec-*/)は数えない(道具の更新はコードの変更ではない。TS-6)。
    spec-init が書いたフックの設定(lock.json の managed)は、後段の検査(after)で中身が書いたときのままなら数えない。
    書く前(PreToolUse)は書いた後の中身が分からないので、数える(手で直すなら記録が要る)。"""
    rel = path.relative_to(root)
    if rel.parts[0] in ("specs", ".decidespec") or path.suffix.lower() in (".md", ".markdown"):
        return False
    if len(rel.parts) > 3 and "/".join(rel.parts[:2]) in (".claude/skills", ".agents/skills") \
            and rel.parts[2].startswith("decidespec-"):
        return False
    managed = _managed(root).get(rel.as_posix()) if after else None
    if managed and path.is_file():
        import hashlib
        try:
            return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest() != managed
        except OSError:
            return True
    return True


def _managed(root):
    try:
        data = json.loads((root / ".decidespec" / "lock.json").read_text(encoding="utf-8"))
        managed = data.get("managed", {})
        return managed if isinstance(managed, dict) else {}
    except (OSError, ValueError, AttributeError):
        return {}


BLOCK = ("decidespec: %s はコードのファイル。このリポは decidespec の仕様駆動なので、書く前に sdd-flow の流れで"
         "変更の記録(specs/_changes/)を作り、実装のフェーズを「進行中」にする(それより前のフェーズは済か"
         "「対象外: 理由」。記録は check.py の E23・E24 を通る形にする)。規模は tiny / bugfix / full から選ぶ(差分を一文で言えれば tiny。迷ったら軽い方)。\n")


def git_run(root, *args):
    import subprocess
    r = subprocess.run(["git", "-c", "core.quotepath=false", *args], cwd=root, capture_output=True, text=True)
    return r.returncode, r.stdout


def diff_paths(root, args):
    """git の差分のパスを、decidespec の根からの絶対パスで返す(改名は元と先の両方)。git が失敗したら None。"""
    rc, top = git_run(root, "rev-parse", "--show-toplevel")
    if rc != 0:
        return None
    rc, out = git_run(root, *args)
    if rc != 0:
        return None
    top = Path(top.strip())
    parts = [x for x in out.split("\0") if x]
    paths, i = [], 0
    while i < len(parts):
        head = parts[i]
        if args[0] == "status":
            code, name = head[:2], head[3:]
            paths.append(top / name)
            if "R" in code or "C" in code:
                i += 1
                paths.append(top / parts[i])
        else:
            code = head
            if code[:1] in ("R", "C"):
                paths += [top / parts[i + 1], top / parts[i + 2]]
                i += 2
            else:
                paths.append(top / parts[i + 1])
                i += 1
        i += 1
    return paths


def scopes(root, names):
    """差分に対応してよい記録の「範囲」(実装が進行中か済の active の記録と、差分に入っている記録)。"""
    out = []
    for p in sorted((root / "specs" / "_changes").glob("*.md")):
        try:
            fm, body = front(p.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError):
            continue
        in_diff = p.resolve() in names
        if not (fm.get("status") == "active" or in_diff):
            continue
        if implementing_one(p, fm, body, done_ok=True) is None:
            continue
        sec = re.search(r"(?m)^##\s+タスク[^\n]*\n(.*?)(?=^##\s|\Z)", body, re.S)
        rows = [[c.strip() for c in line.strip().strip("|").split("|")] for line in (sec.group(1) if sec else "").splitlines()
                if line.lstrip().startswith("|")]
        if not rows:
            continue
        head = rows[0]
        if "範囲" not in head:
            continue
        k = head.index("範囲")
        for r in rows[2:]:
            if len(r) > k:
                out += scope_paths(r[k])
    return out


def scope_paths(cell):
    """範囲の欄からパスを読む。読点(、,,)と空白で区切る。迷ったら範囲を広げない向きに読む。
    注は語の頭で始まる括弧(半角・全角)だけで、欄の最後の閉じ括弧まで続く(閉じていなければ欄の残りを捨てる)。
    バッククォートも語の頭で始まるときだけ囲みで、中を読点・空白で区切る(閉じていなければ欄の残りを捨てる)。
    語にくっついた括弧とバッククォートは、中の空白ごと(括弧は欄の最後の閉じ括弧まで)字どおり語に入れる。"""
    out, tok, i, n = [], "", 0, len(cell)

    def flush():
        nonlocal tok
        if tok.strip():
            out.append(tok.strip())
        tok = ""

    while i < n:
        c = cell[i]
        if c in "、,，" or c.isspace():
            flush()
            i += 1
        elif tok and c in "(（`":
            # 語にくっついた括弧・バッククォートは、中の空白ごと語に入れる。括弧は欄の最後の閉じ括弧まで
            # (注の中の「1)」のような対にならない閉じ括弧で切れて、残りがパスとして漏れないように)。閉じていなければ残り全部
            if c == "`":
                j = cell.find("`", i + 1)
            else:
                j = max(cell.rfind(")"), cell.rfind("）"))
            j = n if j < i else j + 1
            tok += cell[i:j]
            i = j
        elif tok:
            tok += c
            i += 1
        elif c in "(（":
            j = max(cell.rfind(")"), cell.rfind("）"))
            if j < i:
                break
            i = j + 1
        elif c == "`":
            j = cell.find("`", i + 1)
            if j < 0:
                break
            for x in re.split(r"[、,，\s]+", cell[i + 1:j]):
                tok = x
                flush()
            i = j + 1
        else:
            tok = c
            i += 1
    flush()
    return out


def covered(root, path, scope):
    import fnmatch
    rel = path.relative_to(root).as_posix()
    for s in scope:
        s = s.rstrip("/")
        if rel == s or rel.startswith(s + "/") or fnmatch.fnmatch(rel, s):
            return True
    return False


AFTER = ("decidespec: 変更の記録(specs/_changes/)のタスクの「範囲」に入っていないコードの変更がある: %s。"
         "sdd-flow の流れで記録を作るか、タスクの範囲にそのファイルを書き、実装のフェーズを進行中か済にする。\n")


def after_check(argv, data):
    """後段の検査(PH-12)。--pre-commit はステージした差分、--diff <base> は base...HEAD、--stop は作業ツリーの変更。
    差分のコードのファイルが、実装が進行中か済の記録のタスクの「範囲」に入っているかを見る。git が失敗したら止める。"""
    root = find_root(Path(data.get("cwd") or ".").resolve())
    if root is None:
        return 0
    if "--stop" in argv:
        if data.get("stop_hook_active"):
            return 0
        paths = diff_paths(root, ["status", "--porcelain", "-z", "-uall"])
    elif "--pre-commit" in argv:
        paths = diff_paths(root, ["diff", "--cached", "--name-status", "-z", "-M"])
    else:
        i = argv.index("--diff") + 1
        base = argv[i] if i < len(argv) else "origin/main"
        paths = diff_paths(root, ["diff", "--name-status", "-z", "-M", "%s...HEAD" % base])
    if paths is None:
        msg = "decidespec: git の差分を取れなかった(base が無い・git の外など)。検査できないので止める。\n"
        if "--stop" in argv:
            return 0  # 終わるときは止めない(git の外で作業していることがある)
        sys.stderr.write(msg)
        return 1
    mine = [p.resolve() for p in paths]
    inside = [p for p in mine if root in p.parents]
    code = [p for p in inside if is_code(root, p, after=True)]
    scope = scopes(root, set(mine))
    bad = [p.relative_to(root).as_posix() for p in code if not covered(root, p, scope)]
    if not bad:
        return 0
    if "--stop" in argv:
        print(json.dumps({"decision": "block", "reason": AFTER % ", ".join(bad[:5])}, ensure_ascii=False))
        return 0
    sys.stderr.write(AFTER % ", ".join(bad[:5]))
    return 1


def session_start(cwd):
    root = find_root(Path(cwd).resolve())
    if root is not None:
        print("このリポは decidespec の仕様駆動(specs/)。コードを足す・直す・消す依頼は、頼まれなくても sdd-flow スキルの流れ"
              "(変更の記録 specs/_changes/ に7つのフェーズを並べる)で進める。記録の実装が進行中でないと、コードの書き込みはフックが止める。")
    return 0


def main(argv):
    try:
        # コミット・CI から呼ばれたときは標準入力に JSON が来ない(引き継いだ入力を待ち続けないように読まない)
        hook = not (set(argv) & {"--pre-commit", "--diff"})
        data = json.load(sys.stdin) if hook and not sys.stdin.isatty() else {}
    except ValueError:
        data = {}
    if not isinstance(data, dict):
        data = {}
    cwd = Path(data.get("cwd") or ".")
    if "--session-start" in argv:
        return session_start(cwd)
    if set(argv) & {"--pre-commit", "--diff", "--stop"}:
        return after_check(argv, data)
    inp = data.get("tool_input") or {}
    if not isinstance(inp, dict):
        return 0
    if data.get("tool_name") == "Bash":
        targets = bash_targets(inp.get("command") or "", cwd)
    else:
        one = inp.get("file_path") or inp.get("notebook_path")
        targets = [one] if one else []
    for target in targets:
        if target is None:  # 書き込み先が読めない(git apply・patch・字句を読めないコマンド)
            root = find_root(cwd.resolve())
            if root is not None and not implementing(root):
                sys.stderr.write(BLOCK % "(書き込み先を読めないコマンド: git apply・patch・閉じない引用符など)")
                return 2
            continue
        if not target:
            continue
        path = Path(str(target)).expanduser()
        if not path.is_absolute():
            path = cwd / path
        path = path.resolve()
        root = find_root(path.parent)
        if root is None or not is_code(root, path):
            continue
        if not implementing(root):
            sys.stderr.write(BLOCK % path.relative_to(root))
            return 2
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except SystemExit:
        raise
    except Exception as e:  # 門番が落ちたら通さない(黙って素通りにしない)
        sys.stderr.write("decidespec: 門番のスクリプトが落ちた(%s: %s)。書き込みを止めた。flow_guard.py を直す\n"
                         % (type(e).__name__, e))
        sys.exit(2)
