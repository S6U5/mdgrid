---
type: Spec
id: 01M3SC1MJDZHH21QWYARCXHQSR
title: セルと文字の見せ方(cell-view)
description: セルの値をどう見せるか。空・null・キーなしの区別、型と合わない値、改行、寄せ、列の幅、文字の幅、色。
status: active
load_when: セルの描画・列の幅・文字の幅・色を作る・変えるとき
created: 2026-09-30
updated: 2026-10-14
---

# セルと文字の見せ方(cell-view)

表のセルの値をどう見せるか。表の TUI への不満の中心の1つは「日本語や端末の違いで崩れる」なので、幅と色の決まりをここに集める。値の種類(空・null・型と合わない値・改行)を見分けられることは、書き戻しの前に利用者が値を確かめるための芯になる。画面の構成は [screen](../screen/spec.md)、編集は [cell-edit](../cell-edit/spec.md) にある。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 値の見せ方

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CV-1 | null は薄い `∅`、空の文字列は `""`、キーが無いときは空欄にして、null・空の文字列・キーが無い、の3つを見分けられる表示にするべき。 | 3つのノートの表 → 3通りの見え方(ゴールデン `test_CV_1`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |
| CV-2 | 列の型と合わない値(date の列の `someday` など)は、値をそのまま見せて `!` の印を付け、並べ替えでは型の合う値の後ろに置き、編集では列の型の入力を出すべきで、合わない値のまま新しく書かせるべきでない。 | date の列の someday → `!` つきで表示、昇順でも降順でも末尾(`test_CV_2`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |
| CV-3 | 改行を含む値は、1行目の後ろに `…⏎` を付けて表に出し(行を伸ばす設定(CV-8)で伸ばせる行があるときは、改行で行を分けて見せ、入らない分だけ最後の行の後ろに `…` を付ける)、全文は詳細の表示で見せるべき。 | 2行の説明 → 1行目と `…⏎`(ゴールデン `test_CV_3`) | [2026-10-12](../_decisions/2026-10-12-row-lines.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |
| CV-4 | 数の列は右寄せ、チェックボックスの列は中央寄せで見せるべき。 | 数の列のゴールデンで右寄せ(`test_CV_4`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |

## 要件: 版1 — 幅と色

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| SR-9 | 文字の幅は書記素のまとまりで数え、どの行も画面の幅を超えるべきでない。値の中の制御文字は、そのまま端末に出すべきでない。 | 全角・絵文字・結合文字・エスケープ文字を含む値の表 → 全行が画面の幅ちょうど以下(不変条件のテスト `test_SR_9`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| SR-10 | `NO_COLOR` があるときは色を出すべきでなく、`TERM=dumb` では色と絵文字を出すべきでない。標準の入出力が端末でないとき(パイプ)は、画面を出さずに理由を1行出して終わるべき。ただし、画面を出さない起動(`--print`、CLI-5)と、画面は端末(`/dev/tty`)に出して結果だけを標準出力に出す起動(`--pick`、OUT-3)は、標準出力がパイプでも動くべき。 | `NO_COLOR=1` で起動 → エスケープの色指定が無い。標準出力をパイプにして起動 → 画面を出さず理由1行で終わる(`test_SR_10`) | [2026-10-03](../_decisions/2026-10-03-print-pick.md)、[2026-10-01](../_decisions/2026-10-01-edit-view-gaps.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md)、[2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| SR-15 | トゥルーカラーに対応しない端末では 256 色に落とすべきで、ためる変更・読むだけ・選択は、色のほかに印か文字でも示し、色だけで意味を伝えるべきでない。 | `COLORTERM` の無い端末 → 256 色で描き、ためる変更のセルに `*` の印(`test_SR_15`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CV-5 | 列の幅の既定は、見えている行の値の幅から決め、1列の上限を画面の幅の3割にするべき。 | 長い値の列 → 幅は画面の3割で `…` で切れる(`test_CV_5`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |
| CV-6 | East Asian Ambiguous の文字(○● など)の幅は既定で1とし、設定で2にできるべき。 | 設定で2にすると ○ の列がずれない(`test_CV_6`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |
| CV-8 | 表の行は、設定 `display.row_extra_lines`(0〜9 の整数。既定 0)で、長い値と改行のある値を折り返し、データの1行を最大で「1 + その数」の行の高さまで伸ばせるべき(列の幅を広げなくても、長い説明を詳細の表示を開かずに表で読めるように)。行の高さは、その行のセルのうち一番多くの行が要るものに合わせ、行ごとに変わってよい(短い値だけの行は1行のまま)。折り返しは列の幅で、字の幅(CV-6)に従って切り、改行のある値は改行で行を分けるべき。伸ばした行に入りきらない値は、最後の行の後ろに `…` を付けるべき。リストの値の札・まとまりの見出しの行・まとまりの間の空き行は1行のままにしてよい。2行目より下の行には行の印(`>` など)と行番号を出さず、列の区切りと、一行おきの色・選んでいる行の色は行の高さ全部に付けるべき。縦のスクロール・ページ送り・マウスのクリックは伸ばした行の高さで数え、選んでいる行は高さ全部が見えるように表を流すべき(画面より高い行は上から入る分だけ見せる)。値の編集の欄と候補の窓は、その行の1行目に出すべき。0 では今の1行の見え方と1字も変わらないべき。ビューの設定の「表示」の区画(NV-18)で 0・1・2・3 を順に切り替え、ビューごとに上書きできるべき(SR-20 と同じ)。範囲の外の値は警告して既定にするべき。 | `row_extra_lines = 2`、列の幅 20 に 50 字の説明 → その行は3行の高さで、説明が全部見える。61 字 → 3行目の終わりが `…`。2行の説明(改行あり) → 2行の高さで1行目と2行目。短い値だけの行 → 1行。伸ばした行の2行目をクリック → その行を選ぶ。一番下の行を選ぶ → その行の高さ全部が見えるまで表が流れる。`row_extra_lines = 0` → 今と同じ(ゴールデン `test_CV_3` が変わらない)。`row_extra_lines = 12` → 警告して 0(`test_cv_8_*`) | [2026-10-12](../_decisions/2026-10-12-row-lines.md) |
| CV-9 | 決まり(CE-36)に合わないセルにも、型と合わない値(CV-2)と同じ `!` の印を付けるべき(必須のキーが無いノートの空のセルにも付ける)。型と合わない値と決まりに合わない値を、画面と CLI でまとめて「問題」と呼び、理由を「型が合わない(日付ではない など)」「必須のキーが無い」「選べる値ではない」の3つで言い分けるべき。印は色だけに頼らない(SR-15)。 | `[rules.status] values = ["todo", "done"]` → `wip` のセルに `!`。`required = true` で status の無いノートの空欄に `!`。date の列の someday は今までどおり `!`(`test_cv_9_*`) | [2026-10-14](../_decisions/2026-10-14-type-check.md) |

## 範囲外

- テーマ(色の組み合わせ)は screen の SR-26・SR-27。端末のダーク / ライトの自動判定は screen の範囲外を参照。
- 採らない: 表の中での画像の表示。
- CV-7 は 2026-10-11 に削除([記録](../_decisions/2026-10-11-emoji-vs16-revert.md))

## 前提(AI の仮定)

- 空・null・キーなしの見せ方は harlequin(`∅ null`)、型と合わない値の扱いは Obsidian(不正と表示して隠さない)と bases.nvim(型が混ざる列の並べ替え)から採った(2026-09-30 の調査)。
- 列の幅の決め方は csvlens(見えている行で測り、1列の上限は画面の幅の3割)から採った。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-15 | [ビューのタブが帯の1行に入らないとき、次の行に折り返して全部を見せる設定 tab_wrap を足す(tab-wrap)](../_decisions/2026-10-15-tab-wrap.md) | accepted | AI | 人の発言 | NV-23, NV-31, SR-21, CLI-11, CV-8 |
| 2026-10-15 | [折り返すときはタブを全部、検索の欄の行の下に要る行の数だけ並べ、間の空き行を設定 tab_wrap_gap で選べるようにする(tab-wrap-below)](../_decisions/2026-10-15-tab-wrap-below.md) | accepted | 人 | 人の発言 | NV-31, NV-23, SR-21, CLI-11, CV-8 |
| 2026-10-15 | [[display] の項目の数を 12 に直した2つの試験の錠を掛け直す(note-path-locks)](../_decisions/2026-10-15-note-path-locks.md) | accepted | AI | 検証の指摘 | SR-21, CLI-11, CV-8 |
| 2026-10-15 | [ノートの欄の名前を正規表現で抜き出す・削る設定 note_name を足す(note-name)](../_decisions/2026-10-15-note-name.md) | accepted | AI | 人の発言 | SR-21, SR-29, SR-47, CLI-11, CV-8 |
| 2026-10-14 | [型と決まりに合わない値を「問題」として探し、画面と CLI で確かめられるようにする(type-check)](../_decisions/2026-10-14-type-check.md) | accepted | 人 | 人の発言 | CE-36, CV-9, NV-30, CLI-22 |
| 2026-10-12 | [長い値を折り返して、表の行を設定した行の数まで伸ばす(row-lines)](../_decisions/2026-10-12-row-lines.md) | accepted | 人 | 人の発言 | CV-8, CV-3, SR-21 |
| 2026-10-11 | [もとは幅1の字に絵文字の印を付けた書記素は、印を外して1桁で描く(emoji-vs16)](../_decisions/2026-10-11-emoji-vs16.md) | accepted | 人 | 人の発言 | CV-7 |
| 2026-10-11 | [絵文字の印を外して1桁で描く決まりを取り下げ、カラーの絵文字に戻す(emoji-vs16-revert)](../_decisions/2026-10-11-emoji-vs16-revert.md) | accepted | 人 | 人の発言 | CV-7 |
| 2026-10-03 | [--print と --pick を版1に入れる(print-pick)](../_decisions/2026-10-03-print-pick.md) | accepted | AI | AI の提案 | CLI-5, OUT-3, SR-10 |
| 2026-10-01 | [見せ方と編集の食い違いと抜けを直す(edit-view-gaps)](../_decisions/2026-10-01-edit-view-gaps.md) | accepted | AI | 検証の指摘 | SR-10, BV-7, CE-3, CE-5, CE-8, CE-10 |
| 2026-09-30 | [仕様の点検の指摘を直す(v1-review-fixes)](../_decisions/2026-09-30-v1-review-fixes.md) | accepted | AI | 検証の指摘 | SR-13, SR-10, SR-12 ほか 18 |
| 2026-09-30 | [最初の版の仕様を、表の TUI の調査に合わせて書き切る(v1-full-spec)](../_decisions/2026-09-30-v1-full-spec.md) | accepted | AI | 人の発言 | BV-4, BV-9, CE-1 ほか 52 |
| 2026-09-30 | [最初の版の設計(v1-design)](../_decisions/2026-09-30-v1-design.md) | accepted | AI | AI の提案 | BV-1, BV-2, BV-3 ほか 35 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| CV-1 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/ui/test_gaps.rs::test_cv_1_null_is_dim_but_empty_string_and_missing_are_not, src/ui/test_screen.rs::test_cv_1_null_empty_missing, tests/test_source.rs::test_cv_1_get_distinguishes_value_missing_and_null | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CV-2 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/test_big_int_unit.rs::test_cv_2_big_int_boundary_still_int, src/test_big_int_unit.rs::test_cv_2_big_int_kept_as_text, src/test_big_int_write_unit.rs::test_cv_2_big_int_text_stays_quoted ほか 16 | [2026-10-06-datetime-offsets](../_changes/2026-10-06-datetime-offsets.md) 2026-10-06 |
| CV-3 | [2026-10-12](../_decisions/2026-10-12-row-lines.md) | src/ui/test_gaps.rs::test_cv_3_detail_shows_every_line_of_multiline_value, src/ui/test_row_lines.rs::test_cv_3_newline_marked_without_extra_lines, src/ui/test_row_lines.rs::test_cv_8_newline_splits_lines ほか 1 | [2026-10-12-row-lines](../_changes/2026-10-12-row-lines.md) 2026-10-13 |
| CV-4 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/ui/test_grid.rs::test_cv_4_formula_number_and_bool_aligned, src/ui/test_grid.rs::test_cv_4_types_json_number_right_aligned, src/ui/test_screen.rs::test_cv_4_numbers_right_aligned | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| SR-9 | [2026-09-30](../_decisions/2026-09-30-v1-design.md) | src/ui/test_calendar.rs::test_ce_21_narrow_terminal_types_only, src/ui/test_calendar.rs::test_ce_23_no_calendar_shift_right_keeps_date, src/ui/test_calendar.rs::test_ce_24_ambiguous_wide_keeps_year_key ほか 13 | [2026-10-11-emoji-vs16-revert](../_changes/2026-10-11-emoji-vs16-revert.md) 2026-10-11 |
| SR-10 | [2026-10-03](../_decisions/2026-10-03-print-pick.md) | src/test_main.rs::test_sr_10_not_a_terminal, src/test_relmap_unit.rs::test_rel_7_control_chars_are_not_drawn, src/ui/test_gaps.rs::test_sr_10_dumb_term_draws_no_emoji ほか 5 | [2026-10-11-emoji-vs16-revert](../_changes/2026-10-11-emoji-vs16-revert.md) 2026-10-11 |
| SR-15 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_display_options.rs::test_sr_20_zebra_not_without_color, src/ui/test_screen.rs::test_sr_10_no_color, src/ui/test_screen.rs::test_sr_15_pending_mark_and_256_colors ほか 4 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CV-5 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/ui/test_gaps.rs::test_cv_5_width_from_visible_rows_changes_on_scroll, src/ui/test_screen.rs::test_cv_5_long_value_capped | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CV-6 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/ui/test_borders.rs::test_sr_32_ascii_when_ambiguous_wide, src/ui/test_calendar.rs::test_ce_24_ambiguous_wide_keeps_year_key, src/ui/test_display_more.rs::test_sr_20_column_lines_with_ambiguous_wide_keep_positions ほか 2 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CV-8 | [2026-10-12](../_decisions/2026-10-12-row-lines.md) | src/ui/test_row_lines.rs::test_cv_3_newline_marked_without_extra_lines, src/ui/test_row_lines.rs::test_cv_8_bottom_row_fully_visible, src/ui/test_row_lines.rs::test_cv_8_click_on_second_line_selects_row ほか 19 | [2026-10-12-row-lines](../_changes/2026-10-12-row-lines.md) 2026-10-13 |
| CV-9 | [2026-10-14](../_decisions/2026-10-14-type-check.md) | src/ui/test_problems.rs::test_cv_9_required_missing_key_marked, src/ui/test_problems.rs::test_cv_9_rule_violation_marked, tests/test_rules_config.rs::test_cv_9_reason_texts_follow_language ほか 1 | [2026-10-14-type-check](../_changes/2026-10-14-type-check.md) 2026-10-15 |
<!-- decidespec:history:end -->
