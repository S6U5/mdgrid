---
type: Spec
id: 01M799C8HTSJXSW1QFN0KZW52H
title: ボードの画面(board)
description: 列の値ごとのレーンにノートをカードとして並べ、カードを移して値を直す画面の型。
status: active
load_when: ボードの画面・レーンの作り方と並び・カードの中身・カードを移す操作・ボードの段組みとマウスを作る・変えるとき
created: 2026-11-13
updated: 2026-11-13
---

# ボードの画面(board)

タスクの台帳のように、1件1ノートで、進み具合を `status: doing` のような列の値で持つ表を、値ごとのレーンにカードを並べて見せる画面の型(いわゆるカンバン)。表の画面とは別の画面の型(SC-9)で、関係マップ・木の画面と同じくヘッダーのメニューとキーから切り替える。設定は要らず、切り替えたときにカーソルのあった列でまとめる。カードを隣のレーンへ移すと、その列の値を書き換える変更としてためられ、保存の流れ(差分を見せてから書く)に乗る。`.base` の kanban のビューの解釈はしない(SC-8 のまま)。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — ボードの画面

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BD-1 | 表の画面とは別の画面の型(SC-9)として、ボードの画面を持つべき。表の画面で、キー(既定 `B`)・画面の型のメニュー(REL-8)・パレットから切り替えられ、切り替えたときにカーソルのあった列の値ごとに、今の表のビュー(絞り込み・where・計算の列。BV-5・WS-9・WS-10)を通った行だけをレーン(縦の並び)に分け、1行を1枚のカードとして並べるべき。まとめる列にできるのは、ノートのキーの列で、列の型がテキスト(CE-2・CE-35)の列だけにし、ほかの列(`file.*`・計算の列・数・日付・リスト・チェックボックス)では切り替えずに理由を出すべき(型と合わない値を作らないため)。ボードの画面では、同じキー・Esc・`q` で表の画面に戻れるべき。 | 列 status が todo(2行)・doing(1行)・done(1行)の表で、status の列にカーソルを置いて `B` → 「todo」「doing」「done」の3つのレーンに4枚のカードが出る。Esc → 表に戻る。数の列 estimate で `B` → 切り替わらず理由が出る。status = done の行を絞り込みで外してから `B` → done のカードは出ない(`test_bd_1_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-2 | レーンは、まとめる列の値の文字ごとに1つ作り(ためた値があればその値)、見出しに値と、そのレーンのカードの数を出すべき。レーンの並びは、その列の決まりの `values`(CE-36)があればその順を先にし、`values` にある値は、その値の行が無くてもレーンを出すべき(カードを移す先になるため)。残りの値のレーンは文字の順に続け、値が空の行(キーが無い・null・空の文字)のレーン「(なし)」は、空の行があるときだけ最後に出すべき。 | `[rules.status] values = ["todo", "doing", "done"]` と、status が done・todo・review・空 の4行 → レーンは todo(1)・doing(0)・done(1)・review(1)・(なし)(1)の順。決まりが無い → doing・done・todo の文字の順。空の行が無い → 「(なし)」は出ない(`test_bd_2_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-3 | カードには、1行目にノートの名前(表の左のノートの欄と同じ名前。SR-29)を、2行目に、今のビューで見えている列のうち、まとめる列を除いた先頭の2つの列の値を出すべき。ためた変更のある行のカードには、ためた変更の印(`*`。SR-15)を付けるべき。レーンの中のカードの並びは、今のビューの行の並び(並べ替え。NV-3)と同じにするべき。カードを選んで Enter で、表の画面に戻ってその行を選べるべき。 | ビューの列が status・due・owner・tags、ノート `write-spec.md` が due 2026-11-20・owner mika → カードの1行目に write-spec、2行目に 2026-11-20 と mika。due の昇順のビュー → レーンの中も due の昇順。カードで Enter → 表の画面でその行が選ばれている(`test_bd_3_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-4 | ボードの画面で、選んだカードを左・右の隣のレーンへ移せるべき(既定 `H`・`L` と Shift+←・Shift+→)。移すと、その行のまとめる列の値を、移した先のレーンの値にする変更を、ためる変更の1手(WB-10)としてためるべきで、ファイルは保存の流れ(差分を見せてから書く。WB-9)でだけ書かなければならない。レーン「(なし)」へ移すと、値を空にする変更(CE-9)をためるべき。読むだけのセル(CE-8・CE-12・WB-5)の行のカードは移さずに理由を出し、読むだけの起動(WB-15)では移してはならない。移したあとも同じカードを選んだままにし、ボードの画面の中で取り消し・やり直し(WB-10)ができるべき。保存の操作(既定 Ctrl+S)は、表の画面に戻って保存の確認(WB-9)を開くべき。 | todo のレーンのカード write-spec で `L` → doing のレーンに移り、カードに `*`、下の帯に「未保存 1」、ファイルは変わらない。`u` → todo に戻って「未保存 0」。左の端のレーンで `H` → 動かない。Ctrl+S → 保存の確認に `-status: todo`・`+status: doing`。読めないフロントマターのノートのカードで `L` → 移らず理由。`--readonly` で `L` → 移らない(`test_bd_4_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-5 | レーンは横に並べ、レーンの幅は画面の幅をレーンの数で分けた幅(下限 18 桁)にし、入りきらないレーンは、選んでいるレーンが見えるように横に送るべき。レーンの中のカードが高さに入りきらないときは、選んでいるカードが見えるように縦に送るべき。隠れているレーンとカードがあることを、端の印で示すべき。どの幅と高さでも、文字が重なったり、隣のレーンにはみ出したりしてはならない。 | レーンが5つで幅 100 → 5つとも出る。幅 40 → 2つだけ出て、`l` で右へ動くと選んだレーンが見える所まで送られ、左に隠れたレーンの印が出る。1つのレーンにカード 30 枚・高さ 24 → `j` で下へ動くと選んだカードが見える。どの行も画面の幅を超えない(`test_bd_5_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-6 | ボードの画面は、マウスでも操作できるべき。カードのクリックで選び、選んでいるカードをもう一度クリックしたら表の画面のその行に移り、レーンの見出しのクリックでそのレーンを選び、ホイールで選んでいるレーンのカードを上下に選ぶ。 | doing のレーンのカードをクリック → そのカードが選ばれる。もう一度クリック → 表の画面でその行。done の見出しをクリック → done のレーンが選ばれる(`test_bd_6_*`) | [2026-11-13](../_decisions/2026-11-13-board-view.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BD-7 | ワークスペースの印の `[[table]]` に、ボードの設定(まとめる列・レーンの順・カードに出す列)を書いて、列にカーソルを置かなくても同じボードを開けるべき。 | 読んで判定 | [2026-11-13](../_decisions/2026-11-13-board-view.md) |
| BD-8 | レーンの中でカードを上下に動かして並びを変え、並びの列の値を振り直す変更としてためられるべき。 | 読んで判定 | [2026-11-13](../_decisions/2026-11-13-board-view.md) |

## 範囲外

- 次の段: リストの列(`tags` など)でまとめること(1枚のカードが2つ以上のレーンに出る)。版1 はテキストの列だけ(BD-1)。
- 次の段: マウスでカードを掴んで別のレーンへ動かすこと。版1 はキーで移す(BD-4)。
- 採らない: `.base` の kanban のビューを読んでボードにすること(SC-8。mdgrid のボードは、表のビューの上の画面の型)。
- 採らない: 表のテーマの切り替えでボードを作ること(SC-9)。

## 前提(AI の仮定)

- まとめる列は、切り替えたときにカーソルのあった列にする(設定を要らなくするため。印に書く形は BD-7)。
- レーンの順は、列の決まりの `values`(CE-36)を使う(新しい設定を足さず、値の候補の順をそのまま使えるため)。
- カードの2行目に出す列は、ビューの見えている列の先頭の2つ(まとめる列を除く)にする。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-11-13 | [列の値ごとのレーンにノートをカードとして並べる「ボード」の画面を足す(board-view)](../_decisions/2026-11-13-board-view.md) | accepted | 人 | 人の発言 | BD-1, BD-2, BD-3 ほか 5 |
| 2026-11-13 | [ボードの画面の受け入れのテスト2本の誤りを直し、錠を掛け直す(board-tests-locks)](../_decisions/2026-11-13-board-tests-locks.md) | accepted | 人 | 検証の指摘 | BD-1, BD-4 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| BD-1 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_1_action_board_view_switches, src/ui/test_board.rs::test_bd_1_english_words, src/ui/test_board.rs::test_bd_1_esc_q_and_same_key_return_to_table ほか 9 | なし |
| BD-2 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_1_english_words, src/ui/test_board.rs::test_bd_2_missing_null_and_empty_text_share_none_lane, src/ui/test_board.rs::test_bd_2_no_rule_text_order_and_no_none_lane ほか 13 | なし |
| BD-3 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_2_pending_value_decides_lane, src/ui/test_board.rs::test_bd_3_card_name_and_first_two_columns, src/ui/test_board.rs::test_bd_3_cards_follow_view_sort ほか 5 | なし |
| BD-4 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_4_card_actions_move_left_and_right, src/ui/test_board.rs::test_bd_4_edges_do_not_move, src/ui/test_board.rs::test_bd_4_move_out_of_none_lane_sets_value ほか 12 | なし |
| BD-5 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_5_lines_fit_any_width_and_height, src/ui/test_board.rs::test_bd_5_long_text_stays_in_its_lane, src/ui/test_board.rs::test_bd_5_tall_lane_scrolls_to_selected_card ほか 2 | なし |
| BD-6 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | src/ui/test_board.rs::test_bd_6_click_card_selects_then_goes_to_table_row, src/ui/test_board.rs::test_bd_6_click_lane_head_selects_lane, src/ui/test_board.rs::test_bd_6_click_second_line_and_other_card_selects ほか 1 | なし |
| BD-7 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | なし | なし |
| BD-8 | [2026-11-13](../_decisions/2026-11-13-board-view.md) | なし | なし |
<!-- decidespec:history:end -->
