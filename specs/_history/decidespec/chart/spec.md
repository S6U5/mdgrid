---
type: Spec
id: 01M6XN3HMMQTQFDBPJ9VRJQQEF
title: グラフ(今のビューの行を数えて描く画面)
description: 印の設定か列の推定で、今のビューの行を数え、棒・積み上げ棒・折れ線・散布図・ヒートマップで見せる画面の型
status: active
load_when: グラフの画面、グラフの設定と推定、行の数え方と集め方、グラフの描き方を扱うとき
created: 2026-11-08
updated: 2026-11-08
---

# グラフ(今のビューの行を数えて描く画面)

表(と木・関係マップ・ガント・カンバン)に続く画面の型。今のビューの行を、分ける列の値ごとに数えるか、数の列を集めて、棒・積み上げ棒・折れ線・散布図・ヒートマップで見せる。状態ごとの件数、担当ごとの見積もりの合計、月ごとの件数や累積を、表を数えずに一目で見るため。設定は印の `[[table]]` の `chart = { … }`(複数なら配列)で、無ければ列から推定する(GA-7 の一般の形)。種類と列は画面の中のキーでも切り替えられ、項目から `Enter` で表をその値に絞って戻れる。読むだけの画面で、値は表で直す。扱わない: 円グラフ、印への書き戻し、複数のグラフを並べる画面、`.base` の chart のビュー(SC-8)。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CH-1 | ワークスペースの印の `[[table]]` に `chart = { kind = "<種類>", x = "<列>", y = "<列>", agg = "<集め方>", by = "<列>", unit = "<単位>", cumulative = <真偽>, name = "<名前>" }` を書いてよく、グラフを複数書くときは配列(`chart = [{ … }, { … }]`)にしてよい。種類は bar(棒)・stacked(積み上げ棒)・line(折れ線)・scatter(散布図)・heatmap(ヒートマップ)、集め方は count・sum・avg・min・max、単位は day・week・month で、省いた鍵は CH-2 の推定で埋めるべき。書き方が違うとき(表でない・知らない鍵や値・文字でない列の名前)は CLI-3 の形で警告してそのグラフを外し、列が表に無いときは列が出そろってから CLI-3 の形で1回警告してそのグラフを外すべき(行が1つも無い表では列があるかを決めず、警告しない。GA-1 と同じ)。 | `chart = { kind = "bar", x = "status" }` → `P` で status ごとの件数の棒。`chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }]` → `]` で2つ目の折れ線。`kind = "pie"` → 警告1行で、そのグラフは出ない。`x = "none"` → 読み終えて警告1行(`test_ch_1_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-2 | `P`(か画面の型のメニューの「グラフ」)で、今のビューの行(絞り込み・並べ替えのあと。TR-1 と同じ)を数えたグラフの画面を開くべき。印にグラフがあれば最初のグラフで開き、無ければ棒で開いて、分ける列を、状態の列(KB-1 の名前)→ 決まり(CE-36 の `values`)のある列 → チェックボックスの列 → リストの列 → 日付の列 → 最初の列の順に推定してよく、値は件数にするべき。画面の題には種類・分ける列・値と数えた行の数を、推定した鍵があればそのことを出すべき。列が1つも無い表と、行が1つも無いビューでは、理由1行で開かないべき。 | 設定なしで status の列(todo 2・doing 1・done 3)→ 棒が3本で、題に「推定」。列の無い表 → 理由1行で開かない。ビューで絞った行だけを数える(`test_ch_2_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-3 | 分ける列の値は、文字とチェックボックスは値ごと、リストは要素ごと(1行が複数の項目に入る。NV-9 と同じ)、値の無い行は最後の「(なし)」の項目にするべき。日付の列は日・週・月の区切り(開くときは区切りが幅に収まる最も細かい単位。GA-2 と同じ)で、最初から最後までの区切りを、行の無い区切りも 0 として並べ、数の列は等しい幅の区間(10 まで)に分けるべき。日付と数の列で値の無い・読めない行は数えず、外した行の数を示すべき。値は、件数か、数の列(`y`)の合計・平均・最小・最大にし、数でない値の行はその計算から外して、外した数を示すべき。項目は、列の決まりがあればその順、日付と数は小さい順、ほかは値の大きい順(同じなら名前の順)に並べるべき。 | tags(リスト)で 会議 と 家 を持つ行 → 会議にも家にも1。due が 10/01 と 10/20 の2行で週の単位 → 10/01 の週から 10/20 の週まで4つの区切り(間の2つは 0)。estimate の合計で値が "abc" の行 → 合計に入れず「外した 1 行」(`test_ch_3_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-4 | 棒は、1項目を1行の横棒にし、左に項目の名前、右に値を出し、最大の値を棒の全長にするべき(負の値は棒を描かず値だけ出す)。積み上げ棒は、内訳の列(`by`)の値ごとに棒を塗り分けて上に凡例を出し、内訳は全体で多い順に5つまでとし、残りを「その他」にまとめるべき。`%` で、各棒を全長にした内訳の割合に切り替えるべき。項目が高さに収まらないときは、選んでいる項目が見える所までずらすべき。色なしでも、内訳を塗りの字で見分けられ、選んでいる項目に `>` を付けるべき(SR-15)。 | status(todo 2・doing 1・done 3)の棒 → done の棒が全長、todo はその 2/3。積み上げ棒で x = assignee、by = status → 凡例に3つの値と塗りの字。`%` → どの棒も全長。`--no-color` でも塗りの字が内訳ごとに違う(`test_ch_4_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-5 | 折れ線は、分ける列が日付か数の列のとき、区切りごとの値を点で打って線で結び、左に値の目盛り(最小と最大)、下に最初と最後の区切りの名前を出すべき。`c` で累積(その区切りまでの合計)に切り替えるべき。内訳の列(`by`)があれば内訳ごとに線を引き(5本まで)、点の字(`*` `+` `o` `x` `#`)と凡例で見分けられるべき(SR-15)。`h` `l` で区切りを選び、選んだ区切りの名前と値を出すべき。 | due で 2026-10 に3件・2026-11 に1件、単位が月 → 2点を結ぶ線、`l` で「2026-11: 1」。`c` → 3 と 4。by = status → 線が3本で点の字が違う(`test_ch_5_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-6 | 散布図は、分ける列(横)と値の列(縦。`y`)がどちらも数か日付の列のとき、1行を1点で打ち、内訳の列(`by`)があれば点の字で見分けられるべき。どちらかの値が無い・読めない行は外して、外した数を示すべき。`j` `k` で点(横の値の順)を選び、選んだ点の行の名前と2つの値を出し、`Enter` で表に戻ってその行を選ぶべき。 | estimate と actual の数の列で3行 → 3点。actual が空の行 → 外した 1 行。`j` → `Enter` → 表でその行が選ばれる(`test_ch_6_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-7 | ヒートマップは、分ける列(縦)と内訳の列(横。`by`)の値の組ごとの値を、ます目に数で出し、値の大きさを地の濃さで示すべき。色なしでも濃さを字(` ` `░` `▒` `▓` `█`)で示すべき(SR-15)。`h` `j` `k` `l` でます目を選び、選んだます目の組と値を出し、`Enter` でその2つの値の行だけに絞って表に戻るべき(日付と数の区切りでは絞らずに理由を出す)。 | status × assignee で todo かつ alice が2行 → そのます目に 2。`--no-color` で最大のます目に `█`。`Enter` → 表が status = todo かつ assignee = alice の行だけ(`test_ch_7_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-8 | グラフの画面では、`Tab`・`Shift+Tab` で種類、`x` で分ける列、`y` で値(件数 → 数の列)、`a` で集め方、`b` で内訳の列(なし → 列)、`+` `-` で日付の単位、`[` `]` で印に書いたグラフを切り替えてよく、切り替えは画面の中だけにして印は書き換えないべき。種類に合う列が無いとき(折れ線に日付と数の列が無い、散布図に数か日付の列が2つ無い、積み上げ棒とヒートマップに内訳にできる列が無い)は、その種類にせずに理由を出すべき。棒と積み上げ棒では `j` `k` で項目を選び、`Enter` でその値の行だけに絞って表に戻るべき(NV-9 の頻度表の Enter と同じ絞り込み。日付と数の区切りの項目では絞らずに理由を出す)。`Esc`・`q`・`P` で表に戻るべき。マウスは、項目のクリックで選び、選んでいる項目をもう一度クリックすると `Enter` と同じにし、ホイールで選びを動かすべき。グラフの画面では値を直せず、開いているあいだにノートが外で変わったら数え直すべき。 | `Tab` → 積み上げ棒、`b` → 内訳が次の列。done の棒で `Enter` → 表が status = done の行だけになり、ヘッダーに絞り込み。日付の列の無い表で折れ線へ `Tab` → 理由1行で次の種類へ。外で status を書き換える → 数が変わる(`test_ch_8_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |
| CH-9 | 見せ方の一覧(SR-49)・できることの一覧(SR-51)・キーの文書(SR-22)・設定の文書にグラフの項を持ち、組み込みの tasks のスターターに `chart` の設定を同梱し、場面の台本にグラフの場面を、録画の台本にグラフの録画を持つべき。 | docs/manual/ja・en の views.md に「グラフ」の節(画像 images/chart.svg)、features.md に項。starters/tasks の印に `chart`。docs/manual-scenarios.toml に id = "chart"。demos/chart.tape(読んで判定と `test_ch_9_*`) | [2026-11-08](../_decisions/2026-11-08-chart.md) |

## 次の段

- 印に書いたグラフを1画面に並べる(ダッシュボード)。
- 画面で切り替えたグラフを印に書き出す(設定の画面 `o` から選んで保存する形も)。
- 年の単位、面グラフ、箱ひげ図。

## 範囲外

- 円グラフ(端末の文字の格子では形が崩れる。積み上げ棒の割合 `%` で代える)。
- `.base` の chart のビューを開くこと(SC-8)。グラフの設定は印の `[[table]]` の `chart` に書く(GA-7)。
- グラフの画面で値を直すこと。値は表で直す。

## 前提(AI の仮定)

- 行の並びと絞り込みは、表(ビューの設定)が作る行をそのまま使う(グラフで独自の絞り込みを持たない)。親子(NV-27)の畳みは見ず、ビューを通った全部の行を数える。
- 数の列の区間の数(10 まで)と、内訳の数(5 まで)は固定。
- 推定の順(CH-2)と、種類を切り替えたときの列の選び方(その種類に合う最初の列)は固定。外れたら印で書くか、キーで切り替える。
- 平均・最小・最大は、数の値のある行だけで出す。数の値のある行が無い項目は値なし(棒を描かず「-」)とする。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-11-08 | [画面の型「グラフ」を足し、今のビューの行を棒・積み上げ棒・折れ線・散布図・ヒートマップで見る(chart)](../_decisions/2026-11-08-chart.md) | accepted | AI | 人の発言 | CH-1, CH-2, CH-3 ほか 6 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| CH-1 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/test_chart_unit.rs::test_ch_1_agg_names_round_trip, src/test_chart_unit.rs::test_ch_1_kind_names_round_trip_in_cycle_order, src/test_chart_unit.rs::test_ch_1_marker_array_drops_only_bad_one ほか 21 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-2 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/test_chart_unit.rs::test_ch_2_infer_checkbox_then_list_then_date_then_first, src/test_chart_unit.rs::test_ch_2_infer_no_columns_is_none, src/test_chart_unit.rs::test_ch_2_infer_skips_list_status_then_rule_values ほか 10 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-3 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/test_chart_unit.rs::test_ch_3_acc_count_sum_avg_min_max, src/test_chart_unit.rs::test_ch_3_acc_negative_values, src/test_chart_unit.rs::test_ch_3_acc_without_numbers_has_no_value ほか 17 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-4 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/ui/test_chart2.rs::test_ch_4_bar_lengths_follow_values, src/ui/test_chart2.rs::test_ch_4_item_without_numbers_shows_dash, src/ui/test_chart2.rs::test_ch_4_j_k_move_selection_marker ほか 12 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-5 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/ui/test_chart3.rs::test_ch_5_by_draws_one_line_per_value_with_distinct_points, src/ui/test_chart3.rs::test_ch_5_c_toggles_cumulative, src/ui/test_chart3.rs::test_ch_5_cumulative_setting_from_marker ほか 6 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-6 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/ui/test_chart3.rs::test_ch_6_by_uses_point_chars, src/ui/test_chart3.rs::test_ch_6_no_scatter_without_two_numeric_columns, src/ui/test_chart3.rs::test_ch_6_rows_missing_a_value_are_excluded ほか 9 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-7 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/ui/test_chart3.rs::test_ch_7_enter_filters_both_values, src/ui/test_chart3.rs::test_ch_7_enter_on_date_bucket_gives_reason, src/ui/test_chart3.rs::test_ch_7_heatmap_cells_show_values_and_density ほか 4 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-8 | [2026-11-08](../_decisions/2026-11-08-chart.md) | src/ui/test_chart.rs::test_ch_1_array_brackets_switch_saved_charts, src/ui/test_chart3.rs::test_ch_6_no_scatter_without_two_numeric_columns, src/ui/test_chart3.rs::test_ch_8_arrow_keys_move_like_jk ほか 28 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
| CH-9 | [2026-11-08](../_decisions/2026-11-08-chart.md) | tests/test_chart_docs.rs::test_ch_9_config_doc_explains_chart_keys_and_values, tests/test_chart_docs.rs::test_ch_9_demo_tape_records_chart, tests/test_chart_docs.rs::test_ch_9_features_page_has_chart_item ほか 4 | [2026-11-06-chart](../_changes/2026-11-06-chart.md) 2026-11-10 |
<!-- decidespec:history:end -->
