---
type: Spec
id: 01M4DP4Q33645DCWMZMGR5PVY8
title: リレーション(relations)
description: ノートのフロントマターに書いたリンクを、表と表をつなぐリレーションとして見せ・入れ・たどる方法。
status: active
load_when: リンクの値の見せ方・入れ方・行き先を開く・つながった行・表どうしのつながり・関係マップの画面を作る・変えるとき
created: 2026-10-08
updated: 2026-10-09
---

# リレーション(relations)

ノートのフロントマターに書いたリンク(`[[名前]]`・`[文字](パス)`・ただのパス)を、表と表をつなぐリレーションとして扱う。行を見分けるキーはノートのファイル名(パス)で、id の列は使わない。多対多はリンクのリストで書き、中間の表は使わない。登録した表([cli](../cli/spec.md) の CLI-18)をワークスペースの表とみなし、どの表がどの表を指すかはリンクの行き先から見つける。どのノートの集まりも表の形にそろうわけではないので、表にならないノートへのリンクも、リンクのまま扱う。最初は表の中のリレーション(見せる・入れる・たどる・つながった行)を作り、関係マップの画面と画面の切り替えのタブ、幅に合わせた段組みは次の段。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 表の中のリレーション

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| REL-1 | フロントマターの値(文字か、文字のリスト)のうち、`[[名前]]`・`[[パス\|表示]]`・`[[名前#見出し]]` の形、`[文字](相対のパス)` の形、実際にあるノートを指すただのパス(`projects/mdgrid.md`・`../projects/mdgrid`。`.md` は省いてよい)を、行き先のノートへのリンク(リレーション)とみなすべき。行を見分けるキーはノートのファイル名(パス)で、id の列は要らない。行き先は、値を持つノートのフォルダからの相対、開いた根からのパス、ファイル名(同じ名前が複数なら根に近い・パスの短いもの。BV-22 と同じ)の順で解く。`.base` を開いたときもフォルダを開いたときも同じに扱い、行き先は開いたフォルダの外(登録した表のフォルダ)にあってもよい。普通の値を誤ってリンクにしないため、どのノートも指さないただの文字は、リンクとみなしてはならない。 | `project: "[[mdgrid]]"` で projects/mdgrid.md がある → リンク。`parent: ../projects/mdgrid` → projects/mdgrid.md へのリンク。`status: done` で done.md が無い → ただの文字。フォルダを開いても `.base` を開いても同じ(`test_rel_1_*`) | [2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-2 | リンクの値のセルは、行き先のノートの名前(拡張子なし。`[[…\|表示]]` なら表示の文字)で見せ、リストのときは名前を並べて見せるべき。 | `project: "[[projects/mdgrid\|mdgrid]]"` → `mdgrid`。`tags: ["[[tui]]", "[[relations]]"]` → `[tui, relations]`(`test_rel_2_*`) | [2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-10 | `[[…]]` か `[…](…)` の形なのに行き先のノートが見つからないリンクは、書いた文字のまま、行き先が無いと分かる印(SR-15 と同じく色に頼らない)を付けて見せるべき。 | `project: "[[nothing]]"` で nothing.md が無い → 印付きの `nothing`(`test_rel_10_*`) | [2026-10-08](../_decisions/2026-10-08-relations-details.md) |
| REL-3 | リンクの列(値の多くがリンクの列)のセルを直すときは、候補に、その列のリンクの行き先があるフォルダのノートの名前を出し、選んだノートへのリンクを書くべき。書く形は、その列で使われている形(`[[…]]`・`[…](…)`・パス)に合わせ、無ければ `[[名前]]` にする。同じ名前のノートが複数あるときは、パスで区別できる形で書く。リストの列(多対多)では、複数のノートを選んで書けるべき。 | project の列の値がどれも `[[…]]` で projects/ を指す → 候補は projects/ のノート。選ぶと `"[[mdgrid]]"` と書く(`test_rel_3_*`) | [2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-4 | リンクのセルから、行き先のノートを含む表(登録した表のうち行き先を含むもの、無ければ行き先のフォルダ)を開いて、その行を選べるべき(操作の一覧とパレットから)。保存していない変更があれば、移る前に CLI-19 と同じに確かめる。行き先が無いときは、理由を出し、移ってはならない。 | project のセルで「行き先を開く」→ projects の表が開き mdgrid の行が選ばれている。無い行き先 → 理由を出す(`test_rel_4_*`) | [2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-5 | 選んだノートを指しているノート(つながった行・被リンク)を、どの表のどの列から指しているかと一緒に一覧で出し、選んで開けるべき。探す範囲は、開いたフォルダと、ワークスペース(WS-6 の順で決めたもの。無ければ登録した表)の表にするべき。 | projects/mdgrid.md を選んで「つながった行」→ tasks の project の列から指す3行が出る。選ぶとその行を開く(`test_rel_5_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md)、[2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-6 | ワークスペース(WS-6 の順で決めたもの。無ければ登録した表(CLI-18))の表について、ある表の列の値の行き先が別の表のノートなら、設定を書かなくても、その列をその表へのつながり(リストの列なら多対多、そうでなければ多対一)とみなすべき。どのノートの集まりも表の形にそろうわけではないため、行き先がどの表にも無いリンクや、列の形がそろっていないノートの集まりでも、リンクとして扱い、動きを止めてはならない。 | tasks と projects を登録 → tasks.project は projects への多対一。tags の列(リスト)→ 多対多。行き先がどの表にも無いリンク → リンクとして見せ、つながりには数えない(`test_rel_6_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md)、[2026-10-08](../_decisions/2026-10-08-relations.md) |


## 要件: 版1 — 関係マップ

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| REL-7 | 表どうしのつながり(REL-6)を図で見せる関係マップの画面を、表の画面とは別の画面の型(SC-9)として持ち、表ごとの箱(名前・行の数・列)と、列から行き先の表への矢印(1対多・多対多)を描き、選んだつながりの詳細(元の列・行き先の表・種類・逆向きの列)と、つながった行を見せるべき。 | 表の画面で「▦ 表 ▾」を押す → 窓に「● 表」「関係マップ R」。「関係マップ」を選ぶ → 関係マップの画面で、ボタンは「関係マップ ▾」。Esc → 窓が閉じ、画面は変わらない。つながる表が無い → 窓に関係マップが無い(`test_rel_8_*`) | [2026-10-09](../_decisions/2026-10-09-relation-map-now.md)、[2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-8 | 画面の上の端の、画面の型のメニュー(今の型の名前と「▾」のボタン。例「▦ 表 ▾」)を押すと、画面の型(表・関係マップ。関係マップは、つながる表があるときだけ)を並べた小さな窓をボタンのすぐ下に開き、選んだ型に切り替えられるべき(型が増えてもヘッダーが伸びないように)。窓では今の型に印を付け、型に切り替えのキーがあれば添え、Esc と窓の外のクリックで閉じるべき。切り替えのキー(関係マップの `R`)は、窓を開かずに効くべき。 | 読んで判定 | [2026-10-12](../_decisions/2026-10-12-header-menu.md)、[2026-10-09](../_decisions/2026-10-09-relation-map-now.md)、[2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-9 | 左の欄(ワークスペースの表の一覧)・真ん中・右の詳細・下のつながった行のような段組みは、端末の幅と高さに合わせて変えるべき(広ければ並べ、狭ければ1つずつにしてタブかキーで切り替える)。どの幅でも、文字が重なったり切れて読めなくなったりしてはならない。 | 幅 160・100・60 の端末で関係マップの画面を出す → それぞれ並べる・2つ・1つになり、どれも重ならない(`test_rel_9_*`) | [2026-10-09](../_decisions/2026-10-09-relation-map-now.md)、[2026-10-08](../_decisions/2026-10-08-relations.md) |
| REL-12 | 関係マップの画面は、マウスでも操作できるべき。表(左の一覧の行か盤の箱)のクリックでその表を選び、選んでいる表をもう一度クリックしたらその表を開き、盤の線・矢印・札か狭い画面のつながりの行のクリックでそのつながりを選び、つながった行のクリックでそのノートを開き、ホイールで表を上下に選ぶ。 | 盤の projects の箱をクリック → projects を選ぶ。もう一度クリック → projects の表へ移る。tasks.project の矢印をクリック → そのつながりの詳細。枠や空きのクリック → 何も変わらない(`test_rel_12_*`) | [2026-10-09](../_decisions/2026-10-09-relmap-click.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| REL-11 | mdgrid でノートの名前(ファイル名)を変えるときは、そのノートを指すリンク(開いたフォルダと登録した表の中)を、差分を見せてから新しい名前に書き直せるべき。 | 読んで判定 | [2026-10-08](../_decisions/2026-10-08-relations-details.md) |

## 範囲外

- 採らない: id の列(P-01 など)で行き先を指す形。キーはファイル名(REL-1)。
- 採らない: 多対多のための中間の表。リンクのリストで書く(REL-3)。

## 前提(AI の仮定)

- リンクの列かどうか(REL-3)は、その列の空でない値の過半数がリンクかで決める。
- REL-4 の「行き先を開く」の既定のキーは設計で決める(Enter は今のまま編集)。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-12 | [関係マップの「+ 新規」の行と窓の位置が変わる試験の錠を掛け直す(relmap-new-row-locks)](../_decisions/2026-10-12-relmap-new-row-locks.md) | accepted | 人 | AI の提案 | CE-25, REL-8, REL-12 |
| 2026-10-12 | [設定を右上の別のボタンに、画面の型をメニューに、「+ 新規」をビューの帯に移し、ボタンのキーの字を目立たせる(header-menu)](../_decisions/2026-10-12-header-menu.md) | accepted | 人 | 人の発言 | SR-1, SR-42, NV-23, REL-8, CE-25 |
| 2026-10-12 | [余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)](../_decisions/2026-10-12-half-row-locks.md) | accepted | 人 | AI の提案 | BV-7, BV-13, BV-17 ほか 28 |
| 2026-10-10 | [公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)](../_decisions/2026-10-10-config-v2-relock.md) | accepted | 人 | 人の発言 | BV-17, BV-19, BV-20 ほか 42 |
| 2026-10-09 | [ワークスペース(表をまとめる範囲)をアプリの側で持ち、画面とコマンドで管理し、.git と保管庫を検知できるようにする(workspaces)](../_decisions/2026-10-09-workspaces.md) | accepted | 人 | 人の発言 | WS-1, WS-2, WS-3 ほか 5 |
| 2026-10-09 | [関係マップをクリックで操作する(relmap-click)](../_decisions/2026-10-09-relmap-click.md) | accepted | 人 | 人の発言 | REL-12 |
| 2026-10-09 | [関係マップの画面・切り替えのタブ・幅に合わせた段組みを版1に入れる(relation-map-now)](../_decisions/2026-10-09-relation-map-now.md) | accepted | 人 | 人の発言 | REL-7, REL-8, REL-9 |
| 2026-10-08 | [ノートのリンクをリレーションとして扱う(relations)](../_decisions/2026-10-08-relations.md) | accepted | 人 | 人の発言 | REL-1, REL-2, REL-3 ほか 6 |
| 2026-10-08 | [リレーションの細部: 行き先の無いリンクの印と、名前の変更でのリンクの書き直し(relations-details)](../_decisions/2026-10-08-relations-details.md) | accepted | AI | AI の提案 | REL-10, REL-11 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| REL-1 | [2026-10-08](../_decisions/2026-10-08-relations.md) | src/test_relations_unit.rs::test_rel_1_forms_and_resolution, src/test_relations_unit.rs::test_rel_1_percent_with_multibyte_does_not_panic, src/test_relations_unit.rs::test_rel_1_plain_words_are_not_links ほか 4 | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-2 | [2026-10-08](../_decisions/2026-10-08-relations.md) | src/test_relations_unit.rs::test_rel_2_display_names, src/ui/test_relations.rs::test_rel_2_names_in_cells | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-10 | [2026-10-08](../_decisions/2026-10-08-relations-details.md) | src/test_relations_unit.rs::test_rel_2_display_names, src/ui/test_relations.rs::test_rel_10_missing_target_marked, src/ui/test_relations_open.rs::test_rel_10_resolves_again_after_save | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-3 | [2026-10-08](../_decisions/2026-10-08-relations.md) | src/test_relations_unit.rs::test_rel_3_format_matches_form, src/test_relations_unit.rs::test_rel_3_link_column, src/ui/test_relations.rs::test_rel_3_list_column_many_to_many ほか 2 | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-4 | [2026-10-08](../_decisions/2026-10-08-relations.md) | src/ui/test_relations_open.rs::test_rel_4_choose_among_targets, src/ui/test_relations_open.rs::test_rel_4_hidden_row_in_same_table_gives_reason, src/ui/test_relations_open.rs::test_rel_4_menu_has_open_link_on_link_cell ほか 4 | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-5 | [2026-10-09](../_decisions/2026-10-09-workspaces.md) | src/test_relations_unit.rs::test_rel_5_backlinks, src/ui/test_relations_open.rs::test_rel_5_linked_rows_lists_table_and_column, src/ui/test_relations_open.rs::test_rel_5_menu_offers_linked_rows_with_registered_tables ほか 2 | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-6 | [2026-10-09](../_decisions/2026-10-09-workspaces.md) | src/test_relations_unit.rs::test_rel_6_edges | [2026-10-08-relations](../_changes/2026-10-08-relations.md) 2026-10-08 |
| REL-7 | [2026-10-09](../_decisions/2026-10-09-relation-map-now.md) | src/test_relmap_unit.rs::test_rel_7_ascii_lines, src/test_relmap_unit.rs::test_rel_7_back_and_self_links_do_not_panic, src/test_relmap_unit.rs::test_rel_7_control_chars_are_not_drawn ほか 6 | なし |
| REL-8 | [2026-10-12](../_decisions/2026-10-12-header-menu.md) | src/ui/test_header_menu.rs::test_rel_8_menu_back_to_table, src/ui/test_header_menu.rs::test_rel_8_menu_esc_closes_without_change, src/ui/test_header_menu.rs::test_rel_8_menu_lists_types_and_switches ほか 5 | [2026-10-12-header-menu](../_changes/2026-10-12-header-menu.md) 2026-10-12 |
| REL-9 | [2026-10-09](../_decisions/2026-10-09-relation-map-now.md) | src/ui/test_relmap.rs::test_rel_9_layout_by_width, src/ui/test_relmap.rs::test_rel_9_linked_records_when_tall, src/ui/test_relmap_scroll.rs::test_rel_9_narrow_list_scrolls_to_selection | [2026-10-09-relmap-narrow-scroll](../_changes/2026-10-09-relmap-narrow-scroll.md) 2026-10-09 |
| REL-12 | [2026-10-09](../_decisions/2026-10-09-relmap-click.md) | src/ui/test_relmap_click.rs::test_rel_12_click_link_and_linked_row, src/ui/test_relmap_click.rs::test_rel_12_click_selects_and_opens, src/ui/test_relmap_click.rs::test_rel_12_narrow_list_and_wheel ほか 1 | [2026-10-09-relmap-click](../_changes/2026-10-09-relmap-click.md) 2026-10-09 |
| REL-11 | [2026-10-08](../_decisions/2026-10-08-relations-details.md) | なし | なし |
<!-- decidespec:history:end -->
