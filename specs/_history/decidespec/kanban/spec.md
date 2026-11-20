---
type: Spec
id: 01M6BVAQDRX9WNRSKCHGNH0N4X
title: カンバン(状態の列の値ごとのレーンの画面)
description: 印の設定か列の名前の推定で、今のビューの行を状態の列の値ごとのレーンに札として並べ、札を動かして値を直す画面の型
status: active
load_when: カンバンの画面、状態の列の推定、札を動かして値を直すことを扱うとき
created: 2026-11-02
updated: 2026-11-08
---

# カンバン(状態の列の値ごとのレーンの画面)

表(と木・関係マップ・ガント)に続く画面の型。今のビューの行を、状態の列の値ごとのレーン(縦の列)に札として並べ、全部の状態を1枚で見せる。レーンの順は列の決まり(CE-36 の `values`)から取り、札を隣のレーンへ動かすと、表のセルの直しと同じためる変更として状態の値を書き換える。設定は印の `[[table]]` の `kanban = { … }` で、無ければ列の名前から推定する(GA-7 の一般の形)。扱わない: 札に値を添えること、レーンの中の並べ替え、WIP の上限、`.base` の kanban のビュー(SC-8)。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| KB-1 | ワークスペースの印の `[[table]]` に `kanban = { column = "<列>" }` を書いてよく、その列の値ごとに行を分けるべき。設定が無いときは、列の名前から推定してよい(status・state・stage・状態・ステータス・段階。大文字小文字を問わない。リストの型の列は選ばない)。推定でも決まらないときは、理由を出してカンバンを出さないべき。設定の `column` の列が表に無いか、リストの型の列のときは、CLI-3 の形で警告して出さないべき(1行が複数のレーンに属さないように)。ただし、行が1つも無い表(見本なしで作った台帳など)では、列はノートのキーから作るので設定の列があるかを決めず、警告しないべき(画面の型のメニューには出し、開くときに出す行が無いと理由を出す)。 | `kanban = { column = "status" }` の表 → 画面の型のメニューに「カンバン」が出て `B` で開く。設定なしで status の列がある → 推定して開き、画面の題に推定と示す。設定なしで状態らしい列が無い → 理由1行で開かない。`column = "none"` → 警告1行で、メニューに出ない。行が1つも無い表 → 読み終えても警告せず、メニューに出て、開くと理由1行。外からノートを足すと `B` で開く(`test_kb_1_*`) | [2026-11-08](../_decisions/2026-11-08-empty-table-screen-columns.md)、[2026-11-02](../_decisions/2026-11-02-kanban.md) |
| KB-2 | `B`(か画面の型のメニューの「カンバン」)で、今のビューの行(絞り込み・並べ替えのあと。TR-1 と同じ)を、列の値ごとのレーン(縦の列)に札として並べて見せるべき。レーンの順は、その列の決まり(CE-36 の `values`)があればその順(行の無い値もレーンにする)、続いて決まりに無い値を行に現れた順、最後に値の無い行の「(なし)」のレーン(そういう行があるときだけ)にするべき。レーンの見出しには値と札の数を出し、札は行の名前(表のノートの欄と同じ見せ方)の1行で、レーンの中は表の行の順にするべき。レーンが幅に収まらないときは、選んでいるレーンが見える所まで横にずらし、隠れたレーンがあることを示すべき。色なしでも、選んでいる札とレーン、ためる変更のある札を見分けられるべき(SR-15)。 | 決まり `values = ["todo", "doing", "waiting", "done"]` の status で todo 2件・doing 1件・done 1件・状態なし1件 → レーンは todo(2)・doing(1)・waiting(0)・done(1)・(なし)(1) の順。決まりに無い値 blocked の行がある → done と (なし) の間に blocked のレーン。幅 40 で5つのレーン → 選んでいるレーンが見え、隠れた側に印。`--no-color` でも選んでいる札に `>`(`test_kb_2_*`) | [2026-11-02](../_decisions/2026-11-02-kanban.md) |
| KB-3 | カンバンの画面では、`j` `k` でレーンの中の札を選び、`h` `l` でレーンを移り、`Enter` で表に戻ってその行を選び、`Esc`・`q`・`B` で表に戻るべき。マウスは、札のクリックで選び、選んでいる札をもう一度クリックすると表のその行へ、ホイールで札、横のホイールでレーンを動かすべき。 | `B` → `l` → `j` → `Enter` で、表の doing の2件目の行が選ばれる。札のクリック → その札を選ぶ(`test_kb_3_*`) | [2026-11-02](../_decisions/2026-11-02-kanban.md) |
| KB-4 | カンバンの画面では、`H` `L` で選んでいる札を左・右のレーンへ動かし、その行の列の値を、動かした先のレーンの値に書き換えるべき。書き換えは表のセルの直しと同じためる変更(未保存)にし、動かした札は動かした先のレーンで選んだままにするべき。`u` で取り消し、`U` でやり直し、`Ctrl+s` で表に戻って保存の確かめ(差分)を開くべき。「(なし)」のレーンへは動かさず(値を消すのは表で)、読むだけのとき(`--readonly`、読むだけの列や行。CLI-2・BV-7・BV-12)は動かさずに理由を出すべき。 | todo の札で `L` → 札が doing のレーンへ移って選んだまま、下の帯の未保存が 1。`u` → todo に戻る。`--readonly` で `L` → 動かず理由1行。右の隣が (なし) の札で `L` → 動かず理由1行(`test_kb_4_*`) | [2026-11-08](../_decisions/2026-11-08-kanban-move-confirmed.md)、[2026-11-02](../_decisions/2026-11-02-kanban.md) |
| KB-5 | カンバンを開いているあいだにノートが外で変わったら、組み直して同じ行の札を選び直すべき(札が別のレーンへ移っていればそのレーンで)。 | 開いたまま外で status を todo から done に書き換える → 札が done のレーンに移り、選んだまま(`test_kb_5_*`) | [2026-11-02](../_decisions/2026-11-02-kanban.md) |
| KB-6 | 見せ方の一覧(SR-49)・できることの一覧(SR-51)・キーの文書(SR-22)・設定の文書にカンバンの項を持ち、組み込みの tasks のスターターに `kanban` の設定を同梱し、場面の台本にカンバンの場面を持つべき。 | docs/manual/ja・en の views.md に「カンバン」の節(画像 images/kanban.svg)、features.md に札を動かして状態を直す項。starters/tasks の印に `kanban = { column = "status" }`。docs/manual-scenarios.toml に id = "kanban"(読んで判定と `test_kb_6_*`) | [2026-11-02](../_decisions/2026-11-02-kanban.md) |

## 次の段

- 札に値を添える(`kanban = { column, card = ["due", "priority"] }` で札の2行目に)。
- レーンの中の並べ替えと、WIP の上限の印。

## 範囲外

- `.base` の kanban のビューを開くこと(SC-8)。カンバンの設定は印の `[[table]]` の `kanban` に書く(GA-7)。
- 札の値を消すこと(「(なし)」のレーンへ動かすこと)。値を消すのは表のセルの直しでする。

## 前提(AI の仮定)

- 行の並びは表(ビューの設定)が作る行の順をそのまま使う(カンバンで独自の並びを持たない)。親子(NV-27)の畳みは見ず、ビューを通った全部の行を札にする。
- 推定の列の名前の一覧は固定(KB-1)。外れたら印で上書きする。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-11-08 | [カンバンで札を動かして状態の値を書き換える形(KB-4)を、人が今の文のまま決める(kanban-move-confirmed)](../_decisions/2026-11-08-kanban-move-confirmed.md) | accepted | 人 | 人の発言 | KB-4 |
| 2026-11-08 | [行が1つも無い表では、画面の型の設定の列が表に無くても警告しない例外を GA-1・TR-2・KB-1 に書き足す(empty-table-screen-columns)](../_decisions/2026-11-08-empty-table-screen-columns.md) | accepted | 人 | 人の発言 | GA-1, TR-2, KB-1 |
| 2026-11-02 | [画面の型「カンバン」を足し、状態の列の値ごとのレーンに札を並べて、札を動かして値を直す(kanban)](../_decisions/2026-11-02-kanban.md) | accepted | AI | 人の発言 | KB-1, KB-2, KB-3, KB-4, KB-5, KB-6 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| KB-1 | [2026-11-08](../_decisions/2026-11-08-empty-table-screen-columns.md) | src/ui/test_empty_screens.rs::test_kb_1_empty_table_missing_status_column_no_warning_menu_and_reason, src/ui/test_kanban.rs::test_kb_1_bad_setting_shape_warns, src/ui/test_kanban.rs::test_kb_1_infers_japanese_name_and_skips_list_column ほか 17 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| KB-2 | [2026-11-02](../_decisions/2026-11-02-kanban.md) | src/ui/test_kanban.rs::test_kb_2_lanes_follow_rule_order_with_counts, src/ui/test_kanban.rs::test_kb_2_narrow_width_scrolls_lanes_and_marks_hidden_side, src/ui/test_kanban.rs::test_kb_2_no_color_marks_selected_card_and_lane ほか 6 | [2026-11-02-kanban](../_changes/2026-11-02-kanban.md) 2026-11-03 |
| KB-3 | [2026-11-02](../_decisions/2026-11-02-kanban.md) | src/ui/test_kanban.rs::test_kb_3_arrow_keys_move_like_hjkl, src/ui/test_kanban.rs::test_kb_3_click_lane_heading_selects_that_lane, src/ui/test_kanban.rs::test_kb_3_esc_q_b_return_to_table ほか 11 | [2026-11-02-kanban](../_changes/2026-11-02-kanban.md) 2026-11-03 |
| KB-4 | [2026-11-08](../_decisions/2026-11-08-kanban-move-confirmed.md) | src/ui/test_kanban.rs::test_kb_4_ctrl_s_returns_to_table_and_opens_review, src/ui/test_kanban.rs::test_kb_4_does_not_move_into_none_lane, src/ui/test_kanban.rs::test_kb_4_edges_give_reason ほか 11 | [2026-11-02-kanban](../_changes/2026-11-02-kanban.md) 2026-11-03 |
| KB-5 | [2026-11-02](../_decisions/2026-11-02-kanban.md) | src/ui/test_kanban.rs::test_kb_5_external_change_moves_card_and_keeps_selection, src/ui/test_kanban_fix4.rs::test_kb_5_setting_column_removed_outside_closes_kanban, src/ui/test_kanban_fix4.rs::test_kb_5_setting_column_turned_list_outside_closes_kanban ほか 2 | [2026-11-02-kanban](../_changes/2026-11-02-kanban.md) 2026-11-03 |
| KB-6 | [2026-11-02](../_decisions/2026-11-02-kanban.md) | tests/test_kanban_docs.rs::test_kb_6_config_doc_explains_kanban_and_inferred_names, tests/test_kanban_docs.rs::test_kb_6_features_page_has_card_moving_item, tests/test_kanban_docs.rs::test_kb_6_keys_doc_has_kanban_section_and_b_in_table ほか 3 | [2026-11-02-kanban](../_changes/2026-11-02-kanban.md) 2026-11-03 |
<!-- decidespec:history:end -->
