---
type: Change
id: 01M799C8GE6ACT0Q43MTT9SQZX
title: 選んだ列の値を正規表現で探して置き換える(regex-replace)
status: done
size: full
created: 2026-11-13
updated: 2026-11-13
---

# 選んだ列の値を正規表現で探して置き換える(regex-replace)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定](../_decisions/2026-11-13-regex-replace.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人から、機能の改善と追加の依頼があり(会話 2026-11-13)、仕様で「次の段」に残っている候補を選択肢で見せたところ選ばれた4つのうち、「正規表現の置き換え」(CE-15)をこの記録で扱う。

- 何を: 表の今の列の値を、正規表現で探して置き換える操作を足す。置き換えはためる変更にし、保存の流れ(差分を見てから書く)に乗せる。
- なぜ: 接頭語を変える・表記をそろえるような同じ直しを何十行もするとき、1つずつ打ち直すしかない。表の外の道具でファイルを直に書くと、差分を見てから保存する流れを外れる。
- 終わりの条件(例。値は作ったもの):
  - 列 title が `v1 draft`・`v1 final`・`notes` の3行で、行を選ばずに「正規表現で置き換える」→ `v(\d)` → `version $1` → 2行が `version 1 draft`・`version 1 final` になり「未保存 2」。`u` 1回で両方戻る。
  - 探す文字に `(`(読めない正規表現)→ 何も変わらず理由が出る。数の列では始まらない。
- 関係する要件: CE-15(この変更で版1に)・CE-10(一括の設定)・NV-5(行の選択)・WB-9・WB-10・SR-24(操作の一覧)・SR-14(パレット)・WB-15(読むだけ)。

## 不明点と仮定

- 仮定: 行を選んでいなければ、今のビューに出ている全部の行が対象。理由: 列ぜんぶの表記をそろえる使い方がいちばん多く、置き換えはためるだけで、保存の前に差分を見られる。外れたら: 選んだ行だけにする。
- 仮定: 対象は値が文字のセルだけ(リスト・数・日付は変えない)。理由: 置き換えた文字がその型で読めなくなると、型と合わない値を黙って作る。外れたら: リストの要素の置き換えを提案にする。
- 仮定: 既定のキーは割り当てない(パレットと操作の一覧から)。理由: 空いたキーが少なく、たまに使う操作。外れたら: キーの表に足す。
- 仮定: 大文字小文字は正規表現の書き方(`(?i)`)に任せ、別の切り替えは作らない。外れたら: 聞く欄を足す。

## 設計

- 変える場所:
  - `src/ui/replace.rs`(新しいファイル。`impl App` の続き): `start_replace()`(列の型と読むだけを確かめ、続きの入力を開く)、`replace_pattern(query)`(正規表現を組み、読めなければ理由を出してやめ、読めたら置き換える文字を聞く)、`replace_with(col, pattern, with)`(対象の行を決め、行ごとに今の値(ためた値があればその値)が文字なら `Regex::replace_all` を当て、変わった行だけを `Changes::set_each` の1手でためる)。
  - `src/ui/native_io.rs`: 続きの入力 `Ask::ReplacePattern(列)`・`Ask::ReplaceWith { col, pattern }` と、その前置き・候補(無し)・決めたときの分岐。
  - `src/ui/keymap.rs`: `Action::RegexReplace`(名前 `regex_replace`)。`COMMANDS`(ためる変更を作るので、読むだけでは出さない)に足す。既定のキーなし。`src/ui/app.rs` の `apply` でつなぐ。`src/ui/menu.rs` の列の節に足す。
  - `locales/en.toml`・`locales/ja.toml`、`docs/keys.md`・`docs/keys.ja.md`、説明書(`docs/manual/<言語>/features.md`・`tasks.md`・`shots.md`)と場面の台本。
- 変えない場所: `Changes`(`set_each` をそのまま使う)、書き戻し、列の型の推定。
- 選んだ案: 依存にもうある `regex`(検索が使う)で組み、行ごとに `replace_all` を当てて、変わった値だけを `set_each` に渡す。読むだけ・リストのセルの断りは `set_each` の決まり(`Skip`)に任せ、理由をまとめて出す(CE-10 と同じ出し方)。
- 却下した案: セルの入力ボックスに `s/…/…/` の書き方を足す(入力の値と操作の区別がつかなくなる)。`sed` を呼ぶ(ファイルを直に書き、ためる変更にならない)。
- 名前と文言(試験が探す語): 動作の名前は `regex_replace`。パレットと操作の一覧の表示名は、日本語「正規表現で置き換える」・英語「Replace with a regular expression」。聞く前置きは、日本語「探す正規表現: 」「置き換える文字: 」・英語「Find (regular expression): 」「Replace with: 」。結果は、日本語「N 件を置き換えた」・英語「replaced N cell(s)」で始める。一致が無いときは、日本語「一致するセルは無い」・英語「no cell matches」。読めない正規表現は、日本語「正規表現として読めない」・英語「not a valid regular expression」で始める。テキストでない列は、日本語「テキストの列でだけ置き換えられる」・英語「only text columns can be replaced」。
- 要件の対応: CE-15 → `src/ui/replace.rs`(対象・置き換え・ためる)、`native_io.rs`(聞く順)、`keymap.rs`・`menu.rs`(入口)。
- 関わる束: `cargo test --bin mdgrid test_ce_15`・`cargo test --bin mdgrid test_keys_doc`・`cargo test --bin mdgrid test_action_menu` と `cargo test --test test_regex_replace_e2e`
- 全体: `cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`cargo test`・`python3 .decidespec/scripts/check.py .`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 置き換えの操作(入口・聞く順・ためる) | CE-15 | パレットから正規表現と置き換える文字を答えると、変わったセルだけが1手でたまる | src/ui/replace.rs, src/ui/native_io.rs, src/ui/keymap.rs, src/ui/app.rs, src/ui/menu.rs, src/ui/mod.rs, locales/en.toml, locales/ja.toml, docs/keys.md, docs/keys.ja.md | src/ui/test_regex_replace.rs・tests/test_regex_replace_e2e.rs の `test_ce_15_*` | 済 |
| 2 | 文書: できることの一覧・手引き・場面 | CE-15 | 説明書に1項と画面1枚 | docs/manual, docs/manual-scenarios.toml | 読んで判定 | 済 |

## 実装の気づき

- レビュー1回目の版: 59483dcfbe7c3fe4739f79ee2304cc6319eb8f85(作業ツリーには、別の記録 save-undo・board-view の実装も入っている。この記録の範囲はタスクの表のファイル)。
- パレットの表示名の欄は 24 桁の決め打ちだったので、英語の表示名(33 桁)が切れた。見えている候補に長い名前があるときだけ 34 桁まで広げる(`src/ui/help.rs`)。ゴールデン sr_14 の、切れていた日本語の候補が全部出る形に変わる。
- 読めないノートのセル(値が分からず読むだけ)は、一致するかを確かめられないので、飛ばした理由に数える。
- 確かめる回(別の新しい文脈): 指摘 1 は本当(パレットの入力が `halfwidth` を通る筋を、コードで追って確かめた)。直し: 探す正規表現と置き換える文字を聞いている間は、打った字を半角に直さない(1)。空の文字列のセルを外す(2)。候補の無い入力として「(合うものが無い)」を出さない(3)。当てる前に列を確かめ直す(4)。`${1}x` の書き方を説明書とキーの文書に書く(5)。表示名の欄は、動作の名前の欄を削らない幅まで(6)。直しの試験は `src/ui/test_replace_fixes.rs`(実装した本人が書いた。独立したテスト役ではない)。
- 文言の台帳(locales)は、言語の間でキーの並びを同じにする(`tests/test_locales.rs`)。

## レビューの指摘

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 重要 | src/ui/help.rs:509 | 続きの入力はパレットの入力を使うので、全角の英数字・記号と `、`・`・` が半角に直される(SR-17)。置き換える文字に `A・B` と打つと `A/B` がたまり、探す文字の全角の括弧は半角になって読めない正規表現になる。値そのものが黙って変わる。 | 直した |
| 2 | レビュー | 細部 | src/ui/replace.rs:74 | 空の文字列 `key: ""` のセルが、空に一致する正規表現(`^`・`x*`)で変わる。CE-15 は空のセルを変えてはならない。 | 直した |
| 3 | レビュー | 細部 | src/ui/native_io.rs:264 | 聞いている間、入力の下に「(合うものが無い)」が出て、正規表現がどのセルにも一致しないように読める。 | 直した |
| 4 | レビュー | 細部 | src/ui/replace.rs:56 | `replace_with` が列を確かめ直さない。聞いている間に列の型がテキストでなくなっても、文字のセルに当てる。 | 直した |
| 5 | レビュー | 細部 | src/ui/replace.rs:89 | 置き換える文字の `$1abc` は組の名前 `1abc` と読まれて空になる(regex の決まり)。`${1}abc` と書けば引けるが、説明書に注意が無い。 | 直した |
| 6 | レビュー | 細部 | src/ui/help.rs:422 | 表示名の欄を 34 桁に広げると、幅 40 の端末では動作の名前が 3 桁しか出ない。 | 直した |
| 7 | セキュリティ | 細部 | src/ui/replace.rs:43 | 上限ぎりぎりの大きい正規表現を多い行に当てると、その間は画面が止まり、途中でやめる道が無い(regex は線形時間で、大きさの上限は既定で効く。入力は利用者本人のものだけ)。 | 見送り |

## 照合

検証役(実装を見ていない新しい文脈。書込なし)が、実装の前(acf43ac に今の e2e のテストを重ねた worktree)と後(b92e7b0)で確かめた。証拠のスクリプト(`check.py --evidence`)は「前: 要件の手前で落ちた」(単体試験が、実装前には無い名前を使うのでコンパイルで落ちる)として gap を出したので、前で落ちることは e2e で確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-15 | checked | コード: src/ui/replace.rs:15 / テスト: tests/test_regex_replace_e2e.rs::test_ce_15_e2e_palette_replace_then_save_writes_files / 今: 通った(`cargo test --test test_regex_replace_e2e`・`cargo test --bin mdgrid test_ce_15`) / 前: 落ちた(e2e 2本とも。パレットに `regex_replace` の候補が出ずに時間切れ) |

既存のテストの削除・skip・弱体化: なし(既存のテストのファイルは変えていない。ゴールデン `tests/golden/sr_5.txt`・`sr_14.txt` は、足したキーとコマンドの分と、パレットの表示名の欄を広げた分が変わった)

セキュリティの回: 指摘 1 件(細部。重要なし)

全体の走り: 約 4 分(`cargo test --no-fail-fast`。最後の「実装:」の直前。前から落ちる `test_cliff_history_prefixes_all_covered` の1本を除いて通った。この1本は、この変更を外しても落ちる)。`cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`check.py` は通った

確かめた: 1 / 1
