---
type: Change
id: 01M3Y2C8G9CAF4QTJ4WKQVTYN6
title: 行番号・一行おきの色・列の区切り線・上の帯の出し入れ(display-options)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# 行番号・一行おきの色・列の区切り線・上の帯の出し入れ(display-options)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [display-options](../_decisions/2026-10-02-display-options.md)(人)、[column-lines-fix](../_decisions/2026-10-02-column-lines-fix.md)(人)、[display-options-details](../_decisions/2026-10-02-display-options-details.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): 行番号を表示するかしないかなども選べるようにしたい。設定とビューごとの両方で切り替える。ほかに一行おきの色・列の区切り線・上の帯の出し入れ。行の名前の欄を隠すのは今回は入れない(TODO)。

終わりの条件: `[display]` の設定と、ビューの設定の画面(`o`)の「表示」の節で、行番号・一行おきの色・列の区切り線・タブ・検索の欄・設定の帯を切り替えられ、mdgrid のビューに保存でき、帯を隠しても操作は効く。関係する要件: SR-20・SR-21・NV-13・NV-16・NV-23・BV-17・SR-15・CLI-12。

## 不明点と仮定

- 仮定: 行番号は今の表示の並び(絞り込みと並べ替えのあと)の1始まり。グループの見出しの行には付けない。
- 仮定: 一行おきの色は暗めの背景(色を使わない表示では付けない)。
- 仮定: 検索の欄のビューごとの切り替えは、ビューの設定の display に置き、設定の `search_bar` を既定にする。

## 設計

設定(src/config.rs)に `[display]`(row_numbers・zebra・column_lines・tabs・chips)を足し、ITEMS と docs/config.md・config.ja.md に入れる(CLI-12)。ビューの設定(src/settings.rs の Settings)に `display: DisplayOverride`(各項目が Option<bool>。設定と違うものだけ)を足し、mdgrid のビュー・見た目の状態に保存する。ビューの設定の画面(src/ui/settings*.rs)に「表示」の節を足し、6つを切り替える。描画(src/ui/grid.rs・view.rs・bands.rs)は、決まった値(設定 → ビュー)で、行番号の欄を左に足す、一行おきに背景、列の区切り線、帯の行を出さない(表が広がる)。帯を隠しても操作のキーはそのまま。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 表示の設定・ビューごとの上書き・描画・設定の画面 | SR-20, SR-21 | 確かめ方の例の全部、--print-config と文書、golden | src/config.rs, src/config_items.rs, src/display.rs, src/lib.rs, src/settings.rs, src/views.rs, src/test_views_unit.rs, src/ui, docs/config.md, docs/config.ja.md, tests, tests/golden | test_SR_20, test_SR_21 | 済 |

受け入れの試験は実装を見ていない役が先に書く。

## 実装の気づき

- タスク1: 受け入れの2件が試験の側の誤りで落ちる(test_sr_20_toggle_in_settings_applies_to_that_view_only は設定の画面を開いたまま表の行番号を画面で確かめる、test_sr_20_zebra_colors_even_rows の桁 x=13 は「読んだ本」「次の本」の幅2の文字の後ろ半分で ratatui が書かないセル)。ほかは通る。
- 差し戻し1回目: `│` は Ambiguous で ambiguous_wide では幅2になるので幅1の `|` に替え、一行おきの色はノートの行の番号で数えて前景も決めた(Indexed 237/252)。低い画面の「表示」の節は選んだ項目まで流す。今は受け入れの試験も全部通る。
- タスク1(再レビュー): 帯を隠したまま BS で外したあと次に選ばれた条件が見えなかったのを main が直した(外した知らせの後ろに次の条件)。残り(低): 狭い幅でヘッダーの「ビュー 名前」が絞り込みの語を押し出す・切れる、16 色の端末にも 256 色の番号を送る(既存の強調と同じ作り)。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。疑似端末で本物のバイナリを `[display]` の設定で起動し、見た目とビューごとの上書きを確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-20 | checked | コード: src/ui/display.rs:44 / テスト: src/ui/test_display_options.rs::test_sr_20_toggle_in_settings_applies_to_that_view_only / 今: 通った(cargo test) / 前: 落ちた(409a9fc に今の試験を入れると14件中12件が落ちた。変異: 色なしでも zebra・区切りを空白・行番号を出さない・帯の行を詰めない・上書きを保存しない → それぞれ落ちた) |
| SR-21 | checked | コード: src/config.rs:276 / テスト: tests/test_display_options.rs::test_sr_21_reads_each_item / 今: 通った(cargo test) / 前: 落ちた(409a9fc では Config に display が無くコンパイルで落ちる。変異: 値を読まない・--print-config から1項目抜く → 落ちた) |

既存のテストの削除・skip・弱体化: なし(..Default::default() の追加、節が6つになった section の助け、golden の nv_18 の「表示」の節、7d53cbb の受け入れの試験の直し2つはどれも理由と合う)

残した気づき: 7d53cbb で消した確かめにより「反映する前は表が変わらない」は試験で見ていない(設定の画面が表を覆うので画面では見えない)。--print-config は display をインラインの表で出す。

確かめた: 2 / 2
