#!/usr/bin/env python3
"""trace-gate: 要求の正本とテストのコメントの印を突き合わせる検査器。Python の標準ライブラリだけで動く1つのファイル。

使い方(どれも --root DIR と --config FILE を取る。既定は今のフォルダと <root>/trace-gate.json。--root は git の根の下のフォルダでもよく、
git から読む base の側と差分の道は --root からの道にそろえ、--root の外のファイルは見ない。weaken は git の根の .github/workflows/ も見る):
  trace_gate.py check  [--base REF] [--strict-open] [--junit FILE ... [--base-junit FILE ...]] [--format text|json|sarif|github]
      要求の書式(重複・崩れ・base との食い違い・承認の記録との食い違い)と、テストの印(未知・古い・新しすぎる・どこにも付かない・
      skip・空・P0 のテスト無し)を検査する。--base-junit は base の結果と集めた数・飛んだ数を比べる。
  trace_gate.py ledger [--base REF] [--out FILE]
      台帳(JSON。見出しと、要求を名前の順に1行1件)を作る。リポに入れない生成物。
  trace_gate.py weaken --base REF [--head REF] [--junit-base FILE --junit-head FILE] [--format ...]
      base と head(既定は作業ツリー)のテストを比べ、skip・失敗してよい印の追加、断言の減少と緩め、数値の書き換え、
      テストの削除、テストの設定の変更を報告する。Python は構文の木で、ほかは文字の差分で比べる。
  trace_gate.py f2p --junit FILE [--base REF] [--req NAME,...] [--format ...]
      実装前の状態で走らせた JUnit XML から、要求ごとに「断言の失敗で落ちたテスト」があるかを数える。
      数える要求は --req の名前。無ければ、--base があれば base に無い P0・base より版が上がった P0・base で P0 でなかった P0
      (新しく足すか意味を変えたか P0 に上げた要求)だけ、--base も無ければ全部の P0。選んだ名前を JSON の selected と要約に出す。
  trace_gate.py approve --by NAME --meaning TEXT [--req NAME,...]
      人が宣言したときだけ、選んだ要求の名前・版・指紋・承認者・日付・意味を承認の記録(specs/approvals.jsonl)に追記する。
      端末で yes と打ったときだけ書く(AI は走らせない)。記録があれば、check は承認の後に変わった要求を止める。
  trace_gate.py init [--force | --relock] [--hook]
      検査器の写し(tools/trace_gate.py)・設定・要求の雛形・CI の雛形と、写しの錠(tools/trace-gate.lock.json)を置く。
      既にあるファイルは上書きしない(--force は写しと CI の雛形だけを上書き)。--relock は写しの錠だけを書き直す。人が走らせる。
      写しの錠があれば、check は錠の各ファイルを今の中身と照らす(vendor-drift)。
  trace_gate.py ledger --compare PREV.json
      台帳に、前の台帳と比べた要求ごとの new・unchanged・updated・absent を足す。
  trace_gate.py intents [--req NAME,...] [--base REF] [--kind stest] [--format text|json|md|csv|files]
  trace_gate.py results --junit NAME=PATH [--junit NAME=PATH ...] [--format md|csv|json]
      印の付いたテストを要求ごとに並べ、名前・意図(Python の docstring かコメントの1行)・断言の行を出す。
  trace_gate.py convert [--write]
      1行の形の要求の行を、見出しの区切りの形(### 題・ID の行・欄の箇条)に移す。既定は差分を出すだけで、--write のときだけ書く。
      移す前と後で、どの要求の名前・版・優先度・指紋・欄も変わらないことを確かめてから書く(1つでも違えば何も書かずに 1 で終わる)。
  trace_gate.py pr-body --base <基点> [--head <参照>]
      PR の本文の下書き(照らした要求・要求を変えるか・仮決め・上流の変更・人が見るもの)を Markdown で出す。何を変えたか・なぜは人が書く。
  trace_gate.py f2p-run --base REF --cmd "...{junit}..." [--req NAME,...]
      一時の作業ツリーで base に head のテストを写して走らせ(実装前)、head で走らせ(実装後)、断言の失敗で落ちて通ったかを数える。
      数える要求の既定は f2p --base と同じ(base に無いか、版が上がったか、P0 に上げた P0)。0件ならテストを走らせずに 0 で終わる。
  trace_gate.py reverse-run --base REF --cmd "...{junit}..."
      テスト(補助のモジュールとデータを含む)が base から変わったときだけ、一時の作業ツリーで head のコードに base のテストの側を写して
      走らせ、断言の失敗を base-test-failed(警告)で並べる。版の上がった要求の印だけのテストは除く。門にしない(終了コードは 0)。
  trace_gate.py rename 旧い名前 新しい名前 [--write] [--base REF]
      要求の名前を付け替える。正本の ID の行・テストのコメントの印・名前の表(設定の renames)だけを書き換え、版と指紋は変えない。
      既定は差分を出すだけで、--write のときだけ書く。新しい名前が今の正本・比べる先(既定は HEAD)・名前の表にあるか、
      旧い要求の文か版が比べる先と違えば、何も書かずに 1 で終わる。

終了コード: 0 = 問題なし、1 = 問題あり(人が見る)、2 = 使い方か道具の誤り。
"""

from __future__ import annotations

import argparse
import ast
import bisect
import contextlib
import csv
import difflib
import hashlib
import io
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import tokenize
import xml.etree.ElementTree as ET
from collections import Counter
from functools import lru_cache

TOOL_NAME = "trace-gate"
TOOL_VERSION = "0.38.0"
LEDGER_FORMAT = "trace-gate-ledger"
FINDINGS_FORMAT = "trace-gate-findings"
FORMAT_VERSION = 1
CONFIG_NAME = "trace-gate.json"
VENDORED_CHECKER = "tools/trace_gate.py"
CONFIG_VERSION = 1
# 部品(人の決定: 削らずに、使う側が設定の parts で選ぶ。書いていない部品は有効)。値は、切ったときに消える守り
PARTS = {
    "ci": "CI で check と weaken が走らず、合否を手元のフックに頼る(フックは外せる)",
    "hook": "作業の終わりに check が自動で走らない",
    "weaken": "テストを緩めた変更が人に回らず素通りする",
    "f2p": "新しい P0 のテストが実装前に落ちるかを確かめない",
    "approvals": "承認した後に変わった要求を見分けない",
    "upstream": "上流の項目の写し漏れを見つけない",
}
# 前の版(0.18 まで)の init が部品 ci のときに置いた CI の雛形。今の init は置かない(サービス側の雛形は別のプラグインが持つ。人の決定)。
# 写しの錠にこの行が残るあいだは照らし続け、人の init --force・--relock で錠から外し、ci を切れば --prune で消せる
CI_WORKFLOW = ".github/workflows/trace-gate.yml"
SERVICE_GUIDE = "サービス側の守り(CI・CODEOWNERS・ブランチの規則・PR の雛形)は trace-gate では置かない。同じ棚のサービス側の雛形のプラグインが、trace_gate.py guards --format json を読んで作る"
DEFAULT_OFF = {"ci"}  # 書いていなくても切ってある部品(CI は費用と整える手間がかかるので推奨にとどめる。人の決定)

DEFAULT_REQUIREMENTS = ["specs/requirements.md"]
DEFAULT_TESTS = [
    "**/test_*.py",
    "**/*_test.py",
    "**/tests/**/*.py",
    "**/*.test.*",
    "**/*.spec.*",
    "**/__tests__/**",
    "**/*Test.php",
    "**/tests/**/*.php",
    "**/tests/**/*.rs",
    "**/*_test.go",
]
DEFAULT_EXCLUDE = ["**/node_modules/**", "**/vendor/**", "**/target/**", "**/.venv/**", "**/venv/**", "**/dist/**", "**/build/**"]
DEFAULT_APPROVALS = "specs/approvals.jsonl"
DEFAULT_RENAMES = "specs/renames.md"  # 名前の表(付け替えた旧い名前と消した名前。使い回しを止める)
# 基準のファイル(スナップショット・承認の写し・差の台帳)。変えると weaken が人に回す
# Go のゴールデンとファズの種(testdata)・ゴールデンのファイル・見本のフォルダも、期待値を持つ基準として人に回す
DEFAULT_BASELINES = [
    "**/__snapshots__/**",
    "**/*-snapshots/**",
    "**/*.snap",
    "**/*.ambr",
    "**/*.approved.*",
    "specs/diffs.*",
    "**/testdata/**",
    "**/*.golden",
    "**/fixtures/**",
]
DEFAULT_TEST_CONFIG = [
    "**/conftest.py",
    "**/pytest.toml",
    "**/.pytest.toml",
    "**/pytest.ini",
    "**/.pytest.ini",
    "**/tox.ini",
    "**/.coveragerc",
    "**/vitest.config.*",
    "**/vitest.workspace.*",
    "**/vitest.setup.*",
    "**/jest.config.*",
    "**/jest.setup.*",
    "**/playwright.config.*",
    "**/.mocharc*",
    "**/karma.conf.*",
    "**/phpunit.xml",
    "**/phpunit.xml.dist",
    "**/.config/nextest.toml",
    "trace-gate.json",
]
# 共有の設定のファイルは、変わった行がテストに触れるときだけ報告する
SHARED_CONFIG = [
    "**/pyproject.toml",
    "**/setup.cfg",
    "**/package.json",
    "**/composer.json",
    "**/Cargo.toml",
    # アプリと道具の設定にテストの節を混ぜて持つファイル(Vitest は vitest.config が無いと vite.config の test を読む。bun と deno はテストを外す鍵を持つ)
    "**/vite.config.*",
    "**/bunfig.toml",
    "**/deno.json",
    "**/deno.jsonc",
    # テストを走らせる入口(ここで PYTEST_ADDOPTS などを足すとテストを外せる)
    "**/Makefile",
    "**/*.mk",
    "**/justfile",
    "**/Justfile",
    "**/Taskfile.yml",
    "**/Taskfile.yaml",
    "**/noxfile.py",
    "**/ci.sh",
    "**/scripts/ci*.sh",
    "**/scripts/test*.sh",
]
SHARED_CONFIG_WORDS = re.compile(r"(?i)pytest|unittest|jest|vitest|mocha|phpunit|pest|nextest|\btests?\b")
# テストの語を含まずにテストを外せる鍵(bun の最上位の preload は bun test の前に走り、deno の最上位の exclude は deno test にも効く)
SHARED_CONFIG_EXTRA = {"bunfig.toml": re.compile(r"\bpreload\b"), "deno.json": re.compile(r"\bexclude\b"), "deno.jsonc": re.compile(r"\bexclude\b")}
# 依存と生成物の大きなフォルダは中をたどらない(大きなリポで遅いため。中のテストは既定の exclude で外れる)。
# build・dist・coverage はソースのフォルダの名前とぶつかりうるので、たどって exclude で外す
SKIP_DIRS = {".git", "node_modules", ".venv", "venv", "__pycache__", ".mypy_cache", ".pytest_cache", "target", "vendor", ".tox", ".nox", ".next"}

LANG_BY_EXT = {
    ".py": "python",
    ".js": "js",
    ".jsx": "js",
    ".mjs": "js",
    ".cjs": "js",
    ".ts": "js",
    ".tsx": "js",
    ".mts": "js",
    ".cts": "js",
    ".php": "php",
    ".rs": "rust",
    ".go": "go",
}

NAME = r"[a-z0-9][a-z0-9._-]*"
REQ_HINT = re.compile(r"^\s*[-*]\s+`req~")
REQ_LINE = re.compile(
    r"^\s*[-*]\s+`req~(?P<name>" + NAME + r")~(?P<ver>[1-9][0-9]*)`\s+(?P<prio>P[0-2])\s+(?:(?P<origin>【[^】]*】)\s+)?"
    r"(?P<text>\S.*?)\s*受け入れ[:：]\s*(?P<accept>\S.*?)\s*$"
)
OPEN_HINT = re.compile(r"^\s*[-*]\s+`tbd~")  # 1行の形(convert でだけ読む)
OPEN_LINE = re.compile(r"^\s*[-*]\s+`tbd~(?P<name>" + NAME + r")`\s+(?:(?P<origin>【[^】]*】)\s+)?(?P<text>\S.*?)\s*$")
# 未確定事項の区切り: 題の見出し(### か ####)・ID の行(`tbd~名前` だけ)・欄の箇条(決める人 は必須。ほかは任意)
OPEN_ID_HINT = re.compile(r"^\s*`tbd~")
OPEN_ID_LINE = re.compile(r"^\s*`tbd~(?P<name>" + NAME + r")`(?:[ \t]+(?P<origin>【[^】]*】))?[ \t]*$")
OPEN_LABELS = ("決める人", "期限", "仮", "根拠", "区分")
OPEN_KINDS = ("人が先に決める", "仮に決めてよい")
DUE_RE = re.compile(r"期限[:：]\s*(\d{4}-\d{2}-\d{2})")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
DECIDER_RE = re.compile(r"決める人[:：]\s*[^\s。、.,]")
MARK_RE = re.compile(r"\[\s*(utest|itest|stest|ctest)\s*->\s*([^\]\n]*)\]")
TARGET_RE = re.compile(r"^req~([A-Za-z0-9._-]+)~([0-9]+)$")
MARK_HINT = re.compile(r"\[\s*(?:utest|itest|stest|ctest)\b|\breq~[A-Za-z0-9._-]+~")
UP_HINT = re.compile(r"^\s*[-*]\s+`up~")
UP_LINE = re.compile(r"^\s*[-*]\s+`up~(?P<name>" + NAME + r")`\s+(?P<text>\S.*?)\s*$")
UP_REF = re.compile(r"(?<![A-Za-z0-9_~])up~([a-z0-9][a-z0-9._-]*)")

RULES = {
    # TG-1 要求の正本
    "req-malformed": "形は「### 要求の題」の次の行に「`req~名前~版` P0」、続けて「- Acceptance: 確かめられる1文」(1つ以上)と任意の「- Rationale:」「- Needs:」「- Disposition:」「- Input:」を1欄1行で書く(日本語の名前 受け入れ・なぜ・確かめ・扱い・出典 も読む)。名前は英小文字・数字・. - _。1行の形の要求と日本語の名前は convert で今の形に移す",
    "block-stray-text": "区切りの欄の後に、空の行を挟んで字下げした段落・表・コードがある。GitHub では受け入れの項目の中に見えるが、指紋に入らず、変えても版の上げ忘れに気づけない。要る文は受け入れの欄に1行で書くか、字下げを外して区切りの外の段落にする(表は見出しを付けて区切りの外の節へ)",
    "origin-deprecated": "【…】の印は前の版の出所の印で、もう使わない(消してよい。指紋は変わらない)。誰が決めたかは PR のレビューの承認で残す",
    "req-duplicate": "名前は1つの要求に1つ。消した要求の名前も使い回さない(古い印が別の要求に結びつくため)",
    "name-reused": "名前の表(設定の renames)で付け替えた旧い名前か消した名前を、今の要求に使わない(古いテストの印が新しい意味の要求に黙って結びつくため)。新しい名前を付ける",
    "renames-malformed": "名前の表の行は「| `旧い名前` | `新しい名前` |」か「| `旧い名前` | 消した(理由) |」。旧い名前は1行に1つ、行は付け替えた順に足し、前の行の旧い名前を新しい名前に使わない",
    "req-none": "設定の requirements が要求の正本を指しているか確かめる。要求が0件のまま通すと、何も見ずに通る",
    "inputs-present": "設定の tests か requirements に書いた置き場が何にも当たらない(グロブでない道が無い、グロブに当たるファイルが無い、当たったテストが全部 exclude で外れた)。置き場か exclude を直す(0件のまま通すと何も見ずに通る)",
    "open-dangling": "製品のコードのコメントに残った tbd~名前 が正本の未確定事項に無い。決まったなら仮置きのコメントを消し、残っているなら正本に未確定事項を書く",
    "open-used-undecided": "製品のコードが、仮: の無い未確定事項 tbd~名前 を使っている。決める人に決めてもらって要求に書くか、仮に決めてよいなら 仮: に値を書く(仮が無いまま作ると、作る人ごとに推測が割れる)",
    "spec-external-ref": "正本が別紙・別表を名前だけで指している。中身を正本に書くか、リポの中に置いて [名前](道) で指す(作る人や AI が別紙を持っていないと推測で作る)",
    "req-changed-same-version": "要求の文か受け入れ基準を変えたら版を上げる(誤字の直しでも、文が変われば版を上げる。指紋は空白しか寄せないため)",
    "req-changed": "人が中身を見て、要求の変更をレビューで承認する(AI だけで要求を変えない。優先度を上げたら、P0 の選び分けを人が決める)",
    "approval-stale": "承認した後に要求が変わった。人が中身を見て、端末で approve を走らせて承認し直すか、要求を戻す",
    "approval-malformed": "承認の記録の行は1行1つの JSON(name・version・fingerprint・by・date・meaning)。手で直さず、人が approve で書く",
    "approval-rewritten": "承認の記録は追記だけ。書き換えた・消した行を base の形に戻す(取り消すなら、人が新しい行で承認し直す)",
    "req-version-down": "版は上げる向きにだけ変える。戻したいなら新しい版で前の文を書く(下げると、新しい版の印が mark-future になるため)",
    "upstream-unmapped": "上流の項目を、どれかの要求の Input: に書くか、決めていないなら未確定事項(tbd~)の Basis: に up~名前 を書いて保留にする。1行の形の要求の行は読まないので、残っていれば先に convert で移す",
    "upstream-unknown": "出典の up~名前 を上流の正本の名前に合わせる(上流に無い決定を足すなら、上流の変更管理を通す)",
    "upstream-retired": "上流の項目が、扱いが廃止の要求からしか結ばれていない。上流の決定をやめるなら、上流の変更管理を通したかを確かめる",
    "upstream-title-stale": "出典の up~名前(題) の題を、上流の項目の文の頭の写しに直す(終わりを … で切ってよい)。上流の文を変えたなら、要求の側の写しも同じ変更で直す",
    "upstream-malformed": "形は「- `up~名前` 上流の項目の短い文」。名前は英小文字・数字・. - _ で、1つの項目に1つ",
    "open-duplicate": "未確定事項の ID は1つの問いに1つ。2つ目を消すか、別の問いなら別の名前にする(同じ ID の写しは weaken の比べを外すため)",
    "open-malformed": "形は「### 決めていないこと」の次の行に「`tbd~名前`」、続けて「- Decider: 人」(必須)と任意の「- Due: YYYY-MM-DD か出来事」「- Provisional: 値」「- Basis: up~名前 や理由」「- Kind: 人が先に決める|仮に決めてよい」を1欄1行で書く(日本語の名前 決める人・期限・仮・根拠・区分 も読む)。1行の形と日本語の名前は convert で移す",
    "open-item": "出荷の前に、人が決めて要求に書き、未確定事項の区切りを消す",
    # TG-2 テストの印
    "mark-malformed": "形は「[utest -> req~名前~版]」(種類は utest・itest・stest・ctest。複数は , で区切る)",
    "mark-unknown": "印の名前の打ち間違いなら直す。要求を消したなら、その要求だけを指すテストを同じ変更で消す(何も確かめない印が数に入るため)",
    "mark-outdated": "要求の今の版の文に照らしてテストを確かめてから、印の版を直す(印の版だけを替えない)",
    "mark-future": "要求の版を上げ忘れていないか、印の版の打ち間違いでないかを確かめる",
    "mark-orphan": "印はテストの関数の直前(デコレータや属性の上でよい)か、本体の中に置く",
    "test-skipped": "skip や失敗してよい印で緑にしない。要求と食い違うなら、テストを直さずに止めて人に報告する",
    "test-focused": "`.only`・fit・fdescribe はほかのテストを黙って止める。コミットする前に外す",
    "test-empty": "要求の受け入れ基準を確かめる断言(assert・expect など)を書く(断言の無いテストは実装が誤っていても通るため)",
    "test-trivial": "いつも通るか型だけを見る断言しか無い。要求の受け入れ基準の値を比べる断言を書く(型の合う誤った値で通るため)",
    "needs-missing": "要求の Needs: に書いた種類のテストを足す(Needs: から種類を外すのは確かめを弱める要求の変更で、PR で人が承認し、weaken が needs-reduced で人に回す)",
    "needs-unwanted": "要求の Needs: に無い種類の印。要るなら Needs: に足し、要らないならテストか印を見直す",
    "needs-reduced": "人が見て、要る確かめの種類を減らしてよいかを確かめる(減らすのは確かめを弱めること)",
    "p0-no-test": "P0 には、今の版の印を持ち、飛ばない(条件つきの skip も無い)断言のあるテストを1本以上置く(設定の quarantine で隔離したテストは数えない)。環境に頼る所は偽物で差し替えて閉じた形にする",
    "test-not-run": "印のあるテストを、全部が揃うと決めた CI のジョブで走らせる(結果に無いのは、集め方の外か、名前で外されたか)。設定の tests と JUnit の classname が同じテストを指しているかを照らす",
    "test-skipped-at-run": "走らせた結果で飛んだ。名前を組み立てた skip や環境の条件を外し、全部が揃うジョブで走らせる",
    "test-flaky": "再試行で通った(flakyFailure・flakyError)。直すまで隔離するなら、trace-gate.json の quarantine に持ち主と始めた日と理由を書く(人が見る)。再試行の回数を増やして隠さない",
    "test-quarantined": "隔離の一覧(設定の quarantine)にあるテストが、走らせた結果で落ちたか飛んだか結果に無い。持ち主が直して一覧から外す(隔離を延ばすなら extensions を増やし、PR で人が見る)",
    "quarantine-unknown": "隔離の一覧(設定の quarantine)の鍵が、どの印のテストの「道::名前」にも当たらない。テストの名前か道を変えたなら鍵を直し、テストを消したなら項目を消す(当たらない項目は隔離の本数に数えない)",
    "quarantine-passing": "隔離の一覧にあるテストが通った。直ったなら一覧から外す(外さないと次に落ちても止まらない)",
    "test-count-dropped": "走らせた結果の集めたテストが base の結果より減ったか、飛んだテスト(失敗してよい印を含む)が増えた。印の無いテストを設定・名前の選び・収集の外で外していないかを確かめ、消した・飛ばした理由を PR の本文に書いて人に回す(テストを戻すか、base の結果を取り直す)",
    "test-failed-at-run": "印のあるテストが走らせた結果で落ちた。実装を直す(テストを弱めて通さない)。テストが仕様と食い違うなら、どちらも直さずに止めて、場所と両方の読みを書いて人への問いで終える",
    "adr-malformed": "足した ADR は雛形(write-requirement の templates/adr.md)の見出し(背景・決定・理由・選ばなかった案・影響・見直しの条件・確かめ方)を全部持ち、`- 状態:` を 提案・採用・却下・置き換え のどれかにする(置き換えなら `- 置き換え先:` に新しい ADR)。中身は人がレビューで見る。既にある ADR は見ない",
    "spec-heading-missing": "主題のファイルに `## 目的` と `## 要求` を置く(雛形の必須の見出し。任意の見出しは要るときだけ)",
    "spec-heading-order": "見出しを雛形の順(目的・範囲外・用語・要求・未確定事項)に並べる。足した見出しはどこに置いてもよい",
    "spec-line-misplaced": "要求の区切りは `## 要求` の下(まとまりは `###` で分ける)、未確定事項の区切りは `## 未確定事項` の下に置く",
    "accept-not-parts": "受け入れ基準を `- Acceptance:` の下に字下げした `- Given:`(前の状態)・`- When:`(操作1つ)・`- Then:`(見える結果)で書く(require_accept_parts が true)。前の状態と操作の無い要求は Given と When を空にする。1行の文は convert で Then に移せる(指紋は変わらない)。境界は両側を場面に書く",
    "req-no-source": "要求を足す・変える前に、根拠の文書に調べを置き、要求の末尾に Input: up~名前 を書く(取り入れは調べてから)",
    "part-below-floor": "下限の部品(リポの事実から出す)を切るなら、trace-gate.json の waivers に部品ごとの reason(理由)と until(期限)を書く。任意にした守りは入れられず、止めた守りは戻らないため",
    "waiver-expired": "期限の過ぎた切り替え。部品を戻すか、人が理由を見直して期限を延ばす(ふだんは知らせるだけで、切った部品が守る置き場を変えた PR の check --base だけ止める。暦が変わっただけで関係の無い PR を落とさないため。--base では基点と head の早い期限で判断するので、延ばすのはテストに触れない別の変更でする)",
    "retired-has-tests": "扱いが廃止の要求を指す印のテストを同じ変更で消すか、要求の扱いを戻す(廃止の要求は確かめない)",
    # TG-12 テストの弱め
    "skip-added": "人が見て、正当な理由があるか確かめる(理由が無ければ戻す)",
    "xfail-added": "人が見て、正当な理由があるか確かめる(要求との矛盾なら仕様を直す)",
    "assert-reduced": "人が見て、消した断言が要求の確かめに要らないかを確かめる",
    "assert-loosened": "人が見て、緩めた断言で要求の受け入れ基準を確かめられるかを確かめる",
    "literal-changed": "人が見て、受け入れの例の値を変えていないかを確かめる",
    "test-removed": "人が見て、消したテストが確かめていた要求に別のテストがあるかを確かめる",
    "test-kind-changed": "人が見て、テストの関数の種類(def と async def)を変えてよいかを確かめる(unittest は async def の本体を走らせずに通す)",
    "mark-removed": "人が見て、テストの印から種類か要求を外してよいかを確かめる(付け替えは確かめの主張を変える)",
    "req-priority-lowered": "人が見て、要求の優先度を下げてよいかを確かめる(下げると P0 の検査から外れる)。下げる理由と、それを支える独立の証拠(設計で分けたこと・実績の数)を PR の本文に書く",
    "req-removed": "人が見て、要求を正本から消してよいかを確かめる(消すなら、その印のテストも同じ変更で見直す)",
    "open-removed": "人が見て、未確定事項を決めた人と決めた中身を確かめる(人が先に決めるものを AI が消して決めない。決まったなら PR の本文に誰がどう決めたかを書く)",
    "open-downgraded": "人が見て、未確定事項の区分を「仮に決めてよい」に下げてよいか、決める人を変えてよいかを確かめる(下げると、次に消しても人に回らない)",
    "req-renamed": "人が見て、名前の表の付け替えが名前だけの変更かを確かめる(同じ変更で文や版を変えていれば、req-changed も出る)",
    "req-removed-unrecorded": "消した要求の名前を、名前の表(設定の renames)に「| `名前` | 消した(理由) |」の行で書く(付け替えなら「| `旧い名前` | `新しい名前` |」)。書かないと使い回しを止められない",
    "name-unreserved": "人が見て、名前の表の行を消すか変えてよいかを確かめる(消すと、その名前の使い回しが止まらなくなる)",
    "gate-loosened": "人が見て、CI の置き場の変更が検査を緩めていないかを確かめる(走らせる検査器・引き金・手順を見る。緩めるなら理由を ADR に書く。|| true や --no-verify で落ちる検査を通さない)",
    "run-count-dropped": "人が見て、走らせた結果の集めたテストの減りと飛んだテストの増えが意図したもの(テストの消し・環境で飛ぶテスト)かを確かめる(印の無いテストを外す裏口は、テストのファイルの差分に出ないため)",
    "test-unmarked": "足したテストに要求の印が無い。確かめている振る舞いの要求を書いて印を付けるか、要求に結ばないテスト(回帰の見本・既知の限界・補助)なら本体か docstring に [no-req: 理由] を書く",
    "test-renamed": "名前だけが変わった(本体と印は同じ)。弱めではないので、名前が振る舞いを言っているかだけ見る",
    "test-retired": "人が見て、要求の引退が決めたことかを確かめる(引退でなければ、要求とテストを戻す)",
    "test-file-deleted": "人が見て、消したテストが確かめていた要求に別のテストがあるかを確かめる",
    "test-config-changed": "テストの集め方と結果の報告を変えうる。人が中身を見る",
    "upstream-changed": "人が見て、上流の項目の写しの変更が、上流の変更管理(変更管理書の番号)を通ったものかを確かめ、文に挙げた出典の要求を新しい項目の文に照らして見直す",
    "mark-bumped": "人が見て、要求の新しい版の文に照らしてテストを確かめたかを確かめる(本体を変えずに印の版だけを上げると、古い印の止めが外れる)",
    "adr-changed": "人が見て、採用した ADR の決定を書き換えてよいかを確かめる(決定を変えるなら、新しい ADR を足して古い ADR を置き換えにする。誤字やリンクの直しなら、そう PR の本文に書く)",
    "kept-added": "人が見て、足した P0 の要求が本当に既にある振る舞いの写し(リプレースの踏襲)かを確かめる(踏襲は実装前に落ちるかの確かめから外れるので、新しい振る舞いに付けると f2p を黙って外せる)",
    "test-control-added": "人が見て、テストに足した return・握りつぶす try/except・断言を包む if・非同期へ逃がす形・断言の差し替え・空の補助が、断言を走らせなくしていないかを確かめる(断言の行を変えずにテストを殺す形は、断言の数にも期待値にも出ない)。正当なら理由を PR の本文に書く",
    "test-aware-code": "人が見て、テストでないコードがテストの最中を見分けて振る舞いを変えるか、断言を差し替えていないかを確かめる(テストのときだけ正しく返すコードは、テストを変えずにテストを通す)。外への通信やログを切るなどの正当な使い方なら、理由を PR の本文に書く",
    "suppress-added": "人が見て、検査を黙らせる抑止のコメントを足してよいかを確かめる(理由をコメントに書き、要らなくなったら消す)",
    "instructions-changed": "人が見て、エージェントへの指示と設定(AGENTS.md・CLAUDE.md・.claude/・.agents/・.codex/・.mcp.json など)の変更が意図したものかを確かめる(実装役が自分の縛りを緩められるため)",
    "req-added": "人が見て、足した要求とその優先度を確かめる(P0 の選び分けは人が決める)",
    "baseline-changed": "人が見て、基準(スナップショット・承認の写し・差の台帳)の変更が意図した差かを確かめる(一括の更新で差を黙って受け入れない)",
    # TG-3 実装前に落ちるか
    "f2p-not-failing": "実装前に断言の失敗で落ちるテストを置く(スタブは例外でなく型に合う誤った値を返す。例外で落ちるのは振る舞いを確かめていないため)。実装の前から通るなら、選んだ要求が新しい振る舞いかを人に確かめる",
    "base-test-failed": "人が見て、base のテストが head のコードで断言の失敗で落ちた理由を確かめる(テストか補助かデータを実装に合わせて変えていないか。意図した振る舞いの変更なら、その要求の版を上げる)。門ではなく知らせ",
    "f2p-not-passing": "実装前に断言の失敗で落ちたテストが、実装の後に通るようにする。通らないなら実装か要求を見直す(テストを弱めて通さない)",
    # 配り方: 写しの錠
    "vendor-drift": "検査器の写し・CI の雛形・人が写した包み・フックの登録を手で直さない。錠の後に写した物は錠に無いので、人が中身を見て載せる。更新は配る物から人が init --force で取り込み、直したなら人が中身を見て init --relock で写しの錠を書き直す(AI は錠を書かない)",
}

WEAKEN_EXIT = "AI は回避の書き換えをせず、理由を PR の本文に書いて人に回す"
TEST_CONTROL_SEVERITY = "error"  # test-control-added の重さ(使う側のリポの比べで誤検出があれば警告にする。AGENTS.md の決まり)
TEST_AWARE_SEVERITY = "warning"  # test-aware-code の重さ(正当な使い方が多く、検査器そのものの規則の文字列にも当たったので警告)


class UsageError(Exception):
    """使い方か道具の誤り(終了コード 2)。"""


class Finding(object):
    __slots__ = ("rule", "severity", "path", "line", "message", "key")

    def __init__(self, rule, path, line, message, key="", severity="error"):
        self.rule, self.path, self.line, self.message, self.key, self.severity = rule, path, line, message, key, severity

    def to_dict(self):
        return {
            "rule": self.rule,
            "severity": self.severity,
            "path": self.path,
            "line": self.line,
            "message": self.message,
            "help": RULES.get(self.rule, ""),
            "key": self.key,
        }

    def sort_key(self):
        return (self.rule, self.path or "", self.line or 0, self.key)


UNSAFE_CHARS = re.compile("[\x00-\x1f\x7f-\x9f\u200e\u200f\u202a-\u202e\u2066-\u2069]")


def esc(value):
    """外から来た文字列の制御文字と文字の向きの制御を逃がす(ファイルの名前で文字の報告を崩されないように)。"""
    return UNSAFE_CHARS.sub(lambda m: ("\\x%02x" if ord(m.group()) < 0x100 else "\\u%04x") % ord(m.group()), str(value))


def clip(value, n=80):
    """外から来た文字列を、制御文字を逃がして短く切る(検査の結果に指示を混ぜられないように)。"""
    s = esc(value)
    return s if len(s) <= n else s[: n - 1] + "…"


def sha256(data):
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.sha256(data).hexdigest()


# ---------------------------------------------------------------- 設定とファイル


GLOB_MAX_LEN = 256  # グロブの型の長さの上限(字)
GLOB_MAX_DEEP = 4  # 段の丸ごとの ** の数の上限


def glob_problem(pattern):
    """グロブの型が上限を超えていれば、その理由の文。超えていなければ None。"""
    if len(pattern) > GLOB_MAX_LEN:
        return "長さ %d 字が上限 %d 字を超える" % (len(pattern), GLOB_MAX_LEN)
    deep = pattern.split("/").count("**")
    if deep > GLOB_MAX_DEEP:
        return "段の丸ごとの ** が %d 個で、上限 %d 個を超える" % (deep, GLOB_MAX_DEEP)
    return None


@lru_cache(maxsize=4096)
def glob_segments(pattern):
    """型を段に分けた形。段は "**"(段の丸ごと)か、("eq", 文字) か、("glob", [塊の正規表現], [塊の長さ])。
    段の中の `**` は `*` と同じ(gitignore と同じ。続いた * は空の塊を挟むだけで、1つの * と同じに照らす)。塊は * の間の文字と ? だけで、長さが決まっている。"""
    problem = glob_problem(pattern)
    if problem:
        raise UsageError("グロブの型 %s は%s" % (clip(pattern, 60), problem))
    out = []
    for seg in pattern.split("/"):
        if seg == "**":
            out.append("**")
            continue
        if "*" not in seg and "?" not in seg:
            out.append(("eq", seg))
            continue
        chunks = seg.split("*")
        out.append(("glob", [re.compile("".join("[^/]" if c == "?" else re.escape(c) for c in ch)) for ch in chunks], [len(ch) for ch in chunks]))
    return tuple(out)


def segment_matches(seg, name):
    """1段の照らし。最初と最後の塊は端に合わせ、間の塊は左から最初に当たる所を探す(* だけの型では、これで足り、後戻りしない)。"""
    if seg[0] == "eq":
        return seg[1] == name
    rx, lens = seg[1], seg[2]
    if len(rx) == 1:
        return rx[0].fullmatch(name) is not None
    if lens[0] + lens[-1] > len(name) or not rx[0].match(name):
        return False
    pos, end = lens[0], len(name) - lens[-1]
    for r in rx[1:-1]:
        m = r.search(name, pos, end)
        if m is None:
            return False
        pos = m.end()
    return rx[-1].fullmatch(name, end) is not None


def glob_match(pattern, path):
    """道 path が型 pattern に当たるか。段ごとに照らす: 段の丸ごとの ** は0段以上(型の終わりの ** は1段以上)、* と ? は段を越えない。
    たどれる段の位置の集合で進めるので、時間は型の段の数と道の段の数の積に比例する。"""
    segs = glob_segments(pattern)
    parts = path.split("/")
    n, reach = len(parts), {0}
    for i, seg in enumerate(segs):
        if not reach:
            return False
        if seg == "**":
            lo = min(reach)
            if i == len(segs) - 1:
                return lo < n
            reach = set(range(lo, n + 1))
        else:
            reach = {j + 1 for j in reach if j < n and segment_matches(seg, parts[j])}
    return n in reach


def matches(path, patterns):
    return any(glob_match(p, path) for p in patterns)


# 値がグロブの型の鍵(長さと ** の数の上限を、読み込みの時点で確かめる)
GLOB_KEYS = ("requirements", "tests", "exclude", "test_config", "upstream", "baselines", "packages", "approvals", "adr")

# lock は前の版の要求の錠の置き場で、今は読まない(書いてある設定を誤りにしないために受け付ける)
CONFIG_KEYS = {
    "version": int,
    "requirements": list,
    "tests": list,
    "exclude": list,
    "lock": str,
    "approvals": str,
    "renames": str,
    "test_config": list,
    "upstream": list,
    "baselines": list,
    "parts": dict,
    "packages": list,
    "waivers": dict,
    "quarantine": dict,
    "require_sources": bool,
    "require_accept_parts": bool,  # 受け入れの字下げの形を必須にする(既定は無し。init の雛形は true)
    "sections": bool,
    "adr": list,  # 足した ADR の形を check --base で見る置き場(既定は無し。古い写しは知らない鍵で 2 になるので、写しを更新してから足す)
    "assert_helpers": dict,  # 言語ごとの断言のヘルパーの名前(既定は無し。古い写しは知らない鍵で 2 になるので、写しを更新してから足す)
}


def default_config():
    return {
        "version": CONFIG_VERSION,
        "requirements": list(DEFAULT_REQUIREMENTS),
        "tests": list(DEFAULT_TESTS),
        "exclude": list(DEFAULT_EXCLUDE),
        "approvals": DEFAULT_APPROVALS,
        "renames": DEFAULT_RENAMES,
        "test_config": list(DEFAULT_TEST_CONFIG),
        "upstream": [],
        "baselines": list(DEFAULT_BASELINES),
        "parts": {},
        "packages": [],
        "waivers": {},
        "adr": [],
        "quarantine": {},
        "assert_helpers": {},
        "_path": None,
    }


def parts_errors(value):
    if not isinstance(value, dict):
        return ["鍵 parts は、部品の名前から true か false へのオブジェクトにする"]
    out = []
    for name, on in value.items():
        if name not in PARTS:
            near = difflib.get_close_matches(name, list(PARTS), n=1)
            out.append("知らない部品 %s%s(部品は %s)" % (clip(name, 40), "(もしかして %s)" % near[0] if near else "", "・".join(PARTS)))
        elif not isinstance(on, bool):
            out.append("部品 %s の値は真偽値(true か false)にする" % name)
    return out


def waiver_errors(value):
    if not isinstance(value, dict):
        return ['鍵 waivers は、部品の名前から {"reason": 理由, "until": "YYYY-MM-DD"} へのオブジェクトにする']
    out = []
    for name, w in value.items():
        if name not in PARTS:
            out.append("waivers の知らない部品 %s(部品は %s)" % (clip(name, 40), "・".join(PARTS)))
        elif not (
            isinstance(w, dict)
            and isinstance(w.get("reason"), str)
            and w["reason"].strip()
            and isinstance(w.get("until"), str)
            and re.match(r"^\d{4}-\d{2}-\d{2}$", w["until"])
        ):
            out.append("waivers の %s には、空でない reason(理由)と until(期限、YYYY-MM-DD)を書く" % name)
    return out


def calendar_date(text):
    """YYYY-MM-DD の形で暦にある日なら date、そうでなければ None(2026-02-30 は形が合っても None)。"""
    import datetime

    if not isinstance(text, str) or not DATE_RE.match(text):
        return None
    try:
        return datetime.date.fromisoformat(text)
    except ValueError:
        return None


def quarantine_errors(value):
    """隔離の一覧: テストの道::名前 から {owner, since(始めた日), reason, extensions(延長の回数、任意)}。期限は持たない(日付で止めない)。
    since は暦にある日で、明日より後にしない(時差の1日は許す。未来の日は打ち間違いで、日数が負になるため)。"""
    import datetime

    if not isinstance(value, dict):
        return ['鍵 quarantine は、テストの道::名前 から {"owner": 持ち主, "since": "YYYY-MM-DD", "reason": 理由} へのオブジェクトにする']
    out = []
    tomorrow = datetime.date.today() + datetime.timedelta(days=1)
    for node, q in value.items():
        ok = isinstance(q, dict) and all(isinstance(q.get(k), str) and q[k].strip() for k in ("owner", "reason"))
        since = calendar_date(q.get("since")) if isinstance(q, dict) else None
        ext = q.get("extensions", 0) if isinstance(q, dict) else 0
        if not ok or since is None or not (isinstance(ext, int) and not isinstance(ext, bool) and ext >= 0):
            out.append(
                "quarantine の %s には、空でない owner(持ち主)と reason(理由)、since(始めた日、暦にある YYYY-MM-DD)、任意の extensions(0 以上の整数)を書く"
                % clip(node, 80)
            )
        elif since > tomorrow:
            out.append("quarantine の %s の since %s は未来の日。始めた日を書く(明日より後にしない)" % (clip(node, 80), q["since"]))
    return out


HELPER_NAME = re.compile(r"^[A-Za-z_$][\w$]*$")


def helper_errors(value):
    """断言のヘルパー: 言語(python・js・php・rust・go)から関数の名前の並びへ。名前は道や式を含まない識別子だけ。"""
    langs = sorted(set(LANG_BY_EXT.values()))
    if not isinstance(value, dict):
        return ['鍵 assert_helpers は、言語(%s)から関数の名前の配列へのオブジェクトにする(例 {"rust": ["assert_same"]})' % "・".join(langs)]
    out = []
    for lang, names in value.items():
        if lang not in langs:
            out.append("assert_helpers の知らない言語 %s(言語は %s)" % (clip(lang, 40), "・".join(langs)))
        elif not (isinstance(names, list) and all(isinstance(n, str) and HELPER_NAME.match(n) for n in names)):
            out.append("assert_helpers の %s は、関数の名前(識別子。道や括弧を含めない)の配列にする" % lang)
    return out


def part_on(cfg, name):
    """部品が有効か。書いていない部品と、parts の無い設定は、DEFAULT_OFF(ci)を除いて有効。"""
    v = (cfg.get("parts") or {}).get(name)
    return name not in DEFAULT_OFF if v is None else v is not False


def floor_parts(root, cfg):
    """下限の部品と、その理由(リポの事実から機械で出す。自己申告にしない)。切るには waivers に理由と期限が要る。"""
    out = {"weaken": "テストの弱めを人に回す守りは、いつも下限", "f2p": "実装前に落ちるかの確かめは、いつも下限"}
    if os.path.isfile(os.path.join(root, cfg["approvals"])):
        out["approvals"] = "承認の記録 %s がある" % cfg["approvals"]
    if cfg.get("upstream"):
        out["upstream"] = "上流の設定がある"
    return out


WAIVER_TARGETS = {
    "weaken": ("tests", "test_config"),
    "f2p": ("requirements", "tests"),
    "approvals": ("requirements", "approvals"),
    "upstream": ("upstream", "requirements"),
}


def only_waivers_changed(root, rel, base):
    """設定のファイル rel の base から head への変更が、waivers の鍵の中だけか(期限を延ばすだけの変更を、守る置き場の変更と数えないため)。"""
    try:
        before, after = json.loads(git_show(root, base, rel, check=False) or "null"), json.loads(read_text(root, rel) or "null")
    except ValueError:
        return False
    if not (isinstance(before, dict) and isinstance(after, dict)):
        return False
    before.pop("waivers", None)
    after.pop("waivers", None)
    return before == after


def waiver_touched(root, cfg, name, base):
    """切った部品が守る置き場(WAIVER_TARGETS の設定の鍵)のうち、base から変わったファイルの道(無ければ None)。
    設定のファイルの変更が waivers の中だけなら数えない(期限を延ばすだけの変更はテストに触れない)。"""
    globs = [g for k in WAIVER_TARGETS.get(name, ()) for g in ([cfg[k]] if isinstance(cfg.get(k), str) else cfg.get(k) or [])]
    rel = config_rel(root, cfg)
    for _, old, new in git_changes(root, base, None):
        if matches(new, globs) or matches(old, globs):
            if old == new == rel and only_waivers_changed(root, rel, base):
                continue
            return new
    return None


def check_floor(root, cfg, findings, base=None):
    import datetime

    today = datetime.date.today().isoformat()
    where = config_rel(root, cfg) or CONFIG_NAME
    base_waivers = (base_config(root, base, cfg).get("waivers") or {}) if base else {}
    for name, why in floor_parts(root, cfg).items():
        if part_on(cfg, name):
            continue
        w = (cfg.get("waivers") or {}).get(name)
        if w is None:
            findings.append(
                Finding("part-below-floor", where, None, "部品 %s は下限(%s)なのに切られている。切るなら waivers に理由と期限を書く" % (name, why), key=name)
            )
            continue
        # 期限は基点と head の早い側で判断する(検査される変更が、切れた期限を同じ PR で延ばして自分の検査を外せないように)
        b = base_waivers.get(name)
        until, extended = w["until"], ""
        if isinstance(b, dict) and isinstance(b.get("until"), str) and b["until"] < until:
            until, extended = b["until"], "。基点の期限で判断した(head では %s に延ばした。延ばすのはテストに触れない別の変更で)" % w["until"]
        if until < today:
            hit = waiver_touched(root, cfg, name, base) if base else None
            note = (
                "。この変更が切った部品の守る %s を変えた" % clip(hit, 80)
                if hit
                else "。知らせるだけ(切った部品が守る置き場を変えた PR の check --base で止まる)"
            )
            findings.append(
                Finding(
                    "waiver-expired",
                    where,
                    None,
                    "部品 %s を切る期限 %s が過ぎた(理由: %s)%s%s" % (name, until, clip(w["reason"], 80), extended, note),
                    key=name,
                    severity="error" if hit else "warning",
                )
            )


def recommendations(root):
    """リポの事実から出す推奨(下限より上の部品と役)。人が足すか決める。"""
    out = []
    remote = (git(root, "remote", check=False) or "").strip() if os.path.isdir(os.path.join(root, ".git")) else ""
    deploy = []
    wf = os.path.join(root, ".github", "workflows")
    for name in sorted(os.listdir(wf)) if os.path.isdir(wf) else []:
        if name.endswith((".yml", ".yaml")) and re.search(r"(?i)deploy|pages", read_text(wf, name) or ""):
            deploy.append(name)
    if remote or deploy:
        out.append(
            "推奨: CI で check と weaken を回す(%s)。費用と整える手間がかかるので人が決める。入れるなら trace-gate.json の parts.ci を true にし、サービス側の雛形のプラグインで CI を作る"
            % ("git の remote がある" if remote else "デプロイのワークフロー %s がある" % "・".join(deploy))
        )
    deps = []
    pj = read_bytes(os.path.join(root, "package.json"))
    try:
        if pj is not None and json.loads(pj.decode("utf-8")).get("dependencies"):
            deps.append("package.json")
    except (ValueError, UnicodeDecodeError, AttributeError):
        pass
    for name, rx in (
        ("Cargo.toml", r"(?m)^\[dependencies\]\s*\n\s*[A-Za-z0-9_-]+\s*="),
        ("pyproject.toml", r"(?m)^dependencies\s*=\s*\[\s*[\"']"),
        ("go.mod", r"(?m)^require\b"),
        ("composer.json", r'"require"\s*:\s*\{[^}]*"[a-z0-9-]+/[a-z0-9-]+"'),
    ):
        data = read_bytes(os.path.join(root, name))
        if data is not None and re.search(rx, data.decode("utf-8", "replace")):
            deps.append(name)
    if deps:
        out.append("推奨: 依存のライセンスを確かめる役を入れる(依存の宣言 %s がある。読むだけで費用はほぼ無い)" % "・".join(deps))
    ui = sorted(n for n in os.listdir(root) if re.match(r"^(playwright|cypress)\.config\.[a-z]+$", n)) if os.path.isdir(root) else []
    if ui:
        out.append(
            "推奨: E2E を段に分けて回す(画面のテストの道具の設定 %s がある。PR では変えた要求の stest と smoke の数本、全部はマージの前か夜間)" % "・".join(ui)
        )
    return out


def parts_off(cfg):
    return [p for p in PARTS if not part_on(cfg, p)]


ROOT_ONLY_KEYS = ("parts", "packages", "waivers")  # 部品とパッケージの一覧は根の設定だけが持つ


def tree_dirs(root):
    out = set()
    for d, dirs, _ in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
        rel = os.path.relpath(d, root).replace(os.sep, "/")
        if rel != ".":
            out.add(rel)
    return out


def package_dirs(root, cfg):
    """根の設定の packages の glob に当たるフォルダ(root からの道)。当たらない glob は誤り(打ち間違いで黙って何も調べないのを防ぐ)。"""
    pats = cfg.get("packages") or []
    if not pats:
        return []
    dirs, found = tree_dirs(root), []
    for pat in pats:
        clean = pat.strip("/")
        if pat.startswith("/") or ".." in clean.split("/"):
            raise UsageError("packages の %s はリポの中の相対の道にする" % clip(pat, 60))
        hits = sorted(x for x in dirs if glob_match(clean, x))
        if not hits:
            raise UsageError("packages の %s に当たるフォルダが無い" % clip(pat, 60))
        found.extend(hits)
    return uniq(found)


def package_config(root, pkg, cfg):
    """パッケージのフォルダの trace-gate.json(無ければ既定)。部品は根の設定のものを使う。"""
    proot = os.path.join(root, *pkg.split("/"))
    if os.path.realpath(proot) != os.path.join(os.path.realpath(root), *pkg.split("/")):
        raise UsageError("パッケージ %s はリポの中の本物のフォルダにする(シンボリックリンクは使わない)" % pkg)
    path = os.path.join(proot, CONFIG_NAME)
    if os.path.isfile(path):
        try:
            raw = json.loads(read_text(proot, CONFIG_NAME))
        except ValueError:
            raw = {}
        bad = [k for k in ROOT_ONLY_KEYS if isinstance(raw, dict) and k in raw]
        if bad:
            raise UsageError("パッケージの設定 %s/%s に %s は書けない(部品とパッケージの一覧は根の設定だけが持つ)" % (pkg, CONFIG_NAME, "・".join(bad)))
    pcfg = load_config(proot)
    pcfg["parts"] = dict(cfg.get("parts") or {})
    return proot, pcfg


def scopes(root, cfg, pkgs):
    """調べる範囲の一覧 [(道の頭, 根, 設定)]。根は、要求の正本があるときだけ、パッケージのフォルダを外して調べる。"""
    out = []
    rcfg = dict(cfg, exclude=list(cfg["exclude"]) + ["%s/**" % p for p in pkgs])
    if requirement_files(root, rcfg):
        out.append(("", root, rcfg))
    for p in pkgs:
        proot, pcfg = package_config(root, p, cfg)
        out.append((p + "/", proot, pcfg))
    return out


def prefixed(findings, head):
    for f in findings:
        f.path = head + f.path if f.path else (head.rstrip("/") or None)
    return findings


def gate_part_on(root, cfg, base, name):
    """base を取る命令の部品の判定。両方の側で切られた部品だけを切る(PR の中で切っても、その PR 自身は切る前の検査を通る)。"""
    if part_on(cfg, name) or not base:
        return part_on(cfg, name)
    return part_on(base_config(root, base, cfg), name)


def load_config(root, path=None):
    cpath = path or os.path.join(root, CONFIG_NAME)
    if not os.path.isfile(cpath):
        if path:
            raise UsageError("設定のファイルが無い: %s" % clip(path))
        return default_config()
    try:
        with open(cpath, encoding="utf-8") as f:
            text = f.read()
    except OSError as e:
        raise UsageError("設定のファイルを読めない: %s(%s)" % (clip(cpath), clip(e, 120))) from e
    return config_from_text(text, cpath)


def config_from_text(text, cpath, base=False):
    """設定の文を読む。base は base の側の設定で、形の誤り(知らない鍵・型)だけを止め、鍵どうしの条件では止めない
    (古い検査器で正しかった base の設定を、後から足した条件で止めると、直す PR が通らないため)。"""
    cfg = default_config()
    try:
        data = json.loads(text)
    except ValueError as e:
        raise UsageError("設定のファイルを読めない: %s(%s)" % (clip(cpath), clip(e, 120))) from e
    if not isinstance(data, dict):
        raise UsageError("設定のファイルは JSON のオブジェクトにする: %s" % clip(cpath))
    version = data.get("version", CONFIG_VERSION)
    if not isinstance(version, int) or version > CONFIG_VERSION:
        raise UsageError("設定の version %s はこの道具(%s %s)より新しいか不正。道具を更新して" % (clip(version), TOOL_NAME, TOOL_VERSION))
    errors = []
    for key, value in data.items():
        if key == "$schema" or key.startswith("x-"):
            continue
        if key not in CONFIG_KEYS:
            near = difflib.get_close_matches(key, list(CONFIG_KEYS), n=1)
            errors.append(
                "知らない鍵 %s%s(この道具 %s %s が古いなら、更新すると読める鍵かもしれない)"
                % (clip(key, 40), "(もしかして %s)" % near[0] if near else "", TOOL_NAME, TOOL_VERSION)
            )
            continue
        want = CONFIG_KEYS[key]
        if key == "parts":
            errors.extend(parts_errors(value))
            if isinstance(value, dict):
                cfg[key] = value
            continue
        if key == "waivers":
            errors.extend(waiver_errors(value))
            if isinstance(value, dict):
                cfg[key] = value
            continue
        if key == "quarantine":
            errors.extend(quarantine_errors(value))
            if isinstance(value, dict):
                cfg[key] = value
            continue
        if key == "assert_helpers":
            problems = helper_errors(value)
            errors.extend(problems)
            if not problems:
                cfg[key] = value
            continue
        if not isinstance(value, want) or (want is list and not all(isinstance(x, str) for x in value)):
            errors.append("鍵 %s は %s にする" % (key, "文字列の配列" if want is list else want.__name__))
            continue
        if key in GLOB_KEYS:
            for pattern in value if want is list else [value]:
                problem = glob_problem(pattern)
                if problem:
                    errors.append("鍵 %s のグロブ %s は%s" % (key, clip(pattern, 60), problem))
        cfg[key] = value
    if not base and cfg.get("require_sources") and not cfg.get("upstream"):
        errors.append("require_sources は upstream(根拠の文書の置き場)があるときだけ true にできる(無いと照らされない up~名前 を書かされる)")
    if errors:
        raise UsageError("設定の誤り(%s): %s" % (clip(cpath), "; ".join(errors)))
    cfg["_path"] = cpath
    return cfg


def walk_files(root):
    out = []
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
        for name in sorted(files):
            out.append(os.path.relpath(os.path.join(d, name), root).replace(os.sep, "/"))
    return out


def has_glob(p):
    return any(c in p for c in "*?[")


def select(paths, patterns, exclude=()):
    return [p for p in paths if matches(p, patterns) and not matches(p, exclude)]


def read_text(root, rel):
    with open(os.path.join(root, rel), encoding="utf-8", errors="replace") as f:
        return f.read()


# ---------------------------------------------------------------- git


def git(root, *args, check=True):
    try:
        r = subprocess.run(["git", "-C", root] + list(args), capture_output=True)
    except OSError as e:
        raise UsageError("git を起動できない: %s" % clip(e)) from e
    if check and r.returncode != 0:
        raise UsageError("git %s が失敗した: %s" % (args[0], clip(r.stderr.decode("utf-8", "replace").strip(), 200)))
    return r.stdout.decode("utf-8", "replace") if r.returncode == 0 else None


_PREFIX = {}


def git_prefix(root):
    """git の根から root までの道(根なら空、下のフォルダなら "sub/" の形)。git の道を root からの道にそろえるのに使う。"""
    if root not in _PREFIX:
        _PREFIX[root] = git(root, "rev-parse", "--show-prefix").rstrip("\n")
    return _PREFIX[root]


def subdir_prefix(root):
    """git の根から root までの道。git のリポでなければ空(init や錠の書きは git の外でも動く)。"""
    out = git(root, "rev-parse", "--show-prefix", check=False)
    return (out or "").rstrip("\n")


def git_files_at(root, ref):
    """ref の側のファイルを root からの道で返す。root の外のファイルは含めない。"""
    prefix = git_prefix(root)
    out = git(root, "ls-tree", "-r", "-z", "--name-only", "--full-tree", ref)
    return sorted(x[len(prefix) :] for x in out.split("\0") if len(x) > len(prefix) and x.startswith(prefix))


_SHOWN = {}


def git_show(root, ref, path, check=True):
    """ref の側の、root からの道 path の中身。1回の実行の中では、同じ版の同じファイルを2度読まない。
    覚えるのはこの実行の中だけ(実行をまたいで結果を残すと、古い結果を出す害があるため)。"""
    key = (root, ref, path)
    if key not in _SHOWN or (_SHOWN[key] is None and check):
        _SHOWN[key] = git(root, "show", "%s:%s%s" % (ref, git_prefix(root), path), check=check)
    return _SHOWN[key]


# ---------------------------------------------------------------- 要求の正本(TG-1)


class Req(object):
    __slots__ = (
        "name",
        "version",
        "priority",
        "title",
        "why",
        "text",
        "accepts",
        "accept",
        "accept_forms",
        "path",
        "line",
        "fingerprint",
        "needs",
        "treatment",
        "reason",
        "sources",
        "ups",
        "up_titles",
        "dupes",
    )

    @property
    def id(self):
        return "req~%s~%d" % (self.name, self.version)


KINDS = ("utest", "itest", "stest", "ctest")
KIND_ALT = "|".join(KINDS)
TREATMENTS = ("踏襲", "変更", "追加", "廃止")
FENCE_RE = re.compile(r"^\s*(```|~~~)")
# 要求の区切り: 題の見出し(### か ####)・ID の行(名前・版・優先度だけ)・欄の箇条(1欄1行)
REQ_HEAD = re.compile(r"^#{3,4}[ \t]+(?P<title>.*?)(?:[ \t]+#+)?[ \t]*$")
ANY_HEAD = re.compile(r"^\s{0,3}#{1,6}(?:[ \t]|$)")
ID_HINT = re.compile(r"^\s*`req~")
ID_LINE = re.compile(r"^\s*`req~(?P<name>" + NAME + r")~(?P<ver>[1-9][0-9]*)`[ \t]+(?P<prio>P[0-2])(?:[ \t]+(?P<origin>【[^】]*】))?[ \t]*$")
BULLET = re.compile(r"^\s*[-*+][ \t]+(?P<body>.*?)\s*$")
FIELD = re.compile(r"^(?P<label>[^\s:：`]+)[:：][ \t]*(?P<value>.*)$")
LABELS = ("なぜ", "受け入れ", "確かめ", "扱い", "出典")
# 欄の英語の名前(正の形)と、中で使う日本語の名前の対。日本語の名前も同じ欄として読む(convert が英語の名前に替える)
LABEL_ALIAS = {
    "Rationale": "なぜ",
    "Acceptance": "受け入れ",
    "Needs": "確かめ",
    "Disposition": "扱い",
    "Input": "出典",
    "Decider": "決める人",
    "Due": "期限",
    "Provisional": "仮",
    "Basis": "根拠",
    "Kind": "区分",
    "Given": "前提",
    "When": "もし",
    "Then": "ならば",
}
LABEL_EN = {ja: en for en, ja in LABEL_ALIAS.items()}


def canon_label(label):
    """欄の名前を、中で使う日本語の名前にそろえる(英語の名前は対の日本語の名前に替え、ほかはそのまま)。"""
    return LABEL_ALIAS.get(label, label)


NEEDS_VALUE = re.compile(r"^((?:%s)(?:\s*[・,、]\s*(?:%s))*)$" % (KIND_ALT, KIND_ALT))
TREAT_VALUE = re.compile(r"^(踏襲|変更|追加|廃止)\s*(?:[(（]\s*([^()（）\n]*?)\s*[)）])?$")
ENDS = ("。", ".", "!", "?", "！", "？")
OPEN_PAREN = {"(": ")", "（": "）"}


def req_text(title, why):
    """要求の文(指紋の元)。題に「(なぜ)」と終わりの「。」を足す(1行の形の `文(なぜ)。` と同じ文字列にするため)。"""
    text = title.strip() + ("(%s)" % why.strip() if why else "")
    return text if text.endswith(ENDS) else text + "。"


def up_refs(text):
    """文の中の up~名前 の名前の組(終わりの . は外す)。"""
    return {x.rstrip(".") for x in UP_REF.findall(text or "") if x.rstrip(".")}


def up_titles(text):
    """出典の `up~名前(題)` の {名前: 題}。題の括弧は同じ種類の括弧の数で閉じを探す(題の中の括弧を許す)。"""
    out = {}
    for m in UP_REF.finditer(text or ""):
        name, at = m.group(1).rstrip("."), m.end()
        close = OPEN_PAREN.get(text[at : at + 1])
        if not name or not close:
            continue
        depth = 0
        for k in range(at, len(text)):
            if text[k] == text[at]:
                depth += 1
            elif text[k] == close:
                depth -= 1
                if depth == 0:
                    out[name] = text[at + 1 : k].strip()
                    break
    return out


def fingerprint(text, accepts, retired=False):
    """要求の文と受け入れ基準の指紋。版・優先度と、前の版の【…】の印は含めない。
    受け入れ基準が複数なら改行で連ねる(1つのときは 0.11 までの1行の形と同じ値になる)。
    扱いが廃止のときだけ、そのことも含める(廃止にすると P0 の検査から外れるので、版を上げて承認し直させる)。"""
    if isinstance(accepts, str):
        accepts = [accepts]
    body = re.sub(r"\s+", " ", text.strip()) + "\n" + "\n".join(re.sub(r"\s+", " ", a.strip()) for a in accepts)
    return sha256(body + ("\n扱い: 廃止" if retired else ""))[:16]


def deprecated_mark(mark, path, no, findings):
    """【…】の印は前の版の出所の印。警告にして読み飛ばす(文と指紋に含めない)。"""
    if mark is not None:
        findings.append(Finding("origin-deprecated", path, no, "【…】の印 %s はもう使わない(読み飛ばした)" % clip(mark, 20), severity="warning"))


def block_boundary(line):
    """区切りの終わり: 次の見出し・ID の行(要求か未確定事項)・1行の形の行・上流の項目・コードの柵。"""
    return bool(
        FENCE_RE.match(line)
        or ANY_HEAD.match(line)
        or ID_HINT.match(line)
        or OPEN_ID_HINT.match(line)
        or REQ_HINT.match(line)
        or OPEN_HINT.match(line)
        or UP_HINT.match(line)
    )


ACCEPT_PARTS = ("前提", "もし", "ならば")
ACCEPT_JOIN = " → "


def indent_of(line):
    """行の頭の字下げの幅(タブは4つの空白と数える)。"""
    s = line.expandtabs(4)
    return len(s) - len(s.lstrip())


def starts_accept_parts(lines, i, parent_indent):
    """i の行が、`- 受け入れ:` より深く字下げした `- 前提:` などの箇条か(字下げの形の始まりか)。
    それ以外の字下げ(折り返し・知らない箇条)は今までどおり read_fields が誤りにする。"""
    if i >= len(lines) or not lines[i].strip() or indent_of(lines[i]) <= parent_indent:
        return False
    b = BULLET.match(lines[i])
    f = FIELD.match(b.group("body")) if b else None
    return bool(f and canon_label(f.group("label")) in ACCEPT_PARTS)


def read_accept_parts(lines, i, parent_indent, no, errors):
    """`- Acceptance:` の行(no)の次(i)から、それより深く字下げした `- Given:`・`- When:`・`- Then:`(日本語の名前 前提・もし・ならば も読む)を読む。
    (次に読む行, 基準の文 か None)。字下げした行が無ければ i をそのまま返す。
    3行とも書き、前の状態と操作の無い要求は Given と When を空のまま書く。Then は空にできない。
    基準の文は、空でない部分を「前提 → もし → ならば」の順に → でつなぐ(1行に → で書いた形と同じ文字列なので、形を移しても指紋は変わらない。
    Given と When が空なら Then の文だけになり、1行の文の受け入れ基準と同じ指紋になる)。"""
    parts, order, start = {}, [], i
    while i < len(lines):
        line = lines[i]
        if not line.strip() or block_boundary(line) or indent_of(line) <= parent_indent:
            break
        sno = i + 1
        i += 1
        b = BULLET.match(line)
        f = FIELD.match(b.group("body")) if b else None
        if not f or canon_label(f.group("label")) not in ACCEPT_PARTS:
            errors.append(
                (
                    sno,
                    "Acceptance: の下の字下げは %s の箇条だけ(日本語の名前 %s も読む): %s"
                    % ("・".join(LABEL_EN[x] + ":" for x in ACCEPT_PARTS), "・".join(x + ":" for x in ACCEPT_PARTS), clip(line.strip())),
                )
            )
            continue
        shown, value = f.group("label"), f.group("value").strip()
        label = canon_label(shown)
        if not value and label == ACCEPT_PARTS[-1]:
            errors.append((sno, "%s: の中身が無い(Then は空にできない)" % shown))
            continue
        if label in parts:
            errors.append((sno, "%s: は1つの受け入れに1つだけ書く(場面が複数なら `- Acceptance:` を分ける)" % shown))
            continue
        parts[label] = value
        order.append(label)
    if i == start:
        return i, None
    missing = [x for x in ACCEPT_PARTS if x not in parts]
    if missing:
        errors.append(
            (
                no,
                "Acceptance: の字下げの形に %s が無い(%s を1行ずつ書き、前の状態と操作が無ければ Given と When は空のまま書く)"
                % ("・".join(LABEL_EN[x] + ":" for x in missing), " → ".join(LABEL_EN[x] for x in ACCEPT_PARTS)),
            )
        )
        return i, None
    if order != list(ACCEPT_PARTS):
        errors.append((no, "Acceptance: の字下げは %s の順に書く" % " → ".join(LABEL_EN[x] for x in ACCEPT_PARTS)))
        return i, None
    return i, ACCEPT_JOIN.join(parts[x] for x in ACCEPT_PARTS if parts[x])  # 空の Given・When は除く(Then だけなら1行の文と同じ指紋)


def read_fields(lines, i, errors, labels=LABELS, multi=("受け入れ",)):
    """ID の行の次(i)から欄の箇条を読む。(次に読む行, 欄の辞書)。誤りは errors に (行, 文) で足す。
    labels は許すラベル、multi は2つ以上書けるラベル(値は配列)。
    区切りは次の見出し・空の行の後の箇条の外の行・別の行(ID の行・上流の項目・1行の形)で終わる。"""
    fields = {k: [] for k in multi}
    prev_field = True  # ID の行の直後の文も、欄の折り返しと同じく読まない
    while i < len(lines):
        line, no = lines[i], i + 1
        if not line.strip():
            prev_field = False
            i += 1
            continue
        if block_boundary(line):
            break
        b = BULLET.match(line)
        if not b:
            if not prev_field:
                break
            errors.append((no, "区切りの中の箇条の外の行(欄は `- ラベル: 中身` の1行に書き、折り返さない): %s" % clip(line.strip())))
            i += 1
            continue
        prev_field = True
        i += 1
        f = FIELD.match(b.group("body"))
        shown = f.group("label") if f else None
        label = canon_label(shown) if f else None
        if label not in labels:
            if label in ACCEPT_PARTS and "受け入れ" in multi:
                errors.append((no, "%s: は `- Acceptance:` の下に字下げして書く: %s" % (shown, clip(line.strip()))))
            else:
                errors.append(
                    (
                        no,
                        "知らないラベルの欄(ラベルは %s のどれか。日本語の名前 %s も読む): %s"
                        % ("・".join(LABEL_EN[x] + ":" for x in labels), "・".join(x + ":" for x in labels), clip(line.strip())),
                    )
                )
            continue
        value = f.group("value").strip()
        if label == "受け入れ" and label in multi and starts_accept_parts(lines, i, indent_of(line)):
            # 字下げの形(前提・もし・ならば)。読めなかったときは None を置き、「受け入れ: の欄が無い」を重ねて出さない
            i, joined = read_accept_parts(lines, i, indent_of(line), no, errors)
            if value:
                errors.append((no, "Acceptance: に1行の文と字下げの %s を両方書かない" % "・".join(LABEL_EN[x] + ":" for x in ACCEPT_PARTS)))
                joined = None
            fields[label].append(joined)
            fields.setdefault("_forms", []).append("parts")
            continue
        if not value:
            errors.append((no, "%s: の中身が無い" % shown))
            continue
        if label in multi:
            fields[label].append(value)
            if label == "受け入れ":
                fields.setdefault("_forms", []).append("line")
            continue
        if label in fields:
            errors.append((no, "%s: は1つの区切りに1つだけ書く(%s: と %s: は同じ欄)" % (shown, LABEL_EN[label], label)))
            continue
        if label == "区分" and value not in OPEN_KINDS:
            errors.append((no, "区分: は %s のどちらか: %s" % ("・".join(OPEN_KINDS), clip(value))))
            continue
        if label == "確かめ" and not NEEDS_VALUE.match(value):
            errors.append((no, "確かめ: は %s を・で並べる: %s" % ("・".join(KINDS), clip(value))))
            continue
        if label == "扱い":
            m = TREAT_VALUE.match(value)
            if not m:
                errors.append((no, "扱い: は 踏襲・変更・追加・廃止(理由) のどれか: %s" % clip(value)))
                continue
            if m.group(1) == "廃止" and not (m.group(2) or "").strip():
                errors.append((no, "扱い: 廃止 には (理由) が要る(例: 扱い: 廃止(使われていない。2026-11-01 の会議))"))
                continue
        fields[label] = value
    return i, fields


STRAY_INDENT = re.compile(r"^(?: {2,}|\t)\S")


def stray_text(lines, i):
    """欄の読みが止まった行(i。空の行は read_fields が読み飛ばしている)が、字下げした箇条でない行(段落・表・コードの柵)ならその行の番号、でなければ None。
    CommonMark では、欄の後に字下げした行は直前の箇条(受け入れの項目)の中身になるが、区切りの読みでは欄に入らない。"""
    if i >= len(lines):
        return None
    line = lines[i]
    if not STRAY_INDENT.match(line) or BULLET.match(line) or ANY_HEAD.match(line) or ID_HINT.match(line) or OPEN_ID_HINT.match(line):
        return None
    return i + 1


def parse_block(lines, i, head, path, findings):
    """ID の行(i)から1件の要求を読む。(次に読む行, Req か None)。head は直前の見出し (題, 行) か None。"""
    line, no = lines[i], i + 1
    errors = []
    m = ID_LINE.match(line)
    if not m:
        errors.append((no, "ID の行の形が崩れている(`req~名前~版` と P0〜P2 だけを書く): %s" % clip(line.strip())))
    elif head is None:
        errors.append((no, "ID の行の前に、要求の題の見出し(### か ####)が無い: %s" % clip(line.strip())))
    elif not head[0]:
        errors.append((head[1], "要求の題の見出しに文が無い"))
    nxt, fields = read_fields(lines, i + 1, errors)
    if m and not fields["受け入れ"]:
        errors.append((no, "受け入れ: の欄が1つも無い: %s" % clip(line.strip())))
    for eno, msg in errors:
        findings.append(Finding("req-malformed", path, eno, msg))
    if errors:
        return nxt, None
    deprecated_mark(m.group("origin"), path, no, findings)
    stray = stray_text(lines, nxt)
    if stray:
        findings.append(
            Finding(
                "block-stray-text",
                path,
                stray,
                "要求 %s の欄の後に、空の行を挟んで字下げした文がある(指紋に入らない): %s" % (m.group("name"), clip(lines[stray - 1].strip())),
                severity="warning",
            )
        )
    r = Req()
    r.name, r.version, r.priority = m.group("name"), int(m.group("ver")), m.group("prio")
    r.title, r.why, r.path, r.line = head[0], fields.get("なぜ"), path, no
    r.text = req_text(r.title, r.why)
    r.accepts = fields["受け入れ"]
    r.accept_forms = fields.get("_forms", [])
    r.accept = "\n".join(r.accepts)
    r.needs = frozenset(re.findall(KIND_ALT, fields["確かめ"])) if "確かめ" in fields else None
    t = TREAT_VALUE.match(fields["扱い"]) if "扱い" in fields else None
    r.treatment, r.reason = (t.group(1), (t.group(2) or "").strip() or None) if t else (None, None)
    r.sources = fields.get("出典")
    r.ups, r.up_titles = up_refs(r.sources), up_titles(r.sources)
    r.fingerprint = fingerprint(r.text, r.accepts, r.treatment == "廃止")
    r.dupes = []  # 同じ名前の2つ目から後ろの (道, 行)。あれば名前は曖昧で、どちらも確かめたと数えない
    return nxt, r


def open_text(o):
    """未確定事項の、up~名前 を探す文(題と、根拠・仮・決める人の欄)。"""
    return " ".join(x for x in (o.get("text"), o.get("basis"), o.get("provisional"), o.get("decider")) if x)


def parse_open_block(lines, i, head, path, findings):
    """ID の行(i)から1件の未確定事項を読む。(次に読む行, 辞書 か None)。head は直前の見出し (題, 行) か None。"""
    line, no = lines[i], i + 1
    errors = []
    m = OPEN_ID_LINE.match(line)
    if not m:
        errors.append((no, "ID の行の形が崩れている(`tbd~名前` だけを書く): %s" % clip(line.strip())))
    elif head is None:
        errors.append((no, "ID の行の前に、未確定事項の題(決めていないこと)の見出し(### か ####)が無い: %s" % clip(line.strip())))
    elif not head[0]:
        errors.append((head[1], "未確定事項の題の見出しに文が無い"))
    nxt, fields = read_fields(lines, i + 1, errors, OPEN_LABELS, ())
    if m and "決める人" not in fields:
        errors.append((no, "決める人: の欄が無い: %s" % clip(line.strip())))
    for eno, msg in errors:
        findings.append(Finding("open-malformed", path, eno, msg))
    if errors:
        return nxt, None
    deprecated_mark(m.group("origin"), path, no, findings)
    due = fields.get("期限")
    return nxt, {
        "id": "tbd~" + m.group("name"),
        "text": head[0],
        "decider": fields["決める人"],
        "due": due if due and DATE_RE.match(due) else None,
        "due_text": due,
        "provisional": fields.get("仮"),
        "basis": fields.get("根拠"),
        "kind": fields.get("区分"),
        "path": path,
        "line": no,
    }


def bullet_labels(lines, i):
    """i の行から続く箇条(空の行を挟んでよい)の欄のラベルの組。"""
    out = set()
    while i < len(lines) and (not lines[i].strip() or BULLET.match(lines[i])) and not block_boundary(lines[i]):
        b = BULLET.match(lines[i])
        f = FIELD.match(b.group("body")) if b else None
        if f:
            out.add(canon_label(f.group("label")))
        i += 1
    return out


def parse_requirements(text, path, findings):
    """要求と未確定事項の区切り(見出し・ID の行・欄)を読む。1行の形の行は読まずに req-malformed・open-malformed にする。"""
    reqs, opens, fence, head = [], [], False, None
    # 行は改行だけで切る。splitlines は U+2028・U+0085・改頁でも割り、行の番号がずれて要求の文が切れる
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line, no = lines[i], i + 1
        if FENCE_RE.match(line):
            fence, head = not fence, None
            i += 1
            continue
        if fence:
            i += 1
            continue
        if ID_HINT.match(line):
            i, r = parse_block(lines, i, head, path, findings)
            if r is not None:
                reqs.append(r)
            head = None
            continue
        if OPEN_ID_HINT.match(line):
            i, o = parse_open_block(lines, i, head, path, findings)
            if o is not None:
                opens.append(o)
            head = None
            continue
        i += 1
        if not line.strip():
            continue
        m = REQ_HEAD.match(line)
        if m:
            head = (m.group("title").strip(), no)
            continue
        if REQ_HINT.match(line):
            findings.append(Finding("req-malformed", path, no, "1行の形の要求の行はもう読まない(convert で新しい形に移す): %s" % clip(line.strip())))
        elif OPEN_HINT.match(line):
            findings.append(Finding("open-malformed", path, no, "1行の形の未確定事項の行はもう読まない(convert で新しい形に移す): %s" % clip(line.strip())))
        elif head is not None:
            b = BULLET.match(line)
            f = FIELD.match(b.group("body")) if b else None
            if f and canon_label(f.group("label")) in LABELS:
                findings.append(Finding("req-malformed", path, head[1], "見出しの後に ID の行(`req~名前~版` P0)が無く、欄だけがある: %s" % clip(line.strip())))
                while i < len(lines) and (not lines[i].strip() or BULLET.match(lines[i])) and not block_boundary(lines[i]):
                    i += 1
            elif f and canon_label(f.group("label")) in OPEN_LABELS and "決める人" in bullet_labels(lines, i - 1):  # 決める人の無い - 根拠: などは普通の箇条
                findings.append(Finding("open-malformed", path, head[1], "見出しの後に ID の行(`tbd~名前`)が無く、欄だけがある: %s" % clip(line.strip())))
                while i < len(lines) and (not lines[i].strip() or BULLET.match(lines[i])) and not block_boundary(lines[i]):
                    i += 1
        head = None
    return reqs, opens


ADR_SECTIONS = ("背景", "決定", "理由", "選ばなかった案", "影響", "見直しの条件", "確かめ方")  # templates/adr.md の見出し(テストが雛形と照らす)
ADR_STATE_LINE = re.compile(r"^\s*[-*]\s+状態[:：]\s*(?P<value>.*?)\s*$")
ADR_STATE_VALUE = re.compile(r"^(?P<state>提案|採用|却下|置き換え)(?:\s*[(（]\s*→\s*(?P<to>[^()（）\s][^()（）]*?)\s*[)）])?$")
ADR_REPLACED_BY = re.compile(r"^\s*[-*]\s+置き換え先[:：]\s*(?P<to>\S.*?)\s*$")


def adr_number(target):
    """置き換え先が番号を持つか(雛形の {NNNN} のままの置き場所は番号でない)。"""
    return bool(target) and re.search(r"[0-9]", target) is not None and re.search(r"[{}]", target) is None


def adr_problems(text):
    """ADR の文の形の誤り [(行, 文)]: 状態の行の欠けと値、置き換えの置き換え先、雛形の見出しの欠け。中身は見ない。"""
    out, fence, heads, state, replaced_by = [], False, set(), None, False
    for no, line in enumerate(text.split("\n"), 1):
        if FENCE_RE.match(line):
            fence = not fence
            continue
        if fence:
            continue
        m = SPEC_SECTION_RE.match(line)
        if m:
            heads.add(m.group(1))
            continue
        m = ADR_STATE_LINE.match(line)
        if m and state is None:
            state = (no, m.group("value"))
        m = ADR_REPLACED_BY.match(line)
        if m and adr_number(m.group("to")):
            replaced_by = True
    if state is None:
        out.append((1, "状態の行(- 状態: 提案|採用|却下|置き換え)が無い"))
    else:
        v = ADR_STATE_VALUE.match(state[1])
        if not v:
            out.append((state[0], "状態 %s は 提案・採用・却下・置き換え のどれか1つにする" % clip(state[1], 40)))
        elif v.group("state") == "置き換え" and not adr_number(v.group("to")) and not replaced_by:
            out.append((state[0], "状態が置き換えなのに、番号の置き換え先(- 置き換え先: 番号 か 置き換え(→ 番号)。雛形の {NNNN} のままは番号でない)が無い"))
    missing = [h for h in ADR_SECTIONS if h not in heads]
    if missing:
        out.append((1, "雛形の見出しが無い: %s" % "・".join("## " + h for h in missing)))
    return out


def adr_parts(text):
    """ADR の (状態, 状態の行, 決定の節の文(空白を寄せた), 決定の見出しの行)。無いものは None。"""
    state = state_line = decision_line = None
    body, inside, fence = [], False, False
    for no, line in enumerate(text.split("\n"), 1):
        if FENCE_RE.match(line):
            fence = not fence
        m = None if fence else SPEC_SECTION_RE.match(line)
        if m:
            inside = m.group(1) == "決定"
            if inside:
                decision_line = no
            continue
        if inside:
            body.append(line)
        s = None if fence else ADR_STATE_LINE.match(line)
        if s and state is None:
            v = ADR_STATE_VALUE.match(s.group("value"))
            state, state_line = (v.group("state") if v else s.group("value")), no
    return state, state_line, (squash("\n".join(body)) if decision_line else None), decision_line


def adr_changes(root, args, bcfg, cfg, changes):
    """設定の adr(基点と head の和)に当たる、基点で状態が採用の ADR の決定の節の書き換え・消し・置き換えでない状態への変更(adr-changed)。"""
    globs = uniq(list(bcfg.get("adr") or []) + list(cfg.get("adr") or []))
    if not globs:
        return []
    out = []
    for st, old, new in changes:
        if not matches(old, globs) or st == "A":
            continue
        before = adr_parts(read_side(root, args.base, old) or "")
        if before[0] != "採用":
            continue
        if st == "D" or not matches(new, globs):
            out.append(Finding("adr-changed", old, None, "採用の ADR %s が消えたか、ADR の置き場の外へ移った" % old, key=old))
            continue
        after = adr_parts(read_side(root, args.head, new) or "")
        if after[2] != before[2]:
            out.append(Finding("adr-changed", new, after[3], "採用の ADR %s の決定の節が書き換わった" % new, key=new))
        elif after[0] not in ("採用", "置き換え"):
            out.append(Finding("adr-changed", new, after[1], "採用の ADR %s の状態が %s に変わった" % (new, clip(after[0] or "無し", 20)), key=new))
    return out


def check_adr_added(root, cfg, base, files, findings):
    """設定の adr に当たる ADR のうち、base に無い足したものだけの形を見る(既にある ADR と、中身の同じ名前の変更は見ない)。"""
    if not cfg.get("adr"):
        return
    base_files = git_files_at(root, base)
    head = select(files, cfg["adr"], cfg["exclude"])
    gone = [p for p in select(base_files, cfg["adr"], cfg["exclude"]) if p not in set(head)]
    known = set(base_files)
    for p in head:
        if p in known:
            continue
        text = read_text(root, p)
        if any(git_show(root, base, g, check=False) == text for g in gone):
            continue
        for no, msg in adr_problems(text):
            findings.append(Finding("adr-malformed", p, no, "ADR %s の%s" % (p, msg), key="%s:%s" % (p, msg[:20])))


SECTION_ORDER = ["目的", "範囲外", "用語", "要求", "未確定事項"]  # 主題のファイルの雛形の見出しの並び
REQUIRED_SECTIONS = ("目的", "要求")
SPEC_SECTION_RE = re.compile(r"^##(?!#)\s+(.+?)\s*$")


def check_sections(text, path, findings):
    """要求の ID の行か未確定事項の行を持つファイルが、雛形の見出しの並びに従うか(警告。人がレビューで読み方を探さないため)。"""
    current, seen, has_lines, fence, out = None, [], False, False, []
    for no, line in enumerate(text.split("\n"), 1):
        if re.match(r"^\s*(```|~~~)", line):
            fence = not fence
            continue
        if fence:
            continue
        m = SPEC_SECTION_RE.match(line)
        if m:
            current = m.group(1)
            if current in SECTION_ORDER:
                seen.append((current, no))
            continue
        want = "要求" if ID_HINT.match(line) else "未確定事項" if (OPEN_ID_HINT.match(line) or OPEN_HINT.match(line)) else None
        if want:
            has_lines = True
            if current != want:
                out.append(
                    Finding(
                        "spec-line-misplaced",
                        path,
                        no,
                        "%sの ID の行は `## %s` の下に置く(今は %s の下)" % (want, want, "`## %s`" % current if current else "見出しの外"),
                        severity="warning",
                    )
                )
    if not has_lines or not re.search(r"(?m)^#(?!#)\s+\S", text):
        return  # 題(# )を持つ、人が読む文書として書いた正本だけを見る(要求の行だけの最小の正本は見ない)
    findings.extend(out)
    names = [n for n, _ in seen]
    for name in REQUIRED_SECTIONS:
        if name not in names:
            findings.append(Finding("spec-heading-missing", path, None, "見出し `## %s` が無い(雛形の必須の見出し)" % name, key=name, severity="warning"))
    last = -1
    for name, no in seen:
        at = SECTION_ORDER.index(name)
        if at < last:
            findings.append(
                Finding("spec-heading-order", path, no, "見出し `## %s` の順が雛形と違う(%s の順)" % (name, "・".join(SECTION_ORDER)), severity="warning")
            )
            break
        last = at


def requirement_files(root, cfg, files=None):
    out = []
    for p in cfg["requirements"]:
        if has_glob(p):
            out.extend(x for x in (files if files is not None else walk_files(root)) if glob_match(p, x))
        elif (files is not None and p in files) or (files is None and os.path.isfile(os.path.join(root, p))):
            out.append(p)
    return sorted(set(out))


EXTERNAL_REF = re.compile(r"別紙|別表|別添|添付資料")
MD_LINK = re.compile(r"\]\(([^)\s]+)\)")
CODE_SPAN = re.compile(r"(`+)(.+?)\1")  # `道` と `` `道` `` の柵(同じ数の ` で閉じる)


def check_external_refs(root, path, text, findings):
    """正本が別紙・別表を名前だけで指していれば知らせる(警告)。リポの中の実在する道を [名前](道) か `道` で添えていれば知らせない。
    コードの柵の中の行と、`…` の中だけに言葉がある行は見ない。"""
    base = os.path.dirname(path)
    fence = False
    for no, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith(("```", "~~~")):
            fence = not fence
            continue
        if fence or not EXTERNAL_REF.search(CODE_SPAN.sub("", line)):  # `別紙` のように柵の言葉として書いたものは参照と読まない
            continue
        targets = [t.split("#", 1)[0] for t in MD_LINK.findall(line)] + [m.group(2).strip(" `") for m in CODE_SPAN.finditer(line)]
        if any(
            t and not re.match(r"^[a-z]+://", t) and (os.path.exists(os.path.join(root, base, t)) or os.path.exists(os.path.join(root, t))) for t in targets
        ):
            continue
        findings.append(
            Finding(
                "spec-external-ref",
                path,
                no,
                "別紙・別表を名前だけで指している(リポの中の道が無い): %s" % clip(line.strip()),
                key="%s:%d" % (path, no),
                severity="warning",
            )
        )


def load_requirements(root, cfg, findings, files=None):
    reqs, opens, paths = [], [], requirement_files(root, cfg, files)
    for p in paths:
        text = read_text(root, p)
        r, o = parse_requirements(text, p, findings)
        check_external_refs(root, p, text, findings)
        if cfg.get("sections"):
            check_sections(text, p, findings)
        reqs.extend(r)
        opens.extend(o)
    by_name = {}
    for r in reqs:
        if r.name in by_name:
            first = by_name[r.name]
            first.dupes.append((r.path, r.line))
            findings.append(Finding("req-duplicate", r.path, r.line, "要求の名前 %s が2つある(先は %s:%d)" % (r.name, first.path, first.line), key=r.name))
        else:
            by_name[r.name] = r
    first_open = {}
    for o in opens:
        if o["id"] in first_open:
            f = first_open[o["id"]]
            findings.append(
                Finding("open-duplicate", o["path"], o["line"], "未確定事項の ID %s が2つある(先は %s:%d)" % (o["id"], f["path"], f["line"]), key=o["id"])
            )
        else:
            first_open[o["id"]] = o
    if cfg.get("require_accept_parts"):
        for r in by_name.values():
            if "line" in getattr(r, "accept_forms", []):
                findings.append(
                    Finding("accept-not-parts", r.path, r.line, "要求 %s の受け入れ基準に1行の文がある(require_accept_parts が true)" % r.id, key=r.name)
                )
    if cfg.get("require_sources"):
        for r in by_name.values():
            if r.treatment != "廃止" and not r.ups:
                findings.append(Finding("req-no-source", r.path, r.line, "要求 %s に 出典: up~… が無い(require_sources が true)" % r.id, key=r.name))
    if not by_name:
        findings.append(Finding("req-none", (paths or cfg["requirements"])[0], None, "要求が1件も無い(読んだ正本: %s)" % (", ".join(paths) or "無し")))
    return by_name, opens, paths


def compare_versions(reqs, old, where, findings, pairs=None):
    """old は 名前 → {"version", "sha256"}(base の正本)。pairs は {今の名前: 付け替える前の名前}(同じ要求として比べる)。"""
    pairs = pairs or {}
    for name, r in sorted(reqs.items()):
        src = name if name in old else pairs.get(name)
        o = old.get(src) if src else None
        if o is None:
            continue
        ov = o.get("version")
        if not isinstance(ov, int):
            continue
        what = where if src == name else "%s(付け替える前の %s)" % (where, src)
        if r.version < ov:
            findings.append(Finding("req-version-down", r.path, r.line, "要求 %s の版が%sの %d より低い" % (r.id, what, ov), key=name))
        elif r.version == ov and o.get("sha256") != r.fingerprint:
            findings.append(
                Finding("req-changed-same-version", r.path, r.line, "要求 %s の文か受け入れ基準が%sと違うのに、版が %d のまま" % (r.id, what, ov), key=name)
            )


def check_base(root, cfg, base, reqs, findings, renames=None):
    files = git_files_at(root, base)
    old = {}
    for p in requirement_files(root, cfg, files):
        rs, _ = parse_requirements(git_show(root, base, p), p, [])
        for r in rs:
            old.setdefault(r.name, {"version": r.version, "sha256": r.fingerprint})
    pairs = {}
    for name in reqs:
        if name in old:
            continue
        # 名前の表を遡り、base にあって今は無い、いちばん近い名前を付け替える前の名前とみなす(同じ近さに2つあれば併合で、比べない)
        near = [o for depth, o in predecessors(name, renames or {}) if o in old and o not in reqs]
        if near:
            pairs[name] = near[0]
    compare_versions(reqs, old, " base(%s)" % clip(base, 40), findings, pairs)


# ---------------------------------------------------------------- 名前の表(付け替えた旧い名前と消した名前)

RENAME_NAME = re.compile(r"^`(?:req~)?(?P<name>" + NAME + r")`$")
RENAME_GONE = re.compile(r"^消した\s*(?:[(（](?P<a>.*)[)）]|[:：](?P<b>.*))?$")


def parse_renames(text, path, findings):
    """名前の表を {旧い名前: (新しい名前(消した名前なら None), 消した理由, 行)} にする。
    読むのは、1つ目の欄が `名前` の表の行だけ(見出しの行・区切りの行・ほかの行とコードの柵の中は読まない)。
    行は付け替えた順に足すので、前の行の旧い名前を新しい名前にした行は使い回しとして読まない。"""
    out, fence = {}, False
    for no, line in enumerate(text.split("\n"), 1):
        if FENCE_RE.match(line):
            fence = not fence
            continue
        if fence or not line.lstrip().startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        first = RENAME_NAME.match(cells[0])
        if not first:
            continue
        old, second = first.group("name"), cells[1] if len(cells) > 1 else ""
        n, g = RENAME_NAME.match(second), RENAME_GONE.match(second)
        new, reason, bad = None, None, None
        if n:
            new = n.group("name")
            if new == old:
                bad = "新しい名前が旧い名前 %s と同じ" % old
            elif new in out:
                bad = "新しい名前 %s は、前の行(%d 行目)の旧い名前の使い回し" % (new, out[new][2])
        elif g:
            reason = (g.group("a") or g.group("b") or "").strip()
            if not reason:
                bad = "消した名前 %s に理由が無い(「消した(理由)」と書く)" % old
        else:
            bad = "%s の2つ目の欄が `新しい名前` でも「消した(理由)」でもない: %s" % (old, clip(second, 40))
        if bad is None and old in out:
            bad = "旧い名前 %s の行が2つある(先は %d 行目)" % (old, out[old][2])
        if bad:
            findings.append(Finding("renames-malformed", path, no, bad, key="%s:%d" % (old, no)))
            continue
        out[old] = (new, reason, no)
    return out


def load_renames(root, cfg, findings, ref=None):
    """ref の側(None なら作業ツリー)の名前の表。ファイルが無ければ空。"""
    rel = cfg.get("renames") or DEFAULT_RENAMES
    if ref:
        text = git_show(root, ref, rel, check=False)
    else:
        text = read_text(root, rel) if os.path.isfile(os.path.join(root, rel)) else None
    return parse_renames(text, rel, findings) if text is not None else {}


def predecessors(name, renames):
    """名前の表を遡った前の名前を [(近さ, 名前)] で近い順に返す。同じ近さに2つ以上ある段(併合)からは遡らない。"""
    back = {}
    for old, (new, _, _) in renames.items():
        if new:
            back.setdefault(new, []).append(old)
    out, cur, seen, depth = [], name, {name}, 0
    while len(back.get(cur, [])) == 1 and back[cur][0] not in seen:
        cur, depth = back[cur][0], depth + 1
        seen.add(cur)
        out.append((depth, cur))
    return out


def rename_pairs(old_reqs, new_reqs, renames):
    """base の要求のうち、名前の表の付け替えを辿ると head で新しく現れた要求に着くもの {旧い名前: 新しい名前}。
    同じ新しい名前に着く旧い名前が2つ以上あれば(併合)、付け替えとみなさない。"""
    out = {}
    for o in old_reqs:
        if o in new_reqs:
            continue
        cur, seen = o, {o}
        while cur not in new_reqs and cur in renames and renames[cur][0] and renames[cur][0] not in seen:
            cur = renames[cur][0]
            seen.add(cur)
        if cur != o and cur in new_reqs and cur not in old_reqs:
            out[o] = cur
    hits = Counter(out.values())
    return {o: n for o, n in out.items() if hits[n] == 1}


def renamed_note(hit):
    """名前の表の行の短い説明。"""
    new, reason, no = hit
    return "名前の表の %d 行目で%s" % (no, " %s に付け替えた" % new if new else "消した(理由: %s)" % clip(reason, 60))


def check_reused(reqs, renames, findings):
    """名前の表の旧い名前と消した名前を、今の要求に使い回していれば name-reused。"""
    for name, r in sorted(reqs.items()):
        if name in renames:
            findings.append(Finding("name-reused", r.path, r.line, "要求 %s の名前は、%s名前" % (r.id, renamed_note(renames[name])), key=name))


# ---------------------------------------------------------------- 上流の項目(設定したときだけ)


def squash(text):
    """空白の並びを1つの空白にそろえ、前後の空白を外す(指紋と題の照らしで同じ形にするため)。"""
    return re.sub(r"\s+", " ", text).strip()


def title_matches(title, text):
    """出典の題の写しが、上流の項目の文の頭と合うか(終わりの … は切った印。空白はそろえる)。"""
    head = squash(re.sub(r"(?:…|\.\.\.|⋯)+\s*$", "", title or ""))
    return not head or squash(text).startswith(head)


def check_upstream(root, cfg, reqs, opens, findings):
    """上流の項目の正本と、要求の 出典: と未確定事項の文を照らす。設定が無ければ None。"""
    if not cfg["upstream"]:
        return None
    paths = requirement_files(root, {"requirements": cfg["upstream"]})
    if not paths:
        raise UsageError("設定の upstream の上流の正本が無い: %s" % clip(", ".join(cfg["upstream"]), 120))
    items = {}
    for p in paths:
        fence = False
        for no, line in enumerate(read_text(root, p).split("\n"), 1):
            if re.match(r"^\s*(```|~~~)", line):
                fence = not fence
                continue
            if fence or not UP_HINT.match(line):
                continue
            m = UP_LINE.match(line)
            if not m:
                findings.append(Finding("upstream-malformed", p, no, "上流の項目の行の形が崩れている: %s" % clip(line.strip())))
                continue
            name = m.group("name")
            if name in items:
                findings.append(
                    Finding("upstream-malformed", p, no, "上流の項目の名前 up~%s が2つある(先は %s:%d)" % (name, items[name][0], items[name][1]), key=name)
                )
                continue
            items[name] = (p, no, m.group("text"))
    used, retired_used = set(), set()
    for name, r in sorted(reqs.items()):
        for u in sorted(r.ups):
            (retired_used if r.treatment == "廃止" else used).add(u)
            if u not in items:
                findings.append(Finding("upstream-unknown", r.path, r.line, "要求 %s の出典 up~%s が上流の正本に無い" % (r.id, u), key="%s:%s" % (name, u)))
            elif u in r.up_titles and not title_matches(r.up_titles[u], items[u][2]):
                findings.append(
                    Finding(
                        "upstream-title-stale",
                        r.path,
                        r.line,
                        "要求 %s の出典 up~%s の題「%s」が、上流の項目の文の頭「%s」と合わない" % (r.id, u, clip(r.up_titles[u], 40), clip(items[u][2], 40)),
                        key="%s:%s" % (name, u),
                    )
                )
    for o in opens:
        used |= up_refs(open_text(o))
    unmapped = 0
    for name, (p, no, _) in sorted(items.items(), key=lambda x: (x[1][0], x[1][1])):
        if name not in used and name in retired_used:
            findings.append(
                Finding("upstream-retired", p, no, "上流の項目 up~%s は、扱いが廃止の要求からしか結ばれていない" % name, key=name, severity="warning")
            )
        elif name not in used:
            unmapped += 1
            findings.append(Finding("upstream-unmapped", p, no, "上流の項目 up~%s が、どの要求の出典にも未確定事項の文にも無い" % name, key=name))
    return {"items": len(items), "unmapped": unmapped}


# ---------------------------------------------------------------- 承認の記録(人が宣言したときだけ)

APPROVAL_KEYS = ("name", "version", "fingerprint", "by", "date", "meaning")


def read_approvals(text, path, findings):
    """承認の記録の中身を、行ごとの辞書の並びにする。読めない行は approval-malformed にして飛ばす。"""
    rows = []
    for no, line in enumerate(text.split("\n"), 1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except ValueError:
            row = None
        bad_needs = isinstance(row, dict) and (
            ("needs" in row and not (isinstance(row["needs"], list) and all(x in KINDS for x in row["needs"])))
            or ("priority" in row and row["priority"] not in ("P0", "P1", "P2"))
        )
        if not isinstance(row, dict) or any(k not in row for k in APPROVAL_KEYS) or not isinstance(row.get("version"), int) or bad_needs:
            findings.append(Finding("approval-malformed", path, no, "承認の記録の %d 行目を読めない" % no, key="%s:%d" % (path, no)))
            continue
        rows.append(row)
    return rows


def check_approvals(root, cfg, reqs, findings, renames=None):
    """承認の記録があれば、要求ごとの最後の承認と今の版・指紋を比べる。数を返す(記録が無ければ None)。
    要求に自分の承認が無く、名前の表で付け替える前の名前に承認があれば、いちばん近い前の名前の最後の承認と比べる
    (版と指紋が同じときだけ引き継ぎ、違えば失った承認として approval-stale)。"""
    rel = cfg["approvals"]
    path = os.path.join(root, rel)
    if not os.path.isfile(path):
        return None
    last = {}
    for row in read_approvals(read_text(root, rel), rel, findings):
        last[row["name"]] = row
    out = Counter()
    for name, r in sorted(reqs.items()):
        a, via = last.get(name), ""
        if a is None:
            src = next((o for _, o in predecessors(name, renames or {}) if o in last and o not in reqs), None)
            if src is not None:
                a, via = last[src], "付け替える前の %s の" % src
        if a is None:
            out["unapproved"] += 1
        elif a["version"] != r.version or a["fingerprint"] != r.fingerprint:
            out["stale"] += 1
            findings.append(
                Finding(
                    "approval-stale",
                    r.path,
                    r.line,
                    "要求 %s は%s承認した後に変わった(承認は 版 %s・%s が %s に%s)"
                    % (r.id, via, clip(a["version"], 10), clip(a["by"], 40), clip(a["date"], 20), "。引き継げない" if via else ""),
                    key=name,
                )
            )
        elif "priority" in a and r.priority > a["priority"]:
            out["stale"] += 1
            findings.append(
                Finding(
                    "approval-stale",
                    r.path,
                    r.line,
                    "要求 %s の優先度が、承認した %s から %s に下がった(承認は %s が %s に)"
                    % (r.id, a["priority"], r.priority, clip(a["by"], 40), clip(a["date"], 20)),
                    key=name,
                )
            )
        elif "needs" in a and (r.needs is None or not set(a["needs"]) <= r.needs):
            out["stale"] += 1
            findings.append(
                Finding(
                    "approval-stale",
                    r.path,
                    r.line,
                    "要求 %s の要る確かめの種類が、承認した %s から減った(承認は %s が %s に)"
                    % (r.id, "・".join(a["needs"]), clip(a["by"], 40), clip(a["date"], 20)),
                    key=name,
                )
            )
        else:
            out["approved"] += 1
    return {"approved": out["approved"], "unapproved": out["unapproved"], "stale": out["stale"]}


def check_approvals_base(root, cfg, base, findings):
    """承認の記録は追記だけ。base の行が head の頭の行と同じでなければ approval-rewritten。"""
    rel = cfg["approvals"]
    try:
        old = git_show(root, base, rel)
    except UsageError:
        return
    if old is None:
        return
    if not os.path.isfile(os.path.join(root, rel)):
        return  # ファイルごと消したら既定に戻す。消したことは weaken が test-config-changed で人に回す
    new = read_text(root, rel)
    a = [x for x in old.split("\n") if x.strip()]
    b = new.split("\n")
    b = [x for x in b if x.strip()]
    for k, line in enumerate(a):
        if k >= len(b) or b[k] != line:
            findings.append(Finding("approval-rewritten", rel, k + 1, "承認の記録の %d 行目が base と違う(追記だけを許す)" % (k + 1), key=rel))
            return


# ---------------------------------------------------------------- 字句(コメントと文字列)


def line_index(text):
    starts = [0]
    for m in re.finditer("\n", text):
        starts.append(m.end())
    return lambda pos: bisect.bisect_right(starts, pos)


def lex_c(text, lang):
    """C の系統の字句を読み、(文字列とコメントを空白にした写し, [(位置, コメントの文)]) を返す。
    文字列は引用符を残して中身だけを空白にする(テストの題を元の文から取るため)。"""
    out, comments, i, n = list(text), [], 0, len(text)
    quotes = {"js": "'\"`", "php": "'\"", "go": "'\"`", "rust": '"'}[lang]

    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if text.startswith("//", i) or (lang == "php" and c == "#" and not text.startswith("#[", i)):
            j = text.find("\n", i)
            j = n if j < 0 else j
            comments.append((i, text[i:j]))
            blank(i, j)
            i = j
            continue
        if text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            comments.append((i, text[i:j]))
            blank(i, j)
            i = j
            continue
        if lang == "rust":
            m = re.match(r'(?<![\w])b?r(#*)"', text[i : i + 12]) if c in "rb" and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")) else None
            if m:
                end = text.find('"' + m.group(1), i + m.end())
                end = n if end < 0 else end
                blank(i + m.end(), end)
                i = end + 1 + len(m.group(1))
                continue
            if c == "'":
                m = re.match(r"'(\\.[^'\n]{0,8}|[^\\'\n])'", text[i : i + 12])
                if m:
                    blank(i + 1, i + m.end() - 1)
                    i += m.end()
                else:
                    i += 1
                continue
        if c in quotes:
            j = i + 1
            raw = lang == "go" and c == "`"
            while j < n:
                if text[j] == "\\" and not raw:
                    j += 2
                    continue
                if text[j] == c or (text[j] == "\n" and c in "'\"" and lang in ("js", "go")):
                    break
                j += 1
            blank(i + 1, min(j, n))
            i = j + 1
            continue
        i += 1
    return "".join(out), comments


def match_close(mask, pos, open_ch, close_ch):
    """mask[pos] の開きに対応する閉じの位置。無ければ末尾。"""
    depth = 0
    for k in range(pos, len(mask)):
        ch = mask[k]
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return k
    return len(mask) - 1


# ---------------------------------------------------------------- テストの宣言(言語ごとの小さな表)


class Test(object):
    __slots__ = ("path", "name", "node", "start", "end", "lang", "skip", "empty", "links", "qual", "trivial", "scope", "platform", "quarantined", "heads")

    def __init__(self, path, name, start, end, lang, skip=None, empty=False, qual=""):
        self.path, self.name, self.start, self.end, self.lang = path, name, start, end, lang
        self.skip, self.empty, self.links, self.qual = skip, empty, [], qual
        self.trivial = False
        self.heads = (
            None  # Rust: #[path] で別のファイルから取り込まれるファイルの、取り込み先の mod の道の候補(rust_path_heads)。無ければファイルの道から決める
        )
        self.scope = ""  # Rust: ファイルの中で囲む mod の名前の並び(a::tests)。走らせた結果の名前の照らしにだけ使い、node には入れない
        self.platform = False  # 条件つきの条件が OS か CPU で決まる(1つの OS の結果で走らないのは誤りでない)
        self.quarantined = False  # 設定の quarantine にある(確かめに数えない)
        self.node = "%s::%s" % (path, (qual + "::" if qual else "") + name)


# OS と CPU で決まる条件の語(言語ごと)。条件つき(conditional)のテストのうち、条件にこれがあるものを OS の条件つきと読む
RUST_OS = re.compile(r"\b(?:windows|unix|target_os|target_family|target_arch|target_env|target_vendor|target_pointer_width|target_endian)\b")
JS_OS = re.compile(r"\bprocess\s*\.\s*(?:platform|arch)\b|\bos\s*\.\s*(?:platform|type|arch)\s*\(")
GO_OS_WORDS = re.compile(r"\bruntime\s*\.\s*(?:GOOS|GOARCH)\b")
PHP_OS = re.compile(r"#\[\s*RequiresOperatingSystem\w*|@requires\s+OS")
PY_OS = {
    "sys.platform",
    "os.name",
    "platform.system",
    "platform.machine",
    "platform.platform",
    "platform.win32_ver",
    "platform.mac_ver",
    "sys.getwindowsversion",
}

# 断言と skip の語(言語ごと)
ASSERT_WORDS = {
    "js": re.compile(r"\bexpect(?:TypeOf)?\s*\(|\bassert\w*\s*[.(]|\.should\b|\bt\.(?:is|deepEqual|true|false|throws)\s*\("),
    "php": re.compile(r"(?:\$this|self|static|parent)\s*(?:->|::)\s*(?:assert\w*|expect\w*|fail)\s*\(|\bAssert::\w+\s*\(|\bexpect\s*\("),
    # Rust: 断言のマクロと、名前が assert・expect で始まる関数の呼び出し(ほかの言語と同じ)。Result と Option の .expect(…) は取り出しで、fn の定義は呼び出しでないので数えない
    "rust": re.compile(
        r"\b(?:assert\w*|debug_assert\w*|prop_assert\w*|expect\w*|panic|unreachable)!\s*[(\[{]|(?<![\w$])(?<!fn )assert\w*\s*\(|(?<![\w$.])(?<!fn )expect\w*\s*\("
    ),
    "go": re.compile(r"\b\w+\.(?:Error|Errorf|Fatal|Fatalf|Fail|FailNow)\s*\(|\b(?:assert|require)\.\w+\s*\("),
    "python": re.compile(r"\bassert\b|\.assert\w*\s*\(|\bpytest\.(?:raises|warns|fail)\s*\(|\bself\.fail\s*\("),
}
SKIP_WORDS = {
    "js": re.compile(r"(?<![\w$.])(?:it|test|describe)\s*\.\s*(?:skip|todo)\b|(?<![\w$.])x(?:it|test|describe)\s*\("),
    "php": re.compile(r"markTest(?:Skipped|Incomplete)\s*\(|->\s*(?:skip|todo)\s*\("),
    "rust": re.compile(r"#\s*\[\s*ignore\b"),
    "go": re.compile(r"\.Skip(?:Now|f)?\s*\("),
    "python": re.compile(r"\b(?:pytest\.mark\.skip\w*|unittest\.skip\w*|pytest\.skip\s*\(|skipTest\s*\()"),
}
NEVER = re.compile(r"(?!x)x")  # 当たらない(その言語に「失敗してよい」の印が無い)
XFAIL_WORDS = {
    "js": re.compile(r"(?<![\w$.])(?:it|test)\s*\.\s*(?:fails|failing)\b"),
    "php": NEVER,
    "rust": NEVER,
    "go": NEVER,
    "python": re.compile(r"\bxfail\b|\bexpectedFailure\b"),
}
DECL_WORDS = {
    "js": re.compile(r"(?<![\w$.])(?:it|test|xit|xtest|fit)\b(?:\s*\.\s*[A-Za-z]+)*\s*\("),
    "php": re.compile(r"\bfunction\s+test\w*\s*\(|(?<![\w$>:])(?:it|test)\s*\("),
    "rust": re.compile(r"#\s*\[\s*(?:[\w:]+::)?test\b"),
    "go": re.compile(r"\bfunc\s+Test\w*\s*\("),
    "python": re.compile(r"^\s*(?:async\s+)?def\s+test\w*\s*\(", re.M),
}

JS_CALL = re.compile(r"(?<![\w$.])(it|test|xit|xtest|fit|describe|xdescribe|fdescribe)\b((?:\s*\.\s*[A-Za-z]+)*)\s*\(")
FOCUS = {
    "js": re.compile(r"(?<![\w$.])(?:(?:it|test|describe|context|suite)\s*\.\s*only\b|f(?:it|describe)\s*\()"),
    "php": re.compile(r"->\s*only\s*\("),
}
PEST_CALL = re.compile(r"(?<![\w$>:])(it|test)\s*\(")
PHP_FUNC = re.compile(r"\bfunction\s+(\w+)\s*\(")
RUST_TEST_ATTR = re.compile(r"#\s*\[\s*(?:[\w:]+::)?(?:test|rstest|test_case)\b[^\]]*\]")
RUST_FN = re.compile(r"\bfn\s+(\w+)")
GO_FUNC = re.compile(r"\bfunc\s+(Test\w*)\s*\(\s*(\w+)\s+\*testing\.T\s*\)")


CFG_TOKEN = re.compile(r'\s*(?:([A-Za-z_][\w]*)|(=)|("(?:\\.|[^"\\])*")|([(),]))')


def cfg_value(text):
    """Rust の cfg の述語の値。True(いつも真)・False(いつも偽)・None(環境と feature で決まるか読めない)。
    test はテストを組むときに真。中身の無い any() は偽、all() は真(Rust のリファレンスの条件つきのコンパイル)。"""
    toks, pos = [], 0
    while pos < len(text):
        m = CFG_TOKEN.match(text, pos)
        if not m or m.end() == pos:
            if text[pos:].strip():
                return None
            break
        toks.append(m.group(1) or m.group(2) or (m.group(3) and '"') or m.group(4))
        pos = m.end()

    def parse(i):
        name = toks[i]
        if name in ("any", "all", "not") and toks[i + 1] == "(":
            i, vals = i + 2, []
            while toks[i] != ")":
                v, i = parse(i)
                vals.append(v)
                if toks[i] == ",":
                    i += 1
            i += 1
            if name == "not":
                return (None if vals[0] is None else not vals[0]), i
            if name == "any":
                return (True if True in vals else False if all(v is False for v in vals) else None), i
            return (False if False in vals else True if all(v is True for v in vals) else None), i
        if i + 1 < len(toks) and toks[i + 1] == "=":
            return None, i + 3
        return {"test": True, "true": True, "false": False}.get(name), i + 1

    try:
        value, end = parse(0)
    except (IndexError, TypeError):
        return None
    return value if end == len(toks) else None


GO_EXPR_TOKEN = re.compile(r"\s*(\|\||&&|!|\(|\)|[\w.]+)")
# go/build の知る OS と CPU の名(ファイルの名の _GOOS・_GOARCH の接尾辞は暗黙の札)
GO_OS = set("aix android darwin dragonfly freebsd hurd illumos ios js linux nacl netbsd openbsd plan9 solaris wasip1 windows zos".split())
GO_ARCH = set(
    "386 amd64 amd64p32 arm armbe arm64 arm64be loong64 mips mipsle mips64 mips64le mips64p32 mips64p32le ppc ppc64 ppc64le riscv riscv64 s390 s390x sparc sparc64 wasm".split()
)


def go_tag(name):
    """札の1語の値。ignore はいつも偽(go test が組まない慣例の札)、ほかは -tags・OS・CPU で決まるので None。"""
    return False if name == "ignore" else None


def go_and(vals):
    return False if False in vals else True if all(v is True for v in vals) else None


def go_or(vals):
    return True if True in vals else False if all(v is False for v in vals) else None


def go_not(v):
    return None if v is None else not v


def go_expr_value(expr):
    """`//go:build` の式の値(True・False・None)。読めなければ None。"""
    toks, pos = [], 0
    while pos < len(expr):
        m = GO_EXPR_TOKEN.match(expr, pos)
        if not m:
            if expr[pos:].strip():
                return None
            break
        toks.append(m.group(1))
        pos = m.end()

    def p_or(i):
        v, i = p_and(i)
        vals = [v]
        while i < len(toks) and toks[i] == "||":
            v, i = p_and(i + 1)
            vals.append(v)
        return go_or(vals), i

    def p_and(i):
        v, i = p_not(i)
        vals = [v]
        while i < len(toks) and toks[i] == "&&":
            v, i = p_not(i + 1)
            vals.append(v)
        return go_and(vals), i

    def p_not(i):
        if toks[i] == "!":
            v, i = p_not(i + 1)
            return go_not(v), i
        if toks[i] == "(":
            v, i = p_or(i + 1)
            return v, i + 1
        return go_tag(toks[i]), i + 1

    try:
        value, end = p_or(0)
    except IndexError:
        return None
    return value if end == len(toks) else None


def go_file_skip(path, text):
    """Go のテストのファイルを go test が組むか。skip(組まない)・conditional(札か OS・CPU の名で決まる)・None。
    札は package の前の行の `//go:build`(無ければ旧い `// +build`)だけを読む(go の Build Constraints)。"""
    base = os.path.basename(path)
    if base.startswith(("_", ".")):
        return "skip"  # go は _ と . で始まるファイルを読まない
    found = None
    stem = base.split(".", 1)[0]
    if "_" in stem:
        parts = stem[stem.index("_") :].split("_")
        if parts and parts[-1] == "test":
            parts = parts[:-1]
        if (len(parts) >= 2 and parts[-2] in GO_OS and parts[-1] in GO_ARCH) or (parts and (parts[-1] in GO_OS or parts[-1] in GO_ARCH)):
            found = "conditional"
    build, plus = [], []
    for line in text.split("\n"):
        s = line.strip()
        if not s:
            continue
        if not s.startswith("//"):
            break
        if s.startswith("//go:build "):
            build.append(s[len("//go:build ") :])
        elif re.match(r"//\s*\+build\s", s):
            plus.append(re.sub(r"^//\s*\+build\s+", "", s))
    if build:
        value = go_expr_value(build[0])  # go は //go:build の行を1つしか許さない
    elif plus:
        value = go_and(
            [go_or([go_and([go_not(go_tag(t[1:])) if t.startswith("!") else go_tag(t) for t in opt.split(",")]) for opt in ln.split()]) for ln in plus]
        )
    else:
        value = True
    if value is False:
        return "skip"
    return "conditional" if value is None else found


def go_file_platform(path, text):
    """Go のテストのファイルの条件が OS か CPU で決まるか: ファイルの名の _GOOS・_GOARCH の接尾辞か、札の式の OS・CPU・unix の名。"""
    parts = os.path.basename(path).split(".", 1)[0].split("_")[1:]
    if parts and parts[-1] == "test":
        parts = parts[:-1]
    if parts and (parts[-1] in GO_OS or parts[-1] in GO_ARCH):
        return True
    for line in text.split("\n"):
        s = line.strip()
        if s and not s.startswith("//"):
            break
        if s.startswith("//go:build ") or re.match(r"//\s*\+build\s", s):
            if any(w in GO_OS or w in GO_ARCH or w == "unix" for w in re.findall(r"[\w.]+", s.split("build", 1)[1])):
                return True
    return False


def js_constant(text):
    """JS の式が定数なら真偽(True・False)、そうでなければ None。"""
    s = text.strip()
    if s in ("true", "!0", "!false"):
        return True
    if s in ("false", "0", "null", "undefined", '""', "''", "``", "!1", "!true"):
        return False
    if re.fullmatch(r"\d+(?:\.\d+)?", s):
        return float(s) != 0
    if re.fullmatch(r"([\'\"`]).+\1", s):
        return True
    return None


def two_stage_skip(text, mask, mods, first):
    """skipIf(条件)・runIf(条件) の条件が定数なら skip か None、定数でなければ conditional。どちらも無ければ None。"""
    stage = [x for x in mods if x in ("skipIf", "runIf")]
    if not stage:
        return None
    value = js_constant(text[first + 1 : match_close(mask, first, "(", ")")])
    if value is None:
        return "conditional"
    return "skip" if value == (stage[0] == "skipIf") else None


def option_skip(text, mask, after):
    """題の後ろの `{ skip: … }`・`{ todo: … }` の選択肢(Vitest と node:test)。いつも飛ぶなら skip、式なら conditional。"""
    m = re.compile(r"\s*,\s*\{([^{}]*)\}\s*,").match(mask, after)
    if not m:
        return None
    found = None
    for k in re.finditer(r"(?<![\w$])(skip|todo)\s*:\s*([^,}]*)", mask[m.start(1) : m.end(1)]):
        raw = text[m.start(1) + k.start(2) : m.start(1) + k.end(2)]
        value = js_constant(raw)
        if value is True:
            return "skip"
        if value is None:
            found = "conditional"
    return found


TWO_STAGE = {"each", "skipIf", "runIf"}  # 1つ目の括弧は表か条件で、題と本体は次の括弧にある
CHAIN_NEXT = re.compile(r"((?:\s*\.\s*[A-Za-z]+)*)\s*\(")


def body_call(mask, p, mods):
    """it.each(表)(…)・it.skipIf(条件)(…)・it.skipIf(条件).each(表)(…) の、題と本体の括弧の位置と修飾。読めなければ位置は None。"""
    step = mods
    while TWO_STAGE.intersection(step):
        nxt = CHAIN_NEXT.match(mask, match_close(mask, p, "(", ")") + 1)
        if not nxt:
            return None, mods
        step = re.findall(r"[A-Za-z]+", nxt.group(1))
        mods, p = mods + step, nxt.end() - 1
    return p, mods


JS_MATCHER = re.compile(r"\s*\.\s*(?P<neg>not\s*\.\s*)?(?P<name>\w+)\s*\(")
JS_ASSERT_METHOD = re.compile(r"\s*(?P<name>\w+)\s*\(")
JS_SAME_TRUE = ("toBe", "toEqual", "toStrictEqual")


def call_args(mask, open_pos):
    """mask の open_pos の開き括弧から閉じまでの中身(空白を1つに寄せた字面)と、閉じの位置。"""
    close = match_close(mask, open_pos, "(", ")")
    return " ".join(mask[open_pos + 1 : close].split()), close


def js_assert_trivial(mask, m):
    """JS の断言の1つ(ASSERT_WORDS の当たり m)が、いつも通るか型だけを見るものか(真を真と比べる・型・typeof・expectTypeOf)。"""
    word = m.group(0)
    if word.startswith("expectTypeOf"):
        return True
    if word.startswith("expect"):
        arg, close = call_args(mask, m.end() - 1)
        c = JS_MATCHER.match(mask, close + 1)
        if not c:
            return False
        marg, _ = call_args(mask, c.end() - 1)
        name, neg = c.group("name"), bool(c.group("neg"))
        if name == "toBeInstanceOf":
            return True
        if neg:
            return False
        if arg == "true" and ((name in JS_SAME_TRUE and marg == "true") or name == "toBeTruthy"):
            return True
        return arg.startswith("typeof ") and name in JS_SAME_TRUE
    if re.match(r"assert\s*\(", word):
        return call_args(mask, m.end() - 1)[0] == "true"
    if re.match(r"assert\s*\.", word):
        c = JS_ASSERT_METHOD.match(mask, m.end())
        return bool(c) and c.group("name") == "ok" and call_args(mask, c.end() - 1)[0] == "true"
    return False


def js_trivial(mask, start, end):
    """JS の本体(mask の start から end まで)の断言が1つ以上あり、どれもいつも通るか型だけを見るものか。"""
    hits = list(ASSERT_WORDS["js"].finditer(mask, start, end))
    return bool(hits) and all(js_assert_trivial(mask, m) for m in hits)


def helper_pattern(names):
    """設定の assert_helpers の名前の呼び出し(Rust はマクロの呼び出しも)。名前が無ければ None。"""
    if not names:
        return None
    return re.compile(r"(?<![\w$])(?:%s)\s*(?:!\s*[(\[{]|\()" % "|".join(re.escape(n) for n in names))


def has_assert(lang, body, hp=None):
    """本体に断言があるか: 言語の断言の語か、設定の断言のヘルパー(hp)の呼び出し。"""
    return bool(ASSERT_WORDS[lang].search(body) or (hp and hp.search(body)))


def call_tests(path, text, mask, line_of, lang, pattern, hp=None):
    """it(...)・test(...) の形の宣言(JS と Pest)。"""
    tests, skipped_spans, conditional_spans = [], [], []
    for m in pattern.finditer(mask):
        base = m.group(1)
        mods = re.findall(r"[A-Za-z]+", m.group(2)) if m.lastindex and m.lastindex >= 2 and m.group(2) else []
        first = m.end() - 1
        p, mods = body_call(mask, first, mods)
        if p is None:
            continue
        close = match_close(mask, p, "(", ")")
        staged = two_stage_skip(text, mask, mods, first)
        cond = text[first + 1 : match_close(mask, first, "(", ")")] if staged == "conditional" else ""
        if base in ("describe", "xdescribe", "fdescribe"):
            if base == "xdescribe" or "skip" in mods or "todo" in mods or "fixme" in mods or staged == "skip":
                skipped_spans.append((p, close))
            elif staged == "conditional":
                conditional_spans.append((p, close, bool(JS_OS.search(cond))))
            continue
        q = re.compile(r"\s*(['\"`])").match(mask, p + 1)
        title = text[q.end() : mask.find(q.group(1), q.end())] if q else "line %d" % line_of(m.start())
        skip = None
        if base in ("xit", "xtest") or "skip" in mods or "todo" in mods or "fixme" in mods:
            skip = "skip"
        elif staged:
            skip = staged
        elif "fails" in mods or "failing" in mods:
            skip = "xfail"
        if skip is None and q:
            after = mask.find(q.group(1), q.end()) + 1
            skip = option_skip(text, mask, after)
            opt = re.compile(r"\s*,\s*\{([^{}]*)\}").match(mask, after) if skip == "conditional" else None
            cond = text[opt.start(1) : opt.end(1)] if opt else cond
        if lang == "php" and re.compile(r"\s*->\s*(?:skip|todo)\b").match(mask, close + 1):
            skip = "skip"
        body = mask[p : close + 1]
        t = Test(path, clip(title, 120), line_of(m.start()), line_of(close), lang, skip, not has_assert(lang, body, hp))
        if lang == "js":
            t.trivial = js_trivial(mask, p, close + 1) and not (hp and hp.search(body))
        t.platform = skip == "conditional" and bool(JS_OS.search(cond))
        tests.append((m.start(), t))
    for pos, t in tests:
        if t.skip is None and any(a <= pos <= b for a, b in skipped_spans):
            t.skip = "skip"
        elif t.skip is None and any(a <= pos <= b for a, b, _ in conditional_spans):
            t.skip = "conditional"
            t.platform = any(os_cond for a, b, os_cond in conditional_spans if a <= pos <= b)
    return [t for _, t in tests]


def brace_skip(body, m):
    """波かっこの本体の中の skip の呼び出し: 一番上の段なら無条件、if などの中なら条件つき。"""
    if not m:
        return None
    before = body[: m.start()]
    return "skip" if before.count("{") - before.count("}") <= 1 else "conditional"


def region_before(mask, pos):
    """宣言の直前の属性やドキュメントの範囲の始まり(前の { } ; の次)。"""
    k = max(mask.rfind(ch, 0, pos) for ch in "{};")
    return k + 1


RUST_CFG = re.compile(r"#\s*\[\s*cfg\s*\(")
RUST_INNER_CFG = re.compile(r"#\s*!\s*\[\s*cfg\s*\(")


def rust_cfg_blocks(mask):
    """cfg の効く範囲: [(cfg の属性の文, 始まり, 終わり)]。外の属性 #[cfg(…)] の付いた mod の本体と、内の属性 #![cfg(…)] を囲む本体(ファイルの頭ならファイル全体)。"""
    out = []
    for m in re.finditer(r"\bmod\s+\w+\s*\{", mask):
        attrs = mask[region_before(mask, m.start()) : m.start()]
        if RUST_CFG.search(attrs):
            brace = m.end() - 1
            out.append((attrs, brace, match_close(mask, brace, "{", "}")))
    for m in RUST_INNER_CFG.finditer(mask):
        close = match_close(mask, m.end() - 1, "(", ")")
        attr = mask[m.start() : close + 1]
        depth, k = 0, m.start() - 1
        while k >= 0:  # 内の属性を囲む波かっこ(無ければファイル全体)
            if mask[k] == "}":
                depth += 1
            elif mask[k] == "{":
                if depth == 0:
                    break
                depth -= 1
            k -= 1
        lo = k if k >= 0 else 0
        hi = match_close(mask, k, "{", "}") if k >= 0 else len(mask)
        out.append((attr.replace("#!", "#", 1), lo, hi))
    return out


def rust_cfg_skip(skip, attr_texts):
    """テストに効く cfg の属性(関数・囲む mod・内の属性)を読み、いつも偽なら skip、環境で決まるなら conditional を足す。"""
    if skip == "skip":
        return skip
    for attrs in attr_texts:
        for m in RUST_CFG.finditer(attrs):
            close = match_close(attrs, m.end() - 1, "(", ")")
            value = cfg_value(attrs[m.end() : close])
            if value is False:
                return "skip"
            if value is None:
                skip = "conditional"
    return skip


def scan_brace_lang(path, text, lang, helpers=()):
    mask, comments = lex_c(text, lang)
    line_of = line_index(text)
    hp = helper_pattern(helpers)
    tests = []
    if lang == "js":
        tests = call_tests(path, text, mask, line_of, lang, JS_CALL, hp)
    elif lang == "php":
        for m in PHP_FUNC.finditer(mask):
            name = m.group(1)
            head = text[region_before(mask, m.start()) : m.start()]
            if not (name.startswith("test") or re.search(r"#\[\s*Test\b|@test\b", head)):
                continue
            sig_close = match_close(mask, m.end() - 1, "(", ")")
            brace, semi = mask.find("{", sig_close), mask.find(";", sig_close)
            if brace < 0 or (0 <= semi < brace):
                continue
            close = match_close(mask, brace, "{", "}")
            body = mask[brace : close + 1]
            skip = brace_skip(body, SKIP_WORDS["php"].search(body))
            if skip is None and re.search(r"#\[\s*Requires\w*|@requires\b", head):
                skip = "conditional"
                platform = bool(PHP_OS.search(head))
            else:
                platform = False
            empty = not (has_assert("php", body, hp) or re.search(r"@expectedException|#\[\s*ExpectedException", head))
            start = line_of(region_before(mask, m.start()) + len(head) - len(head.lstrip()))
            tests.append(Test(path, name, start, line_of(close), lang, skip, empty))
            tests[-1].platform = platform
        tests.extend(call_tests(path, text, mask, line_of, lang, PEST_CALL, hp))
    elif lang == "rust":
        seen = set()
        cfg_blocks = rust_cfg_blocks(mask)
        mods = [(m.group(1), m.end() - 1, match_close(mask, m.end() - 1, "{", "}")) for m in re.finditer(r"\bmod\s+(\w+)\s*\{", mask)]
        for m in RUST_TEST_ATTR.finditer(mask):
            f = RUST_FN.search(mask, m.end())
            if not f or f.start() in seen:
                continue
            seen.add(f.start())
            attrs = mask[region_before(mask, m.start()) : f.start()]
            brace = mask.find("{", f.end())
            if brace < 0:
                continue
            close = match_close(mask, brace, "{", "}")
            body = mask[brace : close + 1]
            skip = "skip" if SKIP_WORDS["rust"].search(attrs) else None
            for ca in re.finditer(r"#\s*\[\s*cfg_attr\s*\((.*)\]", attrs):
                pred, _, rest = ca.group(1).rpartition(",") if "ignore" in ca.group(1) else ("", "", "")
                if skip is None and re.search(r"\bignore\b", rest):
                    value = cfg_value(pred)
                    skip = "skip" if value is True else None if value is False else "conditional"
            cond_attrs = [attrs] + [a for a, lo, hi in cfg_blocks if lo <= f.start() <= hi]
            skip = rust_cfg_skip(skip, cond_attrs)
            empty = not (has_assert("rust", body, hp) or re.search(r"#\s*\[\s*should_panic", attrs))
            first = region_before(mask, m.start())
            first += len(mask[first : m.start()]) - len(mask[first : m.start()].lstrip())
            tests.append(Test(path, f.group(1), line_of(first), line_of(close), lang, skip, empty))
            tests[-1].scope = "::".join(name for name, lo, hi in mods if lo <= f.start() <= hi)
            tests[-1].platform = skip == "conditional" and any(
                RUST_OS.search(x.group(0)) for a in cond_attrs for x in re.finditer(r"#\s*\[\s*cfg(?:_attr)?\s*\(.*", a)
            )
    elif lang == "go":
        for m in GO_FUNC.finditer(mask):
            brace = mask.find("{", m.end())
            if brace < 0:
                continue
            close = match_close(mask, brace, "{", "}")
            body = mask[brace : close + 1]
            var = re.escape(m.group(2))
            skip = brace_skip(body, re.search(r"\b%s\.Skip(?:Now|f)?\s*\(" % var, body))
            platform = skip == "conditional" and bool(GO_OS_WORDS.search(text[brace : close + 1]))
            whole = go_file_skip(path, text)
            if whole == "skip" or (whole and skip is None):
                skip = whole
                platform = whole == "conditional" and go_file_platform(path, text)
            tests.append(Test(path, m.group(1), line_of(m.start()), line_of(close), lang, skip, not has_assert("go", body, hp)))
            tests[-1].platform = platform
    marks_src = [(line_of(pos), txt, pos) for pos, txt in comments]
    blank_lines = set(no for no, ln in enumerate(mask.split("\n"), 1) if not ln.strip())
    return tests, marks_src, blank_lines, line_of


def dotted(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Call):
        return dotted(node.func) + ("." + ".".join(reversed(parts)) if parts else "")
    parts.append(node.id if isinstance(node, ast.Name) else "?")
    return ".".join(reversed(parts))


COND_SKIP = ("skipif", "skipIf", "skipUnless", "importorskip")


def decorator_skip(decorators):
    """skip(無条件)・conditional(条件つき)・xfail か None。pytest.mark.skip は引数に条件があっても無条件(pytest #8384)。"""
    found = None
    for d in decorators:
        last = dotted(d).split(".")[-1]
        if last in ("xfail", "expectedFailure"):
            return "xfail"
        if last == "skip":
            return "skip"
        if last in COND_SKIP and isinstance(d, ast.Call) and (d.args or d.keywords):
            found = "conditional"
        elif last in COND_SKIP:
            return "skip"
    return found


def _skip_kind(n):
    if isinstance(n, ast.Call):
        name = dotted(n.func)
        if name.endswith("pytest.skip") or name.split(".")[-1] == "skipTest":
            return "skip"
        if name.endswith("pytest.importorskip"):
            return "conditional"
        if name.endswith("pytest.xfail"):
            return "xfail"
    if isinstance(n, ast.Raise) and n.exc is not None and dotted(n.exc.func if isinstance(n.exc, ast.Call) else n.exc).split(".")[-1] == "SkipTest":
        return "skip"
    return None


def body_skip(fn):
    """本体の一番上の段で呼ぶ skip は無条件、if などの中は条件つき。"""
    for stmt in fn.body:
        node = stmt.value if isinstance(stmt, ast.Expr) else stmt
        k = _skip_kind(node)
        if k in ("xfail", "skip"):
            return k
    found = None
    for n in ast.walk(fn):
        k = _skip_kind(n)
        if k == "xfail":
            return "xfail"
        if k:
            found = "conditional"
    return found


WEAK_ASSERTS = {"assertTrue", "assertFalse", "assert_", "failUnless", "failIf", "assertIsNotNone", "assert_called", "assert_called_once"}


def py_assertions(fn, helpers=()):
    """(断言の数, 強い断言の数)。強い = 値を比べる断言(assert の比較・assertEqual・pytest.raises など)と、設定の断言のヘルパーの呼び出し。"""
    total = strong = 0
    for n in ast.walk(fn):
        if isinstance(n, ast.Assert):
            total += 1
            strong += isinstance(n.test, ast.Compare)
        elif isinstance(n, ast.Call):
            name = dotted(n.func)
            last = name.split(".")[-1]
            if last.startswith(("assert", "expect")) or name in ("pytest.raises", "pytest.warns", "pytest.fail", "self.fail") or last in helpers:
                total += 1
                strong += last not in WEAK_ASSERTS
    return total, strong


def py_trivial(fn, helpers=()):
    """断言が、いつも通るか型だけを見るものしか無いか(設定の断言のヘルパーの呼び出しは値を比べる断言と見る)。"""
    for n in ast.walk(fn):
        if isinstance(n, ast.Assert):
            t = n.test
            if isinstance(t, ast.Constant) and t.value is True:
                continue
            if isinstance(t, ast.Call) and dotted(t.func) == "isinstance":
                continue
            return False
        if isinstance(n, ast.Call):
            name = dotted(n.func)
            last = name.split(".")[-1]
            if last in helpers:
                return False
            if not (last.startswith(("assert", "expect")) or name in ("pytest.raises", "pytest.warns", "pytest.fail", "self.fail")):
                continue
            if last == "assertIsInstance":
                continue
            if last == "assertTrue" and n.args and isinstance(n.args[0], ast.Constant) and n.args[0].value is True:
                continue
            return False
    return True


def module_skip(tree):
    for n in tree.body:
        if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "pytestmark" for t in n.targets):
            for sub in ast.walk(n.value):
                if isinstance(sub, (ast.Attribute, ast.Call)):
                    last = dotted(sub).split(".")[-1]
                    if last in ("xfail",):
                        return "xfail"
                    if last == "skipif" and isinstance(sub, ast.Call) and (sub.args or sub.keywords):
                        return "conditional"
                    if last in ("skip", "skipif"):
                        return "skip"
    return None


def replaced_tests(tree):
    """定義の後で差し替えるか消したテストの {(修飾名, 名前)}: クラスの外のテストの名への代入・setattr・delattr・del と、
    クラスの中で def の後に同じ名へ代入するか del したもの(unittest も pytest も、差し替えた後の物を走らせる)。"""
    out = set()

    def target(node):
        if isinstance(node, ast.Attribute) and node.attr.startswith("test"):
            out.add((dotted(node.value).replace(".", "::"), node.attr))

    for n in ast.walk(tree):
        if isinstance(n, (ast.Assign, ast.Delete)):
            for t in n.targets:
                target(t)
        elif isinstance(n, (ast.AnnAssign, ast.AugAssign)):
            target(n.target)
        elif isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ("setattr", "delattr") and len(n.args) >= 2:
            name = n.args[1]
            if isinstance(name, ast.Constant) and isinstance(name.value, str) and name.value.startswith("test"):
                out.add((dotted(n.args[0]).replace(".", "::"), name.value))

    def visit(body, qual):
        defined = set()
        for node in body:
            if isinstance(node, ast.ClassDef):
                visit(node.body, (qual + "::" if qual else "") + node.name)
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                defined.add(node.name)
            elif isinstance(node, (ast.Assign, ast.AnnAssign, ast.Delete)):
                for t in getattr(node, "targets", None) or [node.target]:
                    if isinstance(t, ast.Name) and t.id in defined:
                        out.add((qual, t.id))

    visit(tree.body, "")
    return out


def py_tests(tree):
    """[(修飾名, 関数, 継いだ skip)] を返す。定義の後で差し替えるか消したテストは skip を継ぐ。"""
    out, mskip, replaced = [], module_skip(tree), replaced_tests(tree)

    def visit(body, qual, inherited):
        for node in body:
            if isinstance(node, ast.ClassDef):
                visit(node.body, (qual + "::" if qual else "") + node.name, decorator_skip(node.decorator_list) or inherited)
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name.startswith("test"):
                out.append((qual, node, "skip" if (qual, node.name) in replaced else inherited))

    visit(tree.body, "", mskip)
    return out


def py_platform(tree, qual, fn):
    """条件つきの Python のテストの条件(関数とそれを囲むクラスのデコレータ・モジュールの pytestmark・本体)に、OS か CPU の名(PY_OS)があるか。"""
    nodes, body = list(fn.decorator_list), tree.body
    for name in [x for x in qual.split("::") if x]:
        cls = next((n for n in body if isinstance(n, ast.ClassDef) and n.name == name), None)
        if cls is None:
            break
        nodes.extend(cls.decorator_list)
        body = cls.body
    nodes.extend(n.value for n in tree.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "pytestmark" for t in n.targets))
    if body_skip(fn) == "conditional":
        nodes.append(fn)
    return any(isinstance(x, ast.Attribute) and dotted(x) in PY_OS for n in nodes for x in ast.walk(n))


def scan_python(path, text, helpers=()):
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError) as e:
        return None, str(e)
    comments = []
    try:
        for tok in tokenize.generate_tokens(io.StringIO(text).readline):
            if tok.type == tokenize.COMMENT:
                comments.append((tok.start[0], tok.string, None))
    except (tokenize.TokenError, IndentationError) as e:
        return None, str(e)
    tests = []
    for qual, fn, inherited in py_tests(tree):
        start = min([d.lineno for d in fn.decorator_list] + [fn.lineno])
        skip = decorator_skip(fn.decorator_list) or inherited or body_skip(fn)
        total, _ = py_assertions(fn, helpers)
        tests.append(Test(path, fn.name, start, fn.end_lineno, "python", skip, total == 0, qual))
        tests[-1].trivial = total > 0 and py_trivial(fn, helpers)
        tests[-1].platform = skip == "conditional" and py_platform(tree, qual, fn)
    lines = text.split("\n")
    blank_lines = set(no for no, ln in enumerate(lines, 1) if not ln.strip() or ln.strip().startswith("#"))
    return (tests, comments, blank_lines), None


def scan_file(path, text, lang, helpers=()):
    """(テスト, コメント, 空き行) か、読めないとき (None, 理由)。helpers は設定の assert_helpers のその言語の名前。"""
    if lang == "python":
        return scan_python(path, text, helpers)
    tests, comments, blank_lines, _ = scan_brace_lang(path, text, lang, helpers)
    return (tests, [(no, txt, None) for no, txt, _ in comments], blank_lines), None


def bind(tests, line, blank_lines):
    inside = [t for t in tests if t.start <= line <= t.end]
    if inside:
        return min(inside, key=lambda t: t.end - t.start)
    after = sorted((t for t in tests if t.start > line), key=lambda t: t.start)
    if after and all(k in blank_lines for k in range(line + 1, after[0].start)):
        return after[0]
    return None


def parse_marks(comment_line, comment_text):
    """[(行, 種類, [(名前, 版)])] と、崩れた印の [(行, 文)] を返す。"""
    marks, bad, covered = [], [], []
    for m in MARK_RE.finditer(comment_text):
        line = comment_line + comment_text.count("\n", 0, m.start())
        covered.append((m.start(), m.end()))
        targets = []
        for part in m.group(2).split(","):
            t = TARGET_RE.match(part.strip())
            if not t:
                targets = None
                break
            targets.append((t.group(1), int(t.group(2))))
        if targets:
            marks.append((line, m.group(1), targets))
        else:
            bad.append((line, m.group(0)))
    for h in MARK_HINT.finditer(comment_text):
        if not any(a <= h.start() < b for a, b in covered):
            bad.append((comment_line + comment_text.count("\n", 0, h.start()), comment_text.strip()))
            break
    return marks, bad


def scan_tests(root, cfg, findings, files=None):
    files = files if files is not None else walk_files(root)
    test_files = [p for p in select(files, cfg["tests"], cfg["exclude"]) if os.path.splitext(p)[1] in LANG_BY_EXT]
    all_tests, digests = [], []
    for path in test_files:
        text = read_text(root, path)
        digests.append("%s\0%s" % (path, sha256(text)))
        lang = LANG_BY_EXT[os.path.splitext(path)[1]]
        result, err = scan_file(path, text, lang, (cfg.get("assert_helpers") or {}).get(lang) or ())
        if result is None:
            if MARK_HINT.search(text):
                findings.append(Finding("mark-malformed", path, None, "テストのファイルを読めず、印を確かめられない: %s" % clip(err, 100)))
            continue
        tests, comments, blank_lines = result
        if lang in FOCUS:
            mask, _ = lex_c(text, lang)
            line_of = line_index(text)
            for m in FOCUS[lang].finditer(mask):
                findings.append(
                    Finding(
                        "test-focused",
                        path,
                        line_of(m.start()),
                        "%s の %d 行目でテストを絞っている(%s)。ほかのテストが黙って止まる" % (path, line_of(m.start()), clip(m.group(0).strip(), 30)),
                        key="%s:%d" % (path, line_of(m.start())),
                    )
                )
        for no, txt, _ in comments:
            marks, bad = parse_marks(no, txt)
            for line, src in bad:
                findings.append(Finding("mark-malformed", path, line, "印の形が崩れている: %s" % clip(src)))
            for line, kind, targets in marks:
                t = bind(tests, line, blank_lines)
                if t is None:
                    findings.append(
                        Finding(
                            "mark-orphan", path, line, "印がどのテストにも付いていない: [%s -> %s]" % (kind, clip(", ".join("req~%s~%d" % x for x in targets)))
                        )
                    )
                    continue
                for name, ver in targets:
                    t.links.append({"kind": kind, "name": name, "version": ver, "line": line})
        all_tests.extend(tests)
    rust_assign_heads(root, files, all_tests)
    return all_tests, {"files": len(test_files), "sha256": sha256("\n".join(digests))}


RUST_PATH_MOD = re.compile(r"#\s*\[\s*path\s*=\s*\"([^\"]+)\"\s*\]\s*(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;")


def rust_file_head(path):
    """Rust のファイルの道から決まる mod の道(src/a/b.rs は a::b、a/mod.rs は a、lib.rs・main.rs は根)。src と tests の外なら None。"""
    segs = os.path.splitext(path)[0].split("/")
    cut = max((i for i, x in enumerate(segs) if x == "src"), default=-1)
    if cut < 0:
        cut = max((i for i, x in enumerate(segs) if x == "tests"), default=-1)
        return [] if cut >= 0 and len(segs) == cut + 2 else None  # tests/foo.rs は結合のテストのバイナリの根
    after = segs[cut + 1 :]
    if after[:1] == ["bin"]:
        after = after[2:]
    if after and after[-1] in ("mod", "lib", "main"):
        after = after[:-1]
    return after


def rust_assign_heads(root, files, tests):
    """#[path = "x.rs"] mod 名前; で別のファイルから取り込まれる Rust のテストのファイルに、取り込み先の mod の道(親の道 + 名前)を付ける。
    ファイルの道は x.rs でも、結果の名前は親の道から始まる(src/a.rs の #[path = "test_a.rs"] mod tests; は a::tests::名前)。
    取り込み先が複数あれば候補を全部持つ。親も #[path] で取り込まれていれば、その道をたどる。"""
    rs = {t.path for t in tests if t.lang == "rust"}
    if not rs:
        return
    includes = {}
    for parent in files if files is not None else walk_files(root):
        if not parent.endswith(".rs") or "/target/" in "/" + parent:
            continue
        text = read_text(root, parent)
        if "path" not in text:
            continue
        mask, _ = lex_c(text, "rust")
        for m in re.finditer(r"#\s*\[\s*path\s*=", mask):
            x = RUST_PATH_MOD.match(text, m.start())
            if x:
                target = os.path.normpath(os.path.join(os.path.dirname(parent), x.group(1))).replace(os.sep, "/")
                includes.setdefault(target, []).append((parent, x.group(2)))

    def heads_of(path, depth=0):
        out = []
        for parent, name in includes.get(path, []):
            bases = [rust_file_head(parent)] + (heads_of(parent, depth + 1) if depth < 4 and parent in includes else [])
            out.extend(b + [name] for b in bases if b is not None and b + [name] not in out)
        return out

    for t in tests:
        if t.lang == "rust" and t.path in includes:
            t.heads = heads_of(t.path) or None


def link(reqs, tests, findings, renames=None):
    renames = renames or {}
    for t in tests:
        for ln in t.links:
            r = reqs.get(ln["name"])
            if r is None:
                ln["status"] = "orphaned"
                hint = renames.get(ln["name"])
                findings.append(
                    Finding(
                        "mark-unknown",
                        t.path,
                        ln["line"],
                        "%s の印 req~%s~%d の要求が正本に無い%s"
                        % (clip(t.node, 100), clip(ln["name"], 40), ln["version"], "(%s)" % renamed_note(hint) if hint else ""),
                        key=ln["name"],
                    )
                )
            elif r.dupes:
                ln["status"] = "ambiguous"  # 同じ名前の要求が2つあり、どちらを指すか決まらない(OpenFastTrace の Ambiguous)
            elif ln["version"] < r.version:
                ln["status"] = "outdated"
                findings.append(
                    Finding(
                        "mark-outdated",
                        t.path,
                        ln["line"],
                        "%s の印は req~%s~%d だが、要求は版 %d" % (clip(t.node, 100), r.name, ln["version"], r.version),
                        key=r.name,
                    )
                )
            elif ln["version"] > r.version:
                ln["status"] = "predated"
                findings.append(
                    Finding(
                        "mark-future",
                        t.path,
                        ln["line"],
                        "%s の印は req~%s~%d だが、要求はまだ版 %d" % (clip(t.node, 100), r.name, ln["version"], r.version),
                        key=r.name,
                    )
                )
            else:
                ln["status"] = "covers"
        if t.links and t.skip in ("skip", "xfail"):
            findings.append(
                Finding(
                    "test-skipped",
                    t.path,
                    t.start,
                    "印のあるテスト %s が %s になっている" % (clip(t.node, 100), "skip" if t.skip == "skip" else "失敗してよい印(xfail)"),
                    key=t.node,
                )
            )
        if t.links and t.trivial:
            findings.append(
                Finding(
                    "test-trivial",
                    t.path,
                    t.start,
                    "印のあるテスト %s の断言は、いつも通るか型だけを見るものしか無い" % clip(t.node, 100),
                    key=t.node,
                    severity="warning",
                )
            )
        if t.links and t.empty:
            findings.append(Finding("test-empty", t.path, t.start, "印のあるテスト %s に断言が無い" % clip(t.node, 100), key=t.node))
    for name, r in sorted(reqs.items()):
        if r.dupes:
            continue  # req-duplicate で止めている。曖昧な名前のテストは数えない
        linked = [(t, ln) for t in tests for ln in t.links if ln["name"] == name]
        if r.treatment == "廃止":
            for t, ln in linked:
                findings.append(
                    Finding(
                        "retired-has-tests",
                        t.path,
                        ln["line"],
                        "%s の印は、扱いが廃止の要求 %s を指している(理由: %s)" % (clip(t.node, 100), r.id, clip(r.reason, 60)),
                        key="%s:%s" % (name, t.node),
                    )
                )
            continue
        good = [t for t, ln in linked if ln.get("status") == "covers" and not t.skip and not t.empty and not t.quarantined]
        if r.priority == "P0" and not good:
            if not linked:
                why = "印のテストが無い"
            elif any(ln.get("status") == "covers" and not t.skip and not t.empty and t.quarantined for t, ln in linked):
                why = "今の版の印の有効なテストは、どれも設定の quarantine で隔離されている(隔離したテストは確かめに数えない)"
            elif any(t.skip == "conditional" for t, ln in linked):
                why = "印のテストは条件つきの skip だけで、どの環境で走ったかを静的には確かめられない"
            else:
                why = "印のテストはあるが、今の版で skip でなく断言のあるものが無い"
            findings.append(Finding("p0-no-test", r.path, r.line, "P0 の要求 %s に有効なテストが無い(%s)" % (r.id, why), key=name))
            continue
        if r.needs is None:
            continue
        have = {ln["kind"] for t, ln in linked if ln.get("status") == "covers" and not t.skip and not t.empty and not t.quarantined}
        for kind in sorted(r.needs - have):
            findings.append(
                Finding(
                    "needs-missing",
                    r.path,
                    r.line,
                    "要求 %s は %s の確かめを要るが、その種類の有効なテストが無い" % (r.id, kind),
                    key="%s:%s" % (name, kind),
                    severity="error" if r.priority == "P0" else "warning",
                )
            )
        for t, ln in linked:
            if ln["kind"] not in r.needs:
                findings.append(
                    Finding(
                        "needs-unwanted",
                        t.path,
                        ln["line"],
                        "%s の印 %s は、要求 %s の 確かめ: に無い種類" % (clip(t.node, 100), ln["kind"], r.id),
                        key="%s:%s" % (name, t.node),
                        severity="warning",
                    )
                )


# ---------------------------------------------------------------- 検査(check)と台帳(ledger)


TBD_REF = re.compile(r"(?<![A-Za-z0-9_~.-])tbd~([a-z0-9][a-z0-9._-]*)")  # 要求の名前の一部(req~a.pay-tbd~1)は読まない
DANGLING_EXTS = set(LANG_BY_EXT) | {".sql"}


def lex_sql(text):
    """SQL のコメント [(位置, 文)]。'…' と "…" の中の -- と /* はコメントの始まりと読まない。"""
    comments, i, n = [], 0, len(text)
    while i < n:
        if text.startswith("--", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            comments.append((i, text[i:j]))
            i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            comments.append((i, text[i:j]))
            i = j
        elif text[i] in "'\"":
            j = text.find(text[i], i + 1)
            i = n if j < 0 else j + 1
        else:
            i += 1
    return comments


def code_comments(text, ext):
    """コードのコメントを行ごとに [(行, 文)] で返す。文字列の中の記号はコメントと読まず、ブロックのコメントは全部の行を返す。
    Python は字句の読み(tokenize)で # のコメントだけを返し、docstring は返さない。読めなければ []。"""
    if ext == ".py":
        try:
            return [(tok.start[0], tok.string) for tok in tokenize.generate_tokens(io.StringIO(text).readline) if tok.type == tokenize.COMMENT]
        except (tokenize.TokenError, IndentationError, SyntaxError):
            return []
    comments = lex_sql(text) if ext == ".sql" else lex_c(text, LANG_BY_EXT[ext])[1]
    line_of = line_index(text)
    return [(line_of(pos) + k, part) for pos, body in comments for k, part in enumerate(body.split("\n"))]


def check_inputs(root, cfg, files, findings):
    """設定に書いた置き場が何にも当たらないのに通る事故を知らせる: グロブでない道が無い(エラー)、書いたグロブがどれも1つのファイルにも当たらない(警告)、
    tests に当たったファイルが全部 exclude で外れた(警告)。tests の道は、集めるファイルの道(根から、./ なし)と同じときだけ有ると数える。
    既定の置き場(設定に書いていないもの)は見ない(テストの無い P0 は p0-no-test、要求の無い正本は req-none が言う)。"""
    have = set(files)
    for key, default in (("requirements", DEFAULT_REQUIREMENTS), ("tests", DEFAULT_TESTS)):
        if cfg[key] == default:
            continue
        globs = [p for p in cfg[key] if has_glob(p)]
        for p in cfg[key]:
            if has_glob(p) or (os.path.isfile(os.path.join(root, p)) if key == "requirements" else p in have):
                continue
            hint = "(道は根から書き、./ を付けない)" if key == "tests" and os.path.isfile(os.path.join(root, p)) else ""
            findings.append(Finding("inputs-present", p, None, "設定の %s の置き場 %s が無い%s" % (key, clip(p, 80), hint), key="%s:%s" % (key, p)))
        if globs and not any(glob_match(p, x) for p in globs for x in files):
            findings.append(
                Finding(
                    "inputs-present",
                    globs[0],
                    None,
                    "設定の %s のグロブ(%s)に当たるファイルが1つも無い" % (key, clip("・".join(globs), 80)),
                    key="%s:globs" % key,
                    severity="warning",
                )
            )
        hit = [x for x in files if matches(x, cfg[key])]
        if key == "tests" and hit and not select(hit, cfg[key], cfg["exclude"]):
            findings.append(
                Finding(
                    "inputs-present",
                    cfg[key][0],
                    None,
                    "設定の tests に当たったファイル %d 本が、全部 exclude で外れた(テストは0本になる)" % len(hit),
                    key="tests:excluded",
                    severity="warning",
                )
            )


def check_dangling_opens(root, cfg, files, opens, req_paths, findings):
    """製品のコードのコメントに残った tbd~名前 が、正本の未確定事項に無ければ知らせる(警告。決まったのに消し忘れた仮置きを拾う。テストは見ない)。"""
    known = {o["id"][4:]: o for o in opens}
    skip = set(req_paths)
    for p in files:
        if p in skip or os.path.splitext(p)[1] not in DANGLING_EXTS or matches(p, cfg["exclude"]) or matches(p, cfg["upstream"]) or matches(p, cfg["tests"]):
            continue  # 見るのは製品のコード。テストの見本の文字列は見ない
        full = os.path.join(root, p)
        try:
            if os.path.getsize(full) > 1_000_000:
                continue
            with open(full, "rb") as f:
                text = f.read().decode("utf-8", "replace")
        except OSError:
            continue
        if "tbd~" not in text:
            continue
        for no, comment in code_comments(text, os.path.splitext(p)[1]):
            for m in TBD_REF.finditer(comment):
                name = m.group(1).rstrip(".")  # 文の終わりの . は名前に含めない(up_refs とそろえる)
                if name and name not in known:
                    findings.append(Finding("open-dangling", p, no, "tbd~%s が正本の未確定事項に無い" % name, key="%s:%s" % (p, name), severity="warning"))
                elif name and not known[name].get("provisional"):
                    o = known[name]
                    findings.append(
                        Finding(
                            "open-used-undecided",
                            p,
                            no,
                            "tbd~%s は 仮: が無いまま製品のコードで使われている(決める人: %s)" % (name, o["decider"]),
                            key="%s:%s" % (p, name),
                        )
                    )


def analyze(root, cfg, base=None, strict_open=False, junit=None):
    findings = []
    check_vendor_lock(root, findings)
    check_floor(root, cfg, findings, base)
    reqs, opens, req_paths = load_requirements(root, cfg, findings)
    renames = load_renames(root, cfg, findings)
    check_reused(reqs, renames, findings)
    files = walk_files(root)
    check_inputs(root, cfg, files, findings)
    check_dangling_opens(root, cfg, files, opens, req_paths, findings)
    if base:
        check_adr_added(root, cfg, base, files, findings)
    upstream = check_upstream(root, cfg, reqs, opens, findings) if gate_part_on(root, cfg, base, "upstream") else None
    tests, digest = [], {"files": 0, "sha256": sha256("")}
    approvals, quarantined = None, None
    if reqs:
        if gate_part_on(root, cfg, base, "approvals"):
            approvals = check_approvals(root, cfg, reqs, findings, renames)
        if base:
            check_base(root, cfg, base, reqs, findings, renames)
            if gate_part_on(root, cfg, base, "approvals"):
                check_approvals_base(root, cfg, base, findings)
        tests, digest = scan_tests(root, cfg, findings)
        quarantined = check_quarantine(root, cfg, tests, findings)
        link(reqs, tests, findings, renames)
        if junit:
            other_os = check_runs(reqs, tests, junit, findings, cfg.get("quarantine"))
    for o in opens:
        if strict_open:
            findings.append(Finding("open-item", o["path"], o["line"], "未確定事項 %s が残っている" % o["id"], key=o["id"]))
    findings.sort(key=Finding.sort_key)
    res = {
        "reqs": reqs,
        "opens": opens,
        "req_paths": req_paths,
        "tests": tests,
        "digest": digest,
        "findings": findings,
        "approvals": approvals,
        "upstream": upstream,
        "parts_off": parts_off(cfg),
        "waivers": cfg.get("waivers") or {},
        "quarantined": quarantined if reqs else None,
    }
    if junit and reqs:
        res["other_os"] = other_os  # OS か CPU の条件つきで、この結果では走らなかった印のテストの本数
    return res


def tail_fields(r):
    """台帳と intents に足す任意の欄(書いてあるときだけ)。"""
    return {k: v for k, v in (("treatment", r.treatment), ("reason", r.reason), ("sources", r.sources)) if v}


def req_status(r, tests):
    if r.dupes:
        return "ambiguous"
    if r.treatment == "廃止":
        return "retired"
    linked = [(t, ln) for t in tests for ln in t.links if ln["name"] == r.name]
    if not linked:
        return "no-test"
    if any(ln.get("status") == "covers" and not t.skip and not t.empty and not t.quarantined for t, ln in linked):
        return "verified"
    return "unverified"


def test_state(t):
    return "empty" if t.empty and t.skip in (None, "conditional") else (t.skip or "ok")


def target_names(t):
    """テストが指す要求の名前の集まり(同じ要求を2回指す印は1つに数える)。"""
    return {ln["name"] for ln in t.links}


def verified_by_single(r, tests):
    """要求 r を、1つの要求だけを指すテスト(今の版の印で、飛ばず空でない)で確かめたか。"""
    return any(
        ln["name"] == r.name and ln.get("status") == "covers" and not t.skip and not t.empty and not t.quarantined and len(target_names(t)) == 1
        for t in tests
        for ln in t.links
    )


def counts(res):
    reqs, tests, findings = res["reqs"], res["tests"], res["findings"]
    by_p, verified, retired, multi_only = Counter(), Counter(), 0, Counter()
    ambiguous = sum(1 for r in reqs.values() if r.dupes)
    for r in reqs.values():
        if r.treatment == "廃止":
            retired += 1
            continue
        by_p[r.priority] += 1
        if req_status(r, tests) == "verified":
            verified[r.priority] += 1
            if not verified_by_single(r, tests):
                multi_only[r.priority] += 1
    out = {
        "requirements": len(reqs),
        "by_priority": {p: by_p[p] for p in ("P0", "P1", "P2")},
        "verified": {p: verified[p] for p in ("P0", "P1", "P2")},
        "tests_found": len(tests),
        "tests_marked": sum(1 for t in tests if t.links),
        "tests_multi_target": sum(1 for t in tests if len(target_names(t)) > 1),
        "verified_multi_only": {p: multi_only[p] for p in ("P0", "P1", "P2")},
        "open_items": len(res["opens"]),
        "errors": sum(1 for f in findings if f.severity == "error"),
        "warnings": sum(1 for f in findings if f.severity == "warning"),
    }
    if res.get("approvals") is not None:
        out["approvals"] = res["approvals"]
    if retired:
        out["retired"] = retired
    if ambiguous:
        out["ambiguous"] = ambiguous
    if res.get("upstream") is not None:
        out["upstream"] = res["upstream"]
    if "other_os" in res:
        out["tests_other_os"] = res["other_os"]
    return out


def head_commit(root):
    commit = git(root, "rev-parse", "HEAD", check=False)
    if commit is None:
        return None, None
    status = git(root, "status", "--porcelain", check=False)
    return commit.strip(), bool(status and status.strip())


def build_ledger(root, cfg, res):
    commit, dirty = head_commit(root)
    inputs = []
    for rel in uniq(
        ([os.path.relpath(cfg["_path"], root).replace(os.sep, "/")] if cfg["_path"] else [])
        + res["req_paths"]
        + (requirement_files(root, {"requirements": cfg["upstream"]}) if cfg["upstream"] else [])  # 上流の正本も check の結果を変える
        + ([cfg["approvals"]] if os.path.isfile(os.path.join(root, cfg["approvals"])) else [])
    ):
        with open(os.path.join(root, rel), "rb") as f:
            inputs.append({"path": rel, "sha256": sha256(f.read())})
    rows = []
    for name, r in sorted(res["reqs"].items()):
        links = sorted(
            (
                {"node": t.node, "kind": ln["kind"], "mark_version": ln["version"], "link": ln.get("status"), "state": test_state(t)}
                for t in res["tests"]
                for ln in t.links
                if ln["name"] == name
            ),
            key=lambda x: (x["node"], x["kind"]),
        )
        row = {
            "id": r.id,
            "name": name,
            "version": r.version,
            "priority": r.priority,
            "status": req_status(r, res["tests"]),
            "path": r.path,
            "line": r.line,
            "tests": links,
        }
        row.update(tail_fields(r))
        if r.dupes:
            row["duplicates"] = [{"path": p, "line": n} for p, n in r.dupes]
        rows.append(row)
    return [
        ("format", LEDGER_FORMAT),
        ("format_version", FORMAT_VERSION),
        ("tool", TOOL_NAME),
        ("tool_version", TOOL_VERSION),
        ("commit", commit),
        ("dirty", dirty),
        ("inputs", inputs),
        ("tests_digest", res["digest"]),
        ("counts", counts(res)),
        ("requirements", rows),
        ("open_items", sorted(res["opens"], key=lambda o: o["id"])),
        ("findings", [f.to_dict() for f in res["findings"]]),
    ]


def dump_lines(pairs):
    """見出しの鍵を1行ずつ、配列は1件1行で書く(差分が要求ごとの行に閉じるように)。"""
    one = lambda o: json.dumps(o, ensure_ascii=False, sort_keys=True, separators=(", ", ": "))  # noqa: E731
    lines = ["{"]
    for i, (key, value) in enumerate(pairs):
        comma = "," if i < len(pairs) - 1 else ""
        if isinstance(value, list) and value:
            lines.append("%s: [" % json.dumps(key))
            lines.extend(one(v) + ("," if j < len(value) - 1 else "") for j, v in enumerate(value))
            lines.append("]" + comma)
        else:
            lines.append("%s: %s%s" % (json.dumps(key), one(value), comma))
    lines.append("}")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------- 出力


def render_text(findings, summary, max_per_rule):
    out, by_rule = [], {}
    for f in findings:
        by_rule.setdefault(f.rule, []).append(f)
    for rule in sorted(by_rule):
        items = by_rule[rule]
        for f in items[:max_per_rule]:
            out.append("%s[%s]: %s" % (f.severity, f.rule, esc(f.message)))
            if f.path:
                out.append("  --> %s%s" % (esc(f.path), ":%d" % f.line if f.line else ""))
            out.append("   = help: %s" % RULES.get(f.rule, ""))
        if len(items) > max_per_rule:
            out.append("  … %s はほか %d 件(--format json で全件)" % (rule, len(items) - max_per_rule))
    out.append(summary)
    return "\n".join(out) + "\n"


def render_sarif(findings):
    used = sorted(set(f.rule for f in findings))
    results = []
    for f in findings:
        res = {
            "ruleId": f.rule,
            "level": "error" if f.severity == "error" else "warning",
            "message": {"text": f.message},
            "partialFingerprints": {"traceGate/v1": sha256("%s|%s|%s" % (f.rule, f.path, f.key or f.message))[:32]},
        }
        if f.path:
            loc = {"artifactLocation": {"uri": f.path}}
            if f.line:
                loc["region"] = {"startLine": f.line}
            res["locations"] = [{"physicalLocation": loc}]
        results.append(res)
    doc = {
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": TOOL_NAME,
                        "version": TOOL_VERSION,
                        "rules": [{"id": r, "shortDescription": {"text": r}, "help": {"text": RULES.get(r, "")}} for r in used],
                    }
                },
                "results": results,
            }
        ],
    }
    return json.dumps(doc, ensure_ascii=False, indent=1) + "\n"


GH_LIMIT = 10  # GitHub が1ステップで出す注釈の数(エラーと警告それぞれ。1ジョブでは合わせて 50)
GH_SUMMARY_MAX = 900000  # ジョブの要約に書く上限のバイト数(GitHub は1ステップ 1MiB。超えると黙って欠ける)


def gh_escape(value, prop=False):
    """GitHub の workflow command の値を逃がす(改行で別の命令を混ぜられないように。属性は , と : も)。"""
    s = str(value).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
    return s.replace(":", "%3A").replace(",", "%2C") if prop else s


def counts_text(findings):
    return "エラー %d・警告 %d" % (sum(1 for f in findings if f.severity == "error"), sum(1 for f in findings if f.severity == "warning"))


def render_github(findings, summary):
    """注釈(エラーと警告それぞれ GH_LIMIT 件まで)と、注釈にしなかった指摘の行と、その件数の1行と、要約。"""
    out, shown, rest = [], Counter(), []
    for f in findings:
        level = "error" if f.severity == "error" else "warning"
        if shown[level] >= GH_LIMIT:
            rest.append(f)
            continue
        shown[level] += 1
        props = (["file=" + gh_escape(esc(f.path), True)] + (["line=%d" % f.line] if f.line else []) if f.path else []) + ["title=" + gh_escape(f.rule, True)]
        out.append("::%s %s::%s" % (level, ",".join(props), gh_escape(esc(f.message) + "\nhelp: " + RULES.get(f.rule, ""))))
    for f in rest:
        out.append("%s[%s]: %s%s" % (f.severity, f.rule, esc(f.message), "  --> %s%s" % (esc(f.path), ":%d" % f.line if f.line else "") if f.path else ""))
    if rest:
        out.append("… 注釈にしなかった %d 件(GitHub は1ステップにエラーと警告それぞれ %d 件まで。全件は上の行と --format json)" % (len(rest), GH_LIMIT))
    out.append(summary)
    return "\n".join(out) + "\n"


def step_summary(args, findings, summary, extra):
    """GITHUB_STEP_SUMMARY があれば、要約・全部の指摘の表・確かめていない P0 を Markdown で足す(合否には使わない)。"""
    where = os.environ.get("GITHUB_STEP_SUMMARY")
    if not where:
        return
    lines = ["### trace-gate %s" % args.command, ""] + [md_cell(x) for x in summary.split("\n")] + [""]
    unverified = (extra or {}).get("unverified_p0")
    if unverified:
        lines += ["確かめていない P0: " + "・".join("`%s`" % md_cell(x) for x in unverified), ""]
    if findings:
        lines += ["| 重さ | 規則 | 場所 | 文 |", "|---|---|---|---|"]
        size = len("\n".join(lines).encode("utf-8"))
        for i, f in enumerate(findings):
            row = "| %s | %s | %s | %s |" % (f.severity, f.rule, md_cell("%s%s" % (f.path or "", ":%d" % f.line if f.line else "")), md_cell(f.message))
            size += len(row.encode("utf-8")) + 1
            if size > GH_SUMMARY_MAX:
                lines.append("… ほか %d 件(要約の大きさの上限。全件は --format json)" % (len(findings) - i))
                break
            lines.append(row)
    with open(where, "a", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n\n")


def emit(args, findings, summary, extra=None):
    if args.format == "github":
        sys.stdout.write(render_github(findings, summary))
        step_summary(args, findings, summary, extra)
    elif args.format == "json":
        doc = {"format": FINDINGS_FORMAT, "format_version": FORMAT_VERSION, "tool": TOOL_NAME, "tool_version": TOOL_VERSION}
        doc.update(extra or {})
        doc["findings"] = [f.to_dict() for f in findings]
        sys.stdout.write(json.dumps(doc, ensure_ascii=False, sort_keys=True, indent=1) + "\n")
    elif args.format == "sarif":
        sys.stdout.write(render_sarif(findings))
    else:
        sys.stdout.write(render_text(findings, summary, args.max_per_rule))
    return 1 if any(f.severity == "error" for f in findings) else 0


def summary_line(res):
    c = counts(res)
    line = "trace-gate: 要求 %d 件(P0 %d)・印のあるテスト %d 本・確かめた P0 %d / %d・未確定 %d 件・エラー %d・警告 %d" % (
        c["requirements"],
        c["by_priority"]["P0"],
        c["tests_marked"],
        c["verified"]["P0"],
        c["by_priority"]["P0"],
        c["open_items"],
        c["errors"],
        c["warnings"],
    )
    if c["tests_multi_target"]:
        line += "・複数の要求を指すテスト %d 本(それだけで確かめた P0 %d)" % (c["tests_multi_target"], c["verified_multi_only"]["P0"])
    if "approvals" in c:
        a = c["approvals"]
        line += "・承認の記録: 承認済み %d・未承認 %d・再承認が要る %d" % (a["approved"], a["unapproved"], a["stale"])
    if "retired" in c:
        line += "・廃止 %d 件(数えない)" % c["retired"]
    if "ambiguous" in c:
        line += "・名前が曖昧 %d 件(確かめた数に入れない)" % c["ambiguous"]
    if "upstream" in c:
        line += "・上流の項目 %d(写し漏れ %d)" % (c["upstream"]["items"], c["upstream"]["unmapped"])
    if res.get("parts_off"):
        waivers = res.get("waivers") or {}
        line += "・切った部品: %s" % "・".join(
            "%s(理由: %s、期限 %s。%s)" % (p, waivers[p]["reason"], waivers[p]["until"], PARTS[p]) if p in waivers else "%s(%s)" % (p, PARTS[p])
            for p in res["parts_off"]
        )
    return line


def run_note(res, cfg):
    """check の要約に足す、走らせた結果の不安定な本数と隔離の一言と、OS の条件でこの結果では走らなかった本数。"""
    flaky = sum(1 for f in res["findings"] if f.rule == "test-flaky")
    other = res.get("other_os", 0)
    return (
        ("・不安定 %d 本" % flaky if flaky else "")
        + quarantine_note(cfg.get("quarantine"), res.get("quarantined"))
        + ("・この OS では走らない %d 本(OS か CPU の条件つき。ほかの OS の結果で照らす)" % other if other else "")
    )


def unverified_p0(res):
    """確かめていない P0(廃止を除く)の ID。ジョブの要約と JSON に出す。"""
    return [r.id for _, r in sorted(res["reqs"].items()) if r.priority == "P0" and req_status(r, res["tests"]) not in ("verified", "retired")]


def cmd_check(args, root, cfg):
    if args.base_junit and not args.junit:
        raise UsageError("--base-junit は --junit(head の結果)と一緒に書く")
    drops = count_drops("test-count-dropped", args.base_junit, args.junit) if args.base_junit else []
    pkgs = package_dirs(root, cfg)
    if not pkgs:
        res = analyze(root, cfg, base=args.base, strict_open=args.strict_open, junit=args.junit)
        res["findings"] = sorted(res["findings"] + drops, key=Finding.sort_key)
        c = counts(res)
        c["parts_off"] = res["parts_off"]
        return emit(args, res["findings"], summary_line(res) + run_note(res, cfg), {"counts": c, "unverified_p0": unverified_p0(res)})
    every, lines, rows, unverified = list(drops), [], [], []
    for head, sroot, scfg in scopes(root, cfg, pkgs):
        res = analyze(sroot, scfg, base=args.base, strict_open=args.strict_open, junit=args.junit)
        every.extend(prefixed(res["findings"], head))
        lines.append("%s %s" % (head.rstrip("/") or ".", summary_line(res) + run_note(res, scfg)))
        rows.append({"path": head.rstrip("/") or ".", "counts": counts(res)})
        unverified += [head + x for x in unverified_p0(res)]
    every.sort(key=Finding.sort_key)
    lines.append("trace-gate: 全体 %s(パッケージ %d)" % (counts_text(every), len(rows)))
    total = {
        "errors": sum(1 for f in every if f.severity == "error"),
        "warnings": sum(1 for f in every if f.severity == "warning"),
        "parts_off": parts_off(cfg),
    }
    return emit(args, every, "\n".join(lines), {"counts": total, "packages": rows, "unverified_p0": unverified})


def part_switched_off(args, name, where="設定"):
    """部品が切られているときの知らせ。何も調べずに 0 で返す。"""
    msg = "trace-gate %s: 部品 %s は%sで切られている(%s)。何も調べなかった" % (args.command, name, where, PARTS[name])
    return emit(args, [], msg, {"switched_off": name, "message": msg})


def cmd_ledger(args, root, cfg):
    prev = read_ledger(args.compare) if args.compare else None
    res = analyze(root, cfg, base=args.base)
    pairs = build_ledger(root, cfg, res)
    if prev:
        header, changes = compare_ledgers(prev[0], prev[1], dict(pairs)["requirements"])
        at = [k for k, _ in pairs].index("findings")
        pairs[at:at] = [("compare", header), ("changes", changes)]
        sys.stderr.write("前の台帳との比べ: %s\n" % "・".join("%s %d" % kv for kv in header["counts"].items()))
        if header["previous_tool_version"] != TOOL_VERSION:
            sys.stderr.write(
                "前の台帳は %s %s で作った(今は %s)。検査器の読みの違いが updated に混ざりうる\n"
                % (TOOL_NAME, clip(header["previous_tool_version"], 30), TOOL_VERSION)
            )
    text = dump_lines(pairs)
    if args.out and args.out != "-":  # - は標準出力(「-」という名前のファイルを作らない)
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(text)
        sys.stderr.write("台帳を書いた: %s(%s)\n" % (args.out, summary_line(res)))
    else:
        sys.stdout.write(text)
    return 1 if any(f.severity == "error" for f in res["findings"]) else 0


def cmd_approve(args, root, cfg):
    """人が宣言したときだけ、承認の記録に追記する。端末で yes と打ったときだけ書く(AI の Bash には端末が無い)。"""
    if not part_on(cfg, "approvals"):
        raise UsageError("部品 approvals は設定で切られている。承認の記録を使うなら trace-gate.json の parts.approvals を true にする")
    if not sys.stdin.isatty():
        raise UsageError("approve は人が端末で走らせる(承認の記録は AI が書かない)。標準入力が端末でないので書かなかった")
    findings = []
    reqs, _, _ = load_requirements(root, cfg, findings)
    errors = [f for f in findings if f.severity == "error"]
    if errors:
        sys.stdout.write(render_text(errors, "承認しなかった(要求の正本の誤りを先に直す)", 50))
        return 1
    names = sorted(reqs)
    if args.req:
        names = [n.strip() for n in args.req.split(",") if n.strip()]
        for n in names:
            if n not in reqs:
                raise UsageError("--req の要求が正本に無い: %s" % clip(n, 40))
    sys.stdout.write("承認する要求(%d 件。承認者 %s・意味 %s):\n" % (len(names), clip(args.by, 40), clip(args.meaning, 60)))
    for n in names:
        r = reqs[n]
        sys.stdout.write("  %s %s 指紋 %s %s\n" % (r.id, r.priority, r.fingerprint, clip(r.text, 80)))
    sys.stdout.write("中身を読んで承認する場合は yes と打つ: ")
    sys.stdout.flush()
    answer = sys.stdin.readline().strip()
    if answer != "yes":
        sys.stdout.write("\n承認しなかった\n")
        return 1
    today = __import__("datetime").date.today().isoformat()

    def row(r):
        out = {
            "name": r.name,
            "version": r.version,
            "fingerprint": r.fingerprint,
            "by": args.by,
            "date": today,
            "meaning": args.meaning,
            "priority": r.priority,
        }
        if r.needs is not None:
            out["needs"] = sorted(r.needs)  # 承認した確かめの種類。減ったら approval-stale にする
        return out

    lines = [json.dumps(row(reqs[n]), ensure_ascii=False, sort_keys=True) for n in names]
    path = os.path.join(root, cfg["approvals"])
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "a", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    sys.stdout.write("\n承認の記録に %d 行を足した: %s\n" % (len(lines), cfg["approvals"]))
    return 0


# ---------------------------------------------------------------- テストの弱め(TG-12)


def py_info(text):
    """{修飾名: {"skip", "total", "strong", "nums"}} と、ファイルの skip。読めなければ None。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return None
    info = {}
    for qual, fn, inherited in py_tests(tree):
        total, strong = py_assertions(fn)
        nums, lits = Counter(), Counter()
        for n in py_assert_parts(fn):
            for c in ast.walk(n):
                if isinstance(c, ast.Constant) and (isinstance(c.value, (str, bool)) or c.value is None):
                    lits[repr(c.value)] += 1
        for part in list(fn.decorator_list) + list(fn.body):
            for n in ast.walk(part):
                if isinstance(n, ast.Constant) and isinstance(n.value, (int, float, complex)) and not isinstance(n.value, bool):
                    nums[repr(n.value)] += 1
        key = (qual + "::" if qual else "") + fn.name
        info[key] = {
            "skip": decorator_skip(fn.decorator_list) or inherited or body_skip(fn),
            "total": total,
            "strong": strong,
            "nums": nums,
            "lits": lits,
            "line": fn.lineno,
            "shape": fn_shape(fn),
            "kind": type(fn).__name__,
        }
    return info, module_skip(tree)


def py_assert_parts(fn):
    """断言の中で期待値を持つ部分(assert 文の条件、assert で始まる呼び出しの引数)。assert 文の失敗のときの文は除く。"""
    out = []
    for n in ast.walk(fn):
        if isinstance(n, ast.Assert):
            out.append(n.test)
        elif isinstance(n, ast.Call):
            name = getattr(n.func, "attr", None) or getattr(n.func, "id", None) or ""
            if name.startswith("assert"):
                out.extend(list(n.args) + [k.value for k in n.keywords if k.arg != "msg"])
    return out


def fn_shape(fn):
    """名前と docstring を除いた、テストの関数の形(飾り・引数・本体)。改名の見分けに使う。"""
    body = list(fn.body)
    if body and isinstance(body[0], ast.Expr) and isinstance(getattr(body[0], "value", None), ast.Constant) and isinstance(body[0].value.value, str):
        body = body[1:]
    return sha256(type(fn).__name__ + "\n" + "\n".join(ast.dump(x) for x in list(fn.decorator_list) + [fn.args] + body))


def counter_text(c):
    return clip(", ".join("%s×%d" % (k, v) if v > 1 else k for k, v in sorted(c.items())), 60) or "なし"


def weaken_python(path, a, b, findings, ctx=None):
    old, new = py_info(a), py_info(b)
    if old is None or new is None:
        return False
    (oi, oskip), (ni, nskip) = old, new
    ctx = ctx or {}
    om, nm = ctx.get("old_marks", {}), ctx.get("new_marks", {})
    added = [k for k in ni if k not in oi]
    if nskip and not oskip:
        findings.append(Finding("xfail-added" if nskip == "xfail" else "skip-added", path, None, "ファイルの pytestmark に %s が足された" % nskip, key=path))
    for name, o in sorted(oi.items()):
        n = ni.get(name)
        node = "%s::%s" % (path, name)
        if n is None:
            # 改名とみなすのは、同じクラスの中で、本体・印・飛ぶかどうか(クラスから受け継ぐ skip を含む)が同じときだけ
            twin = next(
                (
                    k
                    for k in added
                    if k.rpartition("::")[0] == name.rpartition("::")[0]
                    and ni[k]["shape"] == o["shape"]
                    and ni[k]["skip"] == o["skip"]
                    and nm.get(k) == om.get(name)
                ),
                None,
            )
            if twin is not None:
                added.remove(twin)
                findings.append(
                    Finding(
                        "test-renamed",
                        path,
                        ni[twin]["line"],
                        "テスト %s の名前が %s に変わった(本体と印は同じ)" % (clip(node, 100), clip(twin, 80)),
                        key=node,
                        severity="warning",
                    )
                )
                continue
            if retire(ctx, path, om.get(name), 1):
                continue
            findings.append(Finding("test-removed", path, None, "テスト %s が消えた" % clip(node, 100), key=node))
            continue
        if n["kind"] != o["kind"]:
            findings.append(
                Finding(
                    "test-kind-changed",
                    path,
                    n["line"],
                    "テスト %s の関数の種類が %s から %s に変わった"
                    % (clip(node, 100), "async def" if o["kind"] == "AsyncFunctionDef" else "def", "async def" if n["kind"] == "AsyncFunctionDef" else "def"),
                    key=node,
                )
            )
        lost = sorted((om.get(name) or frozenset()) - (nm.get(name) or frozenset()))
        if lost:
            findings.append(
                Finding(
                    "mark-removed",
                    path,
                    n["line"],
                    "テスト %s の印から %s が外れた" % (clip(node, 100), clip(", ".join("%s -> %s" % (k, r) for k, r in lost), 120)),
                    key=node,
                )
            )
        if n["skip"] and not o["skip"] and not (nskip and not oskip):
            rule = "xfail-added" if n["skip"] == "xfail" else "skip-added"
            findings.append(Finding(rule, path, n["line"], "テスト %s に %s が足された" % (clip(node, 100), n["skip"]), key=node))
        if n["total"] < o["total"]:
            findings.append(
                Finding("assert-reduced", path, n["line"], "テスト %s の断言が %d から %d に減った" % (clip(node, 100), o["total"], n["total"]), key=node)
            )
        elif n["strong"] < o["strong"]:
            findings.append(
                Finding(
                    "assert-loosened",
                    path,
                    n["line"],
                    "テスト %s の値を比べる断言が %d から %d に減った(緩めた)" % (clip(node, 100), o["strong"], n["strong"]),
                    key=node,
                )
            )
        if n["nums"] != o["nums"] or n["lits"] != o["lits"]:
            gone, new_ = (o["nums"] - n["nums"]) + (o["lits"] - n["lits"]), (n["nums"] - o["nums"]) + (n["lits"] - o["lits"])
            findings.append(
                Finding(
                    "literal-changed",
                    path,
                    n["line"],
                    "テスト %s の期待の数値か文字列・真偽値が変わった(消えた: %s / 足された: %s)" % (clip(node, 100), counter_text(gone), counter_text(new_)),
                    key=node,
                )
            )
    return True


NUM_RE = re.compile(r"(?<![\w.])\d+(?:\.\d+)?(?![\w.])")
# 断言の行の期待の文字列と真偽値(引用符の文字列と true・false・null・nil・undefined)
LIT_RE = re.compile(r"(['\"`])(?:\\.|(?!\1)[^\\\n])*\1|(?<![\w$])(?:true|false|null|nil|undefined)(?![\w$])")


def code_lines(text, lang):
    if lang == "python":
        return [re.sub(r"#.*$", "", ln) for ln in text.split("\n")]
    mask, _ = lex_c(text, lang)
    return mask.split("\n")


SKIP_RANK = {None: 0, "conditional": 1, "skip": 2, "xfail": 2}


def weaken_states(old_path, path, a, b, findings):
    """base と head の同じテストを読み、飛ぶかどうかが強まった(走る→条件つき→飛ぶ)テストに skip-added を出す。
    文字の差分の skip の語では見えない書き方(Go の札とファイルの名・Rust の cfg・定数の skipIf)を拾う。文字の差分が同じテストに出したものは重ねない。"""
    lang = LANG_BY_EXT[os.path.splitext(path)[1]]
    before, _ = scan_file(old_path, a, lang)
    after, _ = scan_file(path, b, lang)
    if before is None or after is None:
        return
    olds = {((t.qual + "::") if t.qual else "") + t.name: t for t in before[0]}
    shown = [(f.line or 0) for f in findings if f.path == path and f.rule in ("skip-added", "xfail-added")]
    for t in after[0]:
        o = olds.get(((t.qual + "::") if t.qual else "") + t.name)
        if o is None or SKIP_RANK[t.skip] <= SKIP_RANK[o.skip] or any(t.start <= x <= t.end for x in shown):
            continue
        rule = "xfail-added" if t.skip == "xfail" else "skip-added"
        findings.append(
            Finding(rule, path, t.start, "テスト %s が %s になった(札・cfg・定数の条件・ファイルの名で外れる)" % (clip(t.node, 100), t.skip), key=t.node)
        )


def weaken_text(path, a, b, lang, findings):
    a_raw, b_raw = a.split("\n"), b.split("\n")
    a_code, b_code = code_lines(a, lang), code_lines(b, lang)
    sm = difflib.SequenceMatcher(None, a_raw, b_raw, autojunk=False)
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            continue
        removed, added = "\n".join(a_code[i1:i2]), "\n".join(b_code[j1:j2])
        line = j1 + 1 if j2 > j1 else max(j1, 1)
        for rule, words in (("skip-added", SKIP_WORDS[lang]), ("xfail-added", XFAIL_WORDS[lang])):
            if len(words.findall(added)) > len(words.findall(removed)):
                findings.append(
                    Finding(
                        rule,
                        path,
                        line,
                        "%s の %d 行目に %s の印が足された" % (path, line, "skip" if rule == "skip-added" else "失敗してよい"),
                        key="%s:%d" % (path, line),
                    )
                )
        ra, aa = len(ASSERT_WORDS[lang].findall(removed)), len(ASSERT_WORDS[lang].findall(added))
        if ra > aa:
            findings.append(
                Finding("assert-reduced", path, line, "%s の %d 行目のあたりで断言が %d 個減った" % (path, line, ra - aa), key="%s:%d" % (path, line))
            )
        if tag == "replace":
            rn, an = Counter(NUM_RE.findall(removed)), Counter(NUM_RE.findall(added))
            rl = Counter(m.group(0) for k in range(i1, i2) if ASSERT_WORDS[lang].search(a_code[k]) for m in LIT_RE.finditer(a_raw[k]))
            al = Counter(m.group(0) for k in range(j1, j2) if ASSERT_WORDS[lang].search(b_code[k]) for m in LIT_RE.finditer(b_raw[k]))
            rn, an = rn + rl, an + al
            if rn != an:
                findings.append(
                    Finding(
                        "literal-changed",
                        path,
                        line,
                        "%s の %d 行目のあたりで数値が変わった(消えた: %s / 足された: %s)" % (path, line, counter_text(rn - an), counter_text(an - rn)),
                        key="%s:%d" % (path, line),
                    )
                )
    da, db = len(DECL_WORDS[lang].findall("\n".join(a_code))), len(DECL_WORDS[lang].findall("\n".join(b_code)))
    if db < da:
        findings.append(Finding("test-removed", path, None, "%s のテストの宣言が %d から %d に減った" % (path, da, db), key=path))


def test_spans(path, text):
    """{テストの修飾名: (始まりの行, 終わりの行, {(種類, 要求): 版})}。読めなければ None。"""
    lang = LANG_BY_EXT.get(os.path.splitext(path)[1])
    if lang is None:
        return None
    result, _ = scan_file(path, text, lang)
    if result is None:
        return None
    tests, comments, blank_lines = result
    out = {((t.qual + "::") if t.qual else "") + t.name: (t.start, t.end, {}) for t in tests}
    for no, txt, _ in comments:
        marks, _ = parse_marks(no, txt)
        for line, kind, targets in marks:
            t = bind(tests, line, blank_lines)
            if t is not None:
                vers = out[((t.qual + "::") if t.qual else "") + t.name][2]
                for n, v in targets:
                    vers[(kind, n)] = max(vers.get((kind, n), 0), v)
    return out


def renamed_marks(marks, renamed):
    """{テスト: {(種類, 名前)}} の名前を、付け替えの対 {旧い名前: 新しい名前} で新しい名前に読み替える。"""
    if not marks or not renamed:
        return marks
    return {k: frozenset((kind, renamed.get(n, n)) for kind, n in v) for k, v in marks.items()}


def weaken_marks(path, a, b, lang, findings, renamed=None):
    """印の版だけを上げた変更(mark-bumped。どの言語でも)と、Python 以外の印の外し(mark-removed)。"""
    old, new = test_spans(path, a), test_spans(path, b)
    if old is None or new is None:
        return
    if renamed:
        for k, (s_, e_, vers) in list(old.items()):
            moved = {}
            for (kind, n), v in vers.items():
                key = (kind, renamed.get(n, n))
                moved[key] = max(moved.get(key, 0), v)
            old[k] = (s_, e_, moved)
    ac, bc = code_lines(a, lang), code_lines(b, lang)
    for name, (os_, oe, om) in sorted(old.items()):
        if name not in new:
            continue
        ns, ne, nm = new[name]
        node = "%s::%s" % (path, name)
        lost = sorted(set(om) - set(nm))
        if lost and lang != "python":
            findings.append(
                Finding(
                    "mark-removed", path, ns, "テスト %s の印から %s が外れた" % (clip(node, 100), clip(", ".join("%s -> %s" % k for k in lost), 120)), key=node
                )
            )
        bumped = sorted(k for k in nm if k in om and nm[k] > om[k])
        same_body = [x.strip() for x in ac[os_ - 1 : oe]] == [x.strip() for x in bc[ns - 1 : ne]]
        if bumped and same_body:
            findings.append(
                Finding(
                    "mark-bumped",
                    path,
                    ns,
                    "テスト %s の本体を変えずに、印 %s の版だけが上がった"
                    % (clip(node, 100), clip(", ".join("%s -> req~%s~%d" % (k[0], k[1], nm[k]) for k in bumped), 120)),
                    key=node,
                )
            )


def test_marks(path, text):
    """{テストの修飾名: 印の要求の名前の集合}。Python は「クラス::名前」、ほかの言語は名前。読めなければ None。"""
    lang = LANG_BY_EXT.get(os.path.splitext(path)[1])
    if lang is None:
        return None
    result, _ = scan_file(path, text, lang)
    if result is None:
        return None
    tests, comments, blank_lines = result
    out = {((t.qual + "::") if t.qual else "") + t.name: set() for t in tests}
    for no, txt, _ in comments:
        marks, _ = parse_marks(no, txt)
        for line, kind, targets in marks:
            t = bind(tests, line, blank_lines)
            if t is not None:
                out[((t.qual + "::") if t.qual else "") + t.name].update((kind, n) for n, _ in targets)
    return {k: frozenset(v) for k, v in out.items()}


# ---------------------------------------------------------------- 断言の行を変えずにテストを殺す形(test-control-added)と、テストを見分ける製品のコード(test-aware-code)

CONTROL_WORDS = {
    "return": "断言より前の return",
    "swallow": "断言を包んで握りつぶす try/except・catch",
    "if": "断言を包む if",
    "async": "断言を非同期へ逃がす .then(…)",
}


def py_is_assert(n, helpers=()):
    if isinstance(n, ast.Assert):
        return True
    if isinstance(n, ast.Call):
        name = dotted(n.func)
        last = name.split(".")[-1]
        return last.startswith(("assert", "expect")) or name in ("pytest.raises", "pytest.warns", "pytest.fail", "self.fail") or last in helpers
    return False


def py_swallows(handler, helpers=()):
    """except の節が、raise も断言も持たずに例外を握りつぶすか。"""
    return not any(isinstance(n, ast.Raise) or py_is_assert(n, helpers) for n in ast.walk(handler))


def py_control(fn, helpers=()):
    """テストの関数の中の、断言の行を変えずにテストを殺す形の数 {形: 数}(入れ子の関数の中は見ない)。
    return は最後の断言より前の return、swallow は握りつぶす try(か contextlib.suppress)の中の断言、if は if の中の断言。"""
    asserts, returns, out = [], [], Counter()

    def visit(node, ctx):
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
                continue
            c = ctx
            if isinstance(child, ast.Return):
                returns.append(child.lineno)
            if py_is_assert(child, helpers):
                asserts.append(child.lineno)
                for k in ctx:
                    out[k] += 1
            if isinstance(child, ast.If):
                visit_if(child, ctx)
                continue
            if isinstance(child, ast.Try):
                visit_try(child, ctx)
                continue
            if isinstance(child, (ast.With, ast.AsyncWith)) and any(
                isinstance(i.context_expr, ast.Call) and dotted(i.context_expr.func).split(".")[-1] == "suppress" for i in child.items
            ):
                c = ctx | {"swallow"}
            visit(child, c)

    def visit_if(node, ctx):
        visit(node.test, ctx)
        for part in node.body + node.orelse:
            visit(ast.Module(body=[part], type_ignores=[]), ctx | {"if"})

    def visit_try(node, ctx):
        swallow = any(py_swallows(h, helpers) for h in node.handlers)
        for part in node.body:
            visit(ast.Module(body=[part], type_ignores=[]), ctx | {"swallow"} if swallow else ctx)
        for part in node.handlers + node.orelse + node.finalbody:
            visit(ast.Module(body=[part], type_ignores=[]), ctx)

    visit(ast.Module(body=fn.body, type_ignores=[]), frozenset())
    if asserts:
        out["return"] = sum(1 for r in returns if r < max(asserts))
    return out


def py_controls(text, helpers=()):
    """{修飾名: (行, {形: 数})}。読めなければ None。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return None
    return {(qual + "::" if qual else "") + fn.name: (fn.lineno, py_control(fn, helpers)) for qual, fn, _ in py_tests(tree)}


JS_SWALLOW_CATCH = re.compile(r"\bcatch\s*(?:\([^)]*\)\s*)?\{")
JS_TRY = re.compile(r"\btry\s*\{")
JS_PROMISE_CATCH = re.compile(r"\.\s*catch\s*\(")
JS_THEN = re.compile(r"\.\s*then\s*\(")
JS_IF = re.compile(r"\bif\s*\(")
JS_RETURN = re.compile(r"\breturn\b")
JS_FAILS = re.compile(r"\bthrow\b|\breject\s*\(|\bdone\s*\(\s*\w")


def js_control(body, lang="js"):
    """JS・TS のテストの本体(文字列とコメントを空白にした写し)の、断言の行を変えずにテストを殺す形の数 {形: 数}。"""
    words = ASSERT_WORDS[lang]
    hits = [m.start() for m in words.finditer(body)]
    out = Counter()
    if not hits:
        return out
    last = max(hits)
    out["return"] = sum(1 for m in JS_RETURN.finditer(body) if m.start() < last)
    for m in JS_SWALLOW_CATCH.finditer(body):
        close = match_close(body, m.end() - 1, "{", "}")
        if not JS_FAILS.search(body[m.end() : close]) and not words.search(body[m.end() : close]):
            tries = [t.start() for t in JS_TRY.finditer(body, 0, m.start())]
            out["swallow"] += sum(1 for h in hits if tries[-1] <= h < m.start()) if tries else 0
    for m in JS_PROMISE_CATCH.finditer(body):
        close = match_close(body, m.end() - 1, "(", ")")
        inner = body[m.end() : close]
        if not JS_FAILS.search(inner) and not words.search(inner):
            out["swallow"] += 1
    for m in JS_THEN.finditer(body):
        close = match_close(body, m.end() - 1, "(", ")")
        if words.search(body[m.end() : close]):
            out["async"] += 1
    for m in JS_IF.finditer(body):
        close = match_close(body, m.end() - 1, "(", ")")
        rest = body[close + 1 :]
        stripped = rest.lstrip()
        if stripped.startswith("{"):
            at = close + 1 + (len(rest) - len(stripped))
            block = body[at : match_close(body, at, "{", "}")]
        else:
            block = stripped.split(";", 1)[0]
        out["if"] += len(words.findall(block))
    return out


def js_controls(path, text, lang):
    """{修飾名: (行, {形: 数})}。読めなければ None。"""
    result, _ = scan_file(path, text, lang)
    if result is None:
        return None
    mask = code_lines(text, lang)
    return {((t.qual + "::") if t.qual else "") + t.name: (t.start, js_control("\n".join(mask[t.start - 1 : t.end]), lang)) for t in result[0]}


def weaken_control(path, a, b, lang, findings, helpers=()):
    """base と head の両方にあるテストで、断言より前の return・握りつぶす try・断言を包む if・非同期へ逃がす形が増えたら test-control-added。
    断言の行を変えずにテストを殺す形は、断言の数も期待値も変えないので、ほかの規則に出ないため(Python と JS・TS)。"""
    if lang == "python":
        old, new = py_controls(a, helpers), py_controls(b, helpers)
    elif lang == "js":
        old, new = js_controls(path, a, lang), js_controls(path, b, lang)
    else:
        return
    if old is None or new is None:
        return
    for name, (line, got) in sorted(new.items()):
        if name not in old:
            continue
        more = [k for k in CONTROL_WORDS if got[k] > old[name][1][k]]
        if more:
            node = "%s::%s" % (path, name)
            findings.append(
                Finding(
                    "test-control-added",
                    path,
                    line,
                    "テスト %s に、断言の行を変えずにテストを通す形が足された: %s" % (clip(node, 100), "・".join(CONTROL_WORDS[k] for k in more)),
                    key=node,
                    severity=TEST_CONTROL_SEVERITY,
                )
            )


# 断言の差し替え(assert・expect の名前への代入、TestCase の assert* の差し替え、setattr)と、断言の補助を空にする形
ASSERT_SWAP_RE = re.compile(
    r"(?:\bassert\w*|\brequire\s*\(\s*['\"](?:node:)?assert(?:/strict)?['\"]\s*\))(?:\s*\.\s*\w+)+\s*=(?![=>])"
    r"|\.\s*(?:assert\w*|fail)\s*=(?![=>])"
    r"|\bsetattr\s*\([^,\n]*,\s*['\"](?:assert\w*|fail)['\"]"
    r"|^\s*(?:(?:const|let|var)\s+)?(?:assert|expect)\w*\s*=(?![=>])(?!\s*(?:require\s*\(|await\s+import\s*\())",
    re.M,
)
JS_HELPER_DEF = re.compile(
    r"\bfunction\s+((?:assert|expect)\w*)\s*\(|\b(?:const|let|var)\s+((?:assert|expect)\w*)\s*=\s*(?:async\s*)?(?:function\b|\([^)]*\)\s*=>|[\w$]+\s*=>)"
)


def helper_bodies(text, lang):
    """名前が assert・expect で始まる、テストでない関数(補助)の {名前: 本体が断言か throw・raise を持つか}。"""
    out = {}
    if lang == "python":
        try:
            tree = ast.parse(text)
        except (SyntaxError, ValueError):
            return out
        for n in ast.walk(tree):
            if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith(("assert", "expect")):
                out[n.name] = any(isinstance(x, ast.Raise) or py_is_assert(x) for x in ast.walk(n) if x is not n)
        return out
    if lang != "js":
        return out
    mask = "\n".join(code_lines(text, lang))
    for m in JS_HELPER_DEF.finditer(mask):
        name = m.group(1) or m.group(2)
        brace = mask.find("{", m.end() - 1)
        if brace < 0:
            continue
        body = mask[brace + 1 : match_close(mask, brace, "{", "}")]
        out[name] = bool(JS_FAILS.search(body) or ASSERT_WORDS[lang].search(body))
    return out


def py_swaps(text):
    """Python のファイルの、断言の名前への代入と差し替えの数(構文の木で数える。文字列の中は数えない)。読めなければ 0。
    assert・fail で始まる属性への代入(unittest.TestCase.assertEqual = …)、assert・expect で始まる名前への import でない代入、
    assert・fail で始まる名前の setattr(monkeypatch.setattr を含む)。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return 0
    n = 0
    for node in ast.walk(tree):
        targets = node.targets if isinstance(node, ast.Assign) else [node.target] if isinstance(node, (ast.AugAssign, ast.AnnAssign)) else []
        for t in targets:
            if isinstance(t, ast.Attribute) and t.attr.startswith(("assert", "fail")):
                n += 1
            elif isinstance(t, ast.Name) and t.id.startswith(("assert", "expect")):
                n += 1
        if isinstance(node, ast.Call) and dotted(node.func).split(".")[-1] == "setattr":
            n += sum(1 for x in node.args[1:2] if isinstance(x, ast.Constant) and isinstance(x.value, str) and x.value.startswith(("assert", "fail")))
    return n


def weaken_assert_swaps(path, a, b, lang, findings):
    """テストの側のファイル(テストと補助)で、断言の名前への代入か差し替えが増えたか、断言を持っていた補助が断言も throw も持たなくなったら
    test-control-added(ファイルの行なし)。"""
    if lang not in ("python", "js"):
        return
    what = []
    if lang == "python":
        swapped = py_swaps(b) > py_swaps(a)
    else:
        ca, cb = "\n".join(code_lines(a, lang)), "\n".join(code_lines(b, lang))
        swapped = len(ASSERT_SWAP_RE.findall(cb)) > len(ASSERT_SWAP_RE.findall(ca))
    if swapped:
        what.append("断言の関数か assert の差し替え")
    ha, hb = helper_bodies(a, lang), helper_bodies(b, lang)
    emptied = sorted(n for n, ok in hb.items() if not ok and ha.get(n))
    if emptied:
        what.append("断言を持たなくなった補助 %s" % clip("・".join(emptied), 80))
    if what:
        findings.append(
            Finding(
                "test-control-added",
                path,
                None,
                "%s に、断言の行を変えずにテストを通す形が足された: %s" % (path, "・".join(what)),
                key=path,
                severity=TEST_CONTROL_SEVERITY,
            )
        )


# テストの最中を見分ける目印(製品のコードがテストのときだけ違う振る舞いをする形)
TEST_AWARE_RE = re.compile(
    r"\b(?:NODE_TEST_CONTEXT|JEST_WORKER_ID|VITEST(?:_\w+)?|PYTEST_CURRENT_TEST|PYTEST_VERSION)\b"
    r"|['\"]pytest['\"]\s+in\s+sys\.modules|sys\.modules\s*(?:\.get\s*\(|\[)\s*['\"]pytest['\"]"
    r"|\bcfg!\s*\(\s*test\s*\)|\btesting\.Testing\s*\(\s*\)|\bflag\.Lookup\s*\(\s*['\"]test\.v['\"]"
)
TEST_AWARE_SWAP_RE = re.compile(
    r"(?:\bassert\w*|\brequire\s*\(\s*['\"](?:node:)?assert(?:/strict)?['\"]\s*\))(?:\s*\.\s*\w+)+\s*=(?![=>])"
    r"|\.\s*assert\w*\s*=(?![=>])|\bsetattr\s*\([^,\n]*,\s*['\"]assert\w*['\"]"
)


def test_aware_added(a, b):
    """テストでないコードの差分で、テストの最中を見分ける目印か断言の差し替えが、消えた行より増えたら、足した最初の行(1から)。無ければ 0。"""
    added, removed, first = 0, 0, 0
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        removed += sum(len(TEST_AWARE_RE.findall(x)) + len(TEST_AWARE_SWAP_RE.findall(x)) for x in a[i1:i2])
        for k in range(j1, j2):
            n = len(TEST_AWARE_RE.findall(b[k])) + len(TEST_AWARE_SWAP_RE.findall(b[k]))
            if n and not first:
                first = k + 1
            added += n
    return first if added > removed else 0


def retire(ctx, path, names, count):
    """消したテストの印が、正本から消えた要求だけを指すなら、要求ごとに束ねて True。"""
    gone = ctx.get("gone") or {}
    names = {n for _, n in names} if names else names
    if not names or not all(n in gone for n in names):
        return False
    for n in names:
        ctx.setdefault("retired", {}).setdefault(n, []).append((path, count))
    return True


SECTION_RE = re.compile(r"^\s*\[+\s*([^\]]+?)\s*\]+\s*$")
JSON_KEY_RE = re.compile(r'(?:"([^"]+)"|(?<![\w$.])([A-Za-z_$][\w$]*))\s*:\s*[\[{]')  # JSON の鍵と、JS の設定の引用符の無い鍵


def enclosing(lines, k):
    """k 行目を囲む節の名前(TOML と INI の [節]、JSON の鍵)を、近い順に返す。"""
    out, depth = [], 0
    for i in range(k - 1, -1, -1):
        line = lines[i]
        m = SECTION_RE.match(line)
        if m and not out:
            return [m.group(1)]
        depth += line.count("}") + line.count("]") - line.count("{") - line.count("[")
        if depth < 0:
            km = JSON_KEY_RE.search(line)
            if km:
                out.append(km.group(1) or km.group(2))
            depth = 0
    return out


def shared_config_touched(a, b, extra=None):
    """共有の設定の変わった行か、その行を囲む節に、テストの道具の語(と、そのファイルでテストを外せる鍵 extra)があるか。"""

    def touched(x):
        return SHARED_CONFIG_WORDS.search(x) or (extra is not None and extra.search(x))

    sm = difflib.SequenceMatcher(None, a, b, autojunk=False)
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            continue
        for lines, lo, hi in ((a, i1, i2), (b, j1, j2)):
            for k in range(lo, hi):
                if touched(lines[k]) or any(touched(n) for n in enclosing(lines, k)):
                    return True
    return False


ROLES_CONFIG, ROLES_BASELINE = "roles.json", "roles-baseline.json"  # 言語ごとの道具の役の設定(道具の名前と版の正本)
ROLES_GLOBS = ["**/roles.json", "**/roles.*.json", "**/roles-baseline.json", "**/roles-baseline.*.json"]  # 言語ごとに分けた置き場も含める
WORKFLOWS = [".github/workflows/*.yml", ".github/workflows/*.yaml"]
# CI の置き場(--root と git の根)。変わったら、緩める書き方が無くても人に回す(base の検査器を head の写しに替える1行や、
# composite action の中身の書き換えは、緩めの語を持たないため)。guards の ci の種類にも同じ一覧を出す。
# forge-gate の設定と ruleset の写しも、承認の数とコードオーナーのレビューの求めを決めるので同じに扱う
CI_PLACES = [".github/workflows/**", ".github/actions/**", ".gitlab-ci.yml", "forge-gate.json", ".github/rulesets/**"]
LOOSEN_RE = re.compile(
    r"\|\|\s*(?:true\b|:(?=\s|$|;|\))|exit\s+0\b)|--passWithNoTests\b|continue-on-error\s*:\s*(?:true\b|\$\{\{)|--no-verify\b"
    r"|^\s*(?:-\s*)?if\s*:\s*(?:false\b|\$\{\{\s*false\s*\}\})"
    r"|\|\|\s*echo\b|(?<![\w-])set\s+\+e\b|--grep-invert\b"
    r"|PYTEST_ADDOPTS|PYTEST_DISABLE_PLUGIN_AUTOLOAD|--deselect\b|--ignore(?:-glob)?\b|(?<![\w-])-k\s|(?<![\w-])-m\s+['\"]?not\b|-p\s+no:"
    r"|\bpaths-ignore\s*:|^\s*paths\s*:|^\s*(?:-\s*)?if\s*:"
)
# 引き金の行(消すか書き換えると、検査が走らなくなる)
TRIGGER_LINE = re.compile(
    r"^\s*(?:on\s*:|-\s*(?:pull_request|pull_request_target|push|merge_group)\b|(?:pull_request|pull_request_target|push|merge_group)\s*:)"
)
# エージェントへの指示と設定(Claude Code・Codex・Gemini CLI・Copilot・Cursor の公式の文書が挙げる置き場)
INSTRUCTION_FILES = [
    "**/AGENTS.md",
    "**/AGENTS.override.md",
    "**/CLAUDE.md",
    "**/CLAUDE.local.md",
    "**/GEMINI.md",
    ".claude/**",
    ".agents/**",
    ".codex/**",
    ".gemini/**",
    ".cursor/**",
    ".mcp.json",
    ".github/copilot-instructions.md",
    ".github/instructions/**",
    ".github/agents/**",
    ".github/skills/**",
    ".github/hooks/**",
]
# 検査を黙らせる抑止のコメント
SUPPRESS_RE = re.compile(
    r"#\s*noqa\b|#\s*(?:ruff|flake8)\s*:\s*noqa\b|#\s*type:\s*ignore\b|#\s*mypy:\s*(?:ignore-errors|disable-error-code)\b"
    r"|#\s*pragma:\s*no\s*(?:cover|branch)\b|#\s*pylint:\s*disable\b|#\s*pyright:\s*ignore\b"
    r"|eslint-disable|oxlint-disable|deno-lint-ignore|@ts-ignore\b|@ts-expect-error\b|@ts-nocheck\b|istanbul\s+ignore|c8\s+ignore|biome-ignore"
    r"|#!?\[(?:allow|expect)\(|//\s*nolint\b|@phpstan-ignore|@psalm-suppress\b|@mago-(?:expect|ignore)\b|phpcs:(?:ignore|disable)|\bNOLINT\b"
    # セキュリティの走査の行の抑止(Semgrep・Bandit と gosec・gitleaks・TruffleHog・CodeQL。CodeQL は行の頭の lgtm も抑止と読む)
    r"|\bnosem(?:grep)?\b|#\s*nosec\b|gitleaks:allow\b|trufflehog:ignore\b|\bcodeql\s*\[|\blgtm\s*\[|(?:#|//|/\*)\s*lgtm\b"
)
# テストや検査を走らせる命令の語と、命令でない鍵の行(消しても検査を外さない)
RUN_WORDS = re.compile(r"(?i)(?<![\w-])(?:tests?|pytest|unittest|jest|vitest|playwright|e2e|go\s+test|cargo\s+test|phpunit|trace_gate(?:\.py)?)(?![\w-])")
KEY_LINE = re.compile(
    r"^\s*(?:-\s*)?(?:name|uses|id|if|with|env|on|jobs|runs-on|timeout-minutes|needs|steps|permissions|concurrency|strategy|matrix|shell|working-directory)\s*:|^\s*[\w.-]+\s*:\s*$"
)


def side_bytes(root, ref, path):
    """ref の側(None は作業ツリー)の中身のバイト列。無ければ None。錠の指紋はバイト列から取るので、git の中身もバイト列のまま読む。"""
    if not ref:
        return read_bytes(os.path.join(root, *path.split("/")))
    r = subprocess.run(["git", "-C", root, "show", "%s:%s%s" % (ref, git_prefix(root), path)], capture_output=True)
    return r.stdout if r.returncode == 0 else None


def lock_rows(root, ref):
    """ref の側の写しの錠の {道: 指紋}(フックの登録を除く)。錠が無いか読めなければ空。"""
    data = side_bytes(root, ref, VENDOR_LOCK)
    try:
        doc = json.loads(data.decode("utf-8")) if data is not None else None
    except (ValueError, UnicodeDecodeError):
        return {}
    rows = doc.get("files") if isinstance(doc, dict) and isinstance(doc.get("files"), list) else []
    return {f["path"]: f.get("sha256") for f in rows if isinstance(f, dict) and f.get("part") != HOOK_PART and safe_rel(f.get("path"))}


def locked_copies(root, base, head):
    """weaken の行の点検(抑止の足し・CI の緩め)から外す写しの道の集合。道具が写す決まった写しの道(検査器・包み・前の版の CI の雛形)だけで、
    基点と head の両方の写しの錠に載り、head の中身が head の錠の指紋と一致するもの(錠に任意のファイルを足しても外れない)。"""
    b, h = lock_rows(root, base), lock_rows(root, head)
    out = set()
    for rel in [r for r, _ in VENDORED + VENDORED_OPTIONAL] + sorted(LEGACY_PART):  # 道具が写す決まった写しの道
        if rel in b and rel in h:
            got = side_bytes(root, head, rel)
            if got is not None and sha256(got) == h[rel]:
                out.add(rel)
    return out


def suppress_added(a, b):
    """b で足された行の抑止のコメントが、消えた行の数より増えたら、足した最初の行(1から)。増えなければ 0。"""
    added, removed, first = 0, 0, 0
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        removed += sum(len(SUPPRESS_RE.findall(x)) for x in a[i1:i2])
        for k in range(j1, j2):
            n = len(SUPPRESS_RE.findall(b[k]))
            if n and not first:
                first = k + 1
            added += n
    return first if added > removed else 0


def loosened_lines(a, b):
    """head で足された行のうち、検査を緩める書き方を含む行の番号(1から)。"""
    out = []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag in ("delete", "replace"):
            # テストや検査を走らせる命令の行を消した・書き換えた(同じ文の行が head の同じ塊に残っていれば数えない)
            kept = Counter(x.strip() for x in b[j1:j2])
            for k in range(i1, i2):
                runs = RUN_WORDS.search(a[k]) and not KEY_LINE.match(a[k])
                if (runs or TRIGGER_LINE.match(a[k])) and a[k].strip() and not LOOSEN_RE.search(a[k]):
                    if kept[a[k].strip()] > 0:
                        kept[a[k].strip()] -= 1
                    else:
                        out.append(min(j1 + 1, len(b)) if b else 1)
        if tag in ("insert", "replace"):
            # 同じ塊の base にある同じ文の緩めの行の数だけは数えない(並べ替えや字下げの変更で出さないため)
            left = Counter(x.strip() for x in a[i1:i2] if LOOSEN_RE.search(x))
            for k in range(j1, j2):
                if LOOSEN_RE.search(b[k]):
                    if left[b[k].strip()] > 0:
                        left[b[k].strip()] -= 1
                    else:
                        out.append(k + 1)
    return sorted(set(out))


def git_name_status(root, base, head, pathspecs=()):
    """base と head(None なら作業ツリー。追跡していない新しいファイルも含める)で変わったファイルを、git の根からの道の
    (状態, 古い道, 新しい道) で返す。pathspecs が無ければ、追跡していないファイルは root の下だけを見る。"""
    # git の道は根からの道で受け取る(利用者の diff.relative に左右されないように)
    args = ["-c", "diff.relative=false", "diff", "--name-status", "-z", "-M", base] + ([head] if head else []) + ["--"] + list(pathspecs)
    parts = [x for x in git(root, *args).split("\0") if x != ""]
    found, i = [], 0
    while i < len(parts):
        st = parts[i]
        if st[:1] in ("R", "C"):
            found.append((st[0], parts[i + 1], parts[i + 2]))
            i += 3
        else:
            found.append((st[0], parts[i + 1], parts[i + 1]))
            i += 2
    if not head:
        for p in (git(root, "ls-files", "--others", "--exclude-standard", "--full-name", "-z", "--", *pathspecs) or "").split("\0"):
            if p:
                found.append(("A", p, p))
    return found


def git_changes(root, base, head):
    """base と head(None なら作業ツリー)で変わったファイルを、root からの道で返す。root の外のファイルは見ない。"""
    found = git_name_status(root, base, head)
    prefix = git_prefix(root)
    n, out = len(prefix), []
    for st, old, new in found:
        old_in, new_in = old.startswith(prefix), new.startswith(prefix)
        if old_in and new_in:
            out.append((st, old[n:], new[n:]))
        elif old_in and st == "R":  # root の外へ移った: root から見ると消えた
            out.append(("D", old[n:], old[n:]))
        elif new_in:  # root の外から移ってきたか写した: root から見ると足された
            out.append(("A", new[n:], new[n:]))
    return out


def read_side(root, ref, path):
    if ref:
        return git_show(root, ref, path)
    return read_text(root, path)


def config_rel(root, cfg):
    """リポの中の設定のファイルの道。リポの外を指す --config なら None。"""
    if not cfg["_path"]:
        return CONFIG_NAME
    rel = os.path.relpath(os.path.abspath(cfg["_path"]), os.path.abspath(root)).replace(os.sep, "/")
    return None if rel == ".." or rel.startswith("../") else rel


def base_config(root, base, cfg):
    """base の側の設定。検査される変更(head)が設定を書き換えて検査を緩められないように、判断の元は base に置く。"""
    rel = config_rel(root, cfg)
    if rel is None:
        return cfg
    text = git_show(root, base, rel, check=False)
    if text is None:
        return default_config()
    return config_from_text(text, "%s:%s" % (base, rel), base=True)


def uniq(items):
    return list(dict.fromkeys(items))


def self_rel(root):
    """走っている検査器のファイルのリポの中の道。リポの外なら None。"""
    rel = os.path.relpath(os.path.abspath(__file__), os.path.abspath(root)).replace(os.sep, "/")
    return None if rel == ".." or rel.startswith("../") else rel


def reqs_at(root, ref, cfg):
    """ref の側(None なら作業ツリー)の要求を {名前: Req} で返す。"""
    files = git_files_at(root, ref) if ref else None
    out = {}
    for p in requirement_files(root, cfg, files):
        rs, _ = parse_requirements(read_side(root, ref, p), p, [])
        for r in rs:
            out.setdefault(r.name, r)
    return out


def opens_at(root, ref, cfg, files=None):
    """ref の側(None なら作業ツリー)の未確定事項を {ID: [出現の順の未確定事項]} で返す。files があれば、その中のファイルだけを読む。"""
    if files is None and ref:
        files = git_files_at(root, ref)
    out = {}
    for p in requirement_files(root, cfg, files):
        _, opens = parse_requirements(read_side(root, ref, p), p, [])
        for o in opens:
            out.setdefault(o["id"], []).append(o)  # 同じ ID の2つ目も残す(先の1つだけを比べると、後ろの格下げが隠れる)
    return out


def kept_opens(root, args, bcfg, cfg):
    """head の側の未確定事項。base と head の両方の設定の requirements に当たり、base にもあるファイルからだけ読む
    (head の設定をダミーのファイルに向けて、ID を書くだけで消えていないと見せないため)。"""
    base_files = set(git_files_at(root, args.base))
    files = [p for p in (git_files_at(root, args.head) if args.head else walk_files(root)) if p in base_files]
    # head の設定の正本に当たるファイルを、base の設定でもう一度絞って読む(opens_at は渡した設定の requirements で選ぶ)
    return opens_at(root, args.head, bcfg, files=requirement_files(root, cfg, files))


def people_decide(occurrences):
    """同じ ID の出現のうち、人が先に決めるもの(区分が「人が先に決める」か区分の無いもの)。"""
    return [o for o in occurrences if o.get("kind") != "仮に決めてよい"]


def removed_opens(old, new):
    """base にあって head に無い未確定事項のうち、人が先に決めるもの。"""
    return [people_decide(os_)[0] for i, os_ in sorted(old.items()) if i not in new and people_decide(os_)]


def downgraded_opens(old, new):
    """人が先に決める未確定事項の head のどの出現でも、区分を「仮に決めてよい」に下げたか、決める人を変えたものの (base, head) の組。"""
    out = []
    for i, os_ in sorted(old.items()):
        strict = people_decide(os_)
        deciders = {o["decider"] for o in strict}
        out.extend((strict[0], n) for n in new.get(i, []) if strict and (n.get("kind") == "仮に決めてよい" or n["decider"] not in deciders))
    return out


def cmd_weaken(args, root, cfg):
    bcfg = base_config(root, args.base, cfg)
    if not (part_on(bcfg, "weaken") or part_on(cfg, "weaken")):  # PR の中で切っても、その PR 自身は weaken を通る
        return part_switched_off(args, "weaken", "base と head の両方の側の設定")
    # パッケージの一覧は base と head の両方の側から作る(head で packages を消しても、base のパッケージの設定で調べる)
    pkgs = package_dirs(root, cfg) if cfg.get("packages") else []
    if bcfg.get("packages"):
        dirs = tree_dirs(root)
        pkgs = uniq(pkgs + sorted(x for pat in bcfg["packages"] for x in dirs if glob_match(pat.strip("/"), x)))
    if not pkgs:
        findings = weaken_scope(args, root, cfg, bcfg)
    else:
        # 根は、今あるパッケージのフォルダを外して比べる(消したパッケージのテストは根の側で消しとして出る)
        findings = weaken_scope(args, root, cfg, bcfg, skip=tuple(p + "/" for p in pkgs))
        for p in pkgs:
            proot, pcfg = package_config(root, p, cfg)
            findings.extend(prefixed(weaken_scope(args, proot, pcfg, base_config(proot, args.base, pcfg)), p + "/"))
        findings.sort(key=Finding.sort_key)
    if git_prefix(root):
        findings.extend(git_root_workflows(args, root))
        findings.sort(key=Finding.sort_key)
    if args.junit_base or args.junit_head:
        if not (args.junit_base and args.junit_head):
            raise UsageError("--junit-base と --junit-head は両方を書く(base と head で走らせた JUnit XML)")
        findings = sorted(findings + count_drops("run-count-dropped", args.junit_base, args.junit_head), key=Finding.sort_key)
    summary = "trace-gate weaken: %s → %s で、人が見るテストの変化 %d 件(%s)" % (args.base, args.head or "作業ツリー", len(findings), counts_text(findings))
    if findings:  # 文から消した決まりの出口(どの指摘にも共通。要約の後の最後の1行)
        summary += "\n" + WEAKEN_EXIT
    return emit(args, findings, summary)


def ci_changed(st, old, new, shown, before, after, shown_old=None, lines=True):
    """CI の置き場のファイルの変更の gate-loosened。ワークフローの YAML に緩める書き方が足されたら行ごと、無ければ行なしで1件。
    消すか置き場の外へ移すと行なしの1件。before・after は道を受けて base・head の中身を返す。lines が偽なら行を見ない(錠と一致する写し)。"""
    shown_old = shown_old or old
    if st == "D" or not matches(new, CI_PLACES):
        return [Finding("gate-loosened", shown_old, None, "CI の置き場の %s が消えたか、置き場の外へ移った" % shown_old, key=shown_old)]
    found = []
    if lines and new.endswith((".yml", ".yaml")):
        a = before(old) if st != "A" else ""
        found = loosened_lines(a.split("\n"), (after(new) or "").split("\n"))
    if found:
        return [Finding("gate-loosened", shown, n, "%s の %d 行目で検査を緩める書き方が足された" % (shown, n), key="%s:%d" % (shown, n)) for n in found]
    what = {"A": "足された", "R": "名前が変わった"}.get(st, "変わった")
    why = "緩める書き方は見つからないが" if lines else "写しの錠と一致する写しなので緩めの行は見ていないが"
    return [Finding("gate-loosened", shown, None, "CI の置き場の %s が%s(%s、走らせる検査器と引き金を人が見る)" % (shown, what, why), key=shown)]


def git_root_workflows(args, root):
    """--root が git の根の下のフォルダのとき、git の根の CI の置き場(CI_PLACES)の変更を gate-loosened で返す(GitHub はそこしか読まず、
    サブフォルダで使うリポの CI はそこにあるため)。道は --root からの道(../.github/workflows/…)で出す。"""
    top = git(root, "rev-parse", "--show-toplevel").rstrip("\n")
    up = "/".join([".."] * len(git_prefix(root).rstrip("/").split("/")))  # git の道の段の数だけ上る(シンボリックリンクの道に左右されない)

    def text(ref, path):
        if ref:
            return git(root, "show", "%s:%s" % (ref, path))
        return read_text(top, path)

    findings = []
    tops = [":(top)" + p.replace("/**", "/") for p in CI_PLACES]
    for st, old, new in git_name_status(root, args.base, args.head, tops):
        if matches(old, CI_PLACES) or matches(new, CI_PLACES):
            findings.extend(ci_changed(st, old, new, up + "/" + new, lambda p: text(args.base, p), lambda p: text(args.head, p), up + "/" + old))
    return findings


def upstream_items_at(root, ref, patterns):
    """ref の側(None なら作業ツリー)の上流の項目 {名前: (道, 行, 文)}。形の崩れた行と同じ名前の2つ目は読まない(check が止める)。"""
    out = {}
    for p in select(git_files_at(root, ref) if ref else walk_files(root), patterns):
        fence = False
        for no, line in enumerate((read_side(root, ref, p) or "").split("\n"), 1):
            if FENCE_RE.match(line):
                fence = not fence
                continue
            m = None if fence else UP_LINE.match(line)
            if m:
                out.setdefault(m.group("name"), (p, no, m.group("text")))
    return out


def upstream_item_changes(root, args, patterns, old_reqs, new_reqs):
    """base にあって文が変わった項目と消えた項目ごとの upstream-changed。足した項目・項目の外の文・同じ文での移しは出さない。"""
    old, new = upstream_items_at(root, args.base, patterns), upstream_items_at(root, args.head, patterns)
    out = []
    for name in sorted(old):
        cited = sorted({r.name for reqs in (old_reqs, new_reqs) for r in reqs.values() if name in r.ups})
        who = "出典にした要求: %s" % "・".join(cited) if cited else "出典にした要求は無い"
        if name not in new:
            p, no, _ = old[name]
            out.append(Finding("upstream-changed", p, None, "上流の項目 up~%s が消えた(base の %s:%d。%s)" % (name, p, no, who), key="up~" + name))
        elif squash(old[name][2]) != squash(new[name][2]):
            p, no, _ = new[name]
            out.append(Finding("upstream-changed", p, no, "上流の項目 up~%s の文が変わった(%s)" % (name, who), key="up~" + name))
    return out


NO_REQ = re.compile(r"\[no-req:\s*[^\]\s][^\]]*\]")  # 要求に結ばないテストの印(理由が要る)


def unmarked_added(path, a, b, findings):
    """base の文 a に無く head の文 b にあるテスト(足されたテスト)のうち、要求の印も [no-req: 理由] も無いものを警告にする。
    既にあるテストは見ない(入れた日に、既にあるリポで大量に出さないため)。"""
    old = test_units(path, a) if a else {}
    for name, (node, body, links, _) in sorted(test_units(path, b).items()):
        if name in old or links or NO_REQ.search(body):
            continue
        findings.append(
            Finding(
                "test-unmarked",
                path,
                None,
                "足したテスト %s に要求の印が無い(要求の無い振る舞いを確かめているおそれ)" % clip(node, 100),
                key=node,
                severity="warning",
            )
        )


def weaken_scope(args, root, cfg, bcfg, skip=()):
    """1つの範囲(根かパッケージ)の base と head を比べる。skip の頭で始まる道は見ない(パッケージの側で見るため)。"""
    findings = []
    rel = config_rel(root, cfg)
    # 設定の道・承認の記録・検査器そのものは、どちらの側の test_config に何が書いてあっても必ず人が見る
    always = (rel, CONFIG_NAME, bcfg["approvals"], cfg["approvals"], VENDORED_CHECKER, VENDORED_RUNNER, VENDOR_LOCK, self_rel(root)) + tuple(ROLES_GLOBS)
    strict = uniq(bcfg["test_config"] + cfg["test_config"] + [x for x in always if x])
    tests_glob = uniq(bcfg["tests"] + cfg["tests"])
    baselines = uniq(bcfg["baselines"] + cfg["baselines"])  # どちらかの側が基準とするものは、全部人が見る
    upstream = uniq(bcfg["upstream"] + cfg["upstream"])
    exclude_glob = [x for x in bcfg["exclude"] if x in cfg["exclude"]]  # 両方の側が外すものだけを外す
    py_helpers = tuple(uniq(list((bcfg.get("assert_helpers") or {}).get("python", [])) + list((cfg.get("assert_helpers") or {}).get("python", []))))

    def is_test(p):
        return os.path.splitext(p)[1] in LANG_BY_EXT and matches(p, tests_glob) and not matches(p, exclude_glob)

    old_reqs, new_reqs = reqs_at(root, args.base, bcfg), reqs_at(root, args.head, cfg)
    old_opens, new_opens = opens_at(root, args.base, bcfg), kept_opens(root, args, bcfg, cfg)
    for o in removed_opens(old_opens, new_opens):
        findings.append(
            Finding("open-removed", o["path"], o["line"], "未確定事項 %s(決める人: %s)が正本から消えた" % (o["id"], clip(o["decider"], 40)), key=o["id"])
        )
    for o, n in downgraded_opens(old_opens, new_opens):
        findings.append(
            Finding(
                "open-downgraded",
                n["path"],
                n["line"],
                "未確定事項 %s の区分か決める人が変わった(区分: %s → %s、決める人: %s → %s)"
                % (n["id"], o.get("kind") or "無し", n.get("kind") or "無し", clip(o["decider"], 40), clip(n["decider"], 40)),
                key=n["id"],
            )
        )
    old_names, new_names = load_renames(root, bcfg, [], args.base), load_renames(root, cfg, [], args.head)
    renamed = rename_pairs(old_reqs, new_reqs, new_names)  # 名前の表の付け替えの対は、同じ要求として比べる
    gone = {n: r for n, r in old_reqs.items() if n not in new_reqs and n not in renamed}
    # 扱いを廃止にした要求も、正本から消したのと同じく、その印だけのテストの消しを要求ごとに束ねる
    gone.update({n: r for n, r in new_reqs.items() if r.treatment == "廃止" and n in old_reqs and old_reqs[n].treatment != "廃止"})
    ctx = {"gone": gone, "retired": {}}

    changes, upstream_touched = [], False
    for st, old, new in git_changes(root, args.base, args.head):
        o, n = not (skip and old.startswith(skip)), not (skip and new.startswith(skip))
        if o and n:
            changes.append((st, old, new))
        elif o and st == "R":  # 根からパッケージへ移った: 根から見ると消えた(パッケージの側では足しとして見る)
            changes.append(("D", old, old))
        elif n:  # パッケージから根へ移ったか写した: 根から見ると足された
            changes.append(("A", new, new))
    locked = locked_copies(root, args.base, args.head)  # 錠と一致する決まった写しは、行の点検から外す(錠の変更は test-config-changed で人に回る)
    left_out = sorted(p for p in locked if any(p in (old, new) for _, old, new in changes))
    left_note = "。行の点検から外した写し: %s(基点と head の錠に載り、錠の指紋と中身が一致するので、抑止と CI の緩めの行は見ていない。錠の差分を人が見る)"

    def test_side_file(p):
        return (matches(p, tests_glob) or bool(TEST_DIR_NAMES & set(p.split("/")[:-1]))) and not matches(p, exclude_glob)

    for st, old, new in changes:
        if st == "D" or os.path.splitext(new)[1] not in LANG_BY_EXT or matches(new, exclude_glob) or new in locked:
            continue
        a = read_side(root, args.base, old) if st != "A" else ""
        b = read_side(root, args.head, new)
        line = suppress_added(a.split("\n"), b.split("\n"))
        if line:
            findings.append(
                Finding("suppress-added", new, line, "%s の %d 行目のあたりで、検査を黙らせる抑止のコメントが足された" % (new, line), key="%s:%d" % (new, line))
            )
        if is_test(new):
            unmarked_added(new, a, b, findings)
        if test_side_file(new):
            if st != "A" and test_side_file(old):
                weaken_assert_swaps(new, a, b, LANG_BY_EXT[os.path.splitext(new)[1]], findings)
            continue
        line = test_aware_added(a.split("\n"), b.split("\n"))
        if line:
            findings.append(
                Finding(
                    "test-aware-code",
                    new,
                    line,
                    "テストでないコード %s の %d 行目のあたりで、テストの最中を見分ける目印か断言の差し替えが足された" % (new, line),
                    key="%s:%d" % (new, line),
                    severity=TEST_AWARE_SEVERITY,
                )
            )
    for st, old, new in changes:
        path = new
        if old == VENDOR_LOCK and (st == "D" or new != VENDOR_LOCK):
            # 錠が無いと check は写しを照らさず、写しの書き換えが黙って通る
            findings.append(Finding("vendor-drift", old, None, "写しの錠 %s が base にあったのに消えた(錠が無いと check は写しを照らさない)" % old, key=old))
            continue
        if matches(path, strict) or matches(old, strict):
            what = {"A": "足された", "D": "消された", "R": "名前が変わった"}.get(st, "変わった")
            message = "テストの設定か検査器 %s が%s" % (path, what)
            if left_out and (path == VENDOR_LOCK or path in left_out):
                message += left_note % "・".join([path] if path in left_out else left_out)
            findings.append(Finding("test-config-changed", path, None, message, key=path))
            continue
        if matches(path, INSTRUCTION_FILES) or matches(old, INSTRUCTION_FILES):
            what = {"A": "足された", "D": "消された", "R": "名前が変わった"}.get(st, "変わった")
            findings.append(Finding("instructions-changed", path, None, "エージェントへの指示か設定 %s が%s" % (path, what), key=path))
            continue
        if upstream and (matches(path, upstream) or matches(old, upstream)):
            upstream_touched = True  # 項目ごとに、走査の後で base と head の項目を比べて出す
            continue
        if matches(path, baselines) or matches(old, baselines):
            what = {"A": "足された", "D": "消された", "R": "名前が変わった"}.get(st, "変わった")
            findings.append(Finding("baseline-changed", path, None, "基準のファイル %s が%s" % (path, what), key=path))
            continue
        if matches(old, CI_PLACES) or matches(path, CI_PLACES):
            findings.extend(
                ci_changed(st, old, path, path, lambda p: read_side(root, args.base, p), lambda p: read_side(root, args.head, p), lines=path not in locked)
            )
            continue
        if matches(path, SHARED_CONFIG) or matches(old, SHARED_CONFIG):
            a = read_side(root, args.base, old) if st != "A" else ""
            b = read_side(root, args.head, new) if st != "D" and matches(new, SHARED_CONFIG) else ""
            if shared_config_touched(a.split("\n"), b.split("\n"), SHARED_CONFIG_EXTRA.get(os.path.basename(path))):
                findings.append(Finding("test-config-changed", path, None, "共有の設定 %s のテストに関わる行が変わった" % path, key=path))
            continue
        old_t, new_t = is_test(old), is_test(new)
        if st == "R" and old_t and not new_t:
            findings.append(Finding("test-file-deleted", old, None, "テストのファイル %s が %s へ移り、テストとして数えられなくなった" % (old, new), key=old))
            continue
        if not new_t:
            continue
        lang = LANG_BY_EXT[os.path.splitext(path)[1]]
        if st == "D":
            marks = test_marks(path, read_side(root, args.base, old))
            names = frozenset().union(*marks.values()) if marks else frozenset()
            if marks and all(marks.values()) and retire(ctx, path, names, len(marks)):
                continue
            findings.append(Finding("test-file-deleted", path, None, "テストのファイル %s が消えた" % path, key=path))
            continue
        if st == "A" or (st in ("R", "C") and not old_t):
            continue
        a, b = read_side(root, args.base, old), read_side(root, args.head, new)
        weaken_marks(path, a, b, lang, findings, renamed)
        weaken_control(path, a, b, lang, findings, py_helpers)
        if lang == "python":
            ctx["old_marks"], ctx["new_marks"] = renamed_marks(test_marks(path, a) or {}, renamed), test_marks(path, b) or {}
            if weaken_python(path, a, b, findings, ctx):
                continue
        weaken_text(path, a, b, lang, findings)
        weaken_states(old, path, a, b, findings)  # 文字の差分の後(同じテストに重ねない)
    if upstream_touched:
        findings.extend(upstream_item_changes(root, args, upstream, old_reqs, new_reqs))
    findings.extend(adr_changes(root, args, bcfg, cfg, changes))
    for name, where in sorted(ctx["retired"].items()):
        r = ctx["gone"][name]
        findings.append(
            Finding(
                "test-retired",
                r.path,
                None,
                "要求 %s を正本から消し、その印だけを持つテスト %d 本も消した(%s)"
                % (r.id, sum(c for _, c in where), clip(", ".join(sorted({p for p, _ in where})), 120)),
                key=name,
            )
        )
    for name, n in sorted(new_reqs.items()):
        if name not in old_reqs and name not in renamed.values():
            findings.append(Finding("req-added", n.path, n.line, "要求 %s(%s)が足された" % (n.id, n.priority), key=name))
            if n.priority == "P0" and n.treatment == "踏襲":
                findings.append(
                    Finding(
                        "kept-added",
                        n.path,
                        n.line,
                        "足した P0 の要求 %s に 扱い: 踏襲 が付いていて、実装前に落ちるかの確かめ(f2p・f2p-run)で数えない" % n.id,
                        key=name,
                    )
                )
    for name, r in sorted(old_reqs.items()):
        n = new_reqs.get(name)
        if n is None and name in renamed:
            n = new_reqs[renamed[name]]
            findings.append(Finding("req-renamed", n.path, n.line, "要求 %s を %s に付け替えた(名前の表)" % (r.id, n.id), key=n.name))
        if n is None:
            if name not in ctx["retired"]:
                note = "(%s)" % renamed_note(new_names[name]) if name in new_names else ""
                findings.append(Finding("req-removed", r.path, None, "要求 %s が正本から消えた%s" % (r.id, note), key=name))
            if name not in new_names:
                findings.append(
                    Finding("req-removed-unrecorded", r.path, None, "要求 %s を正本から消したが、名前の表 %s に行が無い" % (r.id, cfg["renames"]), key=name)
                )
            continue
        what = [
            w
            for w, x in (
                ("文か受け入れ基準", n.fingerprint != r.fingerprint),
                ("版", n.version != r.version),
                ("扱い", n.treatment != r.treatment),
                ("出典", (n.sources or "") != (r.sources or "")),
                ("優先度(%s から %s)" % (r.priority, n.priority), n.priority < r.priority),
            )
            if x
        ]
        if what:
            findings.append(Finding("req-changed", n.path, n.line, "要求 %s の%sが変わった(%s から)" % (n.id, "・".join(what), r.id), key=n.name))
        if n.priority > r.priority:
            findings.append(
                Finding("req-priority-lowered", n.path, n.line, "要求 %s の優先度が %s から %s に下がった" % (n.id, r.priority, n.priority), key=n.name)
            )
        if not r.needs:
            continue
        lost = r.needs - (n.needs or frozenset())
        if lost:
            findings.append(
                Finding("needs-reduced", n.path, n.line, "要求 %s の要る確かめの種類から %s が外れた" % (n.id, "・".join(sorted(lost))), key=n.name)
            )
    where = cfg.get("renames") or DEFAULT_RENAMES
    for old, (new, _, no) in sorted(old_names.items()):
        h = new_names.get(old)
        if h is None:
            findings.append(Finding("name-unreserved", where, None, "名前の表から %s の行(base の %d 行目)が消えた" % (old, no), key=old))
        elif h[0] != new:
            findings.append(
                Finding("name-unreserved", where, h[2], "名前の表の %s の行が「%s」から「%s」に変わった" % (old, new or "消した", h[0] or "消した"), key=old)
            )
    findings.sort(key=Finding.sort_key)
    return findings


# ---------------------------------------------------------------- 実装前に落ちるか(TG-3 の集計)

ASSERT_FAIL = re.compile(r"AssertionError|ExpectationFailedException|AssertionFailedError|^\s*assert\b|\bDID NOT (?:RAISE|WARN)\b")
NODE_ASSERT_CAUSE = re.compile(r"(?m)^\s*cause:\s*AssertionError\b")  # node:test は断言の失敗を本文の cause にだけ書く
RUST_PANIC = re.compile(r"panicked at [^\n]*:\n\s*([^\n]*)")  # libtest は panic の文を system-out に書く


def rust_outcome(child, case):
    """libtest の panic の1件: 断言の panic と should_panic の不発だけを assert-fail に、todo!()・unwrap などは exception にする。"""
    if "did not panic as expected" in (child.get("message") or ""):
        return "assert-fail"
    text = (child.text or "") + "".join(c.text or "" for c in case if c.tag in ("system-out", "system-err"))
    m = RUST_PANIC.search(text)
    return "assert-fail" if m and m.group(1).startswith("assertion") else "exception"


def junit_outcome(case):
    """JUnit の testcase 1件の結果: assert-fail・exception・error・skipped・passed。"""
    for child in case:
        tag = child.tag
        msg = (child.get("message") or "") + " " + (child.get("type") or "")
        if tag == "skipped":
            return "assert-fail" if "xfail" in msg.lower() else "skipped"
        if tag == "failure":
            if "XPASS" in msg:
                return "passed"
            if child.get("type") == "testCodeFailure":
                return "assert-fail" if NODE_ASSERT_CAUSE.search(child.text or "") else "exception"
            if child.get("type") == "assert" or ((child.get("type") or "").startswith("test failure") and "panicked at" in (child.text or "")):
                # libtest の --format junit と、cargo-nextest(type は "test failure" か "test failure with exit code 101" など。本文に panic の文)
                return rust_outcome(child, case)
            if ASSERT_FAIL.search(msg) or (not msg.strip() and ASSERT_FAIL.search(child.text or "")):
                return "assert-fail"
            return "exception"
        if tag == "error":
            return "error"
    return "passed"


RUST_KEY = "\0rust"  # 結果の表の、Rust のテストの名前の最後の段で引く鍵 (RUST_KEY, 最後の段) -> [(道つきの名前, 結果)]
RUST_ROOT_CLASSES = ("crate", "integration", "unknown")  # libtest の JUnit の classname で、モジュールの道でないもの


def rust_full_name(classname, name):
    """JUnit の testcase の Rust のテストの道つきの名前。nextest は name に道(mod::tests::名前)を入れ、classname はバイナリの名前。
    libtest(--format junit)は単体のテストの道を classname に分け、結合のテストは classname が integration で name に道を入れる。"""
    if "::" in name or not classname or classname in RUST_ROOT_CLASSES:
        return name
    return classname + "::" + name


def rust_wants(t):
    """Rust のテストの、結果の名前と後ろから照らす道つきの名前(先に照らすものから)。ファイルの道と、ファイルの中で囲む mod から作る。
    tests/foo.rs(結合のテストのバイナリの根)は、nextest の classname のバイナリの名前 foo つきを先に、無しを後に照らす。"""
    segs = os.path.splitext(t.path)[0].split("/")
    cut = max((i for i, x in enumerate(segs) if x == "src"), default=-1)
    if cut < 0:
        cut = max((i for i, x in enumerate(segs) if x == "tests"), default=-1)
    after = segs[cut + 1 :]
    heads = [after]
    if t.heads:
        heads = t.heads  # #[path] で取り込まれるファイルは、取り込み先の mod の道から
    elif cut >= 0 and segs[cut] == "tests" and len(after) == 1:
        heads = [after, []]
    elif after[:1] == ["bin"]:
        heads = [after[2:]]  # src/bin/x.rs と src/bin/x/main.rs はバイナリの根
    scope = [x for x in t.scope.split("::") if x]
    out = []
    for head in heads:
        if head and head[-1] in ("mod", "lib", "main"):
            head = head[:-1]
        out.append("::".join(head + scope + [t.name]))
    return out


def rust_outcomes(t, outcomes):
    """Rust のテストの結果: 名前の最後の段が同じ testcase のうち、道(ファイルと囲む mod)が後ろから一致するものを優先し、
    無ければ道の無い名前(クレートの根のテスト)だけを合わせる。別の道の同じ名前の結果は借りない(結果に無いと読む)。"""
    cands = outcomes.get((RUST_KEY, t.name))
    if not cands:
        return None
    for want in rust_wants(t):
        same = [o for full, o in cands if full == want or full.endswith("::" + want)]
        if same:
            return same
    return [o for full, o in cands if "::" not in full] or None


def case_keys(t):
    if t.lang == "python":
        module = os.path.splitext(t.path)[0].replace("/", ".")
        cls = t.qual.replace("::", ".")
        return {(module + ("." + cls if cls else ""), t.name)}
    return {("", t.name)}


def junit_doc(path):
    """JUnit XML を1回だけ読み、(根の要素, 生のバイト列)。DOCTYPE・ENTITY の宣言があれば読まない。"""
    try:
        with open(path, "rb") as f:
            data = f.read()
        if re.search(rb"<!DOCTYPE|<!ENTITY", data, re.I):
            raise UsageError("JUnit XML に DOCTYPE か ENTITY の宣言がある。外の実体と膨張を避けるため読まない: %s" % clip(path))
        return ET.fromstring(data), data
    except (OSError, ET.ParseError) as e:
        raise UsageError("JUnit XML を読めない: %s(%s)" % (clip(path), clip(e, 120))) from e


def junit_outcomes(doc, only=None):
    """JUnit XML の根を {(classname, name): [結果]} と {("", name): [結果]} と、Rust の名前の最後の段の {(RUST_KEY, 段): [(道つきの名前, 結果)]} にする。
    only があれば、その testcase だけを読む。"""
    outcomes = {}
    for case in doc.iter("testcase"):
        if only is not None and not only(case):
            continue
        name = re.sub(r"\[.*\]$", "", case.get("name") or "")
        o = junit_outcome(case)
        outcomes.setdefault((case.get("classname") or "", name), []).append(o)
        outcomes.setdefault(("", name), []).append(o)
        full = rust_full_name(case.get("classname") or "", name)
        outcomes.setdefault((RUST_KEY, full.rsplit("::", 1)[-1]), []).append((full, o))
    return outcomes


def read_junit(path):
    """JUnit XML を junit_outcomes の表にする。"""
    return junit_outcomes(junit_doc(path)[0])


def run_counts(paths):
    """JUnit XML の束の (集めたテストの数, skipped の要素を持つテストの数)。xfail も skipped の要素なので飛んだ数に入る。"""
    collected = skipped = 0
    for p in paths:
        for case in junit_doc(p)[0].iter("testcase"):
            collected += 1
            skipped += case.find("skipped") is not None
    return collected, skipped


def count_drops(rule, base_paths, head_paths):
    """base と head の結果の数を比べ、集めた数の減りと飛んだ数の増えを rule の指摘にする(check と weaken の共通の口)。"""
    (bc, bs), (hc, hs) = run_counts(base_paths), run_counts(head_paths)
    out = []
    if hc < bc:
        out.append(Finding(rule, None, None, "走らせた結果で、集めたテストが base の %d 本から %d 本に減った" % (bc, hc), key="collected"))
    if hs > bs:
        out.append(Finding(rule, None, None, "走らせた結果で、飛んだテストが base の %d 本から %d 本に増えた" % (bs, hs), key="skipped"))
    return out


FLAKY_TAGS = ("flakyFailure", "flakyError")  # 再試行で通った記録(Maven Surefire と cargo-nextest の JUnit)


def junit_flaky(paths):
    """再試行の記録(FLAKY_TAGS)を持つ testcase だけの結果の表(junit_outcomes の形)。落ちと飛びは check_runs が先に見る。"""
    out = {}
    for p in paths:
        for k, v in junit_outcomes(junit_doc(p)[0], lambda case: any(c.tag in FLAKY_TAGS for c in case)).items():
            out.setdefault(k, []).extend(v)
    return out


def quarantine_days(q, today=None):
    """始めた日からの日数。時差で明日の日付を書いた項目も 0 にする(負にしない)。"""
    import datetime

    today = today or datetime.date.today()
    return max(0, (today - datetime.date.fromisoformat(q["since"])).days)


def quarantine_note(quarantine, known=None):
    """要約に足す隔離の一言(本数・始めた日からの最長の日数・延長の回数の合計)。日付では止めない。
    known があれば、その印のテストに当たる項目だけを数える(当たらない項目は quarantine-unknown で知らせる)。"""
    quarantine = {k: q for k, q in (quarantine or {}).items() if known is None or k in known}
    if not quarantine:
        return ""
    days = max(quarantine_days(q) for q in quarantine.values())
    return "・隔離 %d 本(最長 %d 日・延長 %d 回)" % (len(quarantine), days, sum(q.get("extensions", 0) for q in quarantine.values()))


def check_quarantine(root, cfg, tests, findings):
    """隔離の一覧の項目を印のテストと照らす: 当たらない鍵は quarantine-unknown(警告)で知らせ、当たったテストに隔離の印を付ける
    (隔離したテストは要求の確かめに数えない)。当たった鍵の集まりを返す。"""
    quarantine = cfg.get("quarantine") or {}
    nodes = {t.node: t for t in tests}
    for t in tests:
        t.quarantined = t.node in quarantine
    unknown = sorted(k for k in quarantine if k not in nodes)
    if unknown:
        findings.append(
            Finding(
                "quarantine-unknown",
                config_rel(root, cfg) or CONFIG_NAME,
                None,
                "隔離の一覧の %d 件がどのテストにも当たらない: %s" % (len(unknown), clip("・".join(unknown), 160)),
                key="quarantine",
                severity="warning",
            )
        )
    return {k for k in quarantine if k in nodes}


def check_runs(reqs, tests, paths, findings, quarantine=None):
    """走らせた結果(JUnit XML)と印のあるテストを照らす(廃止の要求を指すテストは、先に retired-has-tests で止まる)。
    隔離の一覧のテストは止めずに知らせ、再試行で通ったテストは test-flaky で知らせる。
    OS か CPU の条件つきのテストが結果に無いか全部飛んだのは、この OS で走らないのが正しいので止めず、その本数を返す。"""
    other_os = 0
    outcomes = {}
    for p in paths:
        for k, v in read_junit(p).items():
            outcomes.setdefault(k, []).extend(v)
    flaky, quarantine = junit_flaky(paths), quarantine or {}
    for t in tests:
        if not t.links:
            continue
        got = outcomes_of(t, outcomes) or []
        node = clip(t.node, 100)
        q = quarantine.get(t.node)
        if q is not None:
            ok = bool(got) and all(o == "passed" for o in got)
            what = "通った" if ok else "落ちたか飛んだか結果に無い"
            findings.append(
                Finding(
                    "quarantine-passing" if ok else "test-quarantined",
                    t.path,
                    t.start,
                    "隔離したテスト %s が、走らせた結果で%s(持ち主: %s、始めて %d 日、延長 %d 回)"
                    % (node, what, clip(q["owner"], 40), quarantine_days(q), q.get("extensions", 0)),
                    key=t.node,
                    severity="warning",
                )
            )
            continue
        if t.skip == "conditional" and t.platform and all(o == "skipped" for o in got):
            other_os += 1  # 結果に無い(組まれない)か全部飛んだ。ほかの OS のジョブの結果で照らす
            continue
        if not got:
            findings.append(Finding("test-not-run", t.path, t.start, "印のあるテスト %s が、走らせた結果(JUnit XML)に無い" % node, key=t.node))
        elif any(o in ("assert-fail", "exception", "error") for o in got):
            findings.append(Finding("test-failed-at-run", t.path, t.start, "印のあるテスト %s が、走らせた結果で落ちた" % node, key=t.node))
        elif all(o == "skipped" for o in got):
            findings.append(Finding("test-skipped-at-run", t.path, t.start, "印のあるテスト %s が、走らせた結果で飛んだ" % node, key=t.node))
        elif outcomes_of(t, flaky):
            findings.append(Finding("test-flaky", t.path, t.start, "印のあるテスト %s が、再試行で通った" % node, key=t.node, severity="warning"))
    return other_os


def first_adoption(root, base, cfg):
    """trace-gate を入れる最初の PR か。基点の側だけで決める: 基点に、決まった置き場の trace-gate.json も、head の設定のファイルの道も、
    要求の正本(既定の置き場と head の設定の requirements)も無いときだけ真。head で設定を動かすか --config で別の道を渡しても、
    基点に設定か正本が1つでもあれば偽(設定を動かす・消す変更は weaken が test-config-changed で人に回す)。"""
    files = set(git_files_at(root, base))
    if ({CONFIG_NAME, config_rel(root, cfg)} - {None}) & files:
        return False
    patterns = uniq(DEFAULT_REQUIREMENTS + cfg["requirements"])
    return not any(glob_match(p, f) if has_glob(p) else p == f for p in patterns for f in files)


def select_reqs(args, root, cfg, reqs):
    """数える要求の名前(名前の順)と、選び方の説明。どれが新しい振る舞いかを実装役の申告に任せず、機械で決める。
    --req があればその名前。無ければ、--base があれば base に無い P0・base より版が上がった P0・base で P0 でなかった P0
    (新しく足すか意味を変えたか P0 に上げた要求。P2 で足して次の変更で P0 に上げ、実装前に落ちる確かめを外す道を塞ぐ)、
    --base も無ければ全部の P0。base の要求は、--root が git の根の下のフォルダでも --root からの道で読む(git_prefix)。"""
    if args.req:
        names = [n.strip() for n in args.req.split(",") if n.strip()]
        for n in names:
            if n not in reqs:
                raise UsageError("--req の要求が正本に無い: %s" % clip(n, 40))
        return sorted(set(names)), "--req で指定した要求"
    p0 = {n: r for n, r in reqs.items() if r.priority == "P0" and r.treatment not in ("廃止", "踏襲")}
    kept = sorted(n for n, r in reqs.items() if r.priority == "P0" and r.treatment == "踏襲")
    note = "。扱いが踏襲の P0(既にある振る舞い)は数えない: %s" % ", ".join(kept) if kept else ""
    if not args.base:
        return sorted(p0), "全部の P0" + note
    git(root, "rev-parse", "--verify", "--quiet", args.base + "^{commit}")
    if first_adoption(root, args.base, cfg):
        return [], "基点(%s)に trace-gate の設定も要求の正本も無い。入れる最初の PR なので数えない(要求は既にある振る舞いを写したもの)" % clip(args.base, 40)
    old = reqs_at(root, args.base, cfg)
    names = sorted(n for n, r in p0.items() if n not in old or r.version > old[n].version or old[n].priority != "P0")
    return names, "base(%s)に無いか、版が上がったか、P0 に上げた P0%s" % (clip(args.base, 40), note)


def nothing_to_count(args, command, how, extra=None):
    """選ばれた要求が0件のとき。指摘0件・終了コード 0 で、要約に「数える要求なし」と出す。"""
    doc = {"requirements": {}, "selected": []}
    doc.update(extra or {})
    return emit(args, [], "trace-gate %s: 数える要求なし(選び方: %s)" % (command, how), doc)


def cmd_f2p(args, root, cfg):
    if not gate_part_on(root, cfg, args.base, "f2p"):
        return part_switched_off(args, "f2p")
    res = analyze(root, cfg)
    if not res["reqs"]:
        return emit(args, res["findings"], "trace-gate f2p: 要求が無い")
    outcomes = read_junit(args.junit)
    wanted, how = select_reqs(args, root, cfg, res["reqs"])
    if not wanted:
        return nothing_to_count(args, "f2p", how)
    findings, report = [], {}
    for name in wanted:
        r = res["reqs"][name]
        tally = Counter()
        for t in res["tests"]:
            if not any(ln["name"] == name and ln.get("status") == "covers" for ln in t.links):
                continue
            got = outcomes_of(t, outcomes)
            if got is None:
                tally["missing"] += 1
            else:
                tally.update(got)
        report[r.id] = dict(sorted(tally.items()))
        if not tally["assert-fail"]:
            detail = "、".join("%s %d" % (k, v) for k, v in sorted(tally.items())) or "印のテストなし"
            findings.append(Finding("f2p-not-failing", r.path, r.line, "要求 %s に、実装前に断言の失敗で落ちたテストが無い(%s)" % (r.id, detail), key=name))
    summary = "trace-gate f2p: 確かめた %d / %d(断言の失敗で落ちたテストのある要求)。数える要求(%s): %s。%s" % (
        len(wanted) - len(findings),
        len(wanted),
        how,
        ", ".join(wanted),
        counts_text(findings),
    )
    return emit(args, findings, summary, {"requirements": report, "selected": wanted})


RESULT_WORDS = {"ok": "OK", "ng": "NG", "partial": "一部未実施", "none": "未実施", "no-test": "テストなし"}


def run_result(got):
    """1本のテストの結果の並び(パラメータの組ごと)を、OK・NG・一部未実施・未実施 の1つにする。"""
    got = got or []
    if any(o in ("assert-fail", "exception", "error") for o in got):
        return "ng"
    passed = sum(1 for o in got if o == "passed")
    if passed and passed == len(got):
        return "ok"
    return "partial" if passed else "none"


def junit_meta(doc, data):
    """(sha256, 最初の testsuite の時刻)。ホスト名と失敗の文は読まない。"""
    stamp = ""
    for suite in doc.iter("testsuite"):
        stamp = suite.get("timestamp") or ""
        if stamp:
            break
    return sha256(data), clip(stamp, 40)


def cmd_results(args, root, cfg):
    """JUnit XML の実施結果を、要求とテストの組ごとの表(テスト成績書)にする。intents の表と同じ行のキーでつなぐ。"""
    runs = []
    for spec in args.junit or []:
        label, sep, path = spec.partition("=")
        label = label.strip()
        if not sep or not label or not path:
            raise UsageError("--junit は 名前=道 の形で書く: %s" % clip(spec, 60))
        if label in [x[0] for x in runs]:
            raise UsageError("--junit の名前が重なっている: %s" % clip(label, 40))
        doc, data = junit_doc(path)
        runs.append((label, path, junit_outcomes(doc), junit_meta(doc, data)))
    if not runs:
        raise UsageError("--junit 名前=道 を1つ以上書く")
    res = analyze(root, cfg)
    head = ["要求", "テスト", "状態"]
    for label, _, _, _ in runs:
        head += [label, label + " 実行数"]
    rows = []
    for name, r in sorted(res["reqs"].items()):
        tests = sorted((t for t in res["tests"] if any(ln["name"] == name for ln in t.links)), key=lambda t: t.node)
        if not tests:
            rows.append([r.id, "", ""] + [x for _ in runs for x in (RESULT_WORDS["no-test"], "0")])
        for t in tests:
            link = next((ln for ln in t.links if ln["name"] == name), {})
            row = [r.id, t.node, test_state(t) if link.get("status") == "covers" else "outdated"]
            for _, _, outcomes, _ in runs:
                got = outcomes_of(t, outcomes)
                row += [RESULT_WORDS[run_result(got)], str(len(got or []))]
            rows.append(row)
    commit = (git(root, "rev-parse", "HEAD", check=False) or "").strip() or None
    dirty = bool((git(root, "status", "--porcelain", check=False) or "").strip()) if commit else None
    if args.format == "json":
        doc = {
            "format": "trace-gate-results",
            "format_version": FORMAT_VERSION,
            "tool": TOOL_NAME,
            "tool_version": TOOL_VERSION,
            "commit": commit,
            "dirty": dirty,
            "run_id": args.run_id,
            "runs": [{"name": label, "sha256": meta[0], "timestamp": meta[1]} for label, _, _, meta in runs],
            "columns": head,
            "rows": rows,
        }
        sys.stdout.write(json.dumps(doc, ensure_ascii=False, sort_keys=True, indent=1) + "\n")
        return 0
    if args.format == "csv":
        # 来歴の列(コミット・作業ツリーの変更・実行の ID・名前ごとの XML の sha256 と時刻)を各行に付ける。CSV だけを渡しても証跡の条件を満たすため
        meta_head = ["コミット", "作業ツリーの変更", "実行 ID"] + [x for label, _, _, _ in runs for x in (label + " sha256", label + " 時刻")]
        meta_row = [commit or "", "" if dirty is None else ("あり" if dirty else "なし"), args.run_id or ""] + [
            x for _, _, _, meta in runs for x in (meta[0], meta[1])
        ]
        buf = io.StringIO()
        w = csv.writer(buf, lineterminator="\n")
        w.writerow([csv_cell(v) for v in head + meta_head])
        w.writerows([[csv_cell(v) for v in row + meta_row] for row in rows])
        sys.stdout.write(buf.getvalue())
        return 0
    lines = [
        "# テスト成績書",
        "",
        "- 正本とテストのコミット: %s(作業ツリーの変更 %s)" % (commit or "なし(git の外)", "あり" if dirty else "なし"),
        "- 実行の ID: %s" % (md_cell(args.run_id) if args.run_id else "なし"),
    ]
    for label, _, _, meta in runs:
        lines.append("- %s: JUnit XML の sha256 %s、時刻 %s" % (md_cell(label), meta[0], md_cell(meta[1]) or "なし"))
    lines += ["", "| " + " | ".join(md_cell(v) for v in head) + " |", "|" + "---|" * len(head)]
    lines += ["| " + " | ".join(md_cell(v) for v in row) + " |" for row in rows]
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


# ---------------------------------------------------------------- 配り方: init と写しの錠

VENDOR_LOCK = "tools/trace-gate.lock.json"
VENDOR_LOCK_FORMAT = "trace-gate-vendor-lock"
HOOK_SETTINGS = ".claude/settings.json"
HOOK_PART = "trace-gate-hooks"
HOOK_TEMPLATE = "templates/hooks/claude-settings.stop.json"
PLUGIN_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# (リポの中の道, プラグインの中の雛形。None は走っている検査器そのもの)
VENDORED = [(VENDORED_CHECKER, None)]  # 写して錠で固める
VENDORED_RUNNER = "tools/run_roles.py"
VENDORED_OPTIONAL = [(VENDORED_RUNNER, "scripts/run_roles.py")]  # init は置かないが、あれば錠で固める(人が写した包み)
LEGACY_PART = {CI_WORKFLOW: "ci"}  # 前の版の init が置いた部品の写し(錠の行があるあいだ照らし、ci を切れば --prune で片付ける)
SEEDS = [(CONFIG_NAME, "templates/trace-gate.json")]  # 置くだけ。利用者が書くので上書きしない
REQ_SEED = "skills/write-requirement/templates/requirements.md"  # 要求の雛形。置き場は設定の requirements に従う(seed_requirements)
TOOL_VERSION_RE = re.compile(r'(?m)^TOOL_VERSION = "([^"]+)"')


def plugin_source(rel):
    return os.path.abspath(__file__) if rel is None else os.path.join(PLUGIN_DIR, *rel.split("/"))


def read_bytes(path):
    try:
        with open(path, "rb") as f:
            return f.read()
    except OSError:
        return None


def trace_gate_hooks(data):
    """設定の中の trace-gate のフック(命令に trace_gate.py を含むもの)を、比べられる形の一覧で返す。"""
    out = []
    hooks = data.get("hooks") if isinstance(data, dict) else None
    for event, groups in sorted(hooks.items()) if isinstance(hooks, dict) else []:
        for g in groups if isinstance(groups, list) else []:
            for h in g.get("hooks") if isinstance(g, dict) and isinstance(g.get("hooks"), list) else []:
                if isinstance(h, dict) and "trace_gate.py" in str(h.get("command", "")):
                    out.append({"event": event, "matcher": g.get("matcher"), "hook": h})
    return out


def hook_digest(root):
    """.claude/settings.json の trace-gate のフックの指紋(ほかの設定は含めない)。無いか読めなければ None。"""
    data = read_bytes(os.path.join(root, HOOK_SETTINGS))
    try:
        hooks = trace_gate_hooks(json.loads(data.decode("utf-8"))) if data is not None else None
    except (ValueError, UnicodeDecodeError):
        return None
    return None if hooks is None else sha256(json.dumps(hooks, ensure_ascii=False, sort_keys=True))


NO_HOOKS = sha256(json.dumps([]))  # 設定はあるが trace-gate のフックが無いときの指紋


def vendor_entries(root):
    files = []
    for rel, _ in VENDORED + VENDORED_OPTIONAL:
        data = read_bytes(os.path.join(root, rel))
        if data is not None:
            files.append({"path": rel, "sha256": sha256(data)})
    hooks = hook_digest(root)
    if hooks not in (None, NO_HOOKS):
        files.append({"path": HOOK_SETTINGS, "part": HOOK_PART, "sha256": hooks})
    return sorted(files, key=lambda f: f["path"])


def write_vendor_lock(root):
    """写しの錠を今の中身で書く。版は写した検査器の TOOL_VERSION(読めなければ走っている検査器の版)。"""
    m = TOOL_VERSION_RE.search((read_bytes(os.path.join(root, VENDORED_CHECKER)) or b"").decode("utf-8", "replace"))
    files = vendor_entries(root)
    path = os.path.join(root, VENDOR_LOCK)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(
            dump_lines(
                [
                    ("format", VENDOR_LOCK_FORMAT),
                    ("format_version", FORMAT_VERSION),
                    ("tool", TOOL_NAME),
                    ("tool_version", m.group(1) if m else TOOL_VERSION),
                    ("files", files),
                ]
            )
        )
    return files


def safe_rel(p):
    return (
        isinstance(p, str)
        and p != ""
        and not p.startswith("/")
        and "\\" not in p
        and "\0" not in p
        and not re.match(r"^[A-Za-z]:", p)
        and ".." not in p.split("/")
    )


def read_vendor_lock(root):
    data = read_bytes(os.path.join(root, VENDOR_LOCK))
    if data is None:
        return None
    try:
        doc = json.loads(data.decode("utf-8"))
    except (ValueError, UnicodeDecodeError) as e:
        raise UsageError("写しの錠を読めない: %s(%s)" % (VENDOR_LOCK, clip(e, 120))) from e
    if not isinstance(doc, dict) or doc.get("format") != VENDOR_LOCK_FORMAT or not isinstance(doc.get("files"), list):
        raise UsageError("写しの錠の形が違う: %s(format は %s)" % (VENDOR_LOCK, VENDOR_LOCK_FORMAT))
    for f in doc["files"]:
        if not (isinstance(f, dict) and safe_rel(f.get("path")) and isinstance(f.get("sha256"), str) and f.get("part") in (None, HOOK_PART, "ci")):
            raise UsageError("写しの錠の行が不正: %s(道はリポの中の相対の道にする)" % clip(json.dumps(f, ensure_ascii=False), 120))
    # フックの登録の行は、照らす先も書く先も HOOK_SETTINGS なので、ほかの道の行は捨てる(確かめた道と書く道を食い違わせないため)
    doc["files"] = [f for f in doc["files"] if f.get("part") != HOOK_PART or f["path"] == HOOK_SETTINGS]
    return doc


def check_vendor_lock(root, findings):
    """写しの錠があれば、錠の各ファイル(フックの登録は trace-gate のフックの部分)を今の中身と照らす。錠は書かない。"""
    doc = read_vendor_lock(root)
    for f in doc["files"] if doc else []:
        rel = f["path"]
        if f.get("part") == HOOK_PART:
            got, what = hook_digest(root), "%s の trace-gate のフック" % rel
        else:
            data = read_bytes(os.path.join(root, rel))
            got, what = (None if data is None else sha256(data)), rel
        if got is None:
            message = "写しの錠にある %s が無いか読めない" % what
        elif got != f["sha256"]:
            message = "%s が写しの錠と違う(手で直したか、錠を書き直さずに入れ替えた)" % what
        else:
            continue
        if lock_part(f) == "ci":
            message += "。前の版の init が置いた CI の雛形で、CI はサービス側の雛形のプラグインへ移った。人が中身を見て init --relock で錠から外す"
        findings.append(Finding("vendor-drift", rel, None, message, key=rel))
    for rel in unlocked(root, doc):
        findings.append(
            Finding("vendor-drift", rel, None, "%s が写しの錠に無い(錠の後に写したか置いた。人が中身を見て init --relock で錠に載せる)" % rel, key=rel)
        )


def unlocked(root, doc):
    """錠で固める物(検査器の写し・CI の雛形・人が写した包み・trace-gate のフック)のうち、今あるのに写しの錠に無い物の道。錠が無ければ空。"""
    if not doc:
        return []
    locked = {f["path"] for f in doc["files"]}
    return [f["path"] for f in vendor_entries(root) if f["path"] not in locked]


OUTSIDE = "リポの外を指すので書かなかった: %s(道の ../ か絶対の道か、途中のシンボリックリンクがリポの外へ出る。置き場を直すか、人が中身を見て手で置く)"


def inside_root(root, rel):
    """rel がリポの中の相対の道で、途中のシンボリックリンクをたどってもリポの根の下に収まるか(init が書く・消す前に確かめる)。"""
    if not safe_rel(rel):
        return False
    top = os.path.realpath(root)
    dst = os.path.realpath(os.path.join(root, *rel.split("/")))
    return dst.startswith(top.rstrip(os.sep) + os.sep)


def place(root, rel, src, overwrite):
    if not safe_rel(rel):
        return OUTSIDE % rel
    dst = os.path.join(root, *rel.split("/"))
    new = read_bytes(src)
    if os.path.islink(dst):
        return "シンボリックリンクなので残した: %s" % rel
    if not inside_root(root, rel):
        return OUTSIDE % rel
    if os.path.exists(dst):
        if os.path.isfile(dst) and os.path.samefile(src, dst):
            return "走っている検査器そのものなので残した: %s" % rel
        same = read_bytes(dst) == new
        if not overwrite:
            return "既にあるので残した: %s%s" % (rel, "" if same else "(配る物と違う。配る物に替えるなら --force)")
        if same:
            return "配る物と同じなので残した: %s" % rel
        verb = "上書きした"
    else:
        verb = "置いた"
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    with open(dst, "wb") as f:
        f.write(new)
    return "%s: %s" % (verb, rel)


def place_hook(root, update=False):
    """フックの設定を置く。既にあれば trace-gate のフックの項目だけを雛形と比べ、違えば差の行を出し、update のときだけその項目を入れ替える。
    (出力の行の並び, 入れ替えたか)。ほかの設定とほかのフックは中身を変えない(ファイルは JSON として書き直す)。"""
    dst = os.path.join(root, *HOOK_SETTINGS.split("/"))
    if not os.path.lexists(dst):
        return [place(root, HOOK_SETTINGS, plugin_source(HOOK_TEMPLATE), overwrite=False) + "(Stop のフック)"], False
    try:
        data = json.loads(read_bytes(dst).decode("utf-8"))
        template = json.loads(read_bytes(plugin_source(HOOK_TEMPLATE)).decode("utf-8"))
    except (ValueError, UnicodeDecodeError, AttributeError):
        return ["既にあるので残した: %s(JSON として読めない。%s の Stop のフックを人が手で足す)" % (HOOK_SETTINGS, HOOK_TEMPLATE)], False
    have, want = trace_gate_hooks(data), trace_gate_hooks(template)
    if have == want:
        return ["既にあるので残した: %s(trace-gate のフックは雛形と同じ)" % HOOK_SETTINGS], False
    diff = list(
        difflib.unified_diff(
            json.dumps(have, ensure_ascii=False, indent=1).splitlines(),
            json.dumps(want, ensure_ascii=False, indent=1).splitlines(),
            "%s の trace-gate のフック" % HOOK_SETTINGS,
            HOOK_TEMPLATE,
            lineterm="",
        )
    )
    if not update:
        return [
            "既にあるので残した: %s(trace-gate のフックが雛形と違う。人が差を見て、入れ替えるなら init --update-hooks か --force。ほかの項目は変えない)"
            % HOOK_SETTINGS
        ] + diff, False
    if not inside_root(root, HOOK_SETTINGS):
        return [OUTSIDE % HOOK_SETTINGS], False
    replace_trace_gate_hooks(data, template)
    with open(dst, "w", encoding="utf-8") as f:
        f.write(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    return ["入れ替えた: %s の trace-gate のフック(雛形 %s に。ほかの項目は変えていない)" % (HOOK_SETTINGS, HOOK_TEMPLATE)] + diff, True


def is_tg_hook(h):
    return isinstance(h, dict) and "trace_gate.py" in str(h.get("command", ""))


def replace_trace_gate_hooks(data, template):
    """設定の trace-gate のフックを雛形のものに替える。出来事の並びと、ほかの組の並びは変えず、雛形の組は元の trace-gate の組の位置
    (出来事に無かったら終わり)に置く。ほかのフックと同じ組にあった trace-gate のフックは、その組から外して、組の前に雛形の組を置く。"""
    hooks = data.get("hooks") if isinstance(data.get("hooks"), dict) else {}
    wanted = {}
    for event, groups in (template.get("hooks") or {}).items():
        for g in groups:
            mine = [h for h in g.get("hooks") or [] if is_tg_hook(h)]
            if mine:
                wanted.setdefault(event, []).append(dict(g, hooks=mine))
    out = {}
    for event, groups in hooks.items():
        new, placed = [], False
        for g in groups if isinstance(groups, list) else []:
            if isinstance(g, dict) and isinstance(g.get("hooks"), list) and any(is_tg_hook(h) for h in g["hooks"]):
                if not placed:
                    new.extend(wanted.get(event, []))
                    placed = True
                rest = [h for h in g["hooks"] if not is_tg_hook(h)]
                if rest:
                    new.append(dict(g, hooks=rest))
                continue
            new.append(g)
        if not placed:
            new.extend(wanted.get(event, []))
        if new:
            out[event] = new
    for event, groups in wanted.items():
        if event not in out:
            out[event] = list(groups)
    if out:
        data["hooks"] = out
    else:
        data.pop("hooks", None)


def relock_hooks(root):
    """写しの錠のフックの登録の行だけを今の指紋に書き直す(ほかの写しの行は変えない。手で直した写しを錠に取り込まないため)。"""
    doc = read_vendor_lock(root)
    if not doc:
        return
    rows = [f for f in doc["files"] if f.get("part") != HOOK_PART]
    digest = hook_digest(root)
    if digest not in (None, NO_HOOKS):
        rows.append({"path": HOOK_SETTINGS, "part": HOOK_PART, "sha256": digest})
    with open(os.path.join(root, VENDOR_LOCK), "w", encoding="utf-8") as out:
        out.write(
            dump_lines(
                [(k, sorted(rows, key=lambda f: f["path"]) if k == "files" else doc[k]) for k in ("format", "format_version", "tool", "tool_version", "files")]
            )
        )


def seed_requirements(root, cfg):
    """要求の雛形を設定の requirements が指す置き場に置く。既に正本があれば置かず、置き場が型(グロブ)だけならファイルの名を決めずに案内する。"""
    found = requirement_files(root, cfg)
    if found:
        more = "ほか %d" % (len(found) - 1) if len(found) > 1 else ""
        return "要求の正本は既にあるので、雛形を置かなかった: %s%s(利用者の書くファイル。--force でも変えない)" % (found[0], more and "(" + more + ")")
    target = next((p for p in cfg["requirements"] if not has_glob(p)), None)
    if target is None:
        return "要求の正本の置き場 %s に当たるファイルが無い。雛形 %s を写して、置き場に当たる名前で書く(init は型の置き場のファイルの名を決めない)" % (
            "・".join(cfg["requirements"]),
            REQ_SEED,
        )
    return place(root, target, plugin_source(REQ_SEED), overwrite=False)


RUST_CFG_TEST = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")


def rust_unit_test_hint(root, cfg):
    """src の中の Rust のファイルのうち、設定の tests に当たらず #[cfg(test)] を持つもの(単体の試験)があれば、tests に足す glob の案内の1行。
    既定の tests に src を足さないのは、tests に当たるファイルを f2p-run が base に丸ごと写し、本番のコードまで写すため。"""
    hits = []
    for p in select(walk_files(root), ["**/*.rs"], cfg["exclude"]):
        dirs = p.split("/")[:-1]
        if "src" in dirs and not matches(p, cfg["tests"]) and RUST_CFG_TEST.search(lex_c(read_text(root, p), "rust")[0]):
            hits.append(p)
    if not hits:
        return None
    globs = uniq("/".join(p.split("/")[: max(i for i, x in enumerate(p.split("/")[:-1]) if x == "src") + 1]) + "/**/*.rs" for p in hits)  # 最後の src まで
    return (
        "Rust の src に #[cfg(test)] の単体の試験を持つファイルが %d 件(例: %s)。印を読むなら trace-gate.json の tests に %s を足す(置いた設定には足していない。"
        "tests に当たるファイルは f2p-run が base に丸ごと写すので、本番のコードと同じファイルの試験は f2p-run で確かめられない。"
        "f2p-run で確かめるなら、試験を別のファイル(`#[cfg(test)] mod tests;` と tests.rs)に分け、その道だけを足す)"
        % (len(hits), hits[0], "・".join('"%s"' % g for g in globs))
    )


def toml_members(text, section):
    """TOML の [section] の members = [...] の文字列(標準ライブラリだけで読むための小さな読み)。"""
    m = re.search(r"(?ms)^\[%s\]\s*$(.*?)(?=^\[|\Z)" % re.escape(section), text)
    if not m:
        return []
    arr = re.search(r"(?s)\bmembers\s*=\s*\[(.*?)\]", m.group(1))
    return re.findall(r"[\"']([^\"']+)[\"']", arr.group(1)) if arr else []


def workspace_packages(root):
    """ワークスペースの宣言(package.json・pnpm-workspace.yaml・Cargo.toml・go.work・pyproject.toml)からパッケージの glob を読む。"""
    found = []
    data = read_bytes(os.path.join(root, "package.json"))
    if data is not None:
        try:
            ws = json.loads(data.decode("utf-8")).get("workspaces")
        except (ValueError, UnicodeDecodeError, AttributeError):
            ws = None
        ws = ws.get("packages") if isinstance(ws, dict) else ws
        found.extend(x for x in ws or [] if isinstance(x, str))
    data = read_bytes(os.path.join(root, "pnpm-workspace.yaml"))
    if data is not None:
        block = re.search(r"(?ms)^packages:\s*$(.*?)(?=^\S|\Z)", data.decode("utf-8", "replace"))
        if block:
            found.extend(m.strip().strip("\"'") for m in re.findall(r"(?m)^\s*-\s*(.+?)\s*$", block.group(1)))
    for name, section in (("Cargo.toml", "workspace"), ("pyproject.toml", "tool.uv.workspace")):
        data = read_bytes(os.path.join(root, name))
        if data is not None:
            found.extend(toml_members(data.decode("utf-8", "replace"), section))
    data = read_bytes(os.path.join(root, "go.work"))
    if data is not None:
        text = data.decode("utf-8", "replace")
        uses = re.findall(r"(?ms)^use\s*\((.*?)\)", text)
        found.extend(x for b in uses for x in b.split())
        found.extend(re.findall(r"(?m)^use\s+([^\s(]+)\s*$", text))
    out = []
    for x in found:
        x = x[2:] if x.startswith("./") else x
        if x and not x.startswith("!") and x not in out:
            out.append(x.rstrip("/"))
    return out


def lock_part(f):
    """写しの錠の行が属する部品(無ければ None)。古い錠の CI の雛形の行は part が無いので道で見る。"""
    if f.get("part") == HOOK_PART:
        return "hook"
    return f.get("part") or LEGACY_PART.get(f["path"])


def legacy_ci_lines(doc):
    """写しの錠に残る、前の版の init が置いた CI の雛形の行の道。"""
    return [f["path"] for f in (doc or {}).get("files", []) if lock_part(f) == "ci"]


def remove_trace_gate_hooks(root):
    """.claude/settings.json から trace-gate のフックの登録だけを外し、ほかの設定は残す。"""
    if not inside_root(root, HOOK_SETTINGS):
        raise UsageError(OUTSIDE % HOOK_SETTINGS)
    path = os.path.join(root, *HOOK_SETTINGS.split("/"))
    data = json.loads(read_bytes(path).decode("utf-8"))
    strip_trace_gate_hooks(data)
    with open(path, "w", encoding="utf-8") as f:
        f.write(json.dumps(data, ensure_ascii=False, indent=2) + "\n")


def strip_trace_gate_hooks(data):
    """設定の辞書から trace-gate のフック(命令に trace_gate.py を含むもの)だけを外す。空になった組と出来事も外す。"""
    hooks = data.get("hooks") or {}
    for event in list(hooks):
        groups = []
        for g in hooks[event] if isinstance(hooks[event], list) else []:
            if isinstance(g, dict) and isinstance(g.get("hooks"), list):
                g = dict(g, hooks=[h for h in g["hooks"] if not (isinstance(h, dict) and "trace_gate.py" in str(h.get("command", "")))])
                if not g["hooks"]:
                    continue
            groups.append(g)
        if groups:
            hooks[event] = groups
        else:
            del hooks[event]
    if not hooks:
        data.pop("hooks", None)


def prune_parts(root, cfg, dry):
    """切った部品の置いた物のうち、写しの錠と中身が同じ物だけを消して錠から外す。直された物は残して挙げる。"""
    doc = read_vendor_lock(root)
    if not doc:
        return ["写しの錠が無いので、片付ける物は無い: %s" % VENDOR_LOCK]
    keep, lines = [], []
    for f in doc["files"]:
        part = lock_part(f)
        if part is None or part_on(cfg, part):
            keep.append(f)
            continue
        if part == "hook":
            got, what = hook_digest(root), "%s の trace-gate のフック" % f["path"]
        else:
            data = read_bytes(os.path.join(root, f["path"]))
            got, what = (None if data is None else sha256(data)), f["path"]
        if got in (None, NO_HOOKS):
            lines.append("既に無いので錠から外す: %s(部品 %s)" % (what, part))
        elif got != f["sha256"]:
            keep.append(f)
            lines.append("直されているので残した: %s(部品 %s は切られている。消すなら人が中身を見て手で消し、init --relock で錠を書き直す)" % (what, part))
        else:
            target = HOOK_SETTINGS if part == "hook" else f["path"]  # 書くか消す先そのものを確かめる
            if not inside_root(root, target):
                keep.append(f)
                lines.append(OUTSIDE % target)
                continue
            lines.append("%s: %s(部品 %s)" % ("消す" if dry else "消した", what, part))
            if not dry:
                if part == "hook":
                    remove_trace_gate_hooks(root)
                else:
                    os.remove(os.path.join(root, f["path"]))
    if not lines:
        lines.append("切った部品の置いた物は無い(切った部品: %s)" % ("・".join(parts_off(cfg)) or "無し"))
    if not dry and len(keep) != len(doc["files"]) and inside_root(root, VENDOR_LOCK):
        with open(os.path.join(root, VENDOR_LOCK), "w", encoding="utf-8") as out:
            out.write(dump_lines([(k, keep if k == "files" else doc[k]) for k in ("format", "format_version", "tool", "tool_version", "files")]))
        lines.append("写しの錠から外した: %s" % VENDOR_LOCK)
    if dry:
        lines.append("--dry-run なので、何も変えていない")
    return lines


def cmd_init(args, root, cfg):
    lines = []
    if args.dry_run and not args.prune:
        raise UsageError("--dry-run は --prune と一緒に使う")
    if args.prune:
        if args.force or args.relock or args.hook or args.update_hooks:
            raise UsageError("--prune は切った部品の片付けだけをする。--force・--relock・--hook・--update-hooks と一緒に使わない")
        sys.stdout.write("".join(esc(line) + "\n" for line in prune_parts(root, cfg, args.dry_run)))
        return 0
    if (args.hook or args.update_hooks) and not part_on(cfg, "hook"):
        raise UsageError("部品 hook は設定で切られている。フックを置くなら trace-gate.json の parts.hook を true にする")
    if args.relock and (args.hook or args.update_hooks):
        raise UsageError("--relock は写しの錠だけを書き直す。--hook・--update-hooks と一緒に使わない")
    hooks_replaced = False
    hook = part_on(cfg, "hook") and not args.relock
    legacy = legacy_ci_lines(read_vendor_lock(root))
    if not args.relock:
        wanted = [src for _, src in VENDORED + SEEDS] + [REQ_SEED] + ([HOOK_TEMPLATE] if hook else [])
        missing = [s for s in wanted if not os.path.isfile(plugin_source(s))]
        if missing:
            raise UsageError("配る物の雛形が見つからない: %s(プラグインの scripts/trace_gate.py から走らせる)" % missing[0])
        for rel, src in VENDORED:
            lines.append(place(root, rel, plugin_source(src), overwrite=args.force))
        fresh_config = not os.path.lexists(os.path.join(root, CONFIG_NAME))
        for rel, src in SEEDS:
            line = place(root, rel, plugin_source(src), overwrite=False)
            lines.append(line + ("(利用者の書くファイル。--force でも変えない)" if line.startswith("既に") else ""))
        scfg = load_config(root) if fresh_config and not args.config else cfg  # 置いたばかりの設定か、既にある設定の requirements に従う
        lines.append(seed_requirements(root, scfg))
        pkgs = workspace_packages(root) if fresh_config else []
        if pkgs:
            doc = json.loads(read_text(root, CONFIG_NAME))
            doc["packages"] = pkgs
            with open(os.path.join(root, CONFIG_NAME), "w", encoding="utf-8") as f:
                f.write(json.dumps(doc, ensure_ascii=False, indent=2) + "\n")
            lines.append("ワークスペースの宣言から packages を書いた: %s(人が確かめる)" % "・".join(pkgs))
        if hook:
            got, hooks_replaced = place_hook(root, update=args.force or args.update_hooks)
            lines.extend(got)
    if not inside_root(root, VENDOR_LOCK):
        lines.append(OUTSIDE % VENDOR_LOCK)
    elif args.relock or args.force or not os.path.isfile(os.path.join(root, VENDOR_LOCK)):
        files = write_vendor_lock(root)
        lines.append("写しの錠を書いた: %s(%s)" % (VENDOR_LOCK, "・".join(f["path"] for f in files) or "写し無し"))
        for rel in legacy:
            lines.append(
                "写しの錠から外した: %s(前の版の init が置いた CI の雛形。ファイルは消していない。CI はサービス側の雛形のプラグインへ移った。"
                "続けるならそちらで作り直し、やめるなら人が消す)" % rel
            )
    elif hooks_replaced:
        relock_hooks(root)
        lines.append("写しの錠のフックの行だけを書き直した: %s(ほかの写しの行は変えていない)" % VENDOR_LOCK)
    else:
        lines.append("写しの錠は既にあるので変えなかった: %s(書き直すのは人が init --relock で)" % VENDOR_LOCK)
        for rel in unlocked(root, read_vendor_lock(root)):
            lines.append("写しの錠に無い: %s(check は vendor-drift で止める。人が中身を見て init --relock で錠に載せる)" % rel)
        for rel in legacy:
            lines.append(
                "前の版の init が置いた CI の雛形: %s(錠に行があるあいだ check が照らす。CI はサービス側の雛形のプラグインへ移った。"
                "移すなら init --relock で錠から外し、やめるなら parts.ci を false にして init --prune)" % rel
            )
    if not args.relock:
        lines.append(SERVICE_GUIDE + ("(部品 ci が有効: CI もそちらで作る)" if part_on(cfg, "ci") else ""))
        lines.extend(recommendations(root))
        hint = rust_unit_test_hint(root, scfg)
        if hint:
            lines.append(hint)
        lines.append(
            "次: %s に要求を書き、python3 tools/trace_gate.py check を通す。置いた物と写しの錠の差分は人が見てコミットする(AI は錠を書かない)"
            % "・".join(scfg["requirements"])
        )
    sys.stdout.write("".join(esc(line) + "\n" for line in lines))
    return 0


# ---------------------------------------------------------------- サービス側の守りへの口: guards

# サービス側の雛形(CODEOWNERS・CI・ブランチの規則)は別のプラグインが持ち、この出力を読んで作る(人の決定)。
# 外の道具との約束(README の「外の道具との約束」)。種類・検査・鍵を足すのは format_version を変えずに、消す・名前を変える・意味を変えるときは上げる
GUARDS_FORMAT = "trace-gate-guards"
GUARD_KINDS = [
    ("requirements", "要求の正本"),
    ("upstream", "上流・要求の根拠の文書"),
    ("approvals", "承認の記録"),
    ("renames", "名前の表"),
    ("tests", "テストの置き場"),
    ("test_config", "テストの設定"),
    ("shared_config", "依存とテストの設定を混ぜて持つファイル"),
    ("baselines", "基準のファイル・スナップショットとゴールデンと見本"),
    ("roles", "言語ごとの道具の役の設定と基線"),
    ("instructions", "AI への指示のファイル"),
    ("ci", "CI の置き場(ワークフロー・composite action・GitLab の CI)"),
    ("config", "trace-gate の設定"),
    ("vendored", "写した検査器・包みと写しの錠"),
]
PACKAGE_KINDS = ("requirements", "upstream", "approvals", "renames", "tests", "test_config", "baselines", "config")  # パッケージの設定ごとに違う種類
LEDGER_OUT = "trace-gate-ledger.json"
ROLES_CONFIG_RE = re.compile(r"^roles(?:\.([A-Za-z0-9_-]+))?\.json$")  # 根の役の設定(roles-baseline は基線なので当たらない)


def guard_places(root, cfg):
    """守る置き場の種類ごとのグロブ(root からの道)。どの部品の設定でも全部出す(人が見る所を減らさない)。"""
    return {
        "requirements": list(cfg["requirements"]),
        "upstream": list(cfg["upstream"]),
        "approvals": [cfg["approvals"]],
        "renames": [cfg.get("renames") or DEFAULT_RENAMES],
        "tests": list(cfg["tests"]),
        "test_config": list(cfg["test_config"]),
        "shared_config": list(SHARED_CONFIG),
        "baselines": list(cfg["baselines"]),
        "roles": list(ROLES_GLOBS),
        "instructions": list(INSTRUCTION_FILES),
        "ci": list(CI_PLACES),
        "config": [config_rel(root, cfg) or CONFIG_NAME],
        "vendored": [VENDORED_CHECKER, VENDORED_RUNNER, VENDOR_LOCK],
    }


def guard_step(args, when, gate=True, keep=None):
    """検査の段。when は always(どの契機でも)か pull_request。gate が false の段は、前の段が落ちても走らせ、落ちても合否に数えない。"""
    return {"run": args, "when": when, "gate": gate, "keep": keep}


def guard_checks(root, cfg):
    tg = ["python3", VENDORED_CHECKER]
    checks = [
        {
            "name": "check",
            "required": True,
            "steps": [
                guard_step(tg + ["check"], "always"),
                guard_step(tg + ["check", "--base", "{base}"], "pull_request"),
                guard_step(tg + ["ledger", "--out", LEDGER_OUT], "always", gate=False, keep=LEDGER_OUT),
            ],
        }
    ]
    if part_on(cfg, "weaken"):  # 必須にしない。落ちたら人がテストの差分を見て通す
        checks.append({"name": "weaken", "required": False, "steps": [guard_step(tg + ["weaken", "--base", "{base}", "--head", "{head}"], "pull_request")]})
    for name in sorted(os.listdir(root)):
        m = ROLES_CONFIG_RE.match(name)
        if not (m and os.path.isfile(os.path.join(root, name))):
            continue
        rr, out = ["python3", VENDORED_RUNNER, "--config", name], "roles-out" + ("-" + m.group(1) if m.group(1) else "")
        checks.append(
            {
                "name": "roles" + ("-" + m.group(1) if m.group(1) else ""),
                "required": True,
                "steps": [
                    guard_step(rr + ["--format", "json", "--out-dir", out], "always", keep=out),
                    guard_step(rr + ["--check-baseline", "--base", "{base}"], "pull_request"),
                ],
            }
        )
    return checks


def guards_doc(root, cfg):
    got = guard_places(root, cfg)
    for p in package_dirs(root, cfg):
        proot, pcfg = package_config(root, p, cfg)
        mine = guard_places(proot, pcfg)
        for kind in PACKAGE_KINDS:
            got[kind] = uniq(got[kind] + [p + "/" + x for x in mine[kind]])
    return {
        "format": GUARDS_FORMAT,
        "format_version": FORMAT_VERSION,
        "tool": TOOL_NAME,
        "tool_version": TOOL_VERSION,
        "git_prefix": subdir_prefix(root).rstrip("/"),
        "places": [{"kind": kind, "patterns": got[kind]} for kind, _ in GUARD_KINDS],
        "checks": guard_checks(root, cfg),
        "run_from_base": [VENDORED_CHECKER, VENDORED_RUNNER],
        "parts": {p: part_on(cfg, p) for p in PARTS},
        # E2E の段の選び方(PR で触った要求の stest のファイル)。走らせる命令は読む側の設定が持つ
        "select": {
            "stest": {"run": ["python3", VENDORED_CHECKER, "intents", "--base", "{base}", "--kind", "stest", "--format", "files"], "when": "pull_request"}
        },
    }


def cmd_guards(args, root, cfg):
    """守る置き場・必須の検査・部品の状態を出す。サービス側の雛形のプラグインが --format json で読む。何も書かない。"""
    doc = guards_doc(root, cfg)
    if args.format == "json":
        sys.stdout.write(json.dumps(doc, ensure_ascii=False, sort_keys=True, indent=1) + "\n")
        return 0
    notes = dict(GUARD_KINDS)
    lines = ["trace-gate guards(%s %d。サービス側の雛形のプラグインは --format json で読む)" % (GUARDS_FORMAT, FORMAT_VERSION), "守る置き場(--root からの道):"]
    lines += ["  %s(%s): %s" % (p["kind"], notes[p["kind"]], "・".join(p["patterns"]) or "無し") for p in doc["places"]]
    lines.append("検査の仕事({base}・{head} は比べる先と後の git の参照):")
    for c in doc["checks"]:
        steps = [
            "[%s%s] %s" % ("いつも" if s["when"] == "always" else "PR", "" if s["gate"] else "・合否に数えない", " ".join(s["run"]))
            + ("(%s を残す)" % s["keep"] if s["keep"] else "")
            for s in c["steps"]
        ]
        lines.append("  %s(%s): %s" % (c["name"], "必須" if c["required"] else "必須にせず、落ちたら人が見る", " / ".join(steps)))
    lines.append("PR では base の側から取り出して走らせる: %s" % "・".join(doc["run_from_base"]))
    lines += ["選び方 %s(PR で触った要求の印のテストのファイル): %s" % (k, " ".join(v["run"])) for k, v in doc["select"].items()]
    lines.append("部品: %s" % "・".join("%s %s" % (p, "入" if on else "切") for p, on in doc["parts"].items()))
    lines.append("git の根から --root までの道: %s" % (doc["git_prefix"] or "(根)"))
    sys.stdout.write("".join(esc(line) + "\n" for line in lines))
    return 0


# ---------------------------------------------------------------- 運用: 前の台帳との比べ

COMPARE_FIELDS = ("version", "priority", "status", "tests", "treatment")


def read_ledger(path):
    data = read_bytes(path)
    if data is None:
        raise UsageError("前の台帳を読めない: %s" % clip(path))
    try:
        doc = json.loads(data.decode("utf-8"))
    except (ValueError, UnicodeDecodeError) as e:
        raise UsageError("前の台帳を読めない: %s(%s)" % (clip(path), clip(e, 120))) from e
    if not isinstance(doc, dict) or doc.get("format") != LEDGER_FORMAT:
        raise UsageError("前の台帳の形が違う: %s(format は %s)" % (clip(path), LEDGER_FORMAT))
    rows = doc.get("requirements")
    if not isinstance(rows, list) or not all(isinstance(r, dict) and isinstance(r.get("name"), str) for r in rows):
        raise UsageError("前の台帳の requirements が読めない: %s" % clip(path))
    return doc, sha256(data)


def compare_ledgers(prev, prev_sha, rows):
    """要求ごとに new・unchanged・updated・absent。行の番号と道は比べない(ほかの要求の出し入れでずれるため)。"""
    old, new = {r["name"]: r for r in prev["requirements"]}, {r["name"]: r for r in rows}
    same = lambda a, b: json.dumps(a, sort_keys=True) == json.dumps(b, sort_keys=True)  # noqa: E731
    changes, tally = [], Counter()
    for name in sorted(set(old) | set(new)):
        o, n = old.get(name), new.get(name)
        fields = [k for k in COMPARE_FIELDS if not same(o.get(k), n.get(k))] if o and n else []
        change = "new" if o is None else "absent" if n is None else "updated" if fields else "unchanged"
        tally[change] += 1
        changes.append({"change": change, "fields": fields, "name": name, "id": (n or o).get("id"), "was": o.get("id") if o else None})
    header = {
        "previous_commit": prev.get("commit"),
        "previous_tool_version": prev.get("tool_version"),
        "previous_sha256": prev_sha,
        "counts": {k: tally[k] for k in ("new", "unchanged", "updated", "absent")},
    }
    return header, changes


# ---------------------------------------------------------------- 運用: 意図の要約(TG-9)


def is_assert_node(n):
    if isinstance(n, ast.Assert):
        return True
    if isinstance(n, ast.Call):
        name = dotted(n.func)
        return name.split(".")[-1].startswith(("assert", "expect")) or name in ("pytest.raises", "pytest.warns", "pytest.fail", "self.fail")
    return False


def test_intent(text, t):
    """(意図の1行か None, [(行, 断言の行)])。意図は Python だけ: 最初の docstring の1行目か、直前と本体の最初のコメント(印を除く)。"""
    lines = text.split("\n")

    def line_at(no):
        return clip(lines[no - 1].strip(), 160) if 0 < no <= len(lines) else ""

    if t.lang != "python":
        mask, _ = lex_c(text, t.lang)
        line_of = line_index(text)
        nos = sorted({line_of(m.start()) for m in ASSERT_WORDS[t.lang].finditer(mask)} & set(range(t.start, t.end + 1)))
        if t.lang == "js":
            intent = clip(t.name, 160) or None
        else:
            intent, k = None, t.start - 1
            while k >= 1 and re.match(r"^\s*//", lines[k - 1]):
                body = re.sub(r"^\s*///?\s*", "", lines[k - 1]).strip()
                if body and not MARK_HINT.search(body):
                    intent = clip(body, 160)
                k -= 1
        return intent, [(no, line_at(no)) for no in nos]
    try:
        tree = ast.parse(text)
        comments = [(tok.start[0], tok.string) for tok in tokenize.generate_tokens(io.StringIO(text).readline) if tok.type == tokenize.COMMENT]
    except (SyntaxError, ValueError, tokenize.TokenError, IndentationError):
        return None, []
    fn = next((f for q, f, _ in py_tests(tree) if f.name == t.name and q == t.qual and f.end_lineno == t.end), None)
    if fn is None:
        return None, []
    intent = None
    doc = ast.get_docstring(fn)
    if doc and doc.strip():
        intent = doc.strip().splitlines()[0].strip()
    else:
        above, k = set(), t.start - 1
        while k >= 1 and lines[k - 1].strip().startswith("#"):
            above.add(k)
            k -= 1
        for no, c in comments:
            body = c.lstrip("#").strip()
            if body and (no in above or t.start <= no <= t.end) and not MARK_HINT.search(c):
                intent = body
                break
    nos = sorted({n.lineno for n in ast.walk(fn) if is_assert_node(n)})
    return (clip(intent, 160) if intent else None), [(no, line_at(no)) for no in nos]


def wanted_reqs(args, reqs, default):
    if not args.req:
        return default
    names = [n.strip() for n in args.req.split(",") if n.strip()]
    for n in names:
        if n not in reqs:
            raise UsageError("--req の要求が正本に無い: %s" % clip(n, 40))
    return names


DOC_FIELDS = re.compile(r"^\s*(観点|前提)[:：]\s*(\S.*?)\s*$")


ANNOTATION_RE = [
    re.compile(r"type\s*:\s*(['\"])(観点|前提)\1\s*,\s*description\s*:\s*(['\"])((?:(?!\3)[^\n\\])*)\3"),
    re.compile(r"description\s*:\s*(['\"])((?:(?!\1)[^\n\\])*)\1\s*,\s*type\s*:\s*(['\"])(観点|前提)\3"),
]


def doc_fields(text, t):
    """印のテストの観点・前提。Python は docstring の2行目より後ろの「観点:」「前提:」の行、TS・JS は Playwright の annotation の字面(式は読まない)。"""
    if t.lang == "js":
        span = "\n".join(text.split("\n")[t.start - 1 : t.end])
        out = {}
        for m in ANNOTATION_RE[0].finditer(span):
            out.setdefault(m.group(2), clip(m.group(4), 160))
        for m in ANNOTATION_RE[1].finditer(span):
            out.setdefault(m.group(4), clip(m.group(2), 160))
        return out
    if t.lang != "python":
        return {}
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return {}
    fn = next((f for q, f, _ in py_tests(tree) if f.name == t.name and q == t.qual and f.end_lineno == t.end), None)
    doc = ast.get_docstring(fn) if fn is not None else None
    out = {}
    for line in (doc or "").split("\n")[1:]:
        m = DOC_FIELDS.match(line)
        if m and m.group(1) not in out:
            out[m.group(1)] = clip(m.group(2), 160)
    return out


def touched_reqs(root, cfg, base):
    """base から触った要求の名前。base に無い・版が上がった・版が同じで文か受け入れが変わった要求と、本体か印が変わったテストの印の要求。"""
    old, new = reqs_at(root, base, base_config(root, base, cfg)), reqs_at(root, None, cfg)
    names = {n for n, r in new.items() if n not in old or old[n].version != r.version or old[n].fingerprint != r.fingerprint}
    return names | set(changed_test_links(root, cfg, base, None, git_changes(root, base, None)))


def cmd_intents(args, root, cfg):
    res = analyze(root, cfg)
    names = wanted_reqs(args, res["reqs"], list(res["reqs"]))
    if args.base:  # E2E の段の選び方: PR で触った要求だけ(guards の select)
        touched = touched_reqs(root, cfg, args.base)
        names = [n for n in names if n in touched]
    texts, out = {}, []
    for name in sorted(names):
        r, items = res["reqs"][name], []
        linked = (t for t in res["tests"] if any(ln["name"] == name and (not args.kind or ln["kind"] == args.kind) for ln in t.links))
        for t in sorted(linked, key=lambda t: t.node):
            if t.path not in texts:
                texts[t.path] = read_text(root, t.path)
            intent, asserts = test_intent(texts[t.path], t)
            ln = [x for x in t.links if x["name"] == name][0]
            items.append(
                {
                    "node": t.node,
                    "lang": t.lang,
                    "state": test_state(t),
                    "link": ln.get("status"),
                    "mark_version": ln["version"],
                    "intent": intent,
                    "fields": doc_fields(texts[t.path], t),
                    "asserts": [{"line": no, "text": s} for no, s in asserts],
                }
            )
        row = {"id": r.id, "name": name, "priority": r.priority, "text": r.text, "accept": r.accept, "tests": items}
        row.update(tail_fields(r))
        out.append(row)
    if args.format == "files":  # 選んだテストのファイルの道(--root から)を1行ずつ。E2E の走り手に渡す
        paths = sorted({t["node"].split("::", 1)[0] for r in out for t in r["tests"]})
        sys.stdout.write("".join(esc(p) + "\n" for p in paths))
        return 0
    if args.format == "json":
        doc = {"format": "trace-gate-intents", "format_version": FORMAT_VERSION, "tool": TOOL_NAME, "tool_version": TOOL_VERSION, "requirements": out}
        sys.stdout.write(json.dumps(doc, ensure_ascii=False, sort_keys=True, indent=1) + "\n")
        return 0
    if args.format in ("md", "csv"):
        return intents_table(out, args.format)
    lines = []
    for r in out:
        lines.append("%s %s %s" % (r["id"], r["priority"], clip(r["text"], 120)))
        lines.extend("  受け入れ: %s" % clip(a, 160) for a in r["accept"].split("\n"))
        if r.get("treatment"):
            lines.append("  扱い: %s%s" % (r["treatment"], "(%s)" % clip(r["reason"], 80) if r.get("reason") else ""))
        if not r["tests"]:
            lines.append("  (印のテストなし)")
        for t in r["tests"]:
            notes = ([] if t["state"] == "ok" else ["状態: " + t["state"]]) + (
                [] if t["link"] == "covers" else ["印: %s(版 %d)" % (t["link"], t["mark_version"])]
            )
            lines.append("  - %s%s" % (clip(t["node"], 160), "(%s)" % "・".join(notes) if notes else ""))
            if t["lang"] == "python":
                lines.append("      意図: %s" % (esc(t["intent"]) if t["intent"] else "(docstring もコメントも無い)"))
            for a in t["asserts"]:
                lines.append("      断言 %d: %s" % (a["line"], esc(a["text"])))
            if not t["asserts"]:
                lines.append("      断言: (無い)")
    lines.append(
        "trace-gate intents: 要求 %d 件・印のテスト %d 本(意図は人が要求の文と照らして読む)" % (len(out), len({t["node"] for r in out for t in r["tests"]}))
    )
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


def csv_cell(v):
    """CSV のセル。制御文字を逃がし、表計算で式として動く頭の文字には ' を付ける(リポ由来の文が式にならないように)。"""
    v = esc(str(v))
    return "'" + v if v[:1] in ("=", "+", "-", "@", "\t", "\r") else v


def md_cell(v):
    """Markdown の表のセル。制御文字を逃がし、縦棒と改行を崩れない形にする。"""
    return esc(str(v)).replace("|", "\\|").replace("\n", " ")


TABLE_HEAD = ["要求", "優先度", "要求の文", "受け入れ", "テスト", "状態", "意図", "観点", "前提", "断言"]


def intents_table(out, fmt):
    """要求とテストの組ごとに1行(印のテストの無い要求も1行)。表は手で直さない鏡で、時刻を入れない。"""
    rows = []
    for r in out:
        base = [r["id"], r["priority"], r["text"], r["accept"]]
        if not r["tests"]:
            rows.append(base + ["", "印のテストなし", "", "", "", ""])
        for t in r["tests"]:
            f = t.get("fields") or {}
            rows.append(
                base
                + [
                    t["node"],
                    t["state"],
                    t["intent"] or "",
                    f.get("観点", ""),
                    f.get("前提", ""),
                    " / ".join("%d: %s" % (a["line"], a["text"]) for a in t["asserts"]),
                ]
            )
    if fmt == "csv":
        buf = io.StringIO()
        w = csv.writer(buf, lineterminator="\n")
        w.writerow(TABLE_HEAD)
        w.writerows([[csv_cell(v) for v in row] for row in rows])
        sys.stdout.write(buf.getvalue())
        return 0
    lines = ["| " + " | ".join(TABLE_HEAD) + " |", "|" + "---|" * len(TABLE_HEAD)]
    lines += ["| " + " | ".join(md_cell(v) for v in row) + " |" for row in rows]
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


# ---------------------------------------------------------------- 運用: 実装前と実装後の自動の実行(TG-3)


class Terminated(BaseException):
    """SIGTERM か SIGHUP で打ち切られた。finally の片付けを走らせるための例外(Exception の受けに紛れないように BaseException)。"""

    def __init__(self, signum):
        super().__init__(signum)
        self.signum = signum


@contextlib.contextmanager
def terminate_to_cleanup():
    """SIGTERM と SIGHUP を Terminated にして、finally の片付け(一時の作業ツリーとテストの命令のプロセスの組)を走らせる。
    Python は SIGTERM で finally を走らせずに止まるため。2つ目からの合図は片付けの間は受けない。主のスレッドの外では何もしない。"""
    sigs = [s for s in (getattr(signal, "SIGTERM", None), getattr(signal, "SIGHUP", None)) if s is not None]

    def on_signal(signum, frame):
        for s in sigs:
            signal.signal(s, signal.SIG_IGN)
        raise Terminated(signum)

    try:
        old = {s: signal.signal(s, on_signal) for s in sigs}
    except ValueError:  # 主のスレッドの外
        old = None
    try:
        yield
    finally:
        for s, h in (old or {}).items():
            signal.signal(s, h)


def drop_worktree(root, tmp, wt, added):
    """一時の作業ツリーを外し、一時のフォルダを消し、作業ツリーの登録を掃除する(足しかけで止まった登録も)。"""
    if added:
        git(root, "worktree", "remove", "--force", wt, check=False)
    if tmp:
        shutil.rmtree(tmp, ignore_errors=True)
        git(root, "worktree", "prune", check=False)


def kill_group(p):
    try:
        if hasattr(os, "killpg"):
            os.killpg(p.pid, signal.SIGKILL)
        else:
            p.kill()
    except OSError:
        pass


def run_test_command(cmd, cwd, junit, timeout, where):
    """テストを走らせる命令を、打ち切られても子が残らない形(新しいプロセスの組)で走らせ、JUnit XML を書かせる。"""
    kw = {"start_new_session": True} if os.name == "posix" else {}
    try:
        quoted = shlex.quote(junit) if os.name == "posix" else subprocess.list2cmdline([junit])
        p = subprocess.Popen(cmd.replace("{junit}", quoted), shell=True, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, **kw)
    except OSError as e:
        raise UsageError("テストの命令を起動できない: %s" % clip(e)) from e
    try:
        out, _ = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        kill_group(p)
        p.communicate()
        raise UsageError("%s でのテストの実行を %d 秒で打ち切った" % (where, timeout)) from None
    except BaseException:
        kill_group(p)
        p.communicate()
        raise
    if not os.path.isfile(junit):
        tail = out.decode("utf-8", "replace").strip()[-300:]
        raise UsageError("%s で走らせた命令が JUnit XML を {junit} に書かなかった(終了コード %d): %s" % (where, p.returncode, clip(tail, 300)))


def outcomes_of(t, outcomes):
    """テスト t の結果の並び(無ければ None)。Rust は名前の最後の段で引き、道の一致を優先する(rust_outcomes)。"""
    if t.lang == "rust":
        return rust_outcomes(t, outcomes)
    got = None
    for k in sorted(case_keys(t)):
        got = outcomes.get(k) or got
    return got


def cmd_f2p_run(args, root, cfg):
    if not gate_part_on(root, cfg, args.base, "f2p"):
        return part_switched_off(args, "f2p")
    if "{junit}" not in args.cmd:
        raise UsageError("--cmd に {junit}(JUnit XML を書く先)を入れる")
    res = analyze(root, cfg)
    if not res["reqs"]:
        return emit(args, res["findings"], "trace-gate f2p-run: 要求が無い")
    top = os.path.realpath(git(root, "rev-parse", "--show-toplevel").strip())
    git(root, "rev-parse", "--verify", "--quiet", args.base + "^{commit}")
    wanted, how = select_reqs(args, root, cfg, res["reqs"])
    if not wanted:  # 数える要求が無ければ、テストを走らせない
        return nothing_to_count(args, "f2p-run", how, {"base": args.base})
    prefix = os.path.relpath(os.path.realpath(root), top)
    tmp, wt, added = None, None, False
    with terminate_to_cleanup():  # 打ち切られても一時の作業ツリーとテストの命令を残さない
        try:
            tmp = tempfile.mkdtemp(prefix="trace-gate-f2p-")
            wt = os.path.join(tmp, "base")
            git(root, "worktree", "add", "--detach", "--quiet", wt, args.base)
            added = True
            wroot = os.path.normpath(os.path.join(wt, prefix))
            copied = select(walk_files(root), cfg["tests"], cfg["exclude"])
            for rel in copied:  # head のテストのファイルだけを、実装前の base に写す
                dst = os.path.join(wroot, *rel.split("/"))
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                shutil.copyfile(os.path.join(root, rel), dst)
            before, after = os.path.join(tmp, "before.xml"), os.path.join(tmp, "after.xml")
            run_test_command(args.cmd, wroot, before, args.timeout, "base(%s)" % clip(args.base, 40))
            run_test_command(args.cmd, root, after, args.timeout, "head(作業ツリー)")
            ob, oa = read_junit(before), read_junit(after)
        finally:
            drop_worktree(root, tmp, wt, added)
    findings, report = [], {}
    for name in wanted:
        r = res["reqs"][name]
        tb, ta, verified = Counter(), Counter(), 0
        for t in res["tests"]:
            if not any(ln["name"] == name and ln.get("status") == "covers" for ln in t.links):
                continue
            b, a = outcomes_of(t, ob), outcomes_of(t, oa)
            tb.update(b or ["missing"])
            ta.update(a or ["missing"])
            verified += bool(b and "assert-fail" in b and a and all(x == "passed" for x in a))
        report[r.id] = {"before": dict(sorted(tb.items())), "after": dict(sorted(ta.items())), "verified": verified}
        if not tb["assert-fail"]:
            detail = "、".join("%s %d" % kv for kv in sorted(tb.items())) or "印のテストなし"
            findings.append(Finding("f2p-not-failing", r.path, r.line, "要求 %s に、base で断言の失敗で落ちたテストが無い(%s)" % (r.id, detail), key=name))
        elif not verified:
            detail = "、".join("%s %d" % kv for kv in sorted(ta.items()))
            findings.append(
                Finding(
                    "f2p-not-passing", r.path, r.line, "要求 %s のテストは base で断言の失敗で落ちたが、head で通らない(head: %s)" % (r.id, detail), key=name
                )
            )
    summary = (
        "trace-gate f2p-run: 確かめた %d / %d(base で断言の失敗で落ち、head で通ったテストのある要求。base に写したテストのファイル %d)。数える要求(%s): %s。%s"
        % (len(wanted) - len(findings), len(wanted), len(copied), how, ", ".join(wanted), counts_text(findings))
    )
    return emit(args, findings, summary, {"requirements": report, "base": args.base, "selected": wanted})


TEST_DIR_NAMES = {"tests", "test", "__tests__", "testdata"}  # 補助のモジュールとデータを含めて、テストの側と見るフォルダの名前


def test_side(path, cfg):
    """テストの側のファイルか: 設定の tests に当たるか、道に TEST_DIR_NAMES の段を持つ(補助のモジュールとデータ)。"""
    return (matches(path, cfg["tests"]) or bool(TEST_DIR_NAMES & set(path.split("/")[:-1]))) and not matches(path, cfg["exclude"])


def cmd_reverse_run(args, root, cfg):
    """base のテスト(補助とデータを含む)を head のコードで走らせ、断言の失敗だけを人への知らせにする(門にしない)。"""
    if not gate_part_on(root, cfg, args.base, "f2p"):
        return part_switched_off(args, "f2p")
    if "{junit}" not in args.cmd:
        raise UsageError("--cmd に {junit}(JUnit XML を書く先)を入れる")
    git(root, "rev-parse", "--verify", "--quiet", args.base + "^{commit}")
    changed = sorted({p for st, old, new in git_changes(root, args.base, None) for p in (old, new) if test_side(p, cfg)})
    extra = {"base": args.base, "changed": changed, "excluded": [], "not_assertion": 0}
    if not changed:
        return emit(
            args, [], "trace-gate reverse-run: テスト(補助とデータを含む)が base(%s)から変わっていないので、走らせなかった" % clip(args.base, 40), extra
        )
    top = os.path.realpath(git(root, "rev-parse", "--show-toplevel").strip())
    prefix = os.path.relpath(os.path.realpath(root), top)
    head = (git(root, "stash", "create", check=False) or "").strip() or "HEAD"  # 作業ツリーの追跡しているファイルの今の中身(変えない)
    tmp, wt, added = None, None, False
    with terminate_to_cleanup():  # 打ち切られても一時の作業ツリーとテストの命令を残さない
        try:
            tmp = tempfile.mkdtemp(prefix="trace-gate-reverse-")
            wt = os.path.join(tmp, "head")
            git(root, "worktree", "add", "--detach", "--quiet", wt, head)
            added = True
            wroot = os.path.normpath(os.path.join(wt, prefix))
            removed, copied = set(), set()
            for rel in walk_files(wroot):  # head のテストの側を捨て、base のテストの側を写す
                if test_side(rel, cfg):
                    os.remove(os.path.join(wroot, *rel.split("/")))
                    removed.add(rel)
            for rel in git_files_at(root, args.base):
                if test_side(rel, cfg):
                    dst = os.path.join(wroot, *rel.split("/"))
                    os.makedirs(os.path.dirname(dst), exist_ok=True)
                    with open(dst, "wb") as f:  # データのファイルはバイト列のまま写す
                        f.write(subprocess.run(["git", "-C", root, "show", "%s:./%s" % (args.base, rel)], capture_output=True, check=True).stdout)
                    copied.add(rel)
            for rel in sorted(removed - copied):  # Rust はコードの側が #[path] や mod でテストのファイルを指すので、消すとビルドできない。空のファイルで残す
                if rel.endswith(".rs"):
                    open(os.path.join(wroot, *rel.split("/")), "wb").close()
            junit = os.path.join(tmp, "reverse.xml")
            run_test_command(args.cmd, wroot, junit, args.timeout, "head(base のテスト)")
            doc = junit_doc(junit)[0]
            res = analyze(wroot, cfg)  # head の要求と base のテストの印(版の上がった要求の印は outdated になる)
        finally:
            drop_worktree(root, tmp, wt, added)
    by_key = {}
    for t in res["tests"]:
        for k in case_keys(t) | ({(RUST_KEY, t.name)} if t.lang == "rust" else set()):
            by_key.setdefault(k, t)
    findings = []
    for case in doc.iter("testcase"):
        o = junit_outcome(case)
        if o in ("exception", "error"):
            extra["not_assertion"] += 1
        if o != "assert-fail":
            continue
        name = re.sub(r"\[.*\]$", "", case.get("name") or "")
        t = by_key.get((case.get("classname") or "", name)) or by_key.get(("", name)) or by_key.get((RUST_KEY, name.rsplit("::", 1)[-1]))
        if t is not None and t.links and all(ln.get("status") != "covers" for ln in t.links):
            extra["excluded"].append(t.node)  # 版が上がったか消えた要求の印だけのテスト(意図した変更)
            continue
        node = t.node if t is not None else "%s::%s" % (clip(case.get("classname") or "", 80), clip(name, 80))
        findings.append(
            Finding(
                "base-test-failed",
                t.path if t is not None else None,
                t.start if t is not None else None,
                "base のテスト %s が、head のコードで断言の失敗で落ちた" % clip(node, 120),
                key=node,
                severity="warning",
            )
        )
    extra["excluded"] = sorted(set(extra["excluded"]))
    findings.sort(key=Finding.sort_key)
    summary = (
        "trace-gate reverse-run: base(%s)のテストを head のコードで走らせ、断言の失敗 %d 本(版の上がった要求の印だけのテストを除いた %d 本、断言でない落ち %d 本は数えない)。変わったテストの側のファイル %d。%s"
        % (
            clip(args.base, 40),
            len(findings),
            len(extra["excluded"]),
            extra["not_assertion"],
            len(changed),
            counts_text(findings),
        )
    )
    return emit(args, findings, summary, extra)


# ---------------------------------------------------------------- 1行の形から移す(convert)。1行の形はここでだけ読む

NEEDS_RE = re.compile(r"\s*(?<!`)確かめ[:：]\s*((?:%s)(?:\s*[・,、]\s*(?:%s))*)\s*$" % (KIND_ALT, KIND_ALT))
TREAT_RE = re.compile(r"\s*(?<!`)扱い[:：]\s*(踏襲|変更|追加|廃止)\s*(?:[(（]\s*([^()（）\n]*?)\s*[)）])?\s*$")
SOURCE_RE = re.compile(r"\s*(?<!`)出典[:：]\s*((?:(?!確かめ[:：]|扱い[:：]|出典[:：]).)*?)\s*$")
TREAT_HINT = re.compile(r"(?<!`)扱い[:：]")  # `扱い:` のようにコードの字面で書いた語は欄と読まない
MOVE_NOTE = "移す変更は単独の PR にし、人が中身を見て通す(weaken は1行の形の base の要求を読めず、要求の消しと足しが人に回る。一度きり)"
TITLE_NOTE = "移した題は1行の文がそのまま入る。題は短く直し、細部は受け入れ基準か欄へ移す(要求の題を直すと版を上げる。100 字を超える題 %d 件)"


def split_tail(accept):
    """1行の形の受け入れ基準の終わりの「確かめ:」「扱い:」「出典:」を、どの順でも外す。(受け入れ基準, 欄の書いたままの辞書, 誤り か None)。"""
    raw, seen = {}, set()
    while True:
        for key, rx in (("確かめ", NEEDS_RE), ("扱い", TREAT_RE), ("出典", SOURCE_RE)):
            m = rx.search(accept)
            if not m or key in seen:
                continue
            seen.add(key)
            raw[key] = m.group(1).strip() if key != "扱い" else re.sub(r"^\s*扱い[:：]\s*", "", m.group(0)).strip()
            accept = accept[: m.start()].rstrip()
            break
        else:
            break
    if TREAT_HINT.search(accept):
        return accept, raw, "扱い: が受け入れ基準の終わりに1つだけになっていない"
    t = TREAT_VALUE.match(raw.get("扱い", ""))
    if t and t.group(1) == "廃止" and not (t.group(2) or "").strip():
        return accept, raw, "扱い: 廃止 に (理由) が無い"
    if "出典" in raw and not raw["出典"]:
        return accept, raw, "出典: の中身が無い"
    if not accept.strip():
        return accept, raw, "受け入れ基準の中身が無い"
    return accept, raw, None


WHY_SHAPE = re.compile(r"(?:ため|ので|から)。?$")  # なぜ に取り出す括弧の中の終わり(理由の形)


def split_title(text):
    """1行の形の要求の文を (題, なぜ か None) に分ける。文の最後の「(…)」の中が理由の形(ため・ので・から で終わる)なら、
    題はその括弧を外した部分、なぜはその中身。ほかの括弧(要求の名前への参照や規則)は題に残す。
    分けた形から作り直した文(req_text)が元の文と同じになるときだけ返す(指紋を変えないため)。合わなければ None。"""
    s = text.strip()
    body = s[:-1].rstrip() if s.endswith("。") else s
    cands, close = [], body[-1:]
    if close in (")", "）"):
        opener, depth = "(" if close == ")" else "（", 0
        for k in range(len(body) - 1, -1, -1):
            depth += 1 if body[k] == close else -1 if body[k] == opener else 0
            if depth == 0:
                if WHY_SHAPE.search(body[k + 1 : -1].strip()):
                    cands.append((body[:k].strip(), body[k + 1 : -1].strip()))
                break
    cands.append((body, None))
    for title, why in cands:
        if title and (why is None or why) and squash(req_text(title, why)) == squash(s):
            return title, why
    return None


def legacy_line(line):
    """1行の形の要求の行を読む。(中身の辞書 か None, 誤り か None)。"""
    m = REQ_LINE.match(line)
    if not m:
        return None, "1行の形が崩れていて移せない(名前・版・優先度・受け入れ: を確かめる)"
    accept, raw, err = split_tail(m.group("accept"))
    if err:
        return None, err
    parts = split_title(m.group("text"))
    if parts is None:
        return None, "文を題となぜに分けると指紋が変わる(文の終わりを「。」か「(なぜ)。」にして版を上げてから移すか、手で移す)"
    t = TREAT_VALUE.match(raw.get("扱い", ""))
    return {
        "name": m.group("name"),
        "version": int(m.group("ver")),
        "priority": m.group("prio"),
        "origin": m.group("origin"),
        "text": m.group("text"),
        "title": parts[0],
        "why": parts[1],
        "accept": accept,
        "raw": raw,
        "needs": frozenset(re.findall(KIND_ALT, raw["確かめ"])) if "確かめ" in raw else None,
        "treatment": t.group(1) if t else None,
        "reason": ((t.group(2) or "").strip() or None) if t else None,
    }, None


def req_key(name, version, priority, fp, needs, treatment, reason, sources):
    """移す前と後で同じでなければならない中身。"""
    return (name, version, priority, fp, tuple(sorted(needs)) if needs is not None else None, treatment, reason, sources)


OPEN_LABEL_RE = re.compile(r"(?<!`)(決める人|期限|仮|根拠)[:：]")
OPEN_BRACKETS = {"(": ")", "（": "）", "「": "」", "『": "』", "【": "】", "[": "]"}


def outside_brackets(text):
    """位置ごとに、括弧と `…` の外かどうか(括弧の中のラベルを欄の区切りと読まないため)。閉じの無い括弧の後ろは全部中。"""
    out, stack, code = [], [], False
    for c in text:
        out.append(not stack and not code)
        if c == "`":
            code = not code
        elif code:
            continue
        elif c in OPEN_BRACKETS:
            stack.append(OPEN_BRACKETS[c])
        elif stack and c == stack[-1]:
            stack.pop()
    return out


def legacy_open(line):
    """1行の形の未確定事項の行を読む。(中身の辞書 か None, 誤り か None)。
    題は最初のラベルの前の文、欄はラベルごとの文(終わりの「。」は外す)。括弧と `…` の中のラベルは欄に分けない。同じラベルが2つあれば移せない。"""
    m = OPEN_LINE.match(line)
    if not m:
        return None, "1行の形が崩れていて移せない(名前と文を確かめる)"
    text = m.group("text")
    outside = outside_brackets(text)
    hits = [h for h in OPEN_LABEL_RE.finditer(text) if outside[h.start()]]
    labels = [h.group(1) for h in hits]
    if len(labels) != len(set(labels)):
        return None, "同じラベル(%s)が2つあって移せない" % "・".join(sorted({x for x in labels if labels.count(x) > 1}))
    if "決める人" not in labels:
        return None, "決める人: が無いので移せない(先に 決める人: を書く)"
    title = (text[: hits[0].start()] if hits else text).strip().rstrip("。").strip()
    if not title:
        return None, "決めていないことの文(最初のラベルの前)が無いので移せない"
    fields = {}
    for k, h in enumerate(hits):
        end = hits[k + 1].start() if k + 1 < len(hits) else len(text)
        value = text[h.end() : end].strip().rstrip("。").strip()
        if not value:
            return None, "%s: の中身が無いので移せない" % h.group(1)
        fields[h.group(1)] = value
    due = DUE_RE.search(text)
    return {
        "name": m.group("name"),
        "origin": m.group("origin"),
        "title": title,
        "fields": fields,
        "due": due.group(1) if due else None,
        "ups": up_refs(text),
    }, None


OPEN_FIELD_KEYS = (("決める人", "decider"), ("期限", "due_text"), ("仮", "provisional"), ("根拠", "basis"), ("区分", "kind"))
OPEN_KEY_PARTS = ("名前", "題", "欄の文", "期限の日付", "up~ の名前")


def open_key(name, title, fields, due, ups):
    """未確定事項で、移す前と後で同じでなければならない中身(OPEN_KEY_PARTS の順: 名前・題・欄の文・期限の日付・up~ の名前)。"""
    return (name, title, tuple(sorted(fields.items())), due, tuple(sorted(ups)))


def parsed_open_key(o):
    """区切りの形から読んだ未確定事項の open_key。"""
    fields = {label: o[k] for label, k in OPEN_FIELD_KEYS if o.get(k)}
    return open_key(o["id"][4:], o["text"], fields, o["due"], up_refs(open_text(o)))


LABEL_PREFIX = re.compile(r"^(?P<bullet>\s*[-*+][ \t]+)(?P<label>[^\s:：`]+)[:：][ \t]*")


def english_label(line, allowed):
    """区切りの中の欄の行の日本語の名前を英語の名前に替えた行。替えなければ None。allowed はその区切りで許す日本語の名前。"""
    m = LABEL_PREFIX.match(line)
    if not m or m.group("label") not in allowed:
        return None
    rest = line[m.end() :]
    return m.group("bullet") + LABEL_EN[m.group("label")] + (": " + rest if rest else ":")


def then_only_parts(bullet, value):
    """1行の文の受け入れ基準を、Given と When を空にした字下げの形の行に替える。bullet は `- ` までの行の頭。"""
    head = bullet if bullet else "- "
    sub = " " * (len(head.expandtabs(4)) - len(head.lstrip().expandtabs(4)) + 2) + "- "
    return [head + "Acceptance:", sub + "Given:", sub + "When:", sub + "Then: " + value]


def convert_text(text, path, compact=False, parts=False):
    """1行の形の要求と未確定事項の行を区切りの形に写し、区切りの中の日本語の名前の欄を英語の名前に替える。
    (新しい文, 数 (写した要求, 写した未確定事項, 名前を替えた欄の行), 誤り [(行, 名前, 文)])。
    写した後に読み直し、どの要求の名前・版・優先度・指紋・欄も、どの未確定事項の名前・題・欄の文・期限の日付・up~ も写す前と同じかを確かめる。
    compact は区切りの間に空の行を足さない。parts(設定の require_accept_parts)なら、1行の文の受け入れ基準を、
    Given と When を空にした字下げの形の Then に移す(Then だけの受け入れの文は1行の文と同じなので、指紋は変わらない)。"""
    lines = text.split("\n")
    out, errors, expected, open_expected, origin_line = [], [], [], [], {}
    fence, group, pending, count, open_count, renamed = False, False, False, 0, 0, 0
    inside = None  # 今いる区切りの許す日本語の名前(要求は LABELS、未確定事項は OPEN_LABELS)。区切りの外は None
    for idx, line in enumerate(lines):
        no = idx + 1
        if FENCE_RE.match(line):
            fence = not fence
            inside = None
        elif not fence and REQ_HINT.match(line):
            d, err = legacy_line(line)
            if err:
                m = re.match(r"^\s*[-*]\s+`req~([^`~]*)", line)
                errors.append((no, m.group(1) if m else "", err))
                out.append(line)
                continue
            name = d["name"]
            origin_line[name] = no
            block = [
                "%s %s" % ("####" if group else "###", d["title"]),
                "`req~%s~%d` %s%s" % (name, d["version"], d["priority"], " " + d["origin"] if d["origin"] else ""),
            ]
            if d["why"]:
                block.append("- %s: %s" % (LABEL_EN["なぜ"], d["why"]))
            if parts:
                block.extend(then_only_parts("", d["accept"].strip()))
            else:
                block.append("- %s: %s" % (LABEL_EN["受け入れ"], d["accept"].strip()))
            block.extend("- %s: %s" % (LABEL_EN[label], d["raw"][label]) for label in ("確かめ", "扱い", "出典") if label in d["raw"])
            if not compact and out and out[-1].strip():
                out.append("")
            out.extend(block)
            pending, count, inside = not compact, count + 1, None
            fp = fingerprint(d["text"], [d["accept"]], d["treatment"] == "廃止")
            expected.append(req_key(name, d["version"], d["priority"], fp, d["needs"], d["treatment"], d["reason"], d["raw"].get("出典")))
            continue
        elif not fence and OPEN_HINT.match(line):
            d, err = legacy_open(line)
            if err:
                m = re.match(r"^\s*[-*]\s+`tbd~([^`]*)", line)
                errors.append((no, "tbd~" + (m.group(1) if m else ""), err))
                out.append(line)
                continue
            origin_line["tbd~" + d["name"]] = no
            block = ["%s %s" % ("####" if group else "###", d["title"]), "`tbd~%s`%s" % (d["name"], " " + d["origin"] if d["origin"] else "")]
            block.extend("- %s: %s" % (LABEL_EN[label], d["fields"][label]) for label in OPEN_LABELS if label in d["fields"])
            if not compact and out and out[-1].strip():
                out.append("")
            out.extend(block)
            pending, open_count, inside = not compact, open_count + 1, None
            open_expected.append(open_key(d["name"], d["title"], d["fields"], d["due"], d["ups"]))
            continue
        if not fence and inside is not None:
            if block_boundary(line) or (line.strip() and not BULLET.match(line)):
                inside = None
            else:
                en = english_label(line, inside)
                if en is not None:
                    line, renamed = en, renamed + 1
                if parts and inside is not OPEN_LABELS:  # 要求の区切りの中の1行の文の受け入れ基準
                    m = LABEL_PREFIX.match(line)
                    value = line[m.end() :].strip() if m else ""
                    if m and canon_label(m.group("label")) == "受け入れ" and value:
                        line, renamed = "\n".join(then_only_parts(m.group("bullet"), value)), renamed + 1
        if not fence and ID_HINT.match(line):
            inside = LABELS + ACCEPT_PARTS  # 受け入れの字下げの 前提・もし・ならば も替える
        elif not fence and OPEN_ID_HINT.match(line):
            inside = OPEN_LABELS
        if pending and line.strip():
            out.append("")
        pending = False
        out.append(line)
        if not fence and re.match(r"^##(?!#)\s", line):
            group = False
        elif not fence and re.match(r"^###(?!#)\s", line):
            rest = [x for x in lines[idx + 1 :] if x.strip()]
            group = not (rest and (ID_HINT.match(rest[0]) or OPEN_ID_HINT.match(rest[0])))  # 題の見出し(次が ID の行)でなく、まとまりの見出し
    new = "\n".join(out)
    if errors or not (count or open_count or renamed):
        return new, (count, open_count, renamed), errors
    before, after = [], []
    old_blocks, old_opens = parse_requirements(text, path, before)
    new_reqs, new_opens = parse_requirements(new, path, after)
    for r in old_blocks:
        expected.append(req_key(r.name, r.version, r.priority, r.fingerprint, r.needs, r.treatment, r.reason, r.sources))
    got = [req_key(r.name, r.version, r.priority, r.fingerprint, r.needs, r.treatment, r.reason, r.sources) for r in new_reqs]
    for k in sorted(set(expected) ^ set(got), key=lambda k: (origin_line.get(k[0], 0), k[0])):
        errors.append((origin_line.get(k[0], 0), k[0], "移すと名前・版・優先度・指紋・欄のどれかが変わる"))
    open_expected.extend(parsed_open_key(o) for o in old_opens)
    want, got = set(open_expected), {parsed_open_key(o) for o in new_opens}
    for name in sorted({k[0] for k in want ^ got}, key=lambda n: (origin_line.get("tbd~" + n, 0), n)):
        a, b = [k for k in want - got if k[0] == name], [k for k in got - want if k[0] == name]
        parts = [OPEN_KEY_PARTS[i] for i in range(1, len(OPEN_KEY_PARTS)) if a[0][i] != b[0][i]] if len(a) == len(b) == 1 else []
        what = "・".join(parts) if parts else "名前・題・欄の文・期限の日付・up~ の名前のどれか"
        errors.append((origin_line.get("tbd~" + name, 0), "tbd~" + name, "移すと変わる所: %s" % what))
    if sum(f.rule == "req-malformed" for f in after) != sum(f.rule == "req-malformed" for f in before) - count:
        errors.append((0, "", "移した後に読めない区切りがある(移す前の要求の行のまわりを確かめる)"))
    if sum(f.rule == "open-malformed" for f in after) != sum(f.rule == "open-malformed" for f in before) - open_count:
        errors.append((0, "", "移した後に読めない未確定事項の区切りがある(移す前の行のまわりを確かめる)"))
    return new, (count, open_count, renamed), sorted(set(errors)) if errors else errors


def cmd_convert(args, root, cfg):
    pkgs = package_dirs(root, cfg)
    if pkgs:
        targets = [(head + p, sroot, p) for head, sroot, scfg in scopes(root, cfg, pkgs) for p in requirement_files(sroot, scfg)]
    else:
        targets = [(p, root, p) for p in requirement_files(root, cfg)]
    lines, changes, errors, total, open_total, label_total = [], [], [], 0, 0, 0
    for shown, sroot, p in targets:
        text = read_text(sroot, p)
        new, (count, open_count, renamed), errs = convert_text(text, shown, parts=bool(cfg.get("require_accept_parts")))
        errors.extend((shown, no, name, msg) for no, name, msg in errs)
        if (count or open_count or renamed) and not errs:
            changes.append((shown, os.path.join(sroot, p), text, new, count, open_count, renamed))
            total += count
            open_total += open_count
            label_total += renamed
    if errors:
        for shown, no, name, msg in errors:
            shown_name = (" " + name if name.startswith("tbd~") else " req~" + name) if name else ""
            lines.append("移せない: %s%s%s: %s" % (shown, ":%d" % no if no else "", shown_name, msg))
        lines.append("trace-gate convert: 移せない行が %d 件あるので、何も書かなかった(直してからもう一度走らせる)" % len(errors))
        sys.stdout.write("".join(esc(x) + "\n" for x in lines))
        return 1
    if not changes:
        sys.stdout.write("trace-gate convert: 1行の形の要求と未確定事項の行も、日本語の名前の欄も無い(移すものは無い)\n")
        return 0
    for shown, path, text, new, count, open_count, renamed in changes:
        if args.write:
            with open(path, "rb") as f:
                crlf = b"\r\n" in f.read()
            with open(path, "w", encoding="utf-8", newline="\r\n" if crlf else "\n") as f:
                f.write(new)
            lines.append("書き換えた: %s(要求 %d 件・未確定事項 %d 件・欄の名前 %d 行)" % (shown, count, open_count, renamed))
        else:
            diff = difflib.unified_diff(text.split("\n"), new.split("\n"), "a/" + shown, "b/" + shown, lineterm="")
            lines.extend(diff)
    lines.append(
        "trace-gate convert: %d ファイルの要求 %d 件・未確定事項 %d 件・日本語の名前の欄 %d 行を%s(要求は名前・版・優先度・指紋・欄、未確定事項は名前・題・欄の文・期限の日付・up~ が変わらないことを確かめた)"
        % (len(changes), total, open_total, label_total, "書き換えた" if args.write else "移せる。書くのは --write")
    )
    lines.append(MOVE_NOTE)
    added = (ln for _, _, text, new, _, _, _ in changes for ln in set(new.split("\n")) - set(text.split("\n")))
    lines.append(TITLE_NOTE % sum(1 for ln in added if REQ_HEAD.match(ln) and len(REQ_HEAD.match(ln).group("title")) > 100))
    sys.stdout.write("".join(esc(x) + "\n" for x in lines))
    return 0


# ---------------------------------------------------------------- 名前の付け替え(rename)

RENAMES_HEAD = "# 要求の名前の表\n\n付け替えた旧い名前と消した名前。行は付け替えた順に足し、旧い名前は使い回さない(check が name-reused で止める)。\n\n| 旧い名前 | 新しい名前 |\n|---|---|\n"


def comment_spans(path, text):
    """テストのファイルのコメントの [(始まりの位置, 終わりの位置)]。読めなければ None。"""
    lang = LANG_BY_EXT.get(os.path.splitext(path)[1])
    if lang is None:
        return None
    if lang != "python":
        _, comments = lex_c(text, lang)
        return [(pos, pos + len(c)) for pos, c in comments]
    starts = [0]
    for line in text.split("\n"):
        starts.append(starts[-1] + len(line) + 1)
    out = []
    try:
        for tok in tokenize.generate_tokens(io.StringIO(text).readline):
            if tok.type == tokenize.COMMENT:
                at = starts[tok.start[0] - 1] + tok.start[1]
                out.append((at, at + len(tok.string)))
    except (tokenize.TokenError, IndentationError, SyntaxError):
        return None
    return out


def rename_marks(path, text, old, new):
    """コメントの印の中の req~old~版 だけを req~new~版 に替えた文と、替えた数。読めなければ (None, 0)。"""
    spans = comment_spans(path, text)
    if spans is None:
        return None, 0
    target = re.compile(r"(?<![A-Za-z0-9._~-])req~" + re.escape(old) + r"~(?=[0-9])")
    count = [0]

    def in_mark(m):
        body, n = target.subn("req~" + new + "~", m.group(0))
        count[0] += n
        return body

    out = text
    for a, b in sorted(spans, reverse=True):
        out = out[:a] + MARK_RE.sub(in_mark, out[a:b]) + out[b:]
    return out, count[0]


def marks_of(path, text):
    """[(行, 種類, 名前, 版)](コメントの印。テストに付いていない印も含める)。"""
    lang = LANG_BY_EXT[os.path.splitext(path)[1]]
    result, _ = scan_file(path, text, lang)
    if result is None:
        return None
    out = []
    for no, txt, _ in result[1]:
        for line, kind, targets in parse_marks(no, txt)[0]:
            out.extend((line, kind, n, v) for n, v in targets)
    return sorted(out)


def req_shape(r):
    return (r.version, r.priority, r.title, r.fingerprint, r.needs, r.treatment, r.reason, r.sources)


def add_rename_row(text, old, new):
    """名前の表の文に「| `old` | `new` |」の行を足す(最後の表の行の後ろ。表の列の数に合わせて空の欄を足す)。"""
    if text is None:
        return RENAMES_HEAD + "| `%s` | `%s` |\n" % (old, new)
    lines = text.split("\n")
    rows = [i for i, ln in enumerate(lines) if ln.lstrip().startswith("|")]
    if not rows:
        return text.rstrip("\n") + "\n\n| 旧い名前 | 新しい名前 |\n|---|---|\n| `%s` | `%s` |\n" % (old, new)
    last = rows[-1]
    cols = max(2, len(lines[last].strip().strip("|").split("|")))
    row = "| `%s` | `%s` |" % (old, new) + " |" * (cols - 2)
    return "\n".join(lines[: last + 1] + [row] + lines[last + 1 :])


def cmd_rename(args, root, cfg):
    """要求の名前を付け替える。正本の ID の行・テストのコメントの印・名前の表だけを書き換え、版と指紋を変えない。"""
    if cfg.get("packages"):
        raise UsageError("モノレポでは、--root でパッケージのフォルダを指して付け替える")
    old, new = args.old, args.new
    for n in (old, new):
        if not re.fullmatch(NAME, n):
            raise UsageError("名前 %s は英小文字・数字・. - _ で書く" % clip(n, 60))
    refuse, notes = [], []
    found = []
    contents = {}
    for p in requirement_files(root, cfg):
        contents[p] = read_text(root, p)
        found.extend(r for r in parse_requirements(contents[p], p, [])[0])
    same = [r for r in found if r.name == old]
    if not same:
        refuse.append("旧い名前 %s の要求が今の正本に無い" % old)
    elif len(same) > 1:
        refuse.append("旧い名前 %s の要求が %d つある(曖昧)。どちらを付け替えるかと、どのテストの印を替えるかは人が決める" % (old, len(same)))
    if new == old:
        refuse.append("新しい名前が旧い名前と同じ")
    if any(r.name == new for r in found):
        refuse.append("新しい名前 %s の要求が今の正本にある" % new)
    table_rel = cfg.get("renames") or DEFAULT_RENAMES
    bad = []
    table = load_renames(root, cfg, bad)
    if bad:
        refuse.append("名前の表 %s に読めない行がある(%s)。先に check で直す" % (table_rel, ", ".join("%d 行目" % f.line for f in bad)))
    if new in table:
        refuse.append("新しい名前 %s は、%s名前(使い回さない)" % (new, renamed_note(table[new])))
    if old in table:
        refuse.append("旧い名前 %s は、既に%s名前(先に name-reused を直す)" % (old, renamed_note(table[old])))
    base = args.base or "HEAD"
    try:
        base_reqs = reqs_at(root, base, cfg)
        base_table = load_renames(root, cfg, [], base)
    except UsageError as e:
        if args.base:
            raise
        base_reqs, base_table = None, {}
        notes.append("比べる先 HEAD を読めないので、比べる先の正本は確かめなかった(%s)" % clip(e, 80))
    if base_reqs is not None:
        if new in base_reqs:
            refuse.append("新しい名前 %s の要求が比べる先(%s)の正本にある" % (new, clip(base, 40)))
        if new in base_table and new not in table:
            refuse.append("新しい名前 %s は、比べる先(%s)の%s名前" % (new, clip(base, 40), renamed_note(base_table[new])))
        if len(same) == 1 and old in base_reqs and req_shape(base_reqs[old])[:4] != req_shape(same[0])[:4]:
            refuse.append(
                "旧い名前 %s の文か版か優先度が、比べる先(%s)と違う。文の変更と付け替えは別のコミットにする(付け替えに紛れた文の変更を見落とさないため)"
                % (old, clip(base, 40))
            )
    changes = {}
    if not refuse:
        r = same[0]
        lines = contents[r.path].split("\n")
        mark = "`req~%s~" % old
        if lines[r.line - 1].count(mark) != 1:
            refuse.append("%s:%d の ID の行を読めない" % (r.path, r.line))
        else:
            lines[r.line - 1] = lines[r.line - 1].replace(mark, "`req~%s~" % new)
            changes[r.path] = (contents[r.path], "\n".join(lines))
    marks_changed, seen_tests = 0, {}
    if not refuse:
        tests = [x for x in select(walk_files(root), cfg["tests"], cfg["exclude"]) if os.path.splitext(x)[1] in LANG_BY_EXT]
        for path in tests:
            text = read_text(root, path)
            if ("req~%s~" % old) not in text:
                continue
            seen_tests[path] = text
            after, n = rename_marks(path, text, old, new)
            if after is None:
                refuse.append("テストのファイル %s を読めず、印を付け替えられない" % path)
                continue
            if n:
                before_marks, after_marks = marks_of(path, text), marks_of(path, after)
                want = [(ln, k, new if nm == old else nm, v) for ln, k, nm, v in before_marks or []]
                if before_marks is None or after_marks != want:
                    refuse.append("テストのファイル %s の印を確かめられないか、名前のほかにも変わってしまう" % path)
                    continue
                changes[path] = (text, after)
                marks_changed += n
    if not refuse:
        rows_before = read_text(root, table_rel) if os.path.isfile(os.path.join(root, table_rel)) else None
        rows_after = add_rename_row(rows_before, old, new)
        check_rows = []
        if parse_renames(rows_after, table_rel, check_rows).get(old, (None,))[0] != new or check_rows:
            refuse.append("名前の表 %s に、付け替えの行を読める形で足せない" % table_rel)
        changes[table_rel] = (rows_before or "", rows_after)
    if not refuse:
        # 確かめ: 付け替えた要求の版・優先度・題・指紋・欄が同じで、ほかの要求は何も変わらない
        after_reqs = []
        for p in contents:
            after_reqs.extend(parse_requirements(changes.get(p, (None, contents[p]))[1], p, [])[0])
        moved = [x for x in after_reqs if x.name == new]
        rest_before = sorted((x.name,) + req_shape(x)[:4] for x in found if x.name != old)
        rest_after = sorted((x.name,) + req_shape(x)[:4] for x in after_reqs if x.name != new)
        if len(moved) != 1 or req_shape(moved[0]) != req_shape(same[0]) or rest_before != rest_after or any(x.name == old for x in after_reqs):
            refuse.append("付け替えの後に、要求の版・優先度・題・指紋・欄のどれかが変わってしまう")
    if refuse:
        sys.stdout.write("".join("付け替えない: %s\n" % esc(x) for x in refuse))
        sys.stdout.write("trace-gate rename: 何も書かなかった(%s → %s)\n" % (esc(old), esc(new)))
        return 1
    left = []
    for p in sorted(set(contents) | set(seen_tests)):
        if p == table_rel:
            continue
        text = changes[p][1] if p in changes else contents.get(p, seen_tests.get(p, ""))
        for no, line in enumerate(text.split("\n"), 1):
            if re.search(r"(?<![A-Za-z0-9._~-])req~" + re.escape(old) + r"~[0-9]", line):
                left.append("%s:%d" % (p, no))
    out = []
    for p in sorted(changes):
        before, after = changes[p]
        if args.write:
            full = os.path.join(root, p)
            crlf = False
            if os.path.isfile(full):
                with open(full, "rb") as f:
                    crlf = b"\r\n" in f.read()
            os.makedirs(os.path.dirname(full) or ".", exist_ok=True)
            with open(full, "w", encoding="utf-8", newline="\r\n" if crlf else "\n") as f:
                f.write(after)
            out.append("書き換えた: %s" % p)
        else:
            out.extend(difflib.unified_diff(before.split("\n"), after.split("\n"), "a/" + p, "b/" + p, lineterm=""))
    out.extend("書き換えなかった参照(人が見る): %s" % x for x in left)
    out.extend(notes)
    out.append(
        "trace-gate rename: %s → %s(正本の ID の行 1・テストの印 %d・名前の表の行 1)を%s。版と指紋は変えていない。承認の記録・ADR・凍結した記録・要求の文の中の参照は書き換えない(承認は名前の表の対で引き継ぐ)"
        % (old, new, marks_changed, "書き換えた" if args.write else "書き換えられる。書くのは --write")
    )
    sys.stdout.write("".join(esc(x) + "\n" for x in out))
    return 0


# ---------------------------------------------------------------- PR の本文の下書き(pr-body)


def test_units(path, text):
    """{テストの修飾名: (ノード, 本体の行の文, 印の組, 印の要求の名前)}。読めなければ空。"""
    lang = LANG_BY_EXT.get(os.path.splitext(path)[1])
    result, _ = scan_file(path, text, lang) if lang else (None, None)
    if result is None:
        return {}
    tests, comments, blank_lines = result
    links = {t.node: [] for t in tests}
    for no, txt, _ in comments:
        marks, _ = parse_marks(no, txt)
        for line, kind, targets in marks:
            t = bind(tests, line, blank_lines)
            if t is not None:
                links[t.node].extend((kind, n, v) for n, v in targets)
    lines = text.split("\n")
    return {
        ((t.qual + "::") if t.qual else "") + t.name: (
            t.node,
            "\n".join(lines[t.start - 1 : t.end]),
            tuple(sorted(links[t.node])),
            [n for _, n, _ in links[t.node]],
        )
        for t in tests
    }


def changed_test_links(root, cfg, base, head, changes):
    """{要求の名前: [テストのノード]}。base から本体か印が変わったテスト(足したテストを含む)の印だけを、head の側(None なら作業ツリー)から読む。"""
    out = {}
    for st, old, new in changes:
        if st == "D" or not select([new], cfg["tests"], cfg["exclude"]):
            continue
        before = test_units(old, read_side(root, base, old)) if st != "A" else {}
        for qual, (node, body, marks, names) in test_units(new, read_side(root, head, new) or "").items():
            was = before.get(qual)
            if was is not None and (was[1], was[2]) == (body, marks):
                continue
            for name in names:
                out.setdefault(name, []).append(node)
    return out


def cmd_pr_body(args, root, cfg):
    """PR の本文の下書き。検査器の出力だけで埋まる節(照らした要求・要求を変えるか・仮決め・上流・人が見るもの)を埋め、
    何を変えたかと理由は人か AI が書く。題には触れない(接頭辞と言い回しはリポの決まり)。終了コードは 0。"""
    bcfg = base_config(root, args.base, cfg)
    changes = git_changes(root, args.base, args.head)
    changed = [new for st, _, new in changes if st != "D"]
    by_req = changed_test_links(root, cfg, args.base, args.head, changes)
    old_reqs, new_reqs = reqs_at(root, args.base, bcfg), reqs_at(root, args.head, cfg)
    added = sorted(set(new_reqs) - set(old_reqs))
    removed = sorted(set(old_reqs) - set(new_reqs))
    bumped = [(n, old_reqs[n].version, new_reqs[n].version) for n in sorted(set(old_reqs) & set(new_reqs)) if old_reqs[n].version != new_reqs[n].version]
    same_version_changed = [
        n for n in sorted(set(old_reqs) & set(new_reqs)) if old_reqs[n].version == new_reqs[n].version and old_reqs[n].fingerprint != new_reqs[n].fingerprint
    ]
    priority_changed = [
        (n, old_reqs[n].priority, new_reqs[n].priority) for n in sorted(set(old_reqs) & set(new_reqs)) if old_reqs[n].priority != new_reqs[n].priority
    ]

    new_opens = sorted(set(opens_at(root, args.head, cfg)) - set(opens_at(root, args.base, bcfg)))
    code_tbd = []
    for p in changed:
        if os.path.splitext(p)[1] not in DANGLING_EXTS or matches(p, cfg["tests"]):
            continue
        text = read_side(root, args.head, p) or ""
        for no, comment in code_comments(text, os.path.splitext(p)[1]):  # open-dangling と同じ字句の読み(文字列の中は見ない)
            for m in TBD_REF.finditer(comment):
                code_tbd.append("tbd~%s(%s:%d)" % (m.group(1).rstrip("."), p, no))
    weak = weaken_scope(args, root, cfg, bcfg)
    upstream = [f for f in weak if f.rule == "upstream-changed"]
    others = [f for f in weak if f.rule != "upstream-changed"]

    def where(f):
        return "%s%s" % (f.path, ":%d" % f.line if f.line else "")

    lines = [
        "## 何を変えたか・なぜ",
        "",
        "<!-- 何を、なぜ変えたか。外の資料なしで分かるように人か AI が書く(検査器の出力からは出ない) -->",
        "",
        "## 照らした要求",
        "",
    ]
    lines += [
        "- %s(変えたテスト: %s)" % (new_reqs[n].id if n in new_reqs else "req~%s" % n, "・".join(sorted(set(nodes)))) for n, nodes in sorted(by_req.items())
    ] or ["なし(印のあるテストに変更は無い)"]
    lines += ["", "## 要求を変えるか", ""]
    if bumped or added or removed or same_version_changed or priority_changed:
        lines.append("- [x] 変える")
        lines += ["  - 版を上げた: req~%s~%d(%d → %d)。理由: " % (n, nv, ov, nv) for n, ov, nv in bumped]
        lines += ["  - 足した: %s" % new_reqs[n].id for n in added]
        lines += ["  - 消した: %s" % old_reqs[n].id for n in removed]
        lines += ["  - 版を上げずに文か受け入れを変えた(check --base が止める): %s" % new_reqs[n].id for n in same_version_changed]
        lines += ["  - 優先度を変えた: %s(%s → %s)" % (new_reqs[n].id, op, np) for n, op, np in priority_changed]
    else:
        lines.append("- [x] 変えない(不具合の直し・中身の整理)")
    lines += ["", "## この PR の仮決め", ""]
    lines += ["- %s(新しく足した未確定事項。決める権限のある人が承認する)" % o for o in new_opens] + ["- %s(コードのコメント)" % x for x in code_tbd] or [
        "なし"
    ]
    lines += [
        "",
        "## 回帰テスト",
        "",
        "<!-- 不具合の直しなら、直す前に落ちる回帰テストの印と f2p-run --req の結果。そうでなければ「回帰テストなし: 理由」 -->",
        "",
        "## 上流の変更",
        "",
    ]
    lines += ["- %s %s" % (where(f), f.message) for f in upstream] or ["なし"]
    lines += ["", "## 人が見るもの", ""]
    lines += ["- %s %s: %s" % (f.rule, where(f), f.message) for f in others] or ["なし(weaken の指摘は無い)"]
    sys.stdout.write("".join(esc(x) + "\n" for x in lines))
    return 0


# ---------------------------------------------------------------- 入口


class Parser(argparse.ArgumentParser):
    def error(self, message):
        self.print_usage(sys.stderr)
        sys.stderr.write("%s: 引数の誤り: %s\n" % (TOOL_NAME, message))
        sys.exit(2)


def build_parser():
    p = Parser(prog="trace_gate.py", description="要求の正本とテストの印を突き合わせる検査器(%s %s)" % (TOOL_NAME, TOOL_VERSION))
    p.add_argument("--version", action="version", version="%s %s" % (TOOL_NAME, TOOL_VERSION))
    sub = p.add_subparsers(dest="command", parser_class=Parser)

    def common(sp, formats=True):
        sp.add_argument("--root", default=".", help="リポの根(既定は今のフォルダ)")
        sp.add_argument("--config", help="設定のファイル(既定は <root>/%s)" % CONFIG_NAME)
        if formats:
            sp.add_argument("--format", choices=("text", "json", "sarif", "github"), default="text")
            sp.add_argument("--max-per-rule", type=int, default=10, help="text で規則ごとに出す件数")

    sp = sub.add_parser("check", help="要求の書式とテストの印を検査する")
    common(sp)
    sp.add_argument("--base", help="比べる先の git の参照(版を上げずに変えた要求を見つける)")
    sp.add_argument("--strict-open", action="store_true", help="未確定事項が残っていたら止める(出荷の前)")
    sp.add_argument("--junit", action="append", help="走らせた結果の JUnit XML(何度でも)。印のあるテストが結果に無い・飛んだ・落ちたら止める")
    sp.add_argument("--base-junit", action="append", help="base で走らせた JUnit XML(何度でも)。--junit より集めた数が多いか飛んだ数が少なければ止める")
    sp = sub.add_parser("ledger", help="台帳(JSON)を作る")
    common(sp, formats=False)
    sp.add_argument("--base")
    sp.add_argument("--out", help="書き出す先(既定は標準出力)")
    sp.add_argument("--compare", help="前の台帳(JSON)。要求ごとに new・unchanged・updated・absent を足す")
    sp = sub.add_parser("weaken", help="base と head のテストを比べて弱めを報告する")
    common(sp)
    sp.add_argument("--base", required=True)
    sp.add_argument("--head", help="比べる後の git の参照(既定は作業ツリー)")
    sp.add_argument("--junit-base", action="append", help="base で走らせた JUnit XML(何度でも。--junit-head と一緒に)")
    sp.add_argument("--junit-head", action="append", help="head で走らせた JUnit XML(何度でも)。集めた数の減りと飛んだ数の増えを人に回す")
    sp = sub.add_parser("f2p", help="実装前に断言の失敗で落ちたかを数える")
    common(sp)
    sp.add_argument("--junit", required=True, help="実装前の状態で走らせた JUnit XML")
    sp.add_argument("--base", help="比べる先の git の参照。--req が無ければ、base に無い P0・base より版が上がった P0・P0 に上げた要求だけを数える")
    sp.add_argument("--req", help="数える要求の名前(, 区切り。既定は --base があれば新しく足したか版が上がったか P0 に上げた P0、無ければ全部の P0)")
    sp = sub.add_parser("approve", help="人が宣言したときだけ、選んだ要求の承認を承認の記録に追記する(端末で yes と打つ)")
    common(sp, formats=False)
    sp.add_argument("--by", required=True, help="承認する人の名前")
    sp.add_argument("--meaning", required=True, help="承認の意味(例: 顧客承認・リリースの承認)")
    sp.add_argument("--req", help="承認する要求の名前(, 区切り。既定は全部)")
    sp.set_defaults(format="text")
    sp = sub.add_parser("init", help="検査器の写し・設定・雛形・写しの錠をリポに置く(人が走らせる)")
    common(sp, formats=False)
    once = sp.add_mutually_exclusive_group()
    once.add_argument("--force", action="store_true", help="検査器の写しを配る物で上書きし、写しの錠を書き直す")
    once.add_argument("--relock", action="store_true", help="写しは変えず、写しの錠だけを今の中身で書き直す")
    sp.add_argument(
        "--hook", action="store_true", help="Claude Code の Stop のフックを .claude/settings.json に置く(既定で置く。部品 hook を切った設定では使えない)"
    )
    sp.add_argument(
        "--update-hooks",
        action="store_true",
        help="既にある .claude/settings.json の trace-gate のフックの項目だけを雛形に替え、写しの錠のフックの行を書き直す(--force でも替える)",
    )
    sp.add_argument("--prune", action="store_true", help="切った部品の置いた物のうち、写しの錠と中身が同じ物だけを消して錠から外す(直された物は残して挙げる)")
    sp.add_argument("--dry-run", action="store_true", help="--prune で、何も変えずに消す物と残す物を挙げる")
    sp = sub.add_parser("guards", help="守る置き場・必須の検査・部品の状態を出す(サービス側の雛形のプラグインが --format json で読む)")
    common(sp, formats=False)
    sp.add_argument("--format", choices=("text", "json"), default="text")
    sp = sub.add_parser("intents", help="印の付いたテストを要求ごとに並べ、名前・意図・断言の行を出す")
    common(sp, formats=False)
    sp.add_argument("--format", choices=("text", "json", "md", "csv", "files"), default="text")
    sp.add_argument("--req", help="出す要求の名前(, 区切り。既定は全部)")
    sp.add_argument("--base", help="比べる先の git の参照。基点から触った要求(足した・版が上がった・文か受け入れが変わった・印のテストが変わった)だけを出す")
    sp.add_argument("--kind", choices=("utest", "itest", "stest", "ctest"), help="印の種類(stest など)のテストだけを出す")
    sp = sub.add_parser("results", help="JUnit XML の実施結果を、要求とテストの組ごとの表(テスト成績書)にする")
    common(sp, formats=False)
    sp.add_argument("--format", choices=("md", "csv", "json"), default="md")
    sp.add_argument("--junit", action="append", help="名前=JUnit XML の道(何度でも。名前は環境や版。例: ci=out/junit.xml)")
    sp.add_argument("--run-id", help="実行の ID(CI の run の番号や URL)。成績書に書く")
    sp = sub.add_parser("convert", help="1行の形の要求と未確定事項を、見出しの区切りの形に移す(既定は差分だけ。--write で書く)")
    common(sp, formats=False)
    sp.add_argument("--write", action="store_true", help="差分を出す代わりに、要求の正本を書き換える")
    sp = sub.add_parser("rename", help="要求の名前を付け替える(正本の ID の行・テストの印・名前の表。既定は差分だけ。--write で書く)")
    common(sp, formats=False)
    sp.add_argument("old", help="旧い名前")
    sp.add_argument("new", help="新しい名前")
    sp.add_argument("--write", action="store_true", help="差分を出す代わりに書き換える")
    sp.add_argument("--base", help="新しい名前が無いことと旧い要求の文が同じことを確かめる比べる先(既定は HEAD)")
    sp = sub.add_parser("pr-body", help="PR の本文の下書きを出す(検査器の出力で埋まる節だけ。何を変えたかと理由は人が書く)")
    common(sp, formats=False)
    sp.add_argument("--base", required=True)
    sp.add_argument("--head", help="比べる後の git の参照(既定は作業ツリー)")
    sp = sub.add_parser("f2p-run", help="一時の作業ツリーで base に head のテストを写して走らせ、次に head で走らせて数える")
    common(sp)
    sp.add_argument("--base", required=True, help="実装前の git の参照")
    sp.add_argument("--cmd", required=True, help="テストを走らせて JUnit XML を {junit} に書く命令")
    sp.add_argument("--req", help="数える要求の名前(, 区切り。既定は base に無い P0・base より版が上がった P0・P0 に上げた要求)")
    sp.add_argument("--timeout", type=int, default=1800, help="1回の実行の打ち切りの秒数")
    sp = sub.add_parser("reverse-run", help="base のテスト(補助とデータを含む)を head のコードで走らせ、断言の失敗を知らせる(門にしない)")
    common(sp)
    sp.add_argument("--base", required=True, help="テストを取る git の参照")
    sp.add_argument("--cmd", required=True, help="テストを走らせて JUnit XML を {junit} に書く命令")
    sp.add_argument("--timeout", type=int, default=1800, help="実行の打ち切りの秒数")
    return p


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    if not args.command:
        parser.print_help(sys.stderr)
        return 2
    try:
        root = os.path.abspath(args.root)
        if not os.path.isdir(root):
            raise UsageError("--root のフォルダが無い: %s" % clip(args.root))
        cfg = load_config(root, args.config)
        return {
            "check": cmd_check,
            "ledger": cmd_ledger,
            "weaken": cmd_weaken,
            "f2p": cmd_f2p,
            "approve": cmd_approve,
            "init": cmd_init,
            "guards": cmd_guards,
            "intents": cmd_intents,
            "results": cmd_results,
            "f2p-run": cmd_f2p_run,
            "reverse-run": cmd_reverse_run,
            "convert": cmd_convert,
            "pr-body": cmd_pr_body,
            "rename": cmd_rename,
        }[args.command](args, root, cfg)
    except UsageError as e:
        sys.stderr.write("%s: %s\n" % (TOOL_NAME, e))
        return 2
    except Terminated as e:  # 打ち切り。片付けは finally で済んでいる
        sys.stderr.write("%s: 合図 %d で打ち切った(一時の作業ツリーとテストの命令は片付けた)\n" % (TOOL_NAME, e.signum))
        return 128 + e.signum
    except Exception as e:  # 道具の誤りは止まる側に倒す
        sys.stderr.write("%s: 内部の誤り: %s: %s\n" % (TOOL_NAME, type(e).__name__, clip(e, 200)))
        return 2


if __name__ == "__main__":
    sys.exit(main())
