#!/usr/bin/env python3
"""decidespec の決定的検査(形式と、記録から機械で決まる規則だけ。意味の矛盾は検査担当に任せる)。

使い方: python3 <このファイル> [<repo-root>] [--specs <dir>] [--stale-days N] [--write]
  --write  specs/README.md の生成区画を書き直す(区画が無ければ末尾に足す)
  --lock   守られる要件のテストで、まだ錠の無いものに錠をかける。前の取り方の指紋のままの錠は、新しい指紋へ掛け直す(TL-13)
  --verbose  W12・W8・W3 を種類ごとの1行にまとめず、1件ずつ出す
  --lock --relock <決定記録>  変えてよい決定の記録(人の決定か、錨との照合のある AI 自己採択)で変えた・消した要件のテストの錠を掛け直す・外す(TL-5)。
           削除していない要件の錠のあるテストを1つも残さない掛け直しは拒む(TL-15)
  --trail  先頭に人の確認待ち(OM-7。3件ならブレーキ)を並べ、そのあと、どの specs/_bundles/ の記録にも載っていない決着した決定記録を、人の発言の記録を先に並べ、7件以上なら「要約を出す時」を出す(OM-14)
  --approve <決定記録> --find <語句> [--find <語句>]  語句を含む人の発言を会話の記録から探し、approval-evidence と approval-hash を書き込む(語句は記録に残さない)
  --fingerprint <specs/_bundles/…md>  承認の文・束の指紋(囲みの中の本文 + 改行1つの SHA-256 の先頭16文字)を出す(OM-13)
  --settle <提案> accepted|rejected|withdrawn  提案の status と updated を書いて _decisions/ へ移し、相対リンクを直す(CF-6)
  --tests <要件 ID>[,<要件 ID>…]  その要件を指す今のテストを1行ずつ出す(無ければ「なし」。TL-17)
  --overlap <要件 ID>[,<要件 ID>…]  要件の「実装:」のコミットを1つずつ打ち消した版でテストを走らせ、落ちた版で重なりの候補を出す(TL-18)
  --evidence <変更の記録> [--test-cmd "<{file} を含む雛形>"]  要件を指すテストを今と実装前で走らせ、照合の表の行を出す(SF-9)
  --rehash <決定記録>  approval-evidence の指す人の発言から approval-hash を書き込む(伏せ字にした旧い記録の付け直し)

人の承認の照合: decided-by: 人 の記録は、フロントマターの approval-evidence(claude:<会話の ID>/<発言の ID>)で、
承認の言葉を打ち込んだ人の発言を指す。会話の記録(既定 ~/.claude/projects、環境変数 DECIDESPEC_TRANSCRIPTS で変更)に
その発言があれば、出どころが人(origin.kind = human)で、貼り付け(<pasted_content>)の外に承認の言葉があるかを照合する。
サブエージェントの報告・通知・AI の要約の中の言葉は、人の発言として照合に通らない。
終了コード: 0 = エラーなし(警告は可)、1 = エラーあり。依存なし。

構成: specs/README.md、specs/<capability>/spec.md、specs/_proposals/(proposed だけ)、
specs/_decisions/(accepted / rejected / withdrawn と取り込みの記録)、任意の specs/principles.md。
記録の置き場は先頭に _ を付けて、機能のフォルダと見分ける。各 spec.md の末尾には、その機能の決定の一覧と、
要件ごとのつながりの表(最新の決定・テスト・最後に checked にした変更の記録)を --write が書く。

「守られる」要件 = 決定の列のリンク先に decided-by: 人 の記録(旧形式なら decision-makers に【人】)が1本でもあるか、
取り込みの記録で扱いが 人 / 未確認 のもの。AI が自己採択で変えるときは「錨との照合」の節(principles.md のあるリポでは
P 番号つき)が要る(OM-6)。原則(P 番号)と、principles.md のフロントマター gated に載った要件は、AI が自己採択で触れない(OM-13)。

エラー:
  E0  specs/ か索引が無い、引数が不正、旧い形(specs/changes/)のまま、旧い名前(specs/proposals/・specs/decisions/)のまま
  E1  フロントマターが無い(免除: README.md / AGENTS.md / CLAUDE.md。ただし specs/README.md は対象)
  E2  必須キー type / id が無い
  E3  id が ULID でない
  E4  id がリポ内で重複
  E5  updated < created
  E6  status の値が置き場所と合わない
  E7  要件 ID がリポ内で重複
  E8  雛形の穴埋め {…} が残っている
  E9  要件行が4列(ID / 要件 / 確かめ方 / 決定)でない
  E10 決定の列に _decisions/ への実在するリンクが無い
  E11 Markdown リンクの参照が実在しない(他リポは owner/repo:/path で書く)
  E12 提案のフロントマターの値が語彙外か欠けている(decided-by / trigger / trigger-link / touches)
  E13 touches と差分の行の ID の集合が一致しない
  E14 decided-by: 人 なのに、承認の記録に「YYYY-MM-DD 人の承認(要約: …)」の行(旧い記録は伏せ字の行)が無い
  E15 AI 自己採択の記録が、原則か gated の要件(一度でも載ったもの)、またはその時点で守られていた要件に「錨との照合」の節なしで
      触っている。または、人の確認待ち(錨との照合のある、原則か gated に触る AI 自己採択で、照合できた確認の記録に載っていないもの)が
      すでに3件あるのに、原則か gated に触っている(ブレーキ。OM-7)
  E16 採択された「追加」または強める「変更」の強さが しなければならない / してはならない なのに、
      trigger が 人の発言 / 運用で起きた失敗 でない
  E17 最新の記録が取り込みの要件の文言が、取り込んだ文言と違う(変えるなら提案で)
  E18 テストが仕様に無い要件 ID を指している([XX-n] か test_XX_n の形で書く)
  E19 守られる要件に結びついたテスト(Python はテスト関数ごと)が、錠の指紋から変わっている
  E20 錠(specs/test-locks.json)を書き換えた・外したコミットに、変えてよい決定の記録が伴っていない(git の履歴で確かめる)
  E21 Python のテストが構文エラーで読めない(錠の指紋も、テスト関数ごとの要件とのつながりも見られない)
  E22 approval-evidence の指す発言が人の発言でない、または approval-hash(発言の指紋)が合わない(会話の記録が手元にあるときだけ)
  E27 決定記録・提案に人の発言の原文の引用(人「…」、【人】「…」、> 【人】の行)がある(原文は引用せず要約する)
  E29 承認の記録の「承認の文 <パス>、指紋 <16文字>」が、保存した承認の文の今の指紋と合わない、または承認した人の発言が、独立した語としての指紋も、対象の要件 ID と承認を示す語も含まない(選択肢の答えでは、承認を示す語は選んだ答えの側に要る。OM-13)
  E28 決定記録・提案・変更の記録の文が、会話の記録の人の発言と一致する(会話の記録が手元にあるときだけ。日本語の文を8字ずつに切り、7割以上が人の発言にある文)
  E30 .decidespec/lock.json に載ったファイル(spec-init がコピーした道具と入口)の中身が、lock.json の指紋と違うか無い(手で直さない。直すならプラグイン側で直して spec-init で更新する。TS-5)
  E31 principles.md に gated を置いた後に、原則の行か gated、または一度でも gated に載った要件の spec.md の行(要件の文か
      確かめ方。OM-21)を、変えた ID に触る決定の記録(人の決定か、錨との照合のある
      AI 自己採択)なしに変えたコミットがある(git の履歴で確かめる。OM-19)
  E32 採択済みの decided-by: 人 の記録に approval-evidence か approval-hash が無い(--approve <記録> --find <語句> で
      会話の記録から付ける。語句は記録に残らない。伏せ字の承認は、approval-evidence を持つ記録が初めて _decisions/ に
      入ったコミットより前に git に入ったものだけ W20。OM-12)
  E34 _decisions/ に入った記録の status・decided-by・decision-makers・touches を、入ったあとで変えた、または記録を消した
      (git の履歴と作業ツリーで確かめる。本文の伏せ字とリンクの直しは見ない。OM-22)
  E33 確認の記録(specs/_bundles/ の、フロントマターに confirms を持つ記録)が、照合できる人の言葉を指していない
      (会話の記録が手元にあるのに指した会話が無いときを含む)、その言葉が確認の記録の指紋と人の側(打った文か選んだ答え)の
      確認を示す語を含まない(打ち消し「承認しない」「いいえ」などは数えない)、confirms の記録が無いか本文の指紋(- <パス>、指紋 <16文字>)と今の指紋が合わない
      (確認のあとで変わった)、またはほかの記録の承認と同じ発言を指す(OM-13)
  E23 変更の記録(specs/_changes/)のフェーズの表が7行の順(要求・不明点・仕様化・設計・タスク・実装・照合)でない、
      状態が語彙外(未 / 進行中 / 済 / 対象外: 理由)、対象外に理由が無い、要求・実装・照合を対象外にしている、size が語彙外
  E24 変更の記録で、前のフェーズが済か対象外になる前に、後のフェーズが進行中か済になっている
  E25 変更の記録のタスクの表が、仕様に無い要件 ID を指している
  E26 done の変更の記録で、照合が済でない、unassessed が残る、「確かめた: N / M」が表と合わない、
      タスクの要件が照合の表に無い、同じ要件が2行ある、gap に証拠(欠けているもの)が書かれていない

警告:
  W1  README の生成区画が古い(--write で直す)
  W2  ケイパビリティが 10 本を超える
  W3  spec.md の要件が 15 を超える
  W4  確かめ方が空
  W5  spec.md に要件が1行も無い
  W6  提案が --stale-days(既定 30)日以上 proposed のまま
  W7  審議中の提案の「変更」の旧の引用が、現行の要件の文言と一致しない(先に入った変更がある)
  W8  要件の文末(理由の括弧は除く。DC-13)が強さの5語でない(最新の記録が取り込みの要件は除く)
  W9  要件に曖昧語がある(同上)
  W10 旧形式の提案が _proposals/ にある(書き直すか取り下げる)
  W15 spec.md の末尾の生成区画(決定とつながり)が古い(--write で直す)
  W17 approval-evidence の会話の記録が手元に無く、照合できない
  W11 仕様・原則の本文に【人】の印がある(人と AI の区別は決定記録に置く)
  W12 リポにテストがあるのに、要件を指すテストが無い(確かめ方が「読んで判定」「対象外」のものと、最新の記録が取り込みのものは除く)
  W13 守られる要件のテストの指紋が記録されていない(--lock)
  W14 書き直した取り込み(仕様の文言と元の文言が違う行がある)に「## 照合」が無い
  W18 active の変更の記録が --stale-days 日以上更新されていない
  W20 照合の仕組みより前の人の承認(伏せ字で、approval-evidence が無い)。人の決定のまま扱うが照合はできない
  W19 変更の記録の照合で、checked の証拠が型(コード: / テスト: / 今: / 前:、または 読んで判定)に合わない
  W21 AI 自己採択の記録と、同じ要件を守る人の決定が、同じ日に同じコミットで入り、前後が分からない(E15 を確かめられない)
  W22 要件を checked にした最後の変更の記録(done)がコミットされた後に、その要件を指すテストのファイルが変わった
  W23 指紋による承認で、会話の記録が読めず、人の発言が指紋を含むかの照合を飛ばした(OM-18)
  W25 人の確認待ちが3件あるあいだの AI 自己採択(E15 のブレーキ)だが、会話の記録がまったく無い場所(CI など)なので警告にとどめる(OM-13)
  W26 _decisions/ に入った記録の書き換え・消去(E34 の対象)だが、E34 の検査を持つ check.py がリポに入る前の履歴なので警告に
      留める(境は、その check.py が入ったコミット。追跡しているファイルにだけあれば次のコミット。OM-22)
  W24 AI 自己採択が触った要件が、その記録より後に gated に載った(さかのぼって E15 にしない)
      (git の履歴で確かめる。git が無い・記録が git に無い・作業ツリーで変えた記録は起点にしない。
      checked にした記録のどれか1件の後で変わっていなければ出さない)
"""
from __future__ import annotations

import datetime
import re
import sys
from pathlib import Path

ULID_RE = re.compile(r"^[0-9A-HJKMNP-TV-Z]{26}$")
ID_PAT = r"(?:[A-Z]{2,5}-\d+|P\d+)"
REQ_ROW_RE = re.compile(r"^\|\s*(%s)\s*\|" % ID_PAT)
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
PLACEHOLDER_RE = re.compile(r"\{[^{}\n]{1,60}\}")
MD_LINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
CROSS_REPO_RE = re.compile(r"^[\w.-]+/[\w.-]+:/")
# 錠: 要件の文を変えずに、その要件のテストの錠を、変えてよい決定の記録で掛け直す(TL-9)。
DIFF_RE = re.compile(r"^-\s*(追加|変更|削除|錠)\s*[::]\s*(%s)(.*)$" % ID_PAT)
OLD_NEW_RE = re.compile(r"旧「(.*?)」\s*→\s*新「(.*)」(?=\s*(?:\(確かめ方|\(文言|$))")
MUST_RE = re.compile(r"(なければならない|[てで]はならない)[。.]?$")
STRENGTH_RE = re.compile(r"(なければならない|[てで]はならない|べきでない|べき|[てで]よい)[。.]?$")
VAGUE = ("適切に", "なるべく", "できるだけ", "柔軟に", "速やかに", "十分に", "必要に応じて", "適宜")
CONVO_RE = re.compile(r"^会話\s*\d{4}-\d{2}-\d{2}$")
APPROVAL_LINE_RE = re.compile(r"\d{4}-\d{2}-\d{2} 人の承認")
REDACTED = "〔伏せ字"
VERBATIM_RE = re.compile(r"(?m)人「(?!…」|〔)[^」\n]+」|^>\s*【人】|【人】「(?!…」|〔)[^」\n]+」")

EXEMPT_NAMES = {"README.md", "AGENTS.md", "CLAUDE.md"}
SPEC_STATUS = {"draft", "active", "deprecated"}
DECIDED_STATUS = {"accepted", "rejected", "withdrawn"}
DECIDED_BY = {"人", "AI"}
TRIGGERS = ["人の発言", "運用で起きた失敗", "検証の指摘", "AI の提案"]
STRONG_TRIGGERS = {"人の発言", "運用で起きた失敗"}
MAX_REQS = 15
SUMMARIZED = {"W12": "要件を指すテストが無い", "W8": "要件の文末が強さの5語でない", "W3": "spec.md の要件が多い"}
MAX_CAPABILITIES = 10
DEFAULT_STALE_DAYS = 30
TEST_NAME_RE = re.compile(r"(^|/)(test_[^/]*|[^/]*_test\.[a-z]+|[^/]*\.(test|spec)\.[a-z]+)$")
TEST_ID_RES = [re.compile(r"\[(%s)\]" % ID_PAT), re.compile(r"test_([A-Z]{2,5})_(\d+)")]
SKIP_DIRS = {".git", "node_modules", ".venv", "venv", "__pycache__", "specs", ".decidespec"}


def transcript_dirs():
    import os
    env = os.environ.get("DECIDESPEC_TRANSCRIPTS")
    if env:
        return [Path(x).expanduser() for x in env.split(os.pathsep) if x]
    return [Path.home() / ".claude" / "projects"]


def _norm(s):
    return re.sub(r"\s+", "", s)


_ADDED = {}


def added_at(path):
    """記録が git に初めて入った時刻(まだ入っていなければ今)。同じ日付の記録の前後を決める。"""
    import subprocess
    import time
    if path not in _ADDED:
        r = subprocess.run(["git", "log", "--diff-filter=A", "--format=%ct", "--", path.name],
                           cwd=path.parent, capture_output=True, text=True)
        ts = r.stdout.split()
        _ADDED[path] = int(ts[-1]) if r.returncode == 0 and ts else int(time.time())
    return _ADDED[path]


def earlier_or_same(o, rec):
    """記録 o が rec と同じか前に決まったか(日付、同じ日なら git に入った順)。同じ日に同じコミットで入り、
    前後が分からなければ None(履歴の無い書き出し(OM-17)など。id の ULID は起票の時刻で、採択の順でない)。"""
    a, b = o.created or "", rec.created or "9999"
    if a != b:
        return a < b
    ta, tb = added_at(o.path), added_at(rec.path)
    if ta != tb:
        return ta < tb
    return None


_HUMAN_GRAMS = None
GRAM = 8


def human_grams():
    """会話の記録の人の発言(1,500字以下。長いものは貼り付けた引き継ぎ文などとみなす)の8字の断片の集合。"""
    global _HUMAN_GRAMS
    if _HUMAN_GRAMS is None:
        import json
        grams = set()
        for d in transcript_dirs():
            for f in d.glob("*/*.jsonl"):
                for line in f.open(encoding="utf-8", errors="replace"):
                    try:
                        e = json.loads(line)
                    except ValueError:
                        continue
                    t = human_text(e, typed_only=True) if isinstance(e, dict) else None
                    if t and 10 <= len(_norm(t)) <= 1500:
                        t = _norm(t)
                        grams.update(t[i:i + GRAM] for i in range(len(t) - GRAM + 1))
        _HUMAN_GRAMS = grams
    return _HUMAN_GRAMS


def human_sentences(text):
    """文のうち、人の発言と一致するもの(日本語が12字以上で6割以上の文だけを見る)。"""
    grams = human_grams()
    if not grams:
        return []
    hits = []
    for seg in re.split(r"(?<=[。!?！？])|\n|\|", text):
        s = _norm(re.sub(r"〔[^〕]*〕", "", seg))
        cjk = len(re.findall(r"[\u3040-\u30ff\u4e00-\u9fff]", s))
        if cjk < 12 or cjk / max(len(s), 1) < 0.6:
            continue
        ws = [s[i:i + GRAM] for i in range(len(s) - GRAM + 1)]
        if ws and sum(w in grams for w in ws) / len(ws) >= 0.7:
            hits.append(seg.strip())
    return hits


def fingerprint(text):
    """人の発言の指紋: 空白を除いた本文の SHA-256 の先頭16桁(記録には本文を残さない)。"""
    import hashlib
    return hashlib.sha256(_norm(text).encode("utf-8")).hexdigest()[:16]


def human_text(entry, typed_only=False):
    """会話の記録の1行が人の打ち込んだ発言なら、貼り付けを除いた本文を返す。そうでなければ None。

    人の発言は2つの形で残る: ふつうの発言(type: user、本文が文字列、origin.kind: human)と、
    作業中に送った発言(type: attachment、attachment.type: queued_command、attachment.origin.kind: human)。
    サブエージェントの報告(origin.kind: peer)や通知(task-notification)は人の発言ではない。
    選択肢の問いで人が選んだ答え(type: user の toolUseResult に questions と answers があるもの。道具が人の操作として
    書き、AI は書けない)も人の答えとして、「問いの文: 選んだ答え」を改行でつないだ文を返す(OM-12)。
    """
    content = None
    tur = entry.get("toolUseResult") if entry.get("type") == "user" else None
    if isinstance(tur, dict) and isinstance(tur.get("questions"), list) and isinstance(tur.get("answers"), dict):
        if typed_only:
            # 問いの文は AI が書いたもの。人の言葉の写し(E28)を見るときは、選んだ答えだけを人の言葉とする
            return "\n".join(str(a) for a in tur["answers"].values())
        return "\n".join("%s: %s" % (q, a) for q, a in tur["answers"].items())
    if entry.get("type") == "user":
        origin = entry.get("origin") or {}
        c = (entry.get("message") or {}).get("content")
        if origin.get("kind") == "human" and isinstance(c, str):
            content = c
    elif entry.get("type") == "attachment":
        att = entry.get("attachment") or {}
        if isinstance(att, dict) and att.get("type") == "queued_command" \
                and (att.get("origin") or {}).get("kind") == "human" and isinstance(att.get("prompt"), str):
            content = att["prompt"]
    if content is None:
        return None
    return re.sub(r"(?s)<pasted_content[^>]*>.*?</pasted_content[^>]*>", "", content)


def find_session(session_id):
    for d in transcript_dirs():
        for f in d.glob("*/%s.jsonl" % session_id):
            return f
    return None


def approval_quotes(rec):
    """承認の記録のうち「人「…」」の形の引用だけを、承認の言葉として拾う(出典の名前などの「」は拾わない)。"""
    return [q for q in re.findall(r"人「([^」]+)」", section(rec.body, "承認の記録")) if q.strip()]


def _load_human(session_id, uuid, with_time=False):
    """(状態, 人の発言)。with_time なら (状態, 人の発言, 時刻の文字列)。"""
    import json
    f = find_session(session_id)
    if not f:
        return ("unverifiable", None, "") if with_time else ("unverifiable", None)
    for line in f.open(encoding="utf-8"):
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if isinstance(e, dict) and e.get("uuid") == uuid:
            text = human_text(e)
            st = "ok" if text is not None else "not-human"
            return (st, text, str(e.get("timestamp") or "")) if with_time else (st, text)
    return ("absent", None, "") if with_time else ("absent", None)


def _load_entry(session_id, uuid):
    """(状態, 会話の記録の1行)。状態は ok / unverifiable(会話の記録が手元に無い) / absent(発言が無い)。"""
    import json
    f = find_session(session_id)
    if not f:
        return "unverifiable", None
    for line in f.open(encoding="utf-8"):
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if isinstance(e, dict) and e.get("uuid") == uuid:
            return "ok", e
    return "absent", None


def human_parts(entry):
    """人の言葉を (問いの文, 人の側の文) の組で返す。打った発言は ("", 本文)。選択肢の問いの答えは、問いごとに
    (問いの文, 選んだ答え)。問いの文は AI が書いたもので、人の側は選んだ答えだけ(OM-13)。人の言葉でなければ []。"""
    tur = entry.get("toolUseResult") if entry.get("type") == "user" else None
    if isinstance(tur, dict) and isinstance(tur.get("questions"), list) and isinstance(tur.get("answers"), dict):
        return [(str(q), str(a)) for q, a in tur["answers"].items()]
    t = human_text(entry)
    return [("", t)] if t is not None else []


def any_transcripts():
    """会話の記録の置き場に .jsonl が1つでもあるか。無ければ、会話の記録がまったく無い場所(CI など)とみなす(OM-13)。"""
    for d in transcript_dirs():
        if d.is_dir() and next(d.glob("*/*.jsonl"), None) is not None:
            return True
    return False


def verify_evidence(rec):
    """(状態, 説明)。状態は ok / missing / unverifiable / bad。指した発言が人のもので、指紋が合うこと。"""
    items = as_list(rec.fm.get("approval-evidence", ""))
    hashes = as_list(rec.fm.get("approval-hash", ""))
    if not items or not hashes:
        return "missing", ""
    if len(items) != len(hashes):
        return "bad", "approval-evidence と approval-hash の数が合わない"
    for it, h in zip(items, hashes):
        if it.startswith(REDACTED):
            # OM-17: 公開の書き出しで伏せ字にした場所は、照合できない(元のリポで照合済み)
            return "unverifiable", "承認の場所を伏せ字にした"
        m = re.match(r"^claude:([0-9a-f-]{36})/([0-9a-f-]{36})$", it)
        if not m:
            return "bad", "approval-evidence の形が claude:<会話の ID>/<発言の ID> でない(%s)" % it
        state, text = _load_human(m.group(1), m.group(2))
        if state == "unverifiable":
            return "unverifiable", "会話の記録 %s が手元に無い" % m.group(1)
        if state == "not-human":
            return "bad", "発言 %s は人の打ち込んだ発言ではない" % m.group(2)
        if state == "absent":
            return "bad", "会話 %s に発言 %s が無い" % (m.group(1), m.group(2))
        if fingerprint(text) != h:
            return "bad", "発言 %s の指紋が approval-hash と合わない" % m.group(2)
    return "ok", ""


def _set_front(text, key, val):
    if "\n%s:" % key in text:
        return re.sub(r"(?m)^%s:.*$" % re.escape(key), "%s: %s" % (key, val), text, count=1)
    if "\ndecided-by:" in text:
        return text.replace("\ndecided-by:", "\n%s: %s\ndecided-by:" % (key, val), 1)
    i = text.index("\n---", 3)
    return text[:i] + "\n%s: %s" % (key, val) + text[i:]


APPROVAL_TEXT_RE = re.compile(r"承認の文\s*`?(\S+?\.md)`?\s*[、,]\s*指紋\s*`?([0-9a-fA-F]{16}|[0-9a-fA-F]{8})(?![0-9a-fA-F])`?")
# 承認を示す語(小文字にした文で照合する)。英字の語は前後に英字が続かないとき、「はい」は区切りに挟まれたときだけ(OM-13)
APPROVAL_RE = re.compile(r"承認|了承|(?<![a-z])(?:ok|okay|approve|approved)(?![a-z])|(?:^|[\s、。,.!！?？])はい(?=$|[\s、。,.!！?？])")
# 確認・承認を打ち消す言い方(「承認しない」「OKではない」「いいえ」など)。人の側の文にあれば、確認・承認に数えない(OM-13)
NEG_RE = re.compile(r"(?:承認|了承|(?<![a-z])ok|okay|approve|approved)\s*(?:は|を)?\s*(?:しない|しません|せず|できない|できません|"
                    r"ではない|でない|じゃない|ではありません|不可|まだ|以外)|(?<![a-z])(?:not|no|don't|do not|never)\s+(?:ok|okay|approve)"
                    r"|いいえ|差し戻|却下|拒否|見送|保留")
FP_LEN = 16


# 選択肢の答えは AI が書いた選択肢の名前なので、確認・承認に数えるのは、指紋と承認を示す語だけの答えに限る(OM-13)。
# 「承認は保留する」「OK以外」のような、語を含むが断る名前の選択肢を、確認に数えないため
CHOICE_OK_RE = re.compile(r"^(?:を|の)?(?:承認|承認する|承認します|了承|了承する|了承します|ok|okay|はい|approve|approved)$")


def consents(side, choice=False):
    """人の側の文(打った文か選んだ答え)が、確認・承認を示す語を含み、打ち消していないか(OM-13)。
    選択肢の答え(choice)は、指紋・記号・空白を除くと承認を示す語だけであること。"""
    side = side.lower()
    if choice:
        rest = re.sub(r"(?<![0-9a-f])[0-9a-f]{16}(?![0-9a-f])", "", side)
        rest = re.sub(r"[\s、。,.!！()()「」:：]+", "", rest)
        return bool(CHOICE_OK_RE.match(rest))
    return bool(APPROVAL_RE.search(side)) and not NEG_RE.search(side)
LEGACY_FP_BEFORE = "2026-10-12T00:00:00"  # これより前の人の発言にある独立した8文字の指紋は、前の承認として有効(OM-13)
FENCE_RE = re.compile(r"^````markdown\n(.*?)\n````$", re.S | re.M)


def bundle_digest(path):
    """承認の文・束の SHA-256 の16進の全体: 囲み(````markdown)の中の本文 + 改行1つ。囲みが無ければ本文全体。"""
    import hashlib
    text = path.read_text(encoding="utf-8")
    fences = FENCE_RE.findall(text)
    body = "\n".join(fences) + "\n" if fences else split_front(text)[1]  # 囲みが複数なら全部をつなぐ
    return hashlib.sha256(body.encode("utf-8")).hexdigest()


def bundle_fingerprint(path):
    """承認の文・束の指紋: SHA-256 の先頭16文字(OM-13)。前の8文字の指紋は、先頭の8文字と比べて照合する。"""
    return bundle_digest(path)[:FP_LEN]


def record_fingerprint(path):
    """確認した決定記録の指紋: ファイル全体(フロントマターを含む)の SHA-256 の先頭16文字(OM-13)。
    確認のあとで decided-by や touches を変えても分かるように、フロントマターも含める。"""
    import hashlib
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()[:FP_LEN]


def confirm_fingerprint(path):
    """確認の記録の指紋: approval-evidence・approval-hash の行を除いたファイル全体(confirms と本文の指紋の行を含む)の
    SHA-256 の先頭16文字(OM-13)。囲みの外や confirms に、確認のあとで記録を足しても分かるように、全体で取る。"""
    import hashlib
    text = path.read_text(encoding="utf-8").replace("\r\n", "\n")
    end = text.find("\n---", 4) if text.startswith("---\n") else -1
    if end >= 0:  # 除くのはフロントマターの中の行だけ(split_front と同じ読み方。キーの前後の空白も同じに扱う)
        head = "".join(l for l in text[:end + 1].splitlines(keepends=True)
                       if l.split(":", 1)[0].strip() not in ("approval-evidence", "approval-hash") or l.startswith((" ", "#")))
        text = head + text[end + 1:]
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:FP_LEN]


_FP_HISTORY = {}
_APPROVED = {}


def approved_fingerprint(root, rec_path, text_path):
    """記録の git の履歴で、承認の文 text_path を指す承認の行が最初に入った版の指紋(承認した版)。記録がまだ git に無ければ None。"""
    key = (str(rec_path), text_path)
    if key not in _APPROVED:
        rel = rec_path.resolve().relative_to(root.resolve()).as_posix()
        log = git_out(root, "log", "--follow", "--reverse", "--format=%H", "--name-only", "--", rel) or ""
        lines = [l for l in log.splitlines() if l.strip()]
        found = None
        for h, name in zip(lines[0::2], lines[1::2]):
            old = git_out(root, "show", "%s:%s" % (h, name))
            if old is None:
                continue
            for path, fp in APPROVAL_TEXT_RE.findall(section(split_front(old)[1], "承認の記録")):
                if path == text_path:
                    found = fp.lower()
                    break
            if found:
                break
        _APPROVED[key] = found
    return _APPROVED[key]


def fingerprint_history(root, path):
    """承認の文が git に入ってから今までの各版の指紋(古い順)。書き換えられていれば2種類以上になる。git が無ければ []。"""
    import subprocess
    rel = path.resolve().relative_to(root.resolve()).as_posix()
    if rel not in _FP_HISTORY:
        log = git_out(root, "log", "--format=%H", "--", rel) or ""
        fps = []  # (指紋, 伏せ字を含むか)
        for h in reversed(log.split()):
            show = subprocess.run(["git", "-C", str(root), "show", "%s:./%s" % (h, rel)], capture_output=True, text=True)
            if show.returncode == 0:
                import tempfile
                with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8") as tf:
                    tf.write(show.stdout)
                fps.append((bundle_digest(Path(tf.name)), "〔伏せ字" in show.stdout))
                Path(tf.name).unlink()
        _FP_HISTORY[rel] = fps
    return _FP_HISTORY[rel]


_LINK_INLINE = re.compile(r"(\]\(<?)([^)\s<>#]+\.md)((?:#[^)\s<>]*)?>?(?:\s+\"[^\"]*\")?\))")
_LINK_REF = re.compile(r"(?m)^(\[[^\]]+\]:\s*<?)([^\s<>#]+\.md)((?:#\S*)?>?)")
_CODE_SPAN = re.compile(r"(?ms)^(`{3,}|~{3,}).*?^\1\s*$|`[^`\n]+`")


def _relink(text, base_dir, here_dir, target=None, new=None, only=False):
    """Markdown の相対リンクを書き換える(コードの囲みとインラインのコードの中は触らない)。target を渡したら、
    その場所を指すリンクだけを new へ向ける。渡さなければ全部を here_dir から正しく指すように直す(移した記録の中)。"""
    import os

    def one(m):
        t = m.group(2)
        if "://" in t or t.startswith("/"):
            return m.group(0)
        full = os.path.normpath(os.path.join(base_dir, t))
        if only:
            if full != target:
                return m.group(0)
            full = new
        elif target is not None and full == target:
            full = new  # 移した記録が自分を指すリンク
        return m.group(1) + os.path.relpath(full, here_dir).replace(os.sep, "/") + m.group(3)
    out, last = [], 0
    for c in _CODE_SPAN.finditer(text):
        seg = text[last:c.start()]
        out.append(_LINK_REF.sub(one, _LINK_INLINE.sub(one, seg)))
        out.append(c.group(0))
        last = c.end()
    seg = text[last:]
    out.append(_LINK_REF.sub(one, _LINK_INLINE.sub(one, seg)))
    return "".join(out)


def _set_or_add_front(text, key, val):
    """フロントマターの key を val にする(無ければ閉じの前に足す)。"""
    fm_end = text.find("\n---", 3)
    if text.startswith("---") and fm_end > 0 and re.search(r"(?m)^%s:" % re.escape(key), text[:fm_end]):
        return text[:fm_end].replace(re.search(r"(?m)^%s:.*$" % re.escape(key), text[:fm_end]).group(0),
                                     "%s: %s" % (key, val), 1) + text[fm_end:]
    if text.startswith("---") and fm_end > 0:
        return text[:fm_end] + "\n%s: %s" % (key, val) + text[fm_end:]
    return text


def settle(root, specs, prop, status):
    """CF-6: 提案の status と updated を書いて _decisions/ へ移し、この提案を指す相対リンクだけを新しい場所へ向け、
    移した記録の中の相対リンクを新しい場所から正しく指すよう直す。specs/_bundles/(見せたときのままの記録)と
    コードの囲みの中は触らない。読めないファイルがあれば何も変えずに止める。"""
    import os
    prop = prop.resolve()
    rootr = root.resolve()
    if status not in DECIDED_STATUS:
        return 1, "E0 --settle の status は %s のいずれか(今: %s)" % (" | ".join(sorted(DECIDED_STATUS)), status)
    if prop.parent != (specs / PROPS).resolve() or not prop.exists():
        return 1, "E0 --settle には %s/%s/ の提案を渡す(%s)" % (specs.relative_to(root).as_posix(), PROPS, prop)
    new = (specs / DECS).resolve() / prop.name
    if new.exists():
        return 1, "E0 %s が既にある" % new
    bundles = (specs / BUNDLES).resolve()
    plan, skipped, reading = {}, [], prop
    try:
        text = prop.read_text(encoding="utf-8")
        text = _set_or_add_front(_set_or_add_front(text, "status", status), "updated", datetime.date.today().isoformat())
        if split_front(text)[0] is None or split_front(text)[0].get("status") != status:
            return 1, "E0 %s のフロントマターに status を書けない" % prop
        plan[new] = _relink(text, str(prop.parent), str(new.parent), str(prop), str(new))
        for f in sorted(rootr.rglob("*.md")):
            reading = f
            parts = set(f.relative_to(rootr).parts)
            if f == prop or parts & (SKIP_DIRS - {"specs"}) or bundles in f.parents:
                continue
            try:
                t = f.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                if specs.resolve() in f.parents:
                    raise
                skipped.append(os.path.relpath(f, rootr))  # specs/ の外の読めないファイルは飛ばして知らせる
                continue
            n = _relink(t, str(f.parent), str(f.parent), str(prop), str(new), only=True)
            if n != t:
                plan[f] = n
    except (OSError, UnicodeDecodeError) as e:
        return 1, "E0 specs/ の中に読めないファイルがあるので何も変えない(%s): %s" % (os.path.relpath(reading, rootr), e)
    done = {}
    try:
        for f, t in plan.items():
            done[f] = f.read_text(encoding="utf-8") if f.exists() else None
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(t, encoding="utf-8")
        prop.unlink()
    except OSError as e:
        for f, t in done.items():  # 戻す
            if t is None:
                f.unlink()
            else:
                f.write_text(t, encoding="utf-8")
        return 1, "E0 書けなかったので戻した: %s" % e
    changed = [os.path.relpath(f, rootr) for f in plan if f != new]
    return 0, "移した: %s → %s(リンクを直した: %s)%s" % (
        os.path.relpath(prop, rootr), os.path.relpath(new, rootr), ", ".join(changed) or "なし",
        "(読めないので見なかった: %s)" % ", ".join(skipped) if skipped else "")


PRE_REQ_RE = re.compile(r"\b(ImportError|ModuleNotFoundError|SyntaxError|IndentationError)\b")
_RUN_CACHE = {}


def _run_unit(key, where, tmpl):
    """テストの単位を1つ走らせる。(通ったか, 1行, コマンド, 落ち方)。落ち方は assert(要件の振る舞いのアサーションで落ちた)/
    early(読み込み・コンパイル・例外・タイムアウトなど、要件の手前)/ unknown(--test-cmd で見分けられない)/ none(0件)。
    走らせ方が分からなければ None。"""
    import os
    import subprocess
    import sys
    if (key, str(where)) in _RUN_CACHE:
        return _RUN_CACHE[(key, str(where))]
    rel, _, name = key.partition("::")
    cwd = Path(where)
    env = dict(os.environ)
    if tmpl:
        cmd, kind = ["sh", "-c", tmpl.replace("{file}", rel)], "unknown"
    elif rel.endswith(".py"):
        mod = rel[:-3].replace("/", ".")
        cmd, kind = [sys.executable, "-m", "unittest", mod + ("." + name if name else "")], "py"
        # unittest discover と同じく、テストのファイルのフォルダを import の探し先に入れる(隣のテストを import するテストのため)
        env["PYTHONPATH"] = os.pathsep.join([str(cwd / os.path.dirname(rel)), str(cwd)] +
                                            ([env["PYTHONPATH"]] if env.get("PYTHONPATH") else []))
    elif rel.endswith(".rs") and name:
        d = (cwd / rel).parent
        while d != cwd and not (d / "Cargo.toml").exists():
            d = d.parent
        cmd, kind, cwd = ["cargo", "test", name, "--", "--exact"], "rs", d
    else:
        return None
    shown = " ".join(cmd).replace(sys.executable, "python3")
    try:
        r = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, timeout=600, env=env)
        if kind == "rs" and r.returncode == 0 and not re.search(r"running [1-9]", r.stdout + r.stderr):
            cmd = cmd[:3]  # モジュールの中のテスト(path::name)は --exact で当たらないので、名前の一部で走らせ直す
            shown = " ".join(cmd)
            r = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, timeout=600, env=env)
    except (OSError, subprocess.TimeoutExpired) as e:
        res = (False, (str(e).splitlines() or ["走らなかった"])[0][:160], shown, "early")
        _RUN_CACHE[(key, str(where))] = res
        return res
    out = r.stdout + r.stderr
    lines = out.strip().splitlines()
    why = next((l for l in reversed(lines) if ("Error" in l or "error" in l or "panicked" in l)
                and not l.startswith("FAILED (")), lines[-1] if lines else "")
    if kind == "py":
        if r.returncode == 0:
            how = "none" if re.search(r"Ran 0 tests", out) else "ok"
        else:
            how = "assert" if "FAIL:" in out and "ERROR:" not in out and not PRE_REQ_RE.search(out) else "early"
    elif kind == "rs" and "--exact" not in cmd:
        hits = re.findall(r"(?m)^test (\S+) \.\.\. (ok|FAILED|ignored)\s*$", out)
        mine = [st for t, st in hits if t == name or t.endswith("::" + name)]
        if not mine:
            how = "none"
        elif all(st == "ok" for st in mine):
            how = "ok"
        else:
            how = "assert" if "panicked" in out and not re.search(r"error(\[E\d+\])?:", out.replace("error: test failed", "")) else "early"
        r = subprocess.CompletedProcess(r.args, 0 if how == "ok" else 1, r.stdout, r.stderr)
    elif kind == "rs":
        if r.returncode == 0:
            how = "none" if re.search(r"running 0 tests", out) and not re.search(r"running [1-9]", out) else "ok"
        else:
            how = "assert" if "panicked" in out and not re.search(r"error(\[E\d+\])?:", out.replace("error: test failed", "")) else "early"
    else:
        how = "ok" if r.returncode == 0 else ("early" if PRE_REQ_RE.search(out) else "unknown")
    for w in {str(where), str(Path(where).resolve())}:
        why = why.replace(w + os.sep, "").replace(w, ".")
    res = (r.returncode == 0 and how != "none", why.strip()[:160], shown, how)
    _RUN_CACHE[(key, str(where))] = res
    return res


def _overlay_tests(root, before, tests, extra=()):
    """今のテストを「前」に重ねる。重ねるのは、テストのファイルと、「テスト:」のコミットで足した・変えたファイル(補助・fixture)と、
    それらの上のフォルダの __init__.py だけ(コードのファイルを重ねると「前」が今のコードになる)。"""
    import shutil
    files = set(tests) | {x for x in extra if (root / x).is_file()}
    for f in list(files):
        d = Path(f).parent
        while str(d) not in (".", ""):
            if (root / d / "__init__.py").exists():
                files.add((d / "__init__.py").as_posix())
            d = d.parent
    for f in sorted(files):
        if f.endswith("__init__.py") and f not in tests and (before / f).exists():
            continue  # コードのパッケージの __init__.py は、前の版のままにする
        (before / f).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / f, before / f)


def evidence(root, specs, rec_path, reqs, tmpl):
    """SF-9: 変更の記録の要件ごとに、要件を指すテストを今と「前」(最初の「テスト:」のコミットの親に今のテストを重ねたもの)で
    走らせ、照合の表の行を返す。checked は、今で全部通り、前で1つ以上がアサーションで落ちたときだけ。"""
    import io
    import shutil
    import subprocess
    import tarfile
    import tempfile
    rel = rec_path.resolve().relative_to(root.resolve()).as_posix()
    fm, body = split_front(rec_path.read_text(encoding="utf-8"))
    rids = []
    for r in table_rows(section(body, "タスク")):
        if len(r) > 2:
            rids += [x for x in re.findall(ID_PAT, r[2]) if x not in rids]
    for r in verify_rows(body):
        if re.match(ID_PAT + "$", r[0]) and r[0] not in rids:
            rids.append(r[0])
    tests = find_tests(root)
    units = test_units(root, tests)
    log = git_out(root, "log", "--reverse", "--format=%H%x09%s") or ""
    tcommit = next((l.split("\t", 1)[0] for l in log.splitlines()
                    if "\t" in l and l.split("\t", 1)[1].startswith("テスト:") and rel in l), None)
    prefix = (git_out(root, "rev-parse", "--show-prefix") or "").strip()
    code = []
    if tcommit:
        names = (git_out(root, "diff", "--name-only", tcommit, "--", ".") or "").split()
        untracked = (git_out(root, "ls-files", "--others", "--exclude-standard") or "").split()
        for n in names + untracked:
            n = n[len(prefix):] if prefix and n.startswith(prefix) else n
            if not TEST_NAME_RE.search(n) and not n.startswith("specs/") and n not in code:
                code.append(n)
        code.sort(key=lambda n: (n.endswith((".md", ".json")), n))  # 説明と設定のファイルは後ろに
    rows, checked, tmp = [], 0, None
    try:
        before = None
        if tcommit:
            arc = subprocess.run(["git", "-C", str(root), "archive", "%s^:%s" % (tcommit, prefix) if prefix else tcommit + "^"],
                                 capture_output=True)
            if arc.returncode == 0:
                tmp = tempfile.mkdtemp(prefix="decidespec-before-")
                with tarfile.open(fileobj=io.BytesIO(arc.stdout)) as tf:
                    tf.extractall(tmp)
                before = Path(tmp)
                added = (git_out(root, "diff", "--name-only", tcommit + "^", tcommit, "--", ".") or "").split()
                added = [n[len(prefix):] if prefix and n.startswith(prefix) else n for n in added]
                _overlay_tests(root, before, tests, [n for n in added if not n.startswith("specs/")])
        for rid in rids:
            mine = sorted(k for k, (ids, _) in units.items() if rid in ids)
            check = reqs.get(rid, {}).get("check", "")
            if not mine:
                why = "読んで判定(検証役へ)" if "読んで判定" in check else "要件を指すテストが無い(検証役へ)"
                rows.append("| %s | unassessed | %s |" % (rid, why))
                continue
            if before is None:
                rows.append("| %s | unassessed | 「テスト:」のコミット(件名に %s を含む)が見つからず、前を作れない(検証役へ) |" % (rid, rel))
                continue
            now = [(k, _run_unit(k, root, tmpl)) for k in mine]
            if any(r is None for _, r in now):
                rows.append("| %s | unassessed | 走らせ方が分からないテストがある(--test-cmd を渡すか、検証役へ) |" % rid)
                continue
            tests_s = ", ".join(mine[:3]) + (" ほか %d" % (len(mine) - 3) if len(mine) > 3 else "")
            bad_now = [(k, r) for k, r in now if not r[0]]
            if bad_now:
                k, r = bad_now[0]
                what = "0件しか走らなかった" if r[3] == "none" else "落ちた"
                rows.append("| %s | gap | 今: %s(%s: %s) / テスト: %s |" % (rid, what, k, r[1], tests_s))
                continue
            pre = [(k, _run_unit(k, before, tmpl)) for k in mine]
            real = [(k, r) for k, r in pre if r[3] == "assert"]
            unknown = [(k, r) for k, r in pre if r[3] == "unknown"]
            early = [(k, r) for k, r in pre if r[3] in ("early", "none")]
            code_s = (", ".join(code[:5]) + (" ほか %d" % (len(code) - 5) if len(code) > 5 else "")) if code else "なし"
            if real:
                checked += 1
                rows.append("| %s | checked | コード: %s / テスト: %s / 今: 通った(%s ほか) / 前: 落ちた(%s) |"
                            % (rid, code_s, tests_s, now[0][1][2], real[0][1][1] or "アサーション"))
            elif unknown:
                rows.append("| %s | gap | 前: 落ちたが、要件の振る舞いで落ちたかを見分けられない(--test-cmd。%s)。検証役へ / テスト: %s |"
                            % (rid, unknown[0][1][1], tests_s))
            elif early:
                rows.append("| %s | gap | 前: 要件の手前で落ちた(%s)。要件の振る舞いで落ちることを確かめていない / テスト: %s |"
                            % (rid, early[0][1][1], tests_s))
            else:
                rows.append("| %s | gap | 前でも通った(テストが要件を確かめていない) / テスト: %s |" % (rid, tests_s))
    finally:
        if tmp:
            shutil.rmtree(tmp, ignore_errors=True)
    return ["| 要件 | 結果 | 証拠 |", "|---|---|---|"] + rows + ["", "確かめた: %d / %d" % (checked, len(rows))]


def overlap(root, rids, tmpl):
    """TL-18: 要件ごとに、件名が「テスト:」で [ID] を含むコミットの親に今のテストを重ねた版で、要件のテストを走らせる
    (版とテストの組を並べて走らせる)。比べるのは、その要件のテストのうち通るものがある版だけ(全テストが落ちる版は、
    機能がまだ無い版で、見分ける手がかりにならない)。落ちた版が同じテストを組にまとめ、組 X の落ちた版を組 Y が
    すべて落ちているなら「X ⊆ Y」を、2つ以上のテストを持つ組は組そのものを、重なりの候補にする。どの版でも落ちない
    テストも候補。全テストが落ちる版でだけ落ちるテストは「見分けられない」、確かめられない版のあるテストは「比べない」。"""
    import os
    import shutil
    import subprocess
    import tempfile
    from concurrent.futures import ThreadPoolExecutor
    tests = find_tests(root)
    units = test_units(root, tests)
    log = git_out(root, "log", "--reverse", "--format=%H%x09%s") or ""
    prefix = (git_out(root, "rev-parse", "--show-prefix") or "").strip()
    out = []
    for rid in rids:
        mine = sorted(k for k, (ids, _) in units.items() if rid in ids)
        commits = [l.split("\t", 1)[0] for l in log.splitlines()
                   if "\t" in l and l.split("\t", 1)[1].startswith("実装:") and "[%s]" % rid in l.split("\t", 1)[1]]
        out.append("%s: テスト %d 件、版 %d(件名に [%s] を含む「実装:」のコミットを、今のコードから1つずつ打ち消した版)"
                   % (rid, len(mine), len(commits), rid))
        if not mine or not commits:
            out.append("比べられない(%s)" % ("テストが無い" if not mine else "「実装:」のコミットが無い"))
            continue
        # 版ごとに今の HEAD を一時の worktree に出し、その実装のコミットだけを打ち消して、今のテストを重ねる
        where, tmps, broken = {}, [], set()
        for c in commits:
            v = "−" + c[:7]
            tmp = tempfile.mkdtemp(prefix="decidespec-overlap-")
            wt = Path(tmp) / "wt"
            add = subprocess.run(["git", "-C", str(root), "worktree", "add", "--detach", "-q", str(wt), "HEAD"],
                                 capture_output=True, text=True)
            if add.returncode != 0:
                broken.add(v)
                shutil.rmtree(tmp, ignore_errors=True)
                continue
            tmps.append((tmp, wt))
            rv = subprocess.run(["git", "-C", str(wt), "revert", "--no-commit", c], capture_output=True, text=True)
            if rv.returncode != 0:
                broken.add(v)  # 打ち消しが衝突した: その版は作れない
                continue
            here = wt / prefix if prefix else wt
            _overlay_tests(root, here, tests)
            where[v] = here
        if broken:
            out.append("比べない版(打ち消しが衝突して作れない): %s" % ", ".join(sorted(broken)))
        try:
            jobs = [(v, k) for v in where for k in mine]
            with ThreadPoolExecutor(max_workers=os.cpu_count() or 2) as ex:
                results = dict(zip(jobs, ex.map(lambda j: _run_unit(j[1], where[j[0]], tmpl), jobs)))
        finally:
            for tmp, wt in tmps:
                subprocess.run(["git", "-C", str(root), "worktree", "remove", "--force", str(wt)], capture_output=True)
                shutil.rmtree(tmp, ignore_errors=True)
            subprocess.run(["git", "-C", str(root), "worktree", "prune"], capture_output=True)
        versions = ["−" + c[:7] for c in commits]
        fails = {k: set() for k in mine}
        unsure = {k: set() for k in mine}
        passed = {v: 0 for v in versions}
        for (v, k), r in results.items():
            if r is None or (not r[0] and r[3] != "assert"):
                unsure[k].add(v)
            elif not r[0]:
                fails[k].add(v)
            else:
                passed[v] += 1
        usable = {v for v in versions if passed[v]}  # 通るテストがある版だけを比べる
        skipped = sorted(set(versions) - usable - broken)
        if skipped:
            out.append("比べない版(要件のテストがどれも通らない。機能がまだ無い版): %s" % ", ".join(skipped))
        if not usable:
            # 比べる版が無いのに「どの版でも落ちない」と言うと、重なりでないものを候補に出してしまう
            out.append("比べられない(比べられる版が1つも無い。候補は出さない)")
            continue
        for k in mine:
            out.append("%s: 落ちた版 %s / 確かめられない版 %s" % (k, ", ".join(sorted(fails[k])) or "なし",
                                                        ", ".join(sorted(unsure[k])) or "なし"))
        groups = {}
        for k in mine:
            if unsure[k] & usable:
                # その版で、ほかのテストが捕まえない不具合を捕まえていたかもしれない。外して漏れを作らないよう候補にしない
                out.append("比べない: %s は確かめられない版(%s)があるので、重なりの候補にしない"
                           % (k, ", ".join(sorted(unsure[k] & usable))))
                continue
            f = frozenset(fails[k] & usable)
            if fails[k] and not f:
                out.append("見分けられない: %s は、要件のテストがどれも通らない版でだけ落ちる(候補にしない)" % k)
                continue
            groups.setdefault(f, []).append(k)
        for f, ks in sorted(groups.items(), key=lambda x: (sorted(x[0]), x[1])):
            if not f:
                for k in ks:
                    out.append("重なりの候補: %s はどの版でも落ちない" % k)
                continue
            if len(ks) > 1:
                out.append("重なりの候補(同じ版 %s で落ちる組): %s" % (", ".join(sorted(f)), ", ".join(ks)))
            for g, ys in groups.items():
                if g and f < g:
                    out.append("重なりの候補: 組 [%s] ⊆ 組 [%s](前者が落ちる版では後者も落ちる)"
                               % (", ".join(ks), ", ".join(ys)))
    out.append("候補は機械が外さない。外すかは整理(TL-14)で、新しい文脈の検証役の照合を経て決める")
    return out


def first_quote(text):
    """最初の「」の対の中身(入れ子の「」を数える)。差分の行の後ろにある別の「」(範囲外の文を消す注など)を
    要件の文に含めない(CF-17 の E16)。閉じていなければ None。"""
    start = text.find("「")
    if start < 0:
        return None
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "「":
            depth += 1
        elif text[i] == "」":
            depth -= 1
            if depth == 0:
                return text[start + 1:i]
    return None


def strength_tail(text):
    """強さを読む文末: 文末の理由の括弧(半角・全角、中に括弧を持たないもの)と後ろの「。」を、無くなるまで除く(DC-13)。"""
    t = text.strip()
    while True:
        m = re.search(r"[(（][^()（）]*[)）][。.]?$", t)
        if not m:
            return t
        t = t[:m.start()].rstrip()


def is_must(text):
    """必須の強さか: 文末(理由の括弧を除く)が しなければならない / してはならない、または文末に強さの5語が無く
    「必ず」(「必ずしも」は除く)を含む。"""
    t = text.strip()
    tail = strength_tail(t)
    # 除いた括弧の中に必須の語があれば、理由でなく必須の中身として読む(括弧に入れて E16 を黙らせない)
    if re.search(r"なければならない|[てで]はならない|必ず(?!しも)", t[len(tail):]):
        return True
    return bool(MUST_RE.search(tail)) or (bool(re.search(r"必ず(?!しも)", t)) and not STRENGTH_RE.search(tail))


def _as_val(xs):
    return xs[0] if len(xs) == 1 else "[%s]" % ", ".join(xs)


def approve(rec_path, records, phrases):
    """語句ごとに、それを含む人の発言を会話の記録から探し、approval-evidence と approval-hash を書き込む。語句は記録に残さない。"""
    import json
    rec = next((r for p, r in records.items() if p.resolve() == rec_path.resolve()), None)
    if rec is None and rec_path.is_file():
        fm0, b0 = split_front(rec_path.read_text(encoding="utf-8"))
        if fm0 and fm0.get("confirms"):
            rec = Record(rec_path, fm0, b0 or "")
    if rec is None:
        return "記録が見つからない: %s" % rec_path
    if not phrases:
        return "--find <語句> で、承認した人の発言を探す手がかりを渡す(語句は記録に残らない)"
    msgs = []
    for d in transcript_dirs():
        for f in sorted(d.glob("*/*.jsonl")):
            for line in f.open(encoding="utf-8"):
                try:
                    e = json.loads(line)
                except ValueError:
                    continue
                text = human_text(e)
                if text is not None:
                    msgs.append((e.get("timestamp", ""), f.stem, e.get("uuid"), text))
    evs, hs = [], []
    for q in phrases:
        hits = [m for m in msgs if _norm(q) in _norm(m[3])]
        if not hits:
            return "人の発言に見つからない(語句の %d 番目)。人の決定にできない" % (phrases.index(q) + 1)
        same_day = [h for h in hits if h[0][:10] == rec.created]
        ts, sid, uid, text = sorted(same_day or hits)[-1]
        ev = "claude:%s/%s" % (sid, uid)
        if ev not in evs:
            evs.append(ev)
            hs.append(fingerprint(text))
    body = rec_path.read_text(encoding="utf-8")
    body = _set_front(body, "approval-evidence", _as_val(evs))
    body = _set_front(body, "approval-hash", _as_val(hs))
    rec_path.write_text(body, encoding="utf-8")
    return "照合した: %s" % _as_val(evs)


def rehash(rec_path):
    """approval-evidence の指す人の発言から approval-hash を書き込む。"""
    body = rec_path.read_text(encoding="utf-8")
    fm, _ = split_front(body)
    hs = []
    for it in as_list((fm or {}).get("approval-evidence", "")):
        m = re.match(r"^claude:([0-9a-f-]{36})/([0-9a-f-]{36})$", it)
        state, text = _load_human(m.group(1), m.group(2)) if m else ("bad", None)
        if state != "ok":
            return "付け直せない(%s: %s)" % (it, state)
        hs.append(fingerprint(text))
    if not hs:
        return "approval-evidence が無い"
    rec_path.write_text(_set_front(body, "approval-hash", _as_val(hs)), encoding="utf-8")
    return "付け直した: %s" % _as_val(hs)


def git_ignored(root, rels):
    """TL-10: rels(根からの相対のパス)のうち、git が無視するもの。git の作業ツリーでなければ空。"""
    import subprocess
    rels = list(rels)
    if not rels:
        return set()
    try:
        r = subprocess.run(["git", "-C", str(root), "check-ignore", "--stdin"],
                           input="\n".join(rels) + "\n", capture_output=True, text=True)
    except OSError:
        return set()
    if r.returncode not in (0, 1):
        return set()
    return {line for line in r.stdout.splitlines() if line}


def find_tests(root):
    """テストのファイルと、その中で指している要件 ID。git が無視するファイルは除く(TL-10)。"""
    out = {}
    # 中に自分の specs/README.md を持つフォルダ(見本・別の仕様の根)は、別のリポとして探さない
    nested = [d.parent.parent for d in root.rglob("specs/README.md")
              if d.parent.parent != root and not set(d.relative_to(root).parts) & (SKIP_DIRS - {"specs"})]
    for p in root.rglob("*"):
        if not p.is_file() or set(p.relative_to(root).parts) & SKIP_DIRS:
            continue
        if any(n in p.parents for n in nested):
            continue
        rel = p.relative_to(root).as_posix()
        if not TEST_NAME_RE.search(rel):
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        ids = set(TEST_ID_RES[0].findall(text))
        ids |= {"%s-%s" % m for m in TEST_ID_RES[1].findall(text)}
        out[rel] = ids
    for rel in git_ignored(root, out):
        del out[rel]
    return out


def rust_tests(text):
    """Rust のテスト関数(`#[test]` の付いた `fn`)の (名前, 区間の始まり, 終わり) の並び。区間は `#[test]` の行の頭から
    本体の閉じの括弧まで。文字列・生の文字列・文字・行と範囲のコメントの中の括弧は数えない。読めなければ None(TL-8)。"""
    out = []
    i = 0
    n = len(text)
    while True:
        at = text.find("#[test]", i)
        if at < 0:
            return out
        start = text.rfind("\n", 0, at) + 1
        m = re.compile(r"fn\s+([A-Za-z_][A-Za-z0-9_]*)").search(text, at)
        if not m:
            return None
        j = text.find("{", m.end())
        if j < 0:
            return None
        depth = 0
        k = j
        while k < n:
            c = text[k]
            if text.startswith("//", k):
                nl = text.find("\n", k)
                k = n if nl < 0 else nl
                continue
            if text.startswith("/*", k):
                end = text.find("*/", k + 2)
                if end < 0:
                    return None
                k = end + 2
                continue
            raw = re.compile(r'r(#*)"').match(text, k) if c == "r" and (k == 0 or not (text[k - 1].isalnum() or text[k - 1] == "_")) else None
            if raw:
                close = '"' + raw.group(1)
                end = text.find(close, raw.end())
                if end < 0:
                    return None
                k = end + len(close)
                continue
            if c == '"':
                k += 1
                while k < n and text[k] != '"':
                    k += 2 if text[k] == "\\" else 1
                k += 1
                continue
            if c == "'":
                ch = re.compile(r"'(\\.[^']*|[^\\'])'").match(text, k)
                if ch:
                    k = ch.end()
                    continue
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    out.append((m.group(1), start, k + 1))
                    break
            k += 1
        else:
            return None
        i = k + 1


_LEGACY = {}  # 鍵 → 前の取り方(正規化なし)の指紋。TL-13 の掛け直しに使う


def _sha(text):
    import hashlib
    return hashlib.sha256(text.encode()).hexdigest()[:16]


def _words(text):
    """空白の違いを消す: 語と記号に分けて、1つの空白でつなぐ。"""
    return " ".join(re.findall(r"\w+|\S", text))


def _norm_py(src):
    """Python の指紋の元: コメントと空行と空白を落とす(文字列と字下げの構造は残す。TL-5)。"""
    import io
    import tokenize
    lines = src.splitlines(keepends=True)
    starts = [0]
    for ln in lines:
        starts.append(starts[-1] + len(ln))

    def at(pos):
        return starts[pos[0] - 1] + pos[1] if pos[0] - 1 < len(starts) else len(src)
    out, depth, fstart = [], 0, None
    try:
        for t in tokenize.generate_tokens(io.StringIO(src).readline):
            name = tokenize.tok_name.get(t.type, "")
            # Python 3.12 から f-string(3.14 から t-string)は中の字句に分かれる。版で指紋が変わらないよう、
            # 始まりから終わりまでを元のソースの1字句にまとめる(3.11 までと同じ形)。
            if name in ("FSTRING_START", "TSTRING_START"):
                if depth == 0:
                    fstart = at(t.start)
                depth += 1
                continue
            if depth:
                if name in ("FSTRING_END", "TSTRING_END"):
                    depth -= 1
                    if depth == 0:
                        out.append(src[fstart:at(t.end)])
                continue
            if t.type in (tokenize.COMMENT, tokenize.NL, tokenize.ENDMARKER):
                continue
            if t.type in (tokenize.INDENT, tokenize.DEDENT):
                out.append(tokenize.tok_name[t.type])
            elif t.type == tokenize.NEWLINE:
                out.append("\n")
            else:
                out.append(t.string)
    except (tokenize.TokenError, IndentationError, SyntaxError):
        return src  # 読めないときは正規化しない(安全側。コメントだけの変更でも E19 になる)
    return "\x1f".join(out)


_RS_RAW = re.compile(r'b?r(#*)"')
_RS_CHAR = re.compile(r"'(\\.[^']*|[^'\\])'")


def _norm_rs(src):
    """Rust の指紋の元: `//`・`/* */` のコメントと空白を落とす(文字列・文字の中はそのまま。TL-8)。"""
    out, i, n = [], 0, len(src)
    while i < n:
        c = src[i]
        word_before = i > 0 and (src[i - 1].isalnum() or src[i - 1] == "_")
        if src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j
            continue
        if src.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if src.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif src.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
            continue
        m = None if word_before else _RS_RAW.match(src, i)
        if m:
            end = src.find('"' + m.group(1), m.end())
            end = n if end < 0 else end + 1 + len(m.group(1))
            out.append("\x00" + src[i:end] + "\x00")
            i = end
            continue
        if c == '"' or (c == "b" and not word_before and src.startswith('b"', i)):
            j = i + (2 if c == "b" else 1)
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            out.append("\x00" + src[i:j + 1] + "\x00")
            i = j + 1
            continue
        if c == "'":
            m = _RS_CHAR.match(src, i)
            if m:
                out.append("\x00" + m.group(0) + "\x00")
                i = m.end()
                continue
        out.append(c)
        i += 1
    parts = re.split(r"(\x00[^\x00]*\x00)", "".join(out))
    return "\x1f".join(p if p.startswith("\x00") else _words(p) for p in parts if p.strip())


def _units_of(rel, text, file_ids):
    """1つのテストのファイルの錠の単位。{key: (ids, sha, legacy)}。sha はコメントと空白を除いた指紋(TL-5・TL-8)、
    legacy は前の取り方(全文そのまま)の指紋(TL-13)。"""
    import ast
    out = {}
    if rel.endswith(".py"):
        try:
            tree = ast.parse(text)
        except SyntaxError:
            return {rel: (set(), "SYNTAX-ERROR", "SYNTAX-ERROR")}  # 読めないテストは、錠の照合で必ず食い違う
        # 共有部分 = ファイルからテスト関数を除いた残り(補助関数・準備・import)。錠の指紋に含める
        segs = []
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name.startswith("test"):
                segs.append(ast.get_source_segment(text, node) or "")
        shared = text
        for s in segs:
            shared = shared.replace(s, "", 1)
        nshared = _norm_py(shared)

        def walk(node, prefix):
            for ch in ast.iter_child_nodes(node):
                if isinstance(ch, ast.ClassDef):
                    walk(ch, prefix + ch.name + ".")
                elif isinstance(ch, (ast.FunctionDef, ast.AsyncFunctionDef)) and ch.name.startswith("test"):
                    src = ast.get_source_segment(text, ch) or ""
                    ids = set(TEST_ID_RES[0].findall(src)) | {"%s-%s" % m for m in TEST_ID_RES[1].findall(ch.name)}
                    out["%s::%s%s" % (rel, prefix, ch.name)] = (
                        ids, _sha(_norm_py(src) + "\0" + nshared), _sha(src + "\0" + shared))
        walk(tree, "")
        return out
    if rel.endswith(".rs"):
        fns = rust_tests(text)
        if fns:
            # 共有部分 = ファイルからテスト関数の区間を除いた残り(補助関数・use・コメント)。
            shared, last = [], 0
            for _, a, b in fns:
                shared.append(text[last:a])
                last = b
            shared.append(text[last:])
            shared = "".join(shared)
            nshared = _norm_rs(shared)
            # ファイルの共有部分(モジュールの説明 `//!` など)が指す要件は、そのファイルの全部のテスト関数が確かめる
            # ものとして数える(ファイルごとの錠から移しても、守る要件を減らさない)。
            shared_ids = set(TEST_ID_RES[0].findall(shared))
            for name, a, b in fns:
                src = text[a:b]
                ids = shared_ids | set(TEST_ID_RES[0].findall(src)) | {"%s-%s" % m for m in TEST_ID_RES[1].findall(name)}
                out["%s::%s" % (rel, name)] = (
                    ids, _sha(_norm_rs(src) + "\0" + nshared), _sha(src + "\0" + shared))
            return out
    sha = _sha(text)  # Python と Rust 以外は、今までどおりファイルの全文
    return {rel: (file_ids, sha, sha)}


def test_units(root, tests):
    """錠の単位: Python と Rust はテスト関数ごと(指紋は関数の本体 + ファイルの共有部分。TL-8)、それ以外はファイルごと。
    {key: (ids, sha)}。前の取り方の指紋は _LEGACY に入れる。"""
    out = {}
    for rel in tests:
        for key, (ids, sha, legacy) in _units_of(rel, (root / rel).read_text(encoding="utf-8"), tests[rel]).items():
            out[key] = (ids, sha)
            _LEGACY[key] = legacy
    return out


def migrate_file_locks(root, locks, units, protected_ids):
    """TL-8: ファイルごとの錠(鍵に `::` が無い)で、今そのファイルが関数ごとの単位に分かれ、ファイルの中身が錠をかけた
    ときと同じなら、関数ごとの錠に移す(`from` に元の指紋)。移したファイルの鍵の並びを返す。"""
    import hashlib
    moved = []
    for key in [k for k in locks if "::" not in k]:
        fn_keys = [u for u in units if u.startswith(key + "::")]
        path = root / key
        if not fn_keys or not path.exists():
            continue
        sha = hashlib.sha256(path.read_text(encoding="utf-8").encode()).hexdigest()[:16]
        if sha != locks[key].get("sha"):
            continue
        del locks[key]
        for u in fn_keys:
            ids = sorted(units[u][0] & protected_ids)
            if ids:
                locks[u] = {"ids": ids, "sha": units[u][1], "from": sha}
        moved.append(key)
    return moved


def ignored_lock_keys(root, locks):
    """TL-10: git が無視するファイルを指す錠の鍵。"""
    paths = {k.split("::")[0] for k in locks}
    ign = git_ignored(root, paths)
    return {k for k in locks if k.split("::")[0] in ign}


def load_locks(path):
    import json
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def lock_history_errors(root, specs, lock_path, records):
    """E20: 錠の記録の書き換え・削除は、変えてよい決定の記録(by)を伴うときだけ(TL-6)。git の履歴で確かめる。"""
    import json
    need_p, _ = principles_gate(specs)
    import subprocess
    rel = lock_path.relative_to(root).as_posix()
    git = ["git", "-C", str(root)]
    try:
        r = subprocess.run(git + ["log", "--format=%H", "--reverse", "--", rel], capture_output=True, text=True)
    except OSError:
        return []  # git が無い
    if r.returncode != 0:
        return []
    out, prev = [], {}
    ignored = None
    for h in r.stdout.split():
        show = subprocess.run(git + ["show", "%s:./%s" % (h, rel)], capture_output=True, text=True)
        cur = json.loads(show.stdout) if show.returncode == 0 and show.stdout.strip() else {}
        gone = {k.split("::")[0] for k in prev if k not in cur}
        if gone:
            ignored = (ignored or set()) | git_ignored(root, gone)
        ignored = ignored or set()
        for key, entry in prev.items():
            new = cur.get(key)
            if new is not None and new.get("sha") == entry.get("sha"):
                continue
            by = (new or {}).get("by") if new else None
            ok = False
            if by:
                rec = next((x for p, x in records.items() if p.relative_to(root).as_posix() == by), None)
                ok = bool(rec) and rec.may_change_locked(need_p) \
                    and set(entry.get("ids", [])) <= set(rec.touches)
            if new is not None and new.get("legacy") == entry.get("sha") and \
                    _relock_ok(git, h, key, entry.get("sha"), new.get("sha")):
                continue  # TL-13: 前の取り方の指紋から、同じ中身の新しい指紋へ掛け直した
            if new is None and any(k.startswith(key + "::") and v.get("from") == entry.get("sha")
                                   for k, v in cur.items()):
                continue  # TL-8: ファイルごとの錠を、同じ中身の関数ごとの錠に移した
            if new is None and key.split("::")[0] in ignored:
                continue  # TL-10: git が無視するファイルの錠は、そのリポのテストでない
            if new is None:
                # 外すときは、そのコミットで _decisions/ に変えてよい決定の記録が足されていること
                files = subprocess.run(git + ["show", "--name-only", "--relative", "--format=", h],  # リポからの相対(git の最上位でなくても)
                                       capture_output=True, text=True).stdout.split()
                ok = any(f.startswith(specs.relative_to(root).as_posix() + "/" + DECS + "/") and
                         next((x for p, x in records.items() if p.relative_to(root).as_posix() == f
                               and x.may_change_locked(need_p) and set(entry.get("ids", [])) <= set(x.touches)), None)
                         for f in files)
            if not ok:
                out.append("E20 %s(コミット %s): 錠 %s を、変えてよい決定の記録(人の決定か、錨との照合のある AI 自己採択)なしに書き換え・外した"
                           % (rel, h[:7], key))
        prev = cur
    return out


_RELOCK = {}


def _principles_state(text):
    """principles.md の1版から (gated のキーがあるか, gated の ID の集合, P 番号ごとの行の組)。
    同じ P 番号の行が重なれば、その行をすべて組に入れる(差し込みを見逃さない)。無い版は空。"""
    fm, body = split_front(text or "")
    fm = fm or {}
    rows = {}
    for line in (body or "").splitlines():
        m = REQ_ROW_RE.match(line)
        if m and PRINCIPLE_RE.match(m.group(1)):
            rows.setdefault(m.group(1), []).append(line.strip())
    return "gated" in fm, set(as_list(fm.get("gated", ""))), {k: tuple(v) for k, v in rows.items()}


def _principles_changed(old, new):
    return (old[1] ^ new[1]) | {k for k in set(old[2]) | set(new[2]) if old[2].get(k) != new[2].get(k)} | (
        {"gated"} if old[0] and not new[0] else set())


def _human_touches(texts, seen=(), seen_evidence=()):
    """決定記録の (パス, 本文) から、採択済みの人の決定の touches の和。id が seen(それまでの履歴で
    _decisions/ にあった id)に入る記録は、古い決定の使い回し(外に出して戻す・消して書き直す)なので数えない。
    approval-evidence が seen_evidence(それまでの履歴の _decisions/ に現れた承認の場所)に入る記録も、
    本物の人の決定を id だけ変えて写したものなので数えない。"""
    covered = set()
    for path, text in texts:
        fm, body = split_front(text or "")
        if not fm or fm.get("id", "") in seen or set(as_list(fm.get("approval-evidence", ""))) & set(seen_evidence):
            continue
        x = Record(path, fm, body or "")
        if x.legacy:
            continue  # 旧い形(decided-by の無い記録)は過去を読むためだけで、新しく入っても人の決定に数えない
        if x.status == "accepted" and (x.decided_by == "人" or (x.decided_by == "AI" and x.anchor_ok(True))):  # OM-19: 錨との照合のある AI 自己採択も
            covered |= set(x.touches)
    return covered


def _entered(name_status, decs):
    """git diff --name-status の出力から、_decisions/ に入った記録のパス。足したものと、_decisions/ の外から
    移したもの(採択で _proposals/ から移す)だけ。_decisions/ の中での改名は、古い記録の使い回しなので数えない。"""
    out = []
    for line in name_status.splitlines():
        parts = line.split("\t")
        new = parts[-1]
        if not new.startswith(decs):
            continue
        if parts[0].startswith("R") and parts[1].startswith(decs):
            continue
        out.append(new)
    return out


def _req_rows(text):
    """spec.md の要件の表(生成区画を除く)の {ID: (要件の文, 確かめ方)}。決定の列は比べない(採択でリンクが増えるため)。"""
    own = H_RE.sub(lambda m: "\n" * m.group(0).count("\n"), text or "")
    out = {}
    for line in own.splitlines():
        m = REQ_ROW_RE.match(line)
        if m:
            c = cells(line)
            if len(c) == 4:
                out[m.group(1)] = (c[1], c[2])
    return out


def _gated_rows_errors(root, specs, commits, base, state, seen, added_decisions, git):
    """E31(OM-21): gated が初めて置かれたコミットより後に、そのときまでに一度でも gated に載った要件の spec.md の行
    (要件の文か確かめ方)を変えたコミットは、そのコミットで _decisions/ に入った採択済みの決定(人の決定か、錨との照合の
    ある AI 自己採択)で、その ID を touches に持つものを伴うこと。行は、そのコミットで変わったすべての spec.md を合わせて
    ID ごとに比べる(ケイパビリティの改名や行の移動は変更に数えない)。後で gated に足した要件は、足したコミットより後だけを
    見る。まだコミットしていない変更も HEAD と比べて見る。"""
    srel = specs.relative_to(root).as_posix()
    since = {}  # ID -> 初めて gated に載ったコミット
    for c in commits:
        for rid in state(c[0])[1]:
            since.setdefault(rid, c[0])
    p = specs / "principles.md"
    now_gated = _principles_state(p.read_text(encoding="utf-8"))[1] if p.is_file() else set()
    if not since and not now_gated:
        return []
    anc = {}

    def gated_at(par, h):
        """par の時点で載っていた(載ったコミットが par かその祖先)要件と、h で載っている要件。"""
        out = set(state(h)[1])
        for rid, at in since.items():
            if (at, par) not in anc:
                anc[(at, par)] = at == par or git("merge-base", "--is-ancestor", at, par).returncode == 0
            if anc[(at, par)]:
                out.add(rid)
        return out

    def rows_at(rev, files):
        out = {}
        for f in files:
            r = git("show", "%s:./%s" % (rev, f))
            out.update(_req_rows(r.stdout if r.returncode == 0 else ""))
        return out

    # spec.md を変えたコミットだけに先に絞る(すべてのコミットに diff を呼ばない)
    lg = git("log", "-m", "--no-renames", "--format=@%H", "--name-only", "--relative", "--",
             ":(glob)%s/*/spec.md" % srel)
    touched, cur = {}, None
    for line in (lg.stdout if lg.returncode == 0 else "").splitlines():
        if line.startswith("@"):
            cur = line[1:]
        elif line.strip() and cur:
            touched.setdefault(cur, set()).add(line.strip())
    out = []
    for c in commits:
        h, parents = c[0], c[1:]
        if h == base or not parents or h not in touched or git("merge-base", "--is-ancestor", base, h).returncode != 0:
            continue
        changed = None
        for par in parents:  # マージは、どの親とも違う変更だけを、その場の変更とみなす
            files = [f for f in git("diff", "--name-only", "--no-renames", "--relative", par, h, "--", srel).stdout.split()
                     if f.endswith("/spec.md")]
            a, b = rows_at(par, files), rows_at(h, files)
            ch = {rid for rid in gated_at(par, h) if a.get(rid) != b.get(rid)}
            changed = ch if changed is None else changed & ch
        if not changed:
            continue
        covered = set()
        for par in parents:
            covered |= _human_touches(added_decisions(par, h), *seen(par))
        missing = sorted(changed - covered)
        if missing:
            out.append("E31 %s/(コミット %s): gated の要件 %s の行(要件の文か確かめ方)を、その ID に触る決定の記録なしに"
                       "変えた(OM-21)" % (srel, h[:7], ", ".join(missing)))
    head = git("rev-parse", "--verify", "-q", "HEAD")
    if head.returncode == 0 and git("merge-base", "--is-ancestor", base, "HEAD").returncode == 0:
        st = git("diff", "--name-only", "--no-renames", "--relative", "HEAD", "--", srel).stdout.split()
        st += git("ls-files", "--others", "--exclude-standard", "--", srel).stdout.split()
        files = sorted({f for f in st if f.endswith("/spec.md")})
        if files:
            a = rows_at("HEAD", files)
            b = {}
            for f in files:
                if (root / f).is_file():
                    b.update(_req_rows((root / f).read_text(encoding="utf-8")))
            changed = {rid for rid in gated_at("HEAD", "HEAD") | now_gated if a.get(rid) != b.get(rid)}
            if changed:
                decs = srel + "/" + DECS + "/"
                new = _entered(git("diff", "--name-status", "--diff-filter=AR", "-M", "--relative", "HEAD").stdout, decs)
                new += git("ls-files", "--others", "--exclude-standard", "--", decs).stdout.split()
                texts = [(root / q, (root / q).read_text(encoding="utf-8")) for q in new if (root / q).is_file()]
                missing = sorted(changed - _human_touches(texts, *seen("HEAD")))
                if missing:
                    out.append("E31 %s/(まだコミットしていない変更): gated の要件 %s の行を、その ID に触る決定の記録なしに"
                               "変えている(OM-21)" % (srel, ", ".join(missing)))
    return out


SEALED_KEYS = ("status", "decided-by", "touches")


def _sealed_keys(path, text):
    """決着した記録の、書き換えてはならない値。decided-by と touches は Record の読み方で取る(旧い形の記録は
    decision-makers と本文の差分の節から読むので、それも含めて比べる。名前の伏せ字は人か AI かを変えないので比べない)。"""
    fm, body = split_front(text or "")
    if not fm:
        return {k: "" for k in SEALED_KEYS}
    r = Record(path, fm, body or "")
    return {"status": r.status, "decided-by": r.decided_by, "touches": tuple(sorted(r.touches))}


def sealed_errors(root, specs):
    """E34(OM-22): _decisions/ に入った記録は、入ったあとで status・decided-by(旧い形は decision-makers)・touches を
    変えたり、記録を消したり、別の名前に移したりしないこと。比べる元は、その記録が _decisions/ に入ったあとで、決着した
    status(DECIDED_STATUS)を初めて持ったコミットの版(_decisions/ の中での改名はたどる)。proposed のまま入った版は
    比べる元にしない(採択の作業の途中のコミットに、移しただけの版が紛れ込むことがあるため)。proposed のまま入った記録が
    枝ごとに別々に決着したときは、どの枝でも最初に決着した版(祖先に決着した版を持たない版)を比べる元にし、そのどれかと
    違えば E34 にする(マージで片方の決着を書き換えさせないため)。コミットに決着した版が無ければ比べない。今の版は作業ツリー。本文(伏せ字・採択の道具が直すリンク)は比べない。
    マージのコミットの差分と、まだコミットしていない(ステージ済みを含む)消去・改名も見る。git の履歴が無ければ確かめない。
    この検査(def sealed_errors を持つ check.py)がリポに初めて入ったコミット(境)より前のコミットで起きた書き換え・
    消去は、検査の無かったときの履歴なので W26 に留める(境のコミット自身と、その後と、まだコミットしていないものは E34)。
    (エラーと W26 の行, 入ったときの値) を返す。入ったときの値は、書き換えた・移した記録の {今の名前: 値} で、ブレーキの
    数え方はこの値で見る(書き換えで確認待ちから外さないため。W26 に留めた書き換えは今の値で見る)。"""
    import subprocess
    decs = specs.relative_to(root).as_posix() + "/" + DECS + "/"

    def git(*args):
        return subprocess.run(["git", "-C", str(root)] + list(args), capture_output=True, text=True,
                              encoding="utf-8", errors="replace")

    try:
        # --topo-order: 古い日付の枝のコミットが、記録の入ったコミットより前に並んで比べる元から落ちないように
        r = git("log", "--reverse", "--topo-order", "--full-history", "-m", "--format=@%H", "--name-status", "-M",
                "--relative", "--", decs)
    except OSError:
        return [], {}
    if r.returncode != 0:
        return [], {}
    first, gone, cur = {}, {}, None  # first: 今のパス -> (最初に入ったコミット, そのときのパス)
    vers = {}  # 今のパス -> 入ったあとにその記録を変えたコミットの並び [(コミット, そのときのパス)](古い順)

    def take(st, parts, where):
        if st in ("A", "C", "R"):
            new = parts[-1]
            if new.startswith(decs):
                old = parts[1] if st == "R" else None
                if old and old.startswith(decs) and old in first:
                    first.setdefault(new, first[old])  # _decisions/ の中での改名: 元の記録の版と比べる
                    vers.setdefault(new, list(vers.get(old, [])))
                else:
                    first.setdefault(new, (where, new))
                    vers.setdefault(new, [])
                if where is not None:
                    vers[new].append((where, new))
        if st == "M" and parts[1] in vers and where is not None:
            vers[parts[1]].append((where, parts[1]))
        if st in ("D", "R") and parts[1].startswith(decs):
            gone.setdefault(parts[1], [])
            if where not in gone[parts[1]]:
                gone[parts[1]].append(where)

    for line in r.stdout.splitlines():
        if line.startswith("@"):
            cur = line[1:]
            continue
        parts = line.split("\t")
        if len(parts) >= 2:
            take(parts[0][:1], parts, cur)
    # まだコミットしていない(ステージ済みを含む)消去・改名
    w = git("diff", "--name-status", "-M", "--relative", "HEAD", "--", decs)
    for line in (w.stdout if w.returncode == 0 else "").splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            take(parts[0][:1], parts, None)
    for path in git("ls-files", "--deleted", "--", decs).stdout.split():
        gone.setdefault(path, [])
        if None not in gone[path]:
            gone[path].append(None)

    # 境: 検査を持つ check.py(名前が check.py で def sealed_errors を含む)が、親のどれにも無い状態から入ったコミット
    # すべて(マージで入ってきたものは数えない)。境より前とは、そのすべての真の祖先であること(枝の上に別の境を作る、
    # 親の順を入れ替えたマージ、古い日付のコミットでは動かない。枝ごとに別々に入れたリポでは、どちらにも前のものだけ)。
    # コミットに無く、追跡しているファイル(ステージ済みを含む)にだけあれば、次のコミットが境で、コミット済みの履歴は
    # すべて境より前。追跡していないファイルは数えない。どこにも無いリポ(検査を外から走らせるだけ)は免除しない
    has_cache = {}

    def has_check(c):
        if c not in has_cache:
            r2 = git("grep", "-l", "-F", "def sealed_errors", c, "--", ":(glob)**/check.py")
            has_cache[c] = r2.returncode == 0 and bool(r2.stdout.strip())
        return has_cache[c]

    lb = git("log", "--full-history", "-m", "--format=%H", "-S", "def sealed_errors", "--", ":(glob)**/check.py")
    mins = []
    for c in sorted(set(lb.stdout.split()) if lb.returncode == 0 else set()):
        pr = git("rev-list", "--parents", "-n", "1", c).stdout.split()[1:]
        if has_check(c) and not any(has_check(q) for q in pr):
            mins.append(c)
    upcoming = False
    if not mins:
        for q in git("ls-files", "--cached", "--", ":(glob)**/check.py").stdout.split():
            try:
                if "def sealed_errors" in (root / q).read_text(encoding="utf-8", errors="replace"):
                    upcoming = True
                    break
            except OSError:
                continue
    rel_cache = {}

    def anc(x, y):
        if (x, y) not in rel_cache:
            rel_cache[(x, y)] = x == y or git("merge-base", "--is-ancestor", x, y).returncode == 0
        return rel_cache[(x, y)]

    def pre_bound(c):
        """コミット c が境より前か: 境のすべての真の祖先。境が次のコミットなら、コミット済みはすべて前。境が無ければ前は無い。"""
        if c is None:
            return False
        if not mins:
            return upcoming
        return all(c != m and anc(c, m) for m in mins)

    def post_bound(c):
        """コミット c が境の後か: 境のどれかが c の祖先(c 自身を含む)。"""
        return c is not None and any(anc(m, c) for m in mins)

    def at_bounds(paths):
        """境の各コミットとその親(境より前の版)での記録の版(paths のどれかの名前で)。無いところは None。
        境のコミット自身での書き換えを境より前に数えないため、親の版も見る。"""
        vals = []
        points = []
        for m in mins:
            points += [m] + git("rev-list", "--parents", "-n", "1", m).stdout.split()[1:]
        for m in points:
            got = None
            for q in paths:
                r3 = git("show", "%s:./%s" % (m, q))
                if r3.returncode == 0:
                    got = r3.stdout
                    break
            vals.append(got)
        return vals

    out, orig = [], {}
    for path, (h, at) in sorted(first.items()):
        f = root / path
        if not f.is_file() or h is None:
            continue
        b = _sealed_keys(f, f.read_text(encoding="utf-8"))
        settled_vers, seen = [], set()  # 決着した版 [(コミット, 値)]
        for c, cp in vers.get(path, [(h, at)]):
            if c in seen:
                continue
            seen.add(c)
            old = git("show", "%s:./%s" % (c, cp))
            if old.returncode != 0:
                continue
            keys = _sealed_keys(f, old.stdout)
            if keys["status"] in DECIDED_STATUS:
                # 入ったときに決着していた版でも打ち切らない。同じ名前の記録をほかの枝で別に足したときも、その枝で決着した
                # 版を比べる元に入れる(同じ線の後の版は、祖先に決着した版を持つので下で比べる元から外れる)
                settled_vers.append((c, keys))
        # 比べる元: どの枝でも最初に決着した版(ほかの決着した版を祖先に持たない版)
        bases = [(c, keys) for c, keys in settled_vers
                 if not any(d != c and git("merge-base", "--is-ancestor", d, c).returncode == 0
                            for d, _ in settled_vers)]
        if not bases:
            continue
        settled, a = next(((c, keys) for c, keys in bases if any(keys[k] != b[k] for k in SEALED_KEYS)), bases[0])
        diff = [k for k in SEALED_KEYS if a[k] != b[k]]
        if diff:
            # W26: 境のどの時点でも、すでに今の値になっていて、境の後のどの版も、HEAD の版も今の値と同じとき。
            # 境が次のコミットなら、HEAD の版が今の値と同じとき(コミット済みの書き換えはすべて境より前)
            hv = git("show", "HEAD:./%s" % path)
            ok = hv.returncode == 0 and _sealed_keys(f, hv.stdout) == b and (mins or upcoming)
            if ok and mins:
                ok = all(v is not None and _sealed_keys(f, v) == b for v in at_bounds([path, at]))
                for c, cp in vers.get(path, []):
                    if not ok or pre_bound(c):
                        continue
                    # 境より前でない版(境の後と、境と無関係な枝)は、今の値か、親のどれかの版と同じ値(その版では値を
                    # 変えていない)であること。境の後の版は親も境の後なので、たどれば今の値でなければならない
                    old = git("show", "%s:./%s" % (c, cp))
                    if old.returncode != 0:
                        continue
                    kc = _sealed_keys(f, old.stdout)
                    if kc == b:
                        continue
                    same = False
                    for q in git("rev-list", "--parents", "-n", "1", c).stdout.split()[1:]:
                        pv = git("show", "%s:./%s" % (q, cp))
                        if pv.returncode == 0 and _sealed_keys(f, pv.stdout) == kc:
                            same = True
                    ok = same and not post_bound(c)
            if ok:
                out.append("W26 %s: _decisions/ に入ったあとで %s を変えたが、E34 の検査がリポに入る前の履歴なので"
                           "警告に留める(OM-22)" % (path, "・".join(diff)))
                if at != path:
                    orig[Path(path).name] = b
                continue
        if diff or at != path:
            orig[Path(path).name] = a
        if diff:
            out.append("E34 %s: _decisions/ に入ったあとで %s を変えた(%sコミット %s の版と違う。決着した記録は書き換えない。"
                       "OM-22)" % (path, "・".join(diff), "入った" if settled == h else "決着した", settled[:7]))
    for path, hs in sorted(gone.items()):
        if (root / path).exists():
            continue
        # W26: 消したコミットがすべて境より前で、境のどの時点にも記録が無いとき
        if all(pre_bound(h) for h in hs) and not any(v is not None for v in at_bounds([path])):
            out.append("W26 %s: _decisions/ から記録を消したか移した(コミット %s)が、E34 の検査がリポに入る前の履歴なので"
                       "警告に留める(OM-22)" % (path, ", ".join(h[:7] for h in hs)))
        else:
            h = next((h for h in hs if not pre_bound(h)), hs[-1])
            out.append("E34 %s: _decisions/ から記録を消したか移した(%s。決着した記録は消さない。OM-22)"
                       % (path, "コミット %s" % h[:7] if h else "まだコミットしていない変更"))
    return out, orig


def principles_history_errors(root, specs, records):
    """E31(OM-19): principles.md に gated が初めて置かれたコミットより後に、原則の行か gated を変えたコミットは、
    そのコミットで _decisions/ に入った(足した・移した)採択済みの人の決定を伴い、その touches が変えた ID を
    すべて含むこと。各コミットは親の版と比べ(マージは、どの親とも違う変更だけ)、記録はそのコミットの版から読む。
    まだコミットしていない変更(ステージしたものと作業ツリー)も HEAD と比べて見る(pre-commit で止めるため)。"""
    import subprocess
    p = specs / "principles.md"
    rel = p.relative_to(root).as_posix()
    decs = specs.relative_to(root).as_posix() + "/" + DECS + "/"

    def git(*args):
        return subprocess.run(["git", "-C", str(root)] + list(args), capture_output=True, text=True)

    try:
        # パスで絞った git log は履歴を簡略化してマージや枝のコミットを飛ばしうるので、すべてのコミットをたどる。
        r = git("rev-list", "--reverse", "--topo-order", "--parents", "HEAD")
    except OSError:
        return []  # git が無い
    if r.returncode != 0:
        return []  # コミットが無い
    commits = [line.split() for line in r.stdout.splitlines() if line.strip()]
    query = "".join("%s:./%s\n" % (c[0], rel) for c in commits)
    b = subprocess.run(["git", "-C", str(root), "cat-file", "--batch-check=%(objectname) %(objecttype)"],
                       input=query, capture_output=True, text=True)
    blob = {}
    for c, line in zip(commits, b.stdout.splitlines()):
        parts = line.split()
        blob[c[0]] = parts[0] if len(parts) == 2 and parts[1] == "blob" else None
    by_blob = {}

    def state(rev):
        k = blob.get(rev, "?" + rev)
        if k not in by_blob:
            if k is None:
                by_blob[k] = _principles_state("")
            else:
                s = git("show", "%s:./%s" % (rev, rel))
                by_blob[k] = _principles_state(s.stdout if s.returncode == 0 else "")
        return by_blob[k]

    base = next((c[0] for c in commits if blob.get(c[0]) and state(c[0])[0]), None)
    if base is None:
        return []
    seen_cache = {}

    def seen(rev):
        """rev とその祖先で、一度でも _decisions/ にあった記録の id と approval-evidence。"""
        if rev not in seen_cache:
            lg = git("log", "-p", "--no-renames", "--full-history", "--format=", rev, "--", decs)
            # フロントマターと同じ読み方(前後の引用符を外す)で取る
            def grab(key):  # リストの形でも要素に分けて取る
                return {v for m in re.finditer(r"(?m)^[+-]%s:(.*)$" % re.escape(key), lg.stdout)
                        for v in as_list(m.group(1).strip().strip('"').strip("'"))}
            seen_cache[rev] = (grab("id"), grab("approval-evidence"))
        return seen_cache[rev]

    def added_decisions(parent, rev):
        """parent から rev までに _decisions/ に足された・移された記録の (パス, rev の版の本文)。"""
        out = []
        for path in _entered(git("diff", "--name-status", "--diff-filter=AR", "-M", "--relative", parent, rev).stdout, decs):
            s = git("show", "%s:./%s" % (rev, path))
            out.append((root / path, s.stdout if s.returncode == 0 else ""))
        return out

    out = []
    out += _gated_rows_errors(root, specs, commits, base, state, seen, added_decisions, git)
    for c in commits:
        h, parents = c[0], c[1:]
        if h == base or not parents or all(blob.get(h) == blob.get(par) for par in parents):
            continue  # 原則のファイルが、どの親とも同じ
        if git("merge-base", "--is-ancestor", base, h).returncode != 0:
            continue
        cur = state(h)
        changed = None
        for par in parents:  # マージは、検査を経た親(起点かその子孫)のどれとも違う変更だけを、その場の変更とみなす
            ch = _principles_changed(state(par), cur)
            if par != base and git("merge-base", "--is-ancestor", base, par).returncode != 0:
                continue  # 起点より前に分かれた枝の親: その枝の変更は検査を経ていない
            changed = ch if changed is None else changed & ch
        if changed is None:
            changed = _principles_changed(state(parents[0]), cur)
        if not changed:
            continue
        covered = set()
        for par in parents:
            covered |= _human_touches(added_decisions(par, h), *seen(par))
        missing = sorted(changed - covered)
        if missing:
            out.append("E31 %s(コミット %s): 原則か gated の %s を、その ID に触る人の決定なしに変えた(OM-19)"
                       % (rel, h[:7], ", ".join(missing)))
    # まだコミットしていない変更: HEAD の版と作業ツリーを比べ、作業ツリーで _decisions/ に入った記録を数える
    head = git("rev-parse", "--verify", "-q", "HEAD")
    if head.returncode == 0 and (base == head.stdout.strip() or
                                 git("merge-base", "--is-ancestor", base, "HEAD").returncode == 0):
        cur = _principles_state(p.read_text(encoding="utf-8") if p.is_file() else "")
        changed = _principles_changed(state("HEAD"), cur)
        if changed:
            new = _entered(git("diff", "--name-status", "--diff-filter=AR", "-M", "--relative", "HEAD").stdout, decs)
            new += git("ls-files", "--others", "--exclude-standard", "--", decs).stdout.split()
            texts = [(root / q, (root / q).read_text(encoding="utf-8")) for q in new if (root / q).is_file()]
            missing = sorted(changed - _human_touches(texts, *seen("HEAD")))
            if missing:
                out.append("E31 %s(まだコミットしていない変更): 原則か gated の %s を、その ID に触る人の決定なしに"
                           "変えている(OM-19)" % (rel, ", ".join(missing)))
    return out


def _relock_ok(git, commit, key, legacy, sha):
    """TL-13 の掛け直しが正しいか: そのコミットのテストのファイルから、前の取り方と新しい取り方の指紋を
    計算し直し、錠の記録の両方と一致するか。記録の legacy を書き換えただけでは通らない。"""
    import subprocess
    rel = key.split("::")[0]
    if (commit, rel) not in _RELOCK:
        show = subprocess.run(git + ["show", "%s:./%s" % (commit, rel)], capture_output=True, text=True)
        _RELOCK[(commit, rel)] = _units_of(rel, show.stdout, set()) if show.returncode == 0 else {}
    got = _RELOCK[(commit, rel)].get(key)
    return bool(got) and got[2] == legacy and got[1] == sha


def file_hash(path):
    import hashlib
    return hashlib.sha256(path.read_bytes()).hexdigest()[:16]


PROPS, DECS, CHANGES, BUNDLES = "_proposals", "_decisions", "_changes", "_bundles"
CHANGE_STATUS = {"active", "done", "abandoned"}
CHANGE_SIZES = {"tiny", "bugfix", "full"}
PHASES = ["要求", "不明点", "仕様化", "設計", "タスク", "実装", "照合"]
NO_SKIP = {"要求", "実装", "照合"}
VERIFY_RESULTS = {"checked", "gap", "unassessed"}
H_BEGIN, H_END = "<!-- decidespec:history:begin -->", "<!-- decidespec:history:end -->"
H_RE = re.compile(re.escape(H_BEGIN) + r".*?" + re.escape(H_END), re.S)
BEGIN, END = "<!-- decidespec:begin -->", "<!-- decidespec:end -->"
BLOCK_RE = re.compile(r"<!-- (?:decidespec|decidespec):begin -->.*?<!-- (?:decidespec|decidespec):end -->", re.S)  # 旧い名前の目印も読む


def strip_code(text):
    """コードブロックとインラインコードを同じ長さの空白に潰す(書式の説明例を拾わないため)。"""
    def blank(m):
        return re.sub(r"[^\n]", " ", m.group(0))
    text = re.sub(r"(?ms)^```.*?^```", blank, text)
    text = re.sub(r"(?s)<!--.*?-->", blank, text)
    return re.sub(r"`[^`\n]*`", blank, text)


def split_front(text):
    """(フロントマターの dict, 本文) を返す。無ければ (None, text)。"""
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---", 4)
    if end < 0:
        return None, text
    fm = {}
    for line in text[4:end].splitlines():
        if not line or line.startswith((" ", "#")) or ":" not in line:
            continue
        k, v = line.split(":", 1)
        fm[k.strip()] = v.strip().strip('"').strip("'")
    return fm, text[end + 4:]


def as_list(v):
    v = (v or "").strip()
    if v.startswith("[") and v.endswith("]"):
        v = v[1:-1]
    return [x.strip().strip('"').strip("'") for x in v.split(",") if x.strip()]


def cells(line):
    """表の行をセルに分ける(\\| はセル内の縦棒)。"""
    parts = re.split(r"(?<!\\)\|", line.strip().strip("|"))
    return [p.strip().replace("\\|", "|") for p in parts]


def section(body, title):
    """「## title」で始まる節の本文(次の ## まで)。"""
    m = re.search(r"(?m)^##\s+%s[^\n]*\n(.*?)(?=^##\s|\Z)" % re.escape(title), body, re.S)
    return m.group(1) if m else ""


def table_rows(text):
    """節の中の表の行(見出しと区切りを除く)をセルの列で返す。"""
    rows = []
    for line in text.splitlines():
        if not line.lstrip().startswith("|") or re.match(r"^\s*\|[\s:|-]+\|\s*$", line):
            continue
        rows.append(cells(line))
    return rows[1:] if rows else []


def verify_rows(body):
    """変更の記録の照合の表の行(要件の欄が空でないもの)。"""
    return [r for r in table_rows(section(body, "照合")) if r and r[0].strip()]


def git_out(root, *args):
    """git の出力。git が無い・失敗したら None。"""
    import subprocess
    try:
        r = subprocess.run(["git", "-C", str(root), "-c", "log.showSignature=false", *args],
                           capture_output=True, text=True)
    except OSError:
        return None
    return r.stdout if r.returncode == 0 else None


_CHANGED_SINCE = {}


def test_changed_since(root, commit, f):
    """TL-12: コミット commit の後(HEAD まで)にファイル f が変わったか。同じ組は覚えて、重ねて呼ばない。"""
    key = (str(root), commit, f)
    if key not in _CHANGED_SINCE:
        _CHANGED_SINCE[key] = bool((git_out(root, "log", "--format=%H", "%s..HEAD" % commit, "--", f) or "").strip())
    return _CHANGED_SINCE[key]


def checked_records(root, specs, docs):
    """TL-11・TL-12: done の変更の記録が照合で checked にした要件 → その記録の並び(古い順)。
    並びは記録が最後に入ったコミットの新しさ(git rev-list --topo-order HEAD の位置)。git に入っていない記録と
    作業ツリーで変えた記録は updated で並べ、git に入った記録より後(新しい側)に置く(コミットの前後で表が変わらないように)。
    各要素は (記録のパス, updated, コミット or None)。コミットが None の記録は W22 の起点にしない。"""
    order = git_out(root, "rev-list", "--topo-order", "HEAD")
    pos = {h: i for i, h in enumerate(order.split())} if order else {}
    found = []
    for p, (fm, body, text) in docs.items():
        if p.relative_to(specs).parts[0] != CHANGES or fm.get("status") != "done":
            continue
        ids = {r[0] for r in verify_rows(body) if len(r) > 1 and r[1] == "checked" and re.match(ID_PAT + "$", r[0])}
        if not ids:
            continue
        commit = None
        if pos:
            rel = p.relative_to(root).as_posix()
            dirty = (git_out(root, "status", "--porcelain", "--", rel) or "").strip()
            out = git_out(root, "log", "-1", "--format=%H", "--", rel)
            commit = (out or "").strip() or None
            if dirty or commit not in pos:
                commit = None
        updated = fm.get("updated", "")
        key = (0, -pos[commit], updated, p.name) if commit else (1, 0, updated, p.name)
        found.append((key, ids, (p, updated, commit)))
    out = {}
    for key, ids, item in sorted(found, key=lambda x: x[0]):
        for rid in ids:
            out.setdefault(rid, []).append(item)
    return out


def check_change(rel, fm, body, reqs, today, stale_days):
    """変更の記録(specs/_changes/)の検査。フェーズの順序・対象外の理由・照合の完了を見る。"""
    errors, warns = [], []
    status = fm.get("status", "")
    if fm.get("size", "") not in CHANGE_SIZES:
        errors.append("E23 %s: size は %s のいずれか(今: %s)" % (rel, " | ".join(sorted(CHANGE_SIZES)), fm.get("size") or "無し"))
    rows = table_rows(section(body, "フェーズ"))
    names = [r[0] for r in rows]
    if names != PHASES:
        errors.append("E23 %s: フェーズの表が %s の順の7行でない(今: %s)" % (rel, "・".join(PHASES), "・".join(names) or "無し"))
        return errors, warns
    states = {}
    for r in rows:
        name, st = r[0], (r[1] if len(r) > 1 else "")
        if st.startswith("対象外"):
            reason = re.sub(r"^対象外\s*[::]?\s*", "", st)
            if not reason:
                errors.append("E23 %s: %s が対象外なのに理由が無い(「対象外: 理由」)" % (rel, name))
            if name in NO_SKIP:
                errors.append("E23 %s: %s は対象外にできない" % (rel, name))
            states[name] = "対象外"
        elif st in ("未", "進行中", "済"):
            states[name] = st
        else:
            errors.append("E23 %s: %s の状態が語彙外(未 / 進行中 / 済 / 対象外: 理由。今: %s)" % (rel, name, st or "空"))
            states[name] = "未"
    for i, name in enumerate(PHASES):
        if states[name] in ("進行中", "済"):
            early = [n for n in PHASES[:i] if states[n] not in ("済", "対象外")]
            if early:
                errors.append("E24 %s: %s が %s なのに、前のフェーズ %s が終わっていない" % (rel, name, states[name], "・".join(early)))
    task_ids = set()
    for r in table_rows(section(body, "タスク")):
        if len(r) < 3 or not r[0].strip() or set(r[0].strip()) <= set("-: "):
            continue
        for rid in re.findall(ID_PAT, r[2]):
            task_ids.add(rid)
            if rid not in reqs:  # reqs には principles.md の原則の ID も入れてある
                errors.append("E25 %s: タスク %s が仕様に無い要件 %s を指している" % (rel, r[0], rid))
    if status == "done":
        if states["照合"] != "済":
            errors.append("E26 %s: done なのに照合が済でない" % rel)
        allrows = verify_rows(body)
        for r in allrows:
            if not re.match(ID_PAT + "$", r[0]):
                errors.append("E26 %s: 照合の表の行は要件1つずつにする(今: %s)" % (rel, r[0]))
        vrows = [r for r in allrows if re.match(ID_PAT + "$", r[0])]
        seen = {r[0] for r in vrows}
        dup = sorted({r[0] for r in vrows if [x[0] for x in vrows].count(r[0]) > 1})
        if dup:
            errors.append("E26 %s: 照合の表に同じ要件が2行以上ある(%s)" % (rel, ", ".join(dup)))
        checked = 0
        for r in vrows:
            res = r[1] if len(r) > 1 else ""
            if res not in VERIFY_RESULTS:
                errors.append("E26 %s: 照合の %s の結果が checked / gap / unassessed でない(今: %s)" % (rel, r[0], res or "空"))
            if res == "unassessed":
                errors.append("E26 %s: 照合の %s が unassessed のまま" % (rel, r[0]))
            if res == "gap" and (len(r) < 3 or not r[2].strip()):
                errors.append("E26 %s: 照合の %s が gap なのに欠けているものが書かれていない" % (rel, r[0]))
            checked += res == "checked"
            ev = r[2] if len(r) > 2 else ""
            evn = ev.replace("：", ":")
            if res == "checked" and not evn.strip().startswith("読んで判定:") and \
                    not all(k in evn for k in ("コード:", "テスト:", "今:", "前:")):
                warns.append("W19 %s: 照合の %s の証拠が型に合わない(「コード: path:line / テスト: file::name / 今: … / 前: …」か「読んで判定: 何を読んだか」)" % (rel, r[0]))
        missing = sorted(task_ids - seen)
        if missing:
            errors.append("E26 %s: タスクの要件 %s が照合の表に無い" % (rel, ", ".join(missing)))
        m = re.search(r"確かめた[::]\s*(\d+)\s*/\s*(\d+)", section(body, "照合"))
        if not m or (int(m.group(1)), int(m.group(2))) != (checked, len(vrows)):
            errors.append("E26 %s: 「確かめた: N / M」が照合の表(checked %d / %d)と合わない" % (rel, checked, len(vrows)))
    if status == "active":
        u = fm.get("updated", "")
        if DATE_RE.match(u) and (today - datetime.date.fromisoformat(u)).days >= stale_days:
            warns.append("W18 %s: active のまま %s から更新が無い" % (rel, u))
    return errors, warns


def arg_value(argv, flag, default):
    if flag not in argv:
        return default
    i = argv.index(flag)
    if i + 1 >= len(argv) or argv[i + 1].startswith("--"):
        print("E0 %s に値がありません" % flag)
        sys.exit(1)
    return argv[i + 1]


PRINCIPLE_RE = re.compile(r"^P\d+$")
TRAIL_AT = 7  # 軌跡の要約を出す件数(OM-14)
BRAKE_AT = 3  # 人の確認待ちがこの件数あるあいだは、原則と gated に触る AI 自己採択を止める(OM-7)


def _added_order(path):
    """記録がその場所に入った順の鍵: git に入った記録は、そのパスに最初に現れたコミットの深さ。入っていない記録は最後。"""
    import subprocess
    r = subprocess.run(["git", "log", "--reverse", "--format=%H", "--", path.name], cwd=path.parent,
                       capture_output=True, text=True)
    hs = r.stdout.split() if r.returncode == 0 else []
    if not hs:
        return (1, 0, path.name)
    rr = subprocess.run(["git", "rev-list", "--count", hs[0]], cwd=path.parent, capture_output=True, text=True)
    return (0, int(rr.stdout.strip() or 0), path.name)


def pending(root, specs, records, gated, need_p, since=None, done=frozenset(), orig=None):
    """人の確認待ち(OM-7): 錨との照合のある AI 自己採択で、原則か gated の要件(一度でも載ったもの)に触り、
    照合できた確認の記録(done)に載っていないもの。gated に載る前に入った記録(W24)の、その要件への触れは数えない。
    入った順に並べる。"""
    since = since or {}
    orig = orig or {}  # 入ったあとで書き換えた・移した記録は、入ったときの値で数える(OM-22)

    def counts(r):
        for x in (orig[r.path.name]["touches"] if r.path.name in orig else r.touches):
            if PRINCIPLE_RE.match(x):
                return True
            if x in gated and not (x in since and added_before(r.path, since[x])):
                return True
        return False

    def was(r, k, now):
        return orig[r.path.name][k] if r.path.name in orig else now
    out = [r for p, r in records.items() if p.parent.name == DECS and was(r, "status", r.status) == "accepted"
           and was(r, "decided-by", r.decided_by) == "AI" and r.anchor_ok(need_p) and counts(r) and p.name not in done]
    return sorted(out, key=lambda r: _added_order(r.path))


CONFIRM_LINE_RE = re.compile(r"^\s*-\s*`?(\S+?\.md)`?\s*[、,]\s*指紋\s*`?([0-9a-fA-F]{16})`?\s*$", re.M)


def confirm_check(root, specs, records):
    """OM-13: 確認の記録(specs/_bundles/ の、フロントマターに confirms を持つ記録)を照合する。
    (エラー, 警告, 照合できた確認の記録に載った決定記録の名前) を返す。

    確認できるのは、confirms の記録がすべて実在し、本文の「- <パス>、指紋 <16文字>」がその記録の今の指紋と合い、
    指した人の言葉が、確認の記録の指紋(bundle_fingerprint)を独立した語として含み、確認を示す語を人の側(打った文か
    選んだ答え)に含むときだけ。選択肢の答えでは、指紋は問いの文にあってよい。ほかの記録の承認・確認と同じ発言は使えない。
    指した会話の記録が無いとき: 会話の記録がまったく無い場所なら W17 で数えず、ほかの会話の記録があれば E33。"""
    errs, warns, done = [], [], set()
    bdir = specs / BUNDLES
    if not bdir.is_dir():
        return errs, warns, done
    others = {}
    for r in records.values():
        for ev in as_list(r.fm.get("approval-evidence", "")):
            others.setdefault(ev, set()).add(r.path.name)
    for b in sorted(bdir.glob("*.md")):
        fm0, _ = split_front(b.read_text(encoding="utf-8"))
        for ev in as_list((fm0 or {}).get("approval-evidence", "")) if fm0 and fm0.get("confirms") else []:
            others.setdefault(ev, set()).add(b.name)
    by_name = {r.path.name: r for r in records.values() if r.path.parent.name == DECS}
    have_logs = None
    for b in sorted(bdir.glob("*.md")):
        fm, body = split_front(b.read_text(encoding="utf-8"))
        if not fm or not fm.get("confirms"):
            continue
        rel = b.relative_to(root)
        names = as_list(fm.get("confirms", ""))
        listed = {Path(x).name: f.lower() for x, f in CONFIRM_LINE_RE.findall(body or "")}
        bad = ["本文の指紋の行 %s が confirms に無い" % n for n in sorted(set(listed) - {Path(x).name for x in names})]
        for x in names:
            r = by_name.get(Path(x).name)
            if r is None:
                bad.append("確認した記録 %s が _decisions/ に無い" % x)
            elif Path(x).name not in listed:
                bad.append("本文に %s の指紋の行(- <パス>、指紋 <16文字>)が無い" % x)
            elif listed[Path(x).name] != record_fingerprint(r.path):
                bad.append("%s が確認のあとで変わった(本文の指紋 %s、今 %s)" % (x, listed[Path(x).name],
                                                                record_fingerprint(r.path)))
        rec = Record(b, fm, body or "")
        st, why = verify_evidence(rec)
        if st == "missing":
            errs.append("E33 %s: 確認の記録に approval-evidence か approval-hash が無い" % rel); continue
        evs = as_list(fm.get("approval-evidence", ""))
        shared = sorted({n for ev in evs for n in others.get(ev, set())} - {b.name})
        if shared:
            errs.append("E33 %s: 確認の言葉が、ほかの記録の承認・確認と同じ発言を指している(%s)" % (rel, ", ".join(shared)))
            continue
        if bad:
            errs.append("E33 %s: %s" % (rel, "。".join(bad))); continue
        if st == "bad":
            errs.append("E33 %s: %s" % (rel, why)); continue
        if st == "unverifiable":
            if have_logs is None:
                have_logs = any_transcripts()
            if have_logs and not why.startswith("承認の場所を伏せ字"):
                errs.append("E33 %s: %s(ほかの会話の記録はあるので、存在しない会話を指している)" % (rel, why))
            else:
                warns.append("W17 %s: %s(確認を照合できず、確認待ちは減らさない)" % (rel, why))
            continue
        fpb = confirm_fingerprint(b)
        ok = False
        for ev in evs:
            m = re.match(r"^claude:([^/]+)/(.+)$", ev)
            if not m:
                continue
            s2, e = _load_entry(m.group(1), m.group(2))
            if s2 != "ok":
                continue
            for q, a in human_parts(e):
                has_fp = re.search(r"(?<![0-9a-z])%s(?![0-9a-z])" % fpb, (q + "\n" + a).lower())
                if has_fp and consents(a, choice=bool(q)):
                    ok = True
        if not ok:
            errs.append("E33 %s: 確認の言葉が、確認の記録の指紋 %s と、人の側(打った文か選んだ答え)の確認を示す語の両方を"
                        "含まない" % (rel, fpb))
            continue
        done |= {Path(x).name for x in names}
    return errs, warns, done


def trail(specs, records):
    """OM-14: どの _bundles/ の記録にもファイル名が載っていない、決着した決定記録(取り込みを除く)。
    束は _proposals/ のパスで記録を指していることがあるので、ファイル名で照合する。数えるのは軌跡の要約
    (…-trail-N.md)と旧い束(…-bundle-N.md)だけで、事前承認の文(…-approval-N.md など)は要約でない。"""
    listed = set()
    bdir = specs / BUNDLES
    if bdir.is_dir():
        for b in bdir.glob("*.md"):
            if not re.search(r"-(?:trail|bundle)-\d+\.md$", b.name):
                continue
            listed |= set(re.findall(r"[A-Za-z0-9_.-]+\.md", b.read_text(encoding="utf-8")))
    left = [r for p, r in records.items() if p.parent.name == DECS and not r.is_import
            and r.status in ("accepted", "rejected", "withdrawn") and p.name not in listed]
    left.sort(key=lambda r: (r.fm.get("trigger", "") != "人の発言", r.path.name))
    need_p, gated = principles_gate(specs)
    since = gated_since(specs) if need_p else {}
    done = confirm_check(specs.parent, specs, records)[2]
    pend = pending(specs.parent, specs, records, gated | set(since), need_p, since, done,
                   sealed_errors(specs.parent, specs)[1])
    out = []
    if pend:
        out.append("人の確認待ち(%d / %d)%s" % (len(pend), BRAKE_AT, "。ブレーキが掛かっている(原則と gated に触る AI 自己採択を止める)"
                                                       if len(pend) >= BRAKE_AT else ""))
        out += ["- %s" % r.path.relative_to(specs.parent).as_posix() for r in pend]
    out += ["| 記録 | 結果 | 決定者 | きっかけ |", "|---|---|---|---|"]
    out += ["| %s | %s | %s | %s |" % (r.path.relative_to(specs.parent).as_posix(), r.status, r.decided_by,
                                       r.fm.get("trigger", "")) for r in left]
    n = len(left)
    out.append("要約を出す時(%d / %d)" % (n, TRAIL_AT) if n >= TRAIL_AT else "まだ出さない(%d / %d)" % (n, TRAIL_AT))
    return out


def gated_since(specs):
    """gated の要件ごとに、principles.md のフロントマターの gated に初めて載ったコミット。
    git の履歴が無ければ空(さかのぼりを確かめられないので、すべての記録に当てる)。"""
    import subprocess
    p = specs / "principles.md"
    git = ["git", "-C", str(p.parent)]
    try:
        r = subprocess.run(git + ["log", "--reverse", "--format=%H", "--", p.name], capture_output=True, text=True)
    except OSError:
        return {}
    if r.returncode != 0:
        return {}
    since = {}
    for h in r.stdout.split():
        show = subprocess.run(git + ["show", "%s:./%s" % (h, p.name)], capture_output=True, text=True)
        if show.returncode != 0:
            continue
        fm, _ = split_front(show.stdout)
        for rid in as_list(fm.get("gated", "")):
            since.setdefault(rid, h)
    return since


def legacy_boundary(specs):
    """照合の仕組みの境(OM-12): _decisions/ の履歴で、照合した approval-evidence(claude:…)の行が初めて現れたコミット。
    今の記録から外しても境は動かない。伏せ字の approval-evidence は数えない。無い・git が無ければ None。"""
    import subprocess
    d = specs / DECS
    if not d.is_dir():
        return None
    try:
        r = subprocess.run(["git", "log", "--reverse", "--format=%H", "-G", "^approval-evidence: *claude:", "--", "."],
                           cwd=d, capture_output=True, text=True)
    except OSError:
        return None
    hs = r.stdout.split() if r.returncode == 0 else []
    return hs[0] if hs else None


def frozen_before(path, commit):
    """記録のすべての版が commit の真の祖先のコミットで入り、作業ツリーでも変わっていないか(W20)。
    境のあとで書き換えた・消して同じ名前で入れ直した記録は、前の承認に数えない。"""
    import subprocess
    try:
        r = subprocess.run(["git", "log", "--format=%H", "--", path.name], cwd=path.parent, capture_output=True, text=True)
        hs = r.stdout.split() if r.returncode == 0 else []
        if not hs:
            return False
        for h in hs:
            if h == commit or subprocess.run(["git", "merge-base", "--is-ancestor", h, commit], cwd=path.parent,
                                             capture_output=True).returncode != 0:
                return False
        return subprocess.run(["git", "diff", "--quiet", "HEAD", "--", path.name], cwd=path.parent,
                              capture_output=True).returncode == 0
    except OSError:
        return False


def added_before(path, commit):
    """記録が git に入ったコミットが、commit の真の祖先か(W24)。時刻は書き換えられるので、親子の関係で見る。"""
    import subprocess
    r = subprocess.run(["git", "log", "--diff-filter=A", "--format=%H", "--", path.name],
                       cwd=path.parent, capture_output=True, text=True)
    hs = r.stdout.split() if r.returncode == 0 else []
    if not hs or hs[-1] == commit:
        return False
    return subprocess.run(["git", "merge-base", "--is-ancestor", hs[-1], commit], cwd=path.parent,
                          capture_output=True).returncode == 0


def principles_gate(specs):
    """(principles.md があるか, 事前承認の要件の集合)。一覧は principles.md のフロントマター gated(OM-13)。"""
    p = specs / "principles.md"
    if not p.is_file():
        return False, set()
    fm, _ = split_front(p.read_text(encoding="utf-8"))
    return True, set(as_list(fm.get("gated", "")))


class Record:
    """_proposals/ と _decisions/ の1ファイル。"""

    def __init__(self, path, fm, body):
        self.path, self.fm, self.body = path, fm, body
        self.legacy = "decided-by" not in fm
        self.status = fm.get("status", "")
        self.created = fm.get("created", "")
        self.decided_by = fm.get("decided-by", "")
        if self.legacy:
            makers = fm.get("decision-makers", "")
            self.decided_by = "人" if "【人】" in makers or "人" in as_list(makers) else "AI"
        self.is_import = self.decided_by == "取り込み"
        self.touches = as_list(fm.get("touches", ""))
        if self.legacy:  # 旧形式は「対象」の行と差分の節に出てくる ID を触る要件とみなす
            found = re.findall(ID_PAT, section(body, "差分") + "\n".join(
                l for l in body.splitlines() if "**対象**" in l))
            self.touches = sorted(set(found))
        self.import_rows = {}
        self.rewritten = False  # 仕様の文言と元の文言を分けた(書き直した)取り込み
        if self.is_import:
            for line in section(body, "要件").splitlines():
                if REQ_ROW_RE.match(line):
                    c = cells(line)
                    if len(c) >= 3:
                        self.import_rows[c[0]] = (c[1], c[2])
                    if len(c) >= 5 and c[2] != c[3]:
                        self.rewritten = True
        self.diffs = []
        for line in section(body, "差分").splitlines():
            m = DIFF_RE.match(line.strip())
            if m:
                self.diffs.append((m.group(1), m.group(2), m.group(3)))

    def protects(self, rid):
        if self.status != "accepted":
            return False
        if self.is_import:
            row = self.import_rows.get(rid)
            return bool(row) and row[0] in ("人", "未確認")
        return self.decided_by == "人" and rid in self.touches

    def anchor_ok(self, need_p):
        """OM-6: 「錨との照合」の節が空・雛形のままでなく、principles.md のあるリポでは P 番号を挙げている。"""
        text = re.sub(r"<!--.*?-->", "", section(self.body, "錨との照合"), flags=re.S).strip()
        if not text or re.fullmatch(r"\{[^{}]*\}", text):
            return False
        return not need_p or bool(re.search(r"(?<![A-Za-z0-9-])P\d+(?!\d)", text))

    def may_change_locked(self, need_p):
        """TL-5・TL-6: 錠を掛け直し・外してよい決定の記録(人の決定か、錨との照合を持つ AI 自己採択)。"""
        return self.status == "accepted" and (
            self.decided_by == "人" or (self.decided_by == "AI" and self.anchor_ok(need_p)))


def vendored_errors(root):
    """E30: spec-init がコピーした道具と入口が、.decidespec/lock.json の指紋のままか。載っていないファイルが
    .decidespec/ と入口の置き場に足されていないか(TS-5)。指紋は改行を LF にそろえた中身の SHA-256。"""
    import hashlib
    import json
    lock = root / ".decidespec" / "lock.json"
    if not lock.is_file():
        return []
    try:
        data = json.loads(lock.read_text(encoding="utf-8"))
        files = data.get("files", {})
        plugin = data.get("plugin", "decidespec")
        assert isinstance(files, dict)
    except (ValueError, AttributeError, AssertionError, UnicodeDecodeError):
        return ["E30 .decidespec/lock.json: 読めない(spec-init を走らせ直す)"]
    out = []
    for rel, h in sorted(files.items()):
        path = root / rel
        parts = Path(rel).parts
        if ".." in parts or Path(rel).is_absolute() or not (
                (parts[:1] == (".decidespec",) and len(parts) > 1) or
                (len(parts) > 3 and "/".join(parts[:2]) in (".claude/skills", ".agents/skills")
                 and parts[2].startswith(plugin + "-"))):
            out.append("E30 %s: lock.json が道具の置き場の外を指している" % rel)
        elif not path.is_file():
            out.append("E30 %s: lock.json に載っているのに無い(spec-init を走らせ直す)" % rel)
        elif hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest() != h:
            out.append("E30 %s: lock.json の指紋と中身が違う(手で直さない。プラグイン側で直して spec-init で更新する)" % rel)
    tops = [root / ".decidespec"] + [d for base in (".claude/skills", ".agents/skills")
                                    for d in sorted((root / base).glob(plugin + "-*")) if d.is_dir()]
    for top in tops:
        for p in sorted(top.rglob("*")):
            rel = p.relative_to(root).as_posix()
            if p.is_file() and rel not in files and rel != ".decidespec/lock.json" \
                    and "__pycache__" not in p.parts and p.suffix != ".pyc" and p.name != ".DS_Store":
                out.append("E30 %s: lock.json に載っていないファイルが道具の置き場にある(道具の外に移す)" % rel)
    return out


SELF = Path(__file__).resolve()


def main(argv):
    for stream in (sys.stdout, sys.stderr):  # Windows の cp1252 などでも日本語を出せるように
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")
    root = Path(argv[1]) if len(argv) > 1 and not argv[1].startswith("--") else Path(".")
    specs = root / arg_value(argv, "--specs", "specs")
    stale_days = int(arg_value(argv, "--stale-days", DEFAULT_STALE_DAYS))
    write = "--write" in argv
    today = datetime.date.today()
    errors, warns, ids = [], [], {}

    if "--fingerprint" in argv:
        target = root / arg_value(argv, "--fingerprint", "")
        if not target.is_file():
            print("E0 %s が無い" % target)
            return 1
        # 決定記録はファイル全体の指紋(確認の記録の本文に並べる)、承認の文・確認の記録は本文の指紋
        if target.parent.name == DECS:
            print(record_fingerprint(target))
        else:
            fm, _ = split_front(target.read_text(encoding="utf-8"))
            print(confirm_fingerprint(target) if fm and fm.get("confirms") else bundle_fingerprint(target))
        return 0
    if not specs.is_dir():
        print("E0 %s が無い" % specs)
        return 1
    if "--settle" in argv:
        i = argv.index("--settle")
        if i + 2 >= len(argv):
            print("E0 --settle <提案> accepted|rejected|withdrawn")
            return 1
        code, msg = settle(root, specs, root / argv[i + 1], argv[i + 2])
        print(msg)
        return code
    if (specs / "changes").is_dir():
        print("E0 %s/changes/ がある(旧い形)。%s/ と %s/ の形に移してから検査する" % (specs, PROPS, DECS))
        return 1
    old = [n for n in ("proposals", "decisions") if (specs / n).is_dir()]
    if old:
        print("E0 %s に旧い名前のフォルダがある(%s)。%s/ と %s/ に名前を変えてから検査する"
              % (specs, ", ".join(old), PROPS, DECS))
        return 1
    readme = specs / "README.md"
    if not readme.exists():
        errors.append("E0 specs/README.md(索引)が無い")

    errors += vendored_errors(root)

    md_files = sorted(specs.rglob("*.md"))
    docs, records, spec_files = {}, {}, []
    for p in md_files:
        rel = p.relative_to(root)
        text = p.read_text(encoding="utf-8")
        fm, body = split_front(text)
        if p.name in EXEMPT_NAMES and p != readme:
            continue
        if fm is None:
            if p != readme:
                errors.append("E1 %s: フロントマターが無い" % rel)
            docs[p] = ({}, body, text)
            continue
        docs[p] = (fm, body, text)
        for key in ("type", "id"):
            if not fm.get(key):
                errors.append("E2 %s: %s が無い" % (rel, key))
        i = fm.get("id", "")
        if i and not ULID_RE.match(i):
            errors.append("E3 %s: id が ULID ではない(%s)" % (rel, i))
        if i:
            if i in ids:
                errors.append("E4 %s: id が %s と重複" % (rel, ids[i].relative_to(root)))
            ids[i] = p
        c, u = fm.get("created", ""), fm.get("updated", "")
        if DATE_RE.match(c) and DATE_RE.match(u) and u < c:
            errors.append("E5 %s: updated(%s) < created(%s)" % (rel, u, c))

        top = p.relative_to(specs).parts[0]
        st = fm.get("status", "")
        if top in (PROPS, DECS):
            rec = Record(p, fm, body)
            records[p] = rec
            want = {"proposed"} if top == PROPS else DECIDED_STATUS
            if st not in want:
                errors.append("E6 %s: %s/ の status は %s(今: %s)"
                              % (rel, top, " | ".join(sorted(want)), st or "無し"))
        elif top == CHANGES:
            if st not in CHANGE_STATUS:
                errors.append("E6 %s: %s/ の status は %s(今: %s)"
                              % (rel, top, " | ".join(sorted(CHANGE_STATUS)), st or "無し"))
        elif st and st not in SPEC_STATUS:
            errors.append("E6 %s: status は %s のいずれか(今: %s)" % (rel, " | ".join(sorted(SPEC_STATUS)), st))
        if p.name == "spec.md":
            spec_files.append(p)
        if p.name == "principles.md":
            for n, line in enumerate(strip_code(text).splitlines(), 1):
                if "【人】" in line and not line.lstrip().startswith(">") and not REQ_ROW_RE.match(line):
                    warns.append("W11 %s:%d: 本文に【人】の印がある(区別は決定記録に置く)" % (rel, n))

    # --- 全ファイル共通: 穴埋めの残骸・リンク切れ ---
    for p, (fm, body, text) in docs.items():
        rel = p.relative_to(root)
        prose = strip_code(text)
        left = sorted({m.group(0) for m in PLACEHOLDER_RE.finditer(prose)})
        if left:
            errors.append("E8 %s: 雛形の穴埋めが残っている(%s)" % (rel, ", ".join(left[:5])))
        for m in MD_LINK_RE.finditer(prose):
            t = m.group(1).split("#")[0]
            if not t or t.startswith(("http://", "https://", "mailto:")) or CROSS_REPO_RE.match(t):
                continue
            if not ((p.parent / t).exists() or (root / t).exists()):
                errors.append("E11 %s: 参照が実在しない(%s)" % (rel, t))

    # --- 要件表を読む ---
    reqs = {}  # rid -> dict(text, check, links, spec, line)
    for p in spec_files:
        rel = p.relative_to(root)
        fm, body, text = docs[p]
        n_req = 0
        # 生成区画(この機能の決定・つながりの表)の行は要件の行として読まない(行番号は保つ)
        own = H_RE.sub(lambda m: "\n" * m.group(0).count("\n"), text)
        for n, line in enumerate(own.splitlines(), 1):
            m = REQ_ROW_RE.match(line)
            if not m:
                if "【人】" in line and not line.lstrip().startswith(">") and not REQ_ROW_RE.match(line):
                    warns.append("W11 %s:%d: 本文に【人】の印がある(区別は決定記録に置く)" % (rel, n))
                continue
            n_req += 1
            rid = m.group(1)
            if rid in reqs:
                errors.append("E7 %s:%d: 要件 ID %s が重複(%s にもある)" % (rel, n, rid, reqs[rid]["spec"]))
                continue
            c = cells(line)
            if len(c) != 4:
                errors.append("E9 %s:%d: %s の行が4列(ID / 要件 / 確かめ方 / 決定)でない" % (rel, n, rid))
                continue
            links = [(p.parent / lm.group(1).split("#")[0]).resolve() for lm in MD_LINK_RE.finditer(c[3])]
            reqs[rid] = dict(text=c[1], check=c[2], links=links, spec=rel, line=n)
            if not c[2] or c[2] in ("—", "-", "ー", "–"):
                warns.append("W4 %s:%d: %s の確かめ方が空(書けないなら「読んで判定」)" % (rel, n, rid))
            dec = [t for t in links if t.parent == (specs / DECS).resolve() and t.exists()]
            if not dec:
                errors.append("E10 %s:%d: %s の決定の列に _decisions/ への実在するリンクが無い" % (rel, n, rid))
        if n_req == 0:
            warns.append("W5 %s: 要件が1行も無い" % rel)
        elif n_req > MAX_REQS:
            warns.append("W3 %s: 要件が %d(目安 %d まで。分割を検討)" % (rel, n_req, MAX_REQS))

    by_path = {p.resolve(): r for p, r in records.items()}

    need_p, gated = principles_gate(specs)
    since = gated_since(specs) if need_p else {}
    ever = gated | set(since)  # 一度でも gated に載った要件(外す提案を含む。OM-7)
    known = set(reqs)
    if (specs / "principles.md").exists():
        known |= {m.group(1) for line in (specs / "principles.md").read_text(encoding="utf-8").splitlines()
                  for m in [REQ_ROW_RE.match(line)] if m}
    for p, (fm, body, text) in docs.items():
        if p.relative_to(specs).parts[0] == CHANGES and p.suffix == ".md":
            e, w = check_change(p.relative_to(root), fm, body, known, today, stale_days)
            if VERBATIM_RE.search(strip_code(body)):
                e.append("E27 %s: 人の発言の原文の引用がある(要約にし、元は会話の記録の場所で指す)" % p.relative_to(root))
            if human_sentences(strip_code(body)):
                e.append("E28 %s: 会話の記録の人の発言と一致する文がある(要約に書き直す)" % p.relative_to(root))
            errors += e
            warns += w

    def latest(rid):
        links = reqs[rid]["links"]
        return by_path.get(links[0]) if links else None

    # --- 要件ごと: 取り込みの文言の照合・書き方の警告 ---
    for rid, r in reqs.items():
        lr = latest(rid)
        where = "%s:%d" % (r["spec"], r["line"])
        if lr and lr.is_import:
            row = lr.import_rows.get(rid)
            if row and row[1] != r["text"]:
                errors.append("E17 %s: %s は取り込んだ文言から変わっている(変えるなら提案で)" % (where, rid))
            continue
        if not STRENGTH_RE.search(strength_tail(r["text"])):
            warns.append("W8 %s: %s の文末が強さの5語でない" % (where, rid))
        vague = [w for w in VAGUE if w in r["text"]]
        if vague:
            warns.append("W9 %s: %s に曖昧語(%s)" % (where, rid, "・".join(vague)))

    ce, cw, done = confirm_check(root, specs, records)
    errors += ce
    warns += cw
    sealed, orig = sealed_errors(root, specs)
    pend = pending(root, specs, records, ever, need_p, since, done, orig)
    brake_over = {r.path.name for r in pend[BRAKE_AT:]}  # 確認待ちの4件目から(OM-7)
    no_logs = bool(brake_over) and not any_transcripts()  # 会話の記録がまったく無い場所では W25(OM-13)
    boundary = None
    # --- 記録ごとの検査 ---
    known_fps = {fp.lower() for r in records.values() for _, fp in APPROVAL_TEXT_RE.findall(section(r.body, "承認の記録"))}
    for p, rec in records.items():
        rel = p.relative_to(root)
        fm = rec.fm
        in_props = p.relative_to(specs).parts[0] == PROPS
        if rec.legacy:
            if in_props:
                warns.append("W10 %s: 旧形式の提案(decided-by が無い)。書き直すか取り下げる" % rel)
            elif rec.decided_by == "人" and rec.status == "accepted":
                if boundary is None:
                    boundary = legacy_boundary(specs) or ""
                # 境が無い(git が無い・照合した人の決定がまだ無い・公開の書き出し)リポでは確かめない
                if boundary and not added_before(rec.path, boundary):
                    errors.append("E32 %s: decided-by の無い旧い形の記録が、照合の仕組みの境より前に git に入っていない"
                                  "(人の決定に数えない。decided-by を書いて --approve で照合する。OM-23)" % rel)
            continue
        if not rec.is_import:
            if rec.decided_by not in DECIDED_BY:
                errors.append("E12 %s: decided-by は 人 | AI | 取り込み(今: %s)" % (rel, rec.decided_by or "無し"))
            if fm.get("trigger", "") not in TRIGGERS:
                errors.append("E12 %s: trigger は %s のいずれか(今: %s)"
                              % (rel, " / ".join(TRIGGERS), fm.get("trigger", "") or "無し"))
            tl = fm.get("trigger-link", "")
            tpath = tl.split("#")[0]
            if not tl:
                errors.append("E12 %s: trigger-link が無い" % rel)
            elif not (CONVO_RE.match(tl) or tl.startswith(("http://", "https://")) or CROSS_REPO_RE.match(tl)
                      or (p.parent / tpath).exists() or (root / tpath).exists()):
                errors.append("E12 %s: trigger-link が実在の場所か「会話 YYYY-MM-DD」でない(%s)" % (rel, tl))
        if not rec.touches:
            errors.append("E12 %s: touches が空" % rel)
        if rec.is_import and rec.rewritten and not section(rec.body, "照合").strip():
            warns.append("W14 %s: 書き直した取り込みに「## 照合」(書込なしの検証役の照合の結果)が無い" % rel)
        if not rec.is_import:
            diff_ids = {d[1] for d in rec.diffs}
            if diff_ids != set(rec.touches):
                errors.append("E13 %s: touches %s と差分の ID %s が一致しない"
                              % (rel, sorted(rec.touches), sorted(diff_ids)))
        appr = section(rec.body, "承認の記録")
        old_approval = REDACTED in appr and not rec.fm.get("approval-evidence")
        if rec.decided_by == "人" and rec.status == "accepted" \
                and not APPROVAL_LINE_RE.search(appr) and REDACTED not in appr:
            errors.append("E14 %s: decided-by: 人 なのに承認の記録に「YYYY-MM-DD 人の承認(要約: …)」の行が無い" % rel)
        if VERBATIM_RE.search(strip_code(rec.body.replace(section(rec.body, "差分"), ""))):  # 差分は要件の文言の引用
            errors.append("E27 %s: 人の発言の原文の引用がある(人「…」・> 【人】。要約にし、元は会話の記録の場所で指す)" % rel)
        if human_sentences(strip_code(rec.body.replace(section(rec.body, "差分"), ""))):
            errors.append("E28 %s: 会話の記録の人の発言と一致する文がある(要約に書き直す)" % rel)
        if rec.decided_by == "人" and rec.status == "accepted" and old_approval:
            if boundary is None:
                boundary = legacy_boundary(specs) or ""
            if boundary and frozen_before(rec.path, boundary):
                warns.append("W20 %s: 照合の仕組みより前の人の承認(伏せ字、照合なし)" % rel)
            else:
                errors.append("E32 %s: 伏せ字の承認が、照合の仕組み(照合した approval-evidence が初めて _decisions/ に"
                              "入ったコミット)より前に git に入ったままの記録でない(--approve %s --find <語句> で照合する。OM-12)"
                              % (rel, rel))
        elif rec.decided_by == "人" and rec.status == "accepted" and not rec.legacy:
            state, why = verify_evidence(rec)
            if state == "missing":
                errors.append("E32 %s: 人の承認が会話の記録で照合されていない(--approve %s --find <語句>、または --rehash %s)"
                             % (rel, rel, rel))
            elif state == "unverifiable":
                warns.append("W17 %s: %s(照合できない)" % (rel, why))
            elif state == "bad":
                errors.append("E22 %s: %s" % (rel, why))
        apsec = section(rec.body, "承認の記録")
        found = APPROVAL_TEXT_RE.findall(apsec)
        if not found and re.search(r"承認の文|指紋\s*`?[0-9a-fA-F]{6}", apsec):
            errors.append("E29 %s: 承認の記録の「承認の文 <パス>、指紋 <16文字>」が読めない(形を直す。OM-13)" % rel)
        for path, fp in found:
            fp = fp.lower()
            target = root / path
            got = bundle_digest(target)[:len(fp)] if target.exists() else None
            if got != fp:
                errors.append("E29 %s: 承認の文 %s の指紋が %s でない(今: %s。承認のあとで文が変わったか、指紋が違う。OM-13)"
                              % (rel, path, fp, got or "文が無い"))
                continue
            first = approved_fingerprint(root, rec.path, path)
            if first and first != fp:
                errors.append("E29 %s: 承認の記録の指紋が、承認したときの %s から %s に付け直されている(OM-13)" % (rel, first, fp))
                continue
            hist = [(h[:len(fp)], red) for h, red in fingerprint_history(root, target)]
            seq = [h for h, _ in hist]
            # 承認した版(記録の指紋と一致する最初の版)から後と、今の作業ツリーの版だけを比べる(承認の前の下書きの直しは数えない)
            after = hist[seq.index(fp):] if fp in seq else hist
            changed = [h for h, red in after if h != fp and not red]
            redacted = [h for h, red in after if h != fp and red]
            if changed or (seq and got != seq[-1] and got != fp):
                errors.append("E29 %s: 承認の文 %s が承認のあとで書き換えられている(指紋 %s。OM-13)"
                              % (rel, path, " → ".join(dict.fromkeys([fp] + changed + [got]))))
                continue
            if redacted:
                warns.append("W17 %s: 承認の文 %s が承認のあとで伏せ字にされ、指紋を照合できない(OM-17)" % (rel, path))
            texts, pairs = [], []
            for ev in as_list(rec.fm.get("approval-evidence", "")):
                m = re.match(r"^claude:([^/]+)/(.+)$", ev)
                if m:
                    st, text, when = _load_human(m.group(1), m.group(2), with_time=True)
                    if st == "ok":
                        texts.append((text.lower(), when))
                        e = _load_entry(m.group(1), m.group(2))[1]
                        pairs.append(human_parts(e) if e else [("", text)])
            touched = set(x.lower() for x in rec.touches)

            def approves(prs, when):
                # 指紋は前後に英数字の続かない独立した語のときだけ。8文字は、この版より前の発言(前の承認)だけ。
                # ID は承認を示す語と同じ発言にあるときだけ。選択肢の答えは問いと答えの組ごとに見て、承認を示す語は
                # 選んだ答えの側に要る。人の側が打ち消していれば数えない(OM-13)
                for q, a in prs:
                    whole = (q + "\n" + a).lower()
                    if NEG_RE.search(a.lower()):
                        continue
                    if re.search(r"(?<![0-9a-z])%s(?![0-9a-z])" % fp, whole) and \
                            (len(fp) == FP_LEN or (when and when < LEGACY_FP_BEFORE)) and (not q or consents(a, choice=True)):
                        return True
                    if touched & set(re.findall(ID_PAT.lower(), whole)) and consents(a, choice=bool(q)):
                        return True
                return False
            if texts and not any(approves(pr, w) for (_, w), pr in zip(texts, pairs)):
                errors.append("E29 %s: 承認した人の発言が、独立した語としての指紋 %s も、対象の要件 ID と承認を示す語も含まない(OM-13)"
                              % (rel, fp))
            if not texts and rec.status == "accepted" and rec.decided_by == "人":
                warns.append("W23 %s: 会話の記録が読めず、人の発言が指紋 %s を含むかの照合を飛ばした(OM-18)" % (rel, fp))
            same = {bundle_digest(target)[:8], bundle_digest(target)[:FP_LEN]}  # 同じ文の8文字・16文字の指紋
            for t, _ in texts:
                for other in set(re.findall(r"(?<![0-9a-z])(?:[0-9a-f]{16}|[0-9a-f]{8})(?![0-9a-z])", t)) - {fp} - same - known_fps:
                    if other.isdigit():
                        continue  # 日付などの8桁の数字
                    errors.append("E29 %s: 承認した人の発言の16進の語 %s が、承認の文の指紋 %s と違う(OM-13)" % (rel, other, fp))
        if rec.status == "accepted" and rec.decided_by == "AI":
            anchored = rec.anchor_ok(need_p)
            for rid in rec.touches:
                if rid in ever and rid in since and added_before(rec.path, since[rid]):
                    warns.append("W24 %s: %s が gated に載る前に入った AI 自己採択。さかのぼって止めない(OM-13)" % (rel, rid))
                    continue
                if (PRINCIPLE_RE.match(rid) or rid in ever) and anchored:
                    if rec.path.name in brake_over and no_logs:
                        warns.append("W25 %s: 人の確認待ちが3件あるあいだの、原則か gated の %s に触る AI 自己採択。会話の記録が"
                                     "まったく無い場所なので、確認を照合できず警告にとどめる(OM-7・OM-13)" % (rel, rid))
                    elif rec.path.name in brake_over:
                        errors.append("E15 %s: 人の確認待ちが3件あるあいだの、原則か gated の %s に触る AI 自己採択(OM-7)" % (rel, rid))
                    continue
                if PRINCIPLE_RE.match(rid) or rid in ever:
                    errors.append("E15 %s: AI 自己採択で、原則か gated の要件 %s に、錨との照合の節なしで触っている"
                                  "(錨との照合の節を置く。OM-6・OM-7)" % (rel, rid))
                    continue
                if anchored:
                    continue  # OM-6: 錨との照合があれば、守られる要件も AI が変えてよい
                others = [o for o in records.values() if o is not rec and o.protects(rid)]
                guards = [o for o in others if earlier_or_same(o, rec)]
                unknown = [o for o in others if earlier_or_same(o, rec) is None]
                if guards:
                    errors.append("E15 %s: AI 自己採択で守られた要件 %s に触り、「錨との照合」の節が無いか足りない(%s。OM-6)"
                                  % (rel, rid, guards[0].path.relative_to(root)))
                elif unknown:
                    warns.append("W21 %s: 同じ日・同じコミットの人の決定 %s と前後が分からず、%s が守られていたかを確かめられない"
                                 "(履歴のあるリポで確かめる)" % (rel, unknown[0].path.relative_to(root), rid))
        if rec.status == "accepted" and not rec.is_import and fm.get("trigger", "") not in STRONG_TRIGGERS:
            for kind, rid, rest in rec.diffs:
                m = first_quote(rest)
                if m is None:  # 閉じない「」は、前の読み方(行の最後の」まで)で見逃さない
                    g = re.search(r"「(.*)」", rest)
                    m = g.group(1) if g else None
                on = OLD_NEW_RE.search(rest)
                if kind == "変更" and on and is_must(on.group(2)) and not is_must(on.group(1)):
                    errors.append("E16 %s: %s を しなければならない 等に強めたが trigger が %s"
                                  "(予防だけの規則は するべき 以下)" % (rel, rid, fm.get("trigger")))
                if kind == "追加" and m is not None and is_must(m):
                    errors.append("E16 %s: %s を しなければならない 等で追加したが trigger が %s"
                                  "(予防だけの規則は するべき 以下で足す)" % (rel, rid, fm.get("trigger")))
        if in_props:
            c = rec.created
            if DATE_RE.match(c) and (today - datetime.date.fromisoformat(c)).days >= stale_days:
                warns.append("W6 %s: %d 日 proposed のまま" % (rel, (today - datetime.date.fromisoformat(c)).days))
            for kind, rid, rest in rec.diffs:
                m = OLD_NEW_RE.search(rest)
                if kind == "変更" and m and rid in reqs and m.group(1) != reqs[rid]["text"]:
                    warns.append("W7 %s: %s の旧の引用が現行の文言と違う(先に入った変更がある)" % (rel, rid))

    # --- テストとのつながり ---
    tests = find_tests(root)
    tested = set().union(*tests.values()) if tests else set()
    for rel, ids in sorted(tests.items()):
        for rid in sorted(ids - set(reqs)):
            errors.append("E18 %s: 仕様に無い要件 %s を指している" % (rel, rid))
    if tests:
        for rid, r in reqs.items():
            lr = latest(rid)
            if rid in tested or (lr and lr.is_import) or "読んで判定" in r["check"] or "対象外" in r["check"]:
                continue
            warns.append("W12 %s:%d: %s を指すテストが無い([%s] か test_%s をテストに書く)"
                         % (r["spec"], r["line"], rid, rid, rid.replace("-", "_")))
    protected_ids = {rid for rid in reqs if any(o.protects(rid) for o in records.values())}
    lock_path = specs / "test-locks.json"
    locks = load_locks(lock_path)
    units = test_units(root, tests)
    want = {key: sorted(ids & protected_ids) for key, (ids, sha) in units.items()
            if ids & protected_ids and sha != "SYNTAX-ERROR"}
    for key, (ids, sha) in sorted(units.items()):
        if sha == "SYNTAX-ERROR":
            errors.append("E21 %s: テストが構文エラーで読めない" % key)

    # --- 照合の証拠の古さ(TL-12): 要件を checked にした最後の記録のコミットの後に、その要件のテストのファイルが変わった ---
    checked_by = checked_records(root, specs, docs)
    unit_keys = {rid: sorted(k for k, (ids, sha) in units.items() if rid in ids) for rid in reqs}
    for rid, r in reqs.items():
        # 起点は git に入っていて作業ツリーで変えていない記録だけ。どれか1件の後でテストのファイルが変わっていなければ出さない
        # (マージで古い記録の側の枝だけが変わったときの誤検知を避ける)。
        bases = [x for x in checked_by.get(rid, []) if x[2]]
        if not bases:
            continue
        p = bases[-1][0]
        for f in sorted({k.split("::")[0] for k in unit_keys[rid]}):
            if all(test_changed_since(root, c, f) for _, _, c in bases):
                warns.append("W22 %s:%d: %s を checked にした %s の後に、テスト %s が変わった"
                             "(照合し直すか、新しい変更の記録で確かめる)" % (r["spec"], r["line"], rid, p.relative_to(root), f))
    if "--lock" in argv:
        by = arg_value(argv, "--relock", "")
        rec = records.get((root / by).resolve()) if by else None
        if by:
            by_rec = next((r for p, r in records.items() if p.resolve() == (root / by).resolve()), None)
            if not by_rec or not by_rec.may_change_locked(need_p):
                print("E0 --relock には、採択済みの人の決定か、「錨との照合」の節を持つ AI 自己採択の記録を指定する(%s。TL-5)" % by)
                return 1
        before = {k: dict(v) for k, v in locks.items()}
        moved = migrate_file_locks(root, locks, units, protected_ids)
        # TL-10: git が無視するファイルを指す錠は、そのリポのテストでないので外す。
        for key in ignored_lock_keys(root, locks):
            del locks[key]
        # TL-13: 前の取り方の指紋のままの錠は、中身が同じなので新しい指紋へ掛け直す(人の決定は要らない)。
        relocked = []
        for key, entry in locks.items():
            if key in units and entry.get("sha") != units[key][1] and entry.get("sha") == _LEGACY.get(key):
                locks[key] = dict(entry, sha=units[key][1], legacy=entry["sha"])
                relocked.append(key)
        added, changed = [], []
        for key, ids in sorted(want.items()):
            sha = units[key][1]
            if key not in locks:
                locks[key] = {"ids": ids, "sha": sha}
                added.append(key)
            elif locks[key]["sha"] != sha and by and set(locks[key]["ids"]) <= set(by_rec.touches):
                locks[key] = {"ids": ids, "sha": sha, "by": by}
                changed.append(key)
        if by:
            for key in [k for k in locks if k not in units]:
                if set(locks[key]["ids"]) <= set(by_rec.touches):
                    del locks[key]
                    changed.append(key + "(外した)")
        if by:
            # TL-15: 決定の記録が消していない要件の、錠のあるテストを1つも残さない掛け直しは拒む
            # 削除と数えるのは、差分に「削除」があり、仕様からも消えている要件だけ(「削除」と書くだけで外さない)
            deleted = {rid for kind, rid, _ in by_rec.diffs if kind == "削除" and rid not in reqs}
            lost = sorted(rid for rid in set(by_rec.touches) - deleted
                          if any(rid in e.get("ids", []) for e in before.values())
                          and not any(rid in e.get("ids", []) for e in locks.values()))
            if lost:
                print("E0 --relock で、%s の錠のあるテストが1つも残らない(テストをまとめるなら移し先を残す。"
                      "要件を消すなら差分に「削除」を書く。TL-15)。錠は書き換えなかった" % ", ".join(lost))
                return 1
        import json
        lock_path.write_text(json.dumps(locks, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print("錠: 足した %d、決定の記録で変えた %d、関数ごとに移した %d、新しい指紋へ掛け直した %d(%s)"
              % (len(added), len(changed), len(moved), len(relocked), lock_path.relative_to(root)))
    skip = ignored_lock_keys(root, locks)
    for key, entry in sorted(locks.items()):
        if key in skip:
            continue
        cur = units.get(key)
        if cur and entry.get("sha") == _LEGACY.get(key):
            errors.append("E19 %s: 錠が前の取り方の指紋のまま(中身は同じ。--lock で新しい指紋へ掛け直す。TL-13)" % key)
        elif not cur or cur[1] != entry.get("sha"):
            errors.append("E19 %s: 守られる要件 %s のテストが変わった(人の決定か、錨との照合のある提案で変え、--lock --relock <決定記録>)"
                          % (key, ", ".join(entry.get("ids", []))))
    for key, ids in sorted(want.items()):
        if key not in locks:
            warns.append("W13 %s: 守られる要件 %s のテストに錠が無い(テストを足して落ちるのを確かめたコミットで --lock)"
                         % (key, ", ".join(ids)))
    for msg in lock_history_errors(root, specs, lock_path, records):
        errors.append(msg)
    errors += principles_history_errors(root, specs, records)
    errors += [x for x in sealed if x.startswith("E")]
    warns += [x for x in sealed if x.startswith("W")]

    if len(spec_files) > MAX_CAPABILITIES:
        warns.append("W2 ケイパビリティが %d 本(目安 %d 本まで)" % (len(spec_files), MAX_CAPABILITIES))

    if "--approve" in argv:
        target = root / arg_value(argv, "--approve", "")
        phrases = [argv[i + 1] for i, a in enumerate(argv[:-1]) if a == "--find"]
        print(approve(target, records, phrases))
        return 0
    if "--rehash" in argv:
        print(rehash(root / arg_value(argv, "--rehash", "")))
        return 0
    if "--trail" in argv:
        for line in trail(specs, records):
            print(line)
        return 0
    if "--tests" in argv:
        units = test_units(root, find_tests(root))
        for rid in [x.strip() for x in arg_value(argv, "--tests", "").split(",") if x.strip()]:
            mine = sorted(k for k, (ids, _) in units.items() if rid in ids)
            print("%s: %s" % (rid, "%d 件" % len(mine) if mine else "なし"))
            for k in mine:
                print(k)
        return 0
    if "--overlap" in argv:
        rids = [x.strip() for x in arg_value(argv, "--overlap", "").split(",") if x.strip()]
        for line in overlap(root, rids, arg_value(argv, "--test-cmd", "")):
            print(line)
        return 0
    if "--evidence" in argv:
        for line in evidence(root, specs, root / arg_value(argv, "--evidence", ""), reqs, arg_value(argv, "--test-cmd", "")):
            print(line)
        return 0

    # --- spec.md ごとの「この機能の決定」 ---
    for sp in spec_files:
        mine = {rid for rid, r in reqs.items() if r["spec"] == sp.relative_to(root)}
        mine |= set(re.findall(r"\b(%s) は \d{4}-\d{2}-\d{2} に削除" % ID_PAT, docs[sp][2]))
        trace = []
        for rid, r in sorted(((rid, r) for rid, r in reqs.items() if r["spec"] == sp.relative_to(root)),
                             key=lambda x: x[1]["line"]):
            lr = latest(rid)
            dec = "[%s](%s)" % (lr.created, rel_link(lr.path, sp)) if lr else "なし"
            ks = unit_keys[rid]
            t = ", ".join(ks) if len(ks) <= 3 else "%s ほか %d" % (", ".join(ks[:3]), len(ks) - 3)
            last = checked_by.get(rid)
            chk = "[%s](%s) %s" % (last[-1][0].stem, rel_link(last[-1][0], sp), last[-1][1]) if last else "なし"
            trace.append((rid, dec, t or "なし", chk))
        hist = history_block(sp, [r for p_, r in records.items()
                                  if p_.relative_to(specs).parts[0] == DECS and set(r.touches) & mine], trace)
        cur_text = sp.read_text(encoding="utf-8")
        cur = H_RE.search(cur_text)
        if write:
            new_text = (cur_text[:cur.start()] + hist + cur_text[cur.end():]) if cur \
                else cur_text.rstrip("\n") + "\n\n" + hist + "\n"
            if new_text != cur_text:
                sp.write_text(new_text, encoding="utf-8")
        elif not cur or cur.group(0) != hist:
            warns.append("W15 %s: 末尾の生成区画(決定とつながり)が古い(--write で直す)" % sp.relative_to(root))

    # --- README の生成区画 ---
    if readme.exists():
        block = render_block(root, specs, docs, spec_files, records, reqs, metrics=write)
        rt = readme.read_text(encoding="utf-8")
        cur = BLOCK_RE.search(rt)
        if write:
            new = rt[:cur.start()] + block + rt[cur.end():] if cur else rt.rstrip("\n") + "\n\n" + block + "\n"
            if new != rt:
                readme.write_text(new, encoding="utf-8")
                print("書き直した: %s の生成区画" % readme.relative_to(root))
        elif not cur or METRICS_RE.sub("", cur.group(0)) != METRICS_RE.sub("", block):
            warns.append("W1 specs/README.md: 生成区画が古い(python3 %s %s --write)" % (SELF, root))

    for e in errors:
        print(e)
    shown = warns
    if "--verbose" not in argv:
        # いつも出続ける種類は、種類ごとの1行にまとめる(--verbose で1件ずつ)。数は1件ずつ数えたまま。
        shown = [w for w in warns if w.split(" ", 1)[0] not in SUMMARIZED]
        for code, what in SUMMARIZED.items():
            n = sum(1 for w in warns if w.split(" ", 1)[0] == code)
            if n:
                shown.append("%s ×%d(%s。--verbose で1件ずつ)" % (code, n, what))
    for w in shown:
        print(w)
    print("-- %d files, %d capabilities, %d requirements, %d errors, %d warnings"
          % (len(md_files), len(spec_files), len(reqs), len(errors), len(warns)))
    return 1 if errors else 0


def rel_link(path, spec_path):
    import os
    return os.path.relpath(path, spec_path.parent).replace(os.sep, "/")


def history_block(spec_path, recs, trace=()):
    import os
    recs = sorted(recs, key=lambda r: (r.created, r.path.name), reverse=True)
    out = [H_BEGIN, "<!-- この区画は check.py --write が書く。手で直さない。 -->", "", "## この機能の決定", ""]
    if recs:
        out += ["| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |", "|---|---|---|---|---|---|"]
        for r in recs:
            link = os.path.relpath(r.path, spec_path.parent).replace(os.sep, "/")
            t = ", ".join(r.touches) if len(r.touches) <= 6 else "%s ほか %d" % (", ".join(r.touches[:3]), len(r.touches) - 3)
            out.append("| %s | [%s](%s) | %s | %s | %s | %s |" % (
                r.created, r.fm.get("title") or r.path.stem, link, r.status, r.decided_by,
                r.fm.get("trigger", "旧形式"), t))
    else:
        out.append("なし")
    out += ["", "## この機能のつながり", ""]
    if trace:
        out += ["| 要件 | 最新の決定 | テスト | 最後の照合 |", "|---|---|---|---|"]
        out += ["| %s | %s | %s | %s |" % row for row in trace]
    else:
        out.append("なし")
    out.append(H_END)
    return "\n".join(out)


METRICS_HEAD = "### 流れの数字"
METRICS_RE = re.compile(r"\n+" + re.escape(METRICS_HEAD) + r".*?(?=\n" + re.escape(END) + ")", re.S)


def flow_metrics(root, records, docs, specs):
    """DC-12: 人が AI 案を変えた割合と、「テスト:」から「照合:」までの時間の中央値(生成区画の末尾に出す)。
    git の履歴で変わる数字なので、--write のときだけ計算し、W1(生成区画が古い)の比較には入れない。"""
    import statistics
    out = ["", METRICS_HEAD, ""]
    if git_out(root, "rev-parse", "--git-dir") is None:
        return out + ["- 数えられない(git の履歴が無い)"]
    if (git_out(root, "rev-parse", "--is-shallow-repository") or "").strip() == "true":
        return out + ["- 数えられない(浅い履歴)"]
    human = [r for p, r in sorted(records.items()) if p.relative_to(specs).parts[0] == DECS
             and r.fm.get("decided-by") == "人" and r.status in ("accepted", "rejected")]
    changed = 0
    for r in human:
        if r.status == "rejected":
            changed += 1
            continue
        # 履歴(新しい順)の (コミット, その時のパス)。起票は最古、決着は _decisions/ に入った最初の版。
        log = git_out(root, "log", "--follow", "--format=%H", "--name-only", "--", r.path.relative_to(root).as_posix())
        lines = [l for l in (log or "").splitlines() if l.strip()]
        hist = list(zip(lines[0::2], lines[1::2]))
        settled = [h for h in hist if "/%s/" % DECS in "/" + h[1]]
        if len(hist) < 2 or not settled:
            continue  # 起票と決着を比べられない(コミット前・旧い記録)
        first = git_out(root, "show", "%s:%s" % hist[-1])
        last = git_out(root, "show", "%s:%s" % settled[-1])
        if first is not None and last is not None and \
                section(split_front(first)[1], "差分").strip() != section(split_front(last)[1], "差分").strip():
            changed += 1
    out.append("- 人が AI 案を変えた割合 %d / %d(数え方: decided-by が 人 で採択か却下した決定記録のうち、却下したものと、"
               "起票の版と決着の版で「差分」の節が違うもの。取り込み・旧形式・取り下げは数えない)" % (changed, len(human)))
    log = git_out(root, "log", "--format=%at%x09%s") or ""
    commits = [l.split("\t", 1) for l in log.splitlines() if "\t" in l]
    hours = []
    for p, (fm, body, text) in sorted(docs.items()):
        if p.relative_to(specs).parts[0] != CHANGES or fm.get("status") != "done":
            continue
        rel = p.relative_to(root).as_posix()
        t = [int(c) for c, subj in commits if subj.startswith("テスト:") and rel in subj]
        v = [int(c) for c, subj in commits if subj.startswith("照合:") and rel in subj]
        if t and v and max(v) >= min(t):
            hours.append((max(v) - min(t)) / 3600)
    med = "%.1f 時間" % statistics.median(hours) if hours else "なし"
    out.append("- 「テスト:」から「照合:」までの時間: 中央値 %s(done の変更の記録 %d 件。件名が「テスト:」で記録のパスを含む"
               "最初のコミットから、「照合:」で記録のパスを含む最後のコミットまで。書いた時刻で数える)" % (med, len(hours)))
    return out


def render_block(root, specs, docs, spec_files, records, reqs, metrics=False):
    def link(p):
        return p.relative_to(specs).as_posix()

    def protected(rid):
        return any(r.protects(rid) for r in records.values())

    out = [BEGIN, "<!-- この区画は check.py --write が書く。手で直さない。 -->", "", "### ケイパビリティ", ""]
    out += ["| ケイパビリティ | 内容 | 読むとき | 要件 | うち守られる |", "|---|---|---|---|---|"]
    for p in sorted(spec_files):
        fm = docs[p][0]
        mine = [rid for rid, r in reqs.items() if r["spec"] == p.relative_to(root)]
        out.append("| [%s](%s) | %s | %s | %d | %d |" % (
            p.parent.name, link(p), fm.get("description", ""), fm.get("load_when", ""),
            len(mine), sum(1 for rid in mine if protected(rid))))
    unconfirmed = sorted({rid for rid in reqs for r in records.values()
                          if r.is_import and r.import_rows.get(rid, ("",))[0] == "未確認"})
    out += ["", "取り込みで決定者が未確認の要件: %s" % (", ".join(unconfirmed) if unconfirmed else "なし")]
    out += ["", "### 審議中", ""]
    props = [r for p, r in sorted(records.items()) if p.relative_to(specs).parts[0] == PROPS]
    if props:
        out += ["| 提案 | 触る要件 | きっかけ | 決定者 |", "|---|---|---|---|"]
        for r in props:
            out.append("| [%s](%s) | %s | %s | %s |" % (
                r.fm.get("title") or r.path.stem, link(r.path), ", ".join(r.touches),
                r.fm.get("trigger", "旧形式"), r.decided_by))
    else:
        out.append("なし")
    out += ["", "### 決定の一覧(新しい順)", ""]
    decs = [r for p, r in records.items() if p.relative_to(specs).parts[0] == DECS]
    decs.sort(key=lambda r: (r.created, r.path.name), reverse=True)
    if decs:
        out += ["| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触る要件 |", "|---|---|---|---|---|---|"]
        for r in decs:
            t = ", ".join(r.touches) if len(r.touches) <= 6 else \
                "%s ほか %d" % (", ".join(r.touches[:3]), len(r.touches) - 3)
            out.append("| %s | [%s](%s) | %s | %s | %s | %s |" % (
                r.created, r.fm.get("title") or r.path.stem, link(r.path), r.status, r.decided_by,
                r.fm.get("trigger", "旧形式"), t))
    else:
        out.append("なし")
    if metrics:
        out += flow_metrics(root, records, docs, specs)
    out.append(END)
    return "\n".join(out)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
