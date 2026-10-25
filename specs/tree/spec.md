---
type: Spec
id: 01M5HZRF83S0X0TZR9Z0RJ2CRE
title: 木の画面(tree)
description: 階層を文字の列で持つノートの表を、フォルダの木と、フォルダの中の並びで見せる画面の型。
status: active
load_when: 階層の列の設定・木の画面・フォルダの木の描き方・フォルダの中の並び・木からの URL を開く操作を作る・変えるとき
created: 2026-10-22
updated: 2026-10-22
---

# 木の画面(tree)

ブックマークの台帳のように、1件1ノートで、置き場の階層を「根 > フォルダ > …」のような文字の列で持つ表を、フォルダの木と、選んだフォルダの中の並びで見せる画面の型。表の画面とは別の画面の型(SC-9)で、関係マップと同じくヘッダーのメニューから切り替える。どの列が階層でどの列が並びかは、ワークスペースの印に書く。ブックマーク専用ではなく、階層を文字の列で持つどの台帳にも使う。版1 は見るだけで、行を別のフォルダへ移す・並べ替えるのは次の段。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 木の画面

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| TR-1 | 表の画面とは別の画面の型(SC-9)として、木の画面を持つべき。表に階層の列の設定(TR-2)があるときだけ、画面の型のメニュー(REL-8)に「木」を出して切り替えられ、設定が無い表では出さないべき。木の画面は、今の表のビュー(絞り込み・where・計算の列。BV-5・WS-9・WS-10)を通った行だけを扱うべき(ビューで外した行を木に出さないため)。 | 印に階層の列を書いた表で「▦ 表 ▾」を押す → 窓に「木」がある。選ぶ → 木の画面になる。設定の無い表 → 窓に「木」が無い。ビュー「使う」(status が active だけ)で木を開く → archived の行が木に出ない(`test_tr_1_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-2 | ワークスペースの印の `[[table]]` に `tree = { path = "<列>", separator = "<区切り>", order = "<列>" }` を書いてよく、`path` の列の値を `separator`(省略は ` > `)で切ったものを、その行の置き場の階層とみなすべき。`order` は省略してよい。`path` の列が無い表や、区切りが空の文字のときは、CLI-3 の形で警告して木を出さないべき。 | `tree = { path = "folder", order = "order" }` と folder が `bar > 開発 > Web` の行 → bar の下の 開発 の下の Web に置かれる。`separator = ""` → 警告1行で、メニューに「木」が出ない(`test_tr_2_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-3 | 木の画面の左に、階層の木を描くべき。行の値に出てくる途中の階層もフォルダとして作り(`a > b > c` だけでも a と b を作る)、各フォルダに、その下にある行の数(下の階層も含む)を添え、フォルダを開く・閉じるができるべき。階層の列が空の行は、根の「(なし)」にまとめるべき。同じ名前のフォルダは、親が違えば別のフォルダにするべき。フォルダの並びは名前の順にするべき。 | `bar > 開発`(2行)・`bar > 開発 > Web`(1行)・`other`(1行)・空(1行) → 木は bar(3)・開発(3)・Web(1)・other(1)・(なし)(1)。開発を閉じる → Web が隠れ、数は3のまま(`test_tr_3_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-4 | 木の画面の右に、選んだフォルダの直下にある行を、今のビューの列(`order` の並び)で並べるべき。並びは TR-2 の `order` の列の小さい順(数として読めない値と空は後ろに、同じ値はノートの名前の順)にし、`order` が無ければノートの名前の順にするべき。行を選んで Enter で、表の画面のその行に移れるべき。 | 開発 を選ぶ → order 10・20・空 の3行がその順に出る。order が `abc` の行は空と同じく後ろ。行で Enter → 表の画面に戻り、その行が選ばれている(`test_tr_4_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-5 | 右の欄で選んだ行に `http://` か `https://` で始まる値の列があれば、その URL を OS の既定のブラウザで開く操作を、操作の一覧とパレットから出せるべき。開く前に URL を下の帯に見せ、`http`・`https` 以外の値は開いてはならない(ノートの値から任意のコマンドやファイルを開かないため)。 | url が `https://example.com` の行で「URL を開く」→ 帯に URL が出て、OS の開くコマンドに渡る(試験では開く口を差し替えて渡った値を確かめる)。`file:///etc/passwd`・`javascript:` の行 → 出さないか、理由を出して開かない(`test_tr_5_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-6 | 木の画面の段組み(左の木・右の行)は、端末の幅と高さに合わせて変えるべき(広ければ並べ、狭ければ1つずつにしてキーかタブで切り替える)。どの幅でも、文字が重なったり切れて読めなくなったりしてはならない。 | 幅 160・100・60 で木の画面を出す → 160・100 は並べ、60 は1つずつ。どれも重ならない(`test_tr_6_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-7 | 木の画面は、マウスでも操作できるべき。フォルダのクリックで選び、開閉の印のクリックで開く・閉じる、右の行のクリックで選び、選んでいる行をもう一度クリックしたら表のその行に移り、ホイールで上下に選ぶ。 | 開発 の印をクリック → 閉じる。Web をクリック → 選ぶ。右の行をクリック2回 → 表のその行へ(`test_tr_7_*`) | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| TR-8 | 木の画面で、右の行を別のフォルダへ移せるべき(左のフォルダを選んで移す)。移すと、その行の `path` の列の値を新しいフォルダの階層の文字に書き換える変更としてため、保存の流れ(差分を見せてから書く。WB-9)に乗せるべき。 | 読んで判定 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |
| TR-9 | 木の画面で、右の行を上下に動かして並びを変えられるべき。動かすと、そのフォルダの行の `order` の列を、並びどおりの値(10 刻み)に振り直す変更としてため、保存の流れに乗せるべき。 | 読んで判定 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) |

## 範囲外

- 次の段: フォルダの並びをノートの外の一覧(例: 台帳の設定のフォルダの順)から読むこと。版1 は名前の順(TR-3)。
- 採らない: ブックマーク専用の画面や、ブラウザのブックマークのファイルを直接読むこと(SC-1・SC-2。台帳の側の道具の役目)。
- 採らない: 表のテーマの切り替えで木を作ること(SC-9)。

## 前提(AI の仮定)

- 木の画面への切り替えのキーは設計で決める(関係マップの `R` と同じく、窓を開かずに切り替えるキーを1つ持つ)。
- 階層の列が文字のリスト(`[a, b]`)のときは、版1 では最初の要素だけを使う。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-22 | [階層の列でノートを木に並べて見せる「木」の画面を足す(tree-view)](../_decisions/2026-10-22-tree-view.md) | accepted | 人 | 人の発言 | TR-1, TR-2, TR-3 ほか 6 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| TR-1 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/ui/test_folder_tree.rs::test_tr_1_menu_offers_tree_and_switches, src/ui/test_folder_tree.rs::test_tr_1_no_tree_without_setting, src/ui/test_folder_tree.rs::test_tr_1_tree_uses_view_rows ほか 1 | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-2 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/test_folder_tree_unit.rs::test_tr_2_empty_separator_warns_and_no_tree, src/test_folder_tree_unit.rs::test_tr_2_marker_tree_reads_path_order_and_default_separator, src/test_folder_tree_unit.rs::test_tr_2_split_by_separator ほか 7 | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-3 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/test_folder_tree_unit.rs::test_tr_3_same_name_under_other_parents, src/test_folder_tree_unit.rs::test_tr_3_tree_counts_and_none, src/ui/test_folder_tree.rs::test_tr_3_tree_counts_fold_and_none ほか 1 | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-4 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/test_folder_tree_unit.rs::test_tr_4_rows_by_order_then_name, src/ui/test_folder_tree.rs::test_tr_4_opens_at_current_row, src/ui/test_folder_tree.rs::test_tr_4_rows_by_order_and_enter_jumps ほか 1 | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-5 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/test_folder_tree_unit.rs::test_tr_5_web_url_only_http, src/ui/test_folder_tree.rs::test_tr_5_open_url_shows_and_passes_value | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-6 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/ui/test_folder_tree.rs::test_tr_6_layout_by_width, tests/test_tree_e2e.rs::test_tr_1_tree_screen_counts_and_jump_e2e | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-7 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | src/ui/test_folder_tree.rs::test_tr_7_mouse | [2026-10-22-tree-view](../_changes/2026-10-22-tree-view.md) 2026-10-23 |
| TR-8 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | なし | なし |
| TR-9 | [2026-10-22](../_decisions/2026-10-22-tree-view.md) | なし | なし |
<!-- decidespec:history:end -->
