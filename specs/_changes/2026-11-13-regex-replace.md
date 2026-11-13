---
type: Change
id: 01M799C8GE6ACT0Q43MTT9SQZX
title: 選んだ列の値を正規表現で探して置き換える(regex-replace)
status: active
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
| 不明点 | 進行中 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定](../_decisions/2026-11-13-regex-replace.md) |
| 設計 | 進行中 | この記録の「設計」 |
| タスク | 進行中 | この記録の「タスク」 |
| 実装 | 進行中 | コミット |
| 照合 | 未 | この記録の「照合」 |

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
| 1 | 置き換えの操作(入口・聞く順・ためる) | CE-15 | パレットから正規表現と置き換える文字を答えると、変わったセルだけが1手でたまる | src/ui/replace.rs, src/ui/native_io.rs, src/ui/keymap.rs, src/ui/app.rs, src/ui/menu.rs, src/ui/mod.rs, locales/en.toml, locales/ja.toml, docs/keys.md, docs/keys.ja.md | src/ui/test_regex_replace.rs・tests/test_regex_replace_e2e.rs の `test_ce_15_*` | 未 |
| 2 | 文書: できることの一覧・手引き・場面 | CE-15 | 説明書に1項と画面1枚 | docs/manual, docs/manual-scenarios.toml | 読んで判定 | 未 |

## 実装の気づき

なし

## レビューの指摘

なし

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

確かめた: 0 / 0
