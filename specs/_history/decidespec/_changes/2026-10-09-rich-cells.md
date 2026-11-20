---
type: Change
id: 01M4E9ZTAK201SXYYKZP689741
title: 表のセルを値の型に合わせた部品で見せる(SR-35)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# 表のセルを値の型に合わせた部品で見せる(SR-35)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-rich-cells.md・2026-10-09-rich-cells-config.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が表をもう少し GUI のようにモダンにしたいと頼み、見本の画面から「セルを部品に」を選んだ(会話 2026-10-09)。SR-35。

## 不明点と仮定

- 仮定: 札の列(種類の少ない短い文字の列)は、全部の行から決める: 文字の列で、値のある行が2つ以上、違う値が 12 以下でくり返しがある(違う値が行の数より少ない)、どの値も 20 字以下で改行なし(初めは「行の数の半分以下」にしたが、行の少ない表で status が札にならなかったので緩めた)、リンクでない。外れたら: しきいを変える。
- 仮定: 札の色は値の文字から決まる8色(どの端末の地でも読めるよう、地と文字の色を両方決める)。
- 仮定: 選んでいるセルは札にせず、今の選びの見た目(SR-33)で文字を見せる(どこを選んでいるか分かるように)。
- 仮定: 検索(NV-1)は見せる文字で一致を探すので、真偽の列は `☑`・`☐` でなく値の文字(true・false)でも一致するようにする。

## 設計

- src/config.rs・src/config_items.rs・docs/config*.md: `cells`(文字列 "rich" / "plain" か表。表は style・checkbox・chips・select・links・icons と [cells.columns])。設定は mdgrid::config::Cells にまとめ、列ごと・種類ごとの判定は Cells のメソッドにする。
- src/ui/grid.rs: 列の札の判定(selects)を列の型と一緒に、全部の行から決める。
- src/ui/cell.rs: Shown に部品(Part: 札の並び・リンク)を足し、rich のとき真偽を `☑`・`☐`、リストと札の列を札の文字にする。幅は今と同じく shown の文字から。
- src/ui/view.rs: 部品のあるセルを、札ごとに色を付けた span で描く。列の見出しに型の印。
- src/ui/chips.rs(新): 札の色(値の文字から8色)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | セルの部品 | SR-35 | 試験が通る | src/config.rs, src/config_items.rs, src/i18n.rs, src/ui/grid.rs, src/ui/cell.rs, src/ui/view.rs, src/ui/chips.rs, src/cells.rs, src/lib.rs, src/test_cells_unit.rs, src/ui/app.rs, src/ui/nav.rs, src/ui/settings.rs, src/ui/mod.rs, src/ui/test_rich_cells.rs, docs/, README.md, README.ja.md, tests/golden/, specs/test-locks.json | test_sr_35_rich_cells(src/ui/test_rich_cells.rs)、test_sr_35_cells_string_and_table(src/test_cells_unit.rs) | 済 |

## 実装の気づき

- 設定は核の mdgrid::cells(Cells・Part・ColStyle・looks_like_select)に置き、画面は App::rich・is_select で引く。
- 選んでいるセルは札にせず選びの見た目で文字を見せる。真偽の列は検索で値の文字(true・false)でも一致させた(nav の search_hit)。
- 札の色は値の文字の指紋から8色。地と文字の両方を決めるので、明るい地の端末でも読める。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-35 | checked | コード: src/cells.rs、src/ui/cell.rs(rich_text・CellPart)、src/ui/view.rs(chip_spans・head_text)、src/ui/chips.rs、src/ui/grid.rs(selects)、src/ui/detail.rs、src/ui/nav.rs(search_matches) / テスト: src/test_cells_unit.rs、src/ui/test_rich_cells.rs、src/ui/test_rich_cells_more.rs |

既存のテストの削除・skip・弱体化: なし

照合は書込なしの検証役(2026-10-09)が gap を出し、直した: 札を強いた列("chip")でリストが札にならない・詳細に札の文字が出る・検索の印が ☑ に付かない・札の列の判定が毎回全部の行を集める(部品を使わない列は数えず、流して読む)・点を含むキーを計算の列と取り違える。直したあとの試験は src/ui/test_rich_cells_more.rs。

確かめた: 1 / 1
