---
type: Change
id: 01M472D8PNPPTDHEAYC52XQFJ1
title: .base の組み込みの集計を表の下に出す(summaries)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# .base の組み込みの集計を表の下に出す(summaries)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [summaries](../_decisions/2026-10-06-summaries.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-13)が列の集計が無いと指摘した。BV-14 を版1に上げた。

終わりの条件: BV-14 の例のとおり。英語と日本語で出る(SR-23)。関係する要件: BV-14・BV-5・BV-7・SR-23・SR-9(どの行も画面の幅以下)。

## 不明点と仮定

- 仮定: 集計の意味は Obsidian のヘルプのとおり。Average・Median・Stddev・Sum・Min・Max は数の値だけ、Earliest・Latest は日付と日時だけ、Range は数なら Max − Min、日付なら Latest − Earliest(期間)、Checked・Unchecked は真偽の true・false の数、Empty・Filled は空(null・空の文字・空のリスト・キーが無い)とそうでないものの数、Unique は空でない値の種類の数。数える値が無ければ空欄。Stddev は母集団の標準偏差(Obsidian と同じか確かめられなければ記録に書く)。
- 仮定: 小数は表のセルと同じ見せ方(長ければ切る)。集計の行は表の最後の行の下・下の帯の上に1行で、列の位置にそろえる(横に流すと一緒に流れる)。左のノートの欄には「集計」(英語は「Summary」)と出す。集計の無いビューでは行を出さない。
- 仮定: `--print` には出さない(表の行ではないため)。docs/obsidian-bases.md(英日)に書く。

## 設計

- src/base.rs: ビューの `summaries` を読む(今は無視して捨てている所)。組み込みの名前は列挙子に、ほかは未対応の理由に。
- 新しい src/summary.rs(核): (集計の名前, 値の並び)→ 結果の値 の純関数。
- src/ui: 表の下に1行を描く(view.rs の表の描き方・列の位置・横の送りを使う)。描く行の数は1行減る(table_height)。
- 文言は src/i18n.rs(集計の名前の見せ方と「集計」の見出し)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 集計 | BV-14, BV-7, SR-23 | BV-14 の例のとおりで、既存の試験と golden が通る | src/base.rs, src/summary.rs, src/lib.rs, src/test_summary_unit.rs, src/ui/view.rs, src/ui/grid.rs, src/ui/app.rs, src/ui/test_summaries.rs, src/ui/mod.rs, src/i18n.rs, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, examples/demo/Tasks.base, docs/manual-scenarios.toml, docs/assets/ | test_bv_14_*(src/test_summary_unit.rs, src/ui/test_summaries.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。試験と実装は同じ実装役が、試験を先に置いて落ちることを確かめてから書いた。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-14 | checked | コード: src/summary.rs の compute、src/base.rs の summaries の読み取り、src/ui/view.rs の集計の行 / テスト: src/test_summary_unit.rs::test_bv_14_*(11本)、src/ui/test_summaries.rs::test_bv_14_*(9本) / 今: 通った(全体 1215 passed) / 前: 落ちた(仮の todo!() で 11本、画面の 8本が落ちた。狭い端末の1本は行が無くて前から通る)。実物: 英語の見本の Open のビューの下に `Summary … Earliest 2026-10-05 … Sum 26` |
| BV-7 | checked | コード: src/base.rs の summaries の読み取り(式の集計は理由に) / テスト: src/ui/test_summaries.rs::test_bv_14_formula_summary_unsupported_view_opens / 今: 通った / 前: 落ちた(式の集計の理由が出ない) |
| SR-23 | checked | コード: src/i18n.rs の集計の名前 / テスト: src/ui/test_summaries.rs::test_bv_14_english_names / 今: 通った / 前: 落ちた(集計の行が無い) |

既存のテストの削除・skip・弱体化: なし(golden は変わらない)

確かめた: 3 / 3

README の画像(docs/assets/demo-*.svg)と demo.gif を撮り直した(見本の Open のビューに estimate の列と集計の行が増えた)。
