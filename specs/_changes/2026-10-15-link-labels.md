---
type: Change
id: 01M4XQS7V2SJ3QN9TZNNSVEGS2
title: リンクの見出しと件数の表が [[…]] のまま、.base から行き先を開けない、関係マップに余分な表
status: done
size: bugfix
created: 2026-10-15
updated: 2026-10-15
---

# リンクの見出しと件数の表が [[…]] のまま、.base から行き先を開けない、関係マップに余分な表

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: bugfix(REL-2・REL-4・REL-7 のとおりに動かない不具合。要件の意味を変えない) | |
| 設計 | 対象外: bugfix | |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

見本の作り直し(specs/_changes/2026-10-15-examples-studio.md の「実装の気づき」)で見つけ、別の作業に出した本体の不具合を直す(人の依頼。会話 2026-10-15、プロジェクトのスレッド)。examples/studio で再現した。

1. リンクの列でまとめたまとまりの見出し(Tasks.base の By project など)と、`%` の件数の表が、リンクを行き先の名前でなく `[[Bakery Website Redesign]]` のように出す。セル(REL-2)と同じ名前で見せるのが終わり。
2. `.base` の表(Tasks.base)から、People のノートへのリンクを「行き先を開く」と、「この表にあるが絞り込みで隠れている」と出て移れない。`.base` の表はルートフォルダの全部のノートを読むので、`.base` の外のフォルダのノートも「今の表のノート」とみなしていた。REL-4 のとおり、行き先を含む登録した表(無ければ行き先のフォルダ)に移るのが終わり。
3. ルートフォルダの下の一部のフォルダだけのワークスペース(`mdgrid Tasks Projects People --add-to Work` → `mdgrid -w Work`)で関係マップを開くと、箱に余分な「Tasks (2)」(ルートフォルダの残りの Clients と Meetings の10件)が出る。今の表のフォルダを、開いたフォルダでなく読み込んだノートの根(ルートフォルダ)で決めていたため。範囲の表だけを出すのが終わり。

関係する要件: REL-2、REL-4、REL-7。

## 不明点と仮定

- 仮定: 見出しの文字は値をつないだもの(リストは `, ` でつなぐ。settings::group_label と base の見出しが同じ)なので、まず全体をリンクとして読み、だめなら `, ` で分けた要素ごとに読む。要素の中に `, ` を含むリンクは名前にならず今までどおり出る(まれ)。相対のリンクは、まとまりの最初の行(件数の表は、その値を持つ最初の行)から解く。外れたら: 見出しを列の値から作り直す。
- 仮定: 「今の表のノート」は、`.base` を開いていれば `.base` の全体の絞り込み(`filters`。ビューのでなく)を通るノート。ビューの絞り込み・簡易の絞り込み・畳んだまとまりで見えないだけなら、今までどおり「隠れている」の理由を出す。
- 仮定: 今の表のフォルダは、フォルダを1つ開いたならそのフォルダ(`.base` はルートフォルダ。relations::Table::new と同じ)。フォルダを2つ以上開いたときは今までどおり読み込んだノートの根。今の表のフォルダが範囲のどれかの表のフォルダの下なら、その表のノートとして数え、今の表の箱を足さない(印のワークスペースで Tasks のフォルダを開いたとき、Tasks.base の箱を選ぶ)。
- 人に聞くことはなし。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | まとまりの見出しと件数の表のリンクを名前で見せる | REL-2 | 見出しが `project: mdgrid`、件数の表の項目が `mdgrid`・`?ghost` | src/ui/relations.rs src/ui/view.rs src/ui/freq.rs src/ui/test_link_labels.rs | test_rel_2_group_heading_shows_link_names・test_rel_2_list_group_heading_shows_link_names・test_rel_2_frequency_shows_link_names | 済 |
| 2 | `.base` の表から外のフォルダのノートへのリンクを開く | REL-4 | 登録した Projects に移る / 無ければ行き先のフォルダ / ビューで隠れたものは理由 | src/base.rs src/ui/relations.rs src/ui/test_link_labels.rs | test_rel_4_open_link_from_base_to_registered_table・test_rel_4_open_link_from_base_without_registered_tables・test_rel_4_base_row_hidden_by_view_filter_gives_reason | 済 |
| 3 | 今の表のフォルダを開いたフォルダにし、範囲の表の中なら箱を足さない | REL-7 | 一部のフォルダの範囲で箱は3つ、今の表を選ぶ | src/ui/relations.rs src/ui/relmap.rs src/ui/test_link_labels.rs | test_rel_7_partial_workspace_has_no_extra_table・test_rel_7_folder_inside_base_table_has_no_extra_table | 済 |

## 実装の気づき

- 今の表のフォルダを開いたフォルダにすると、ノートの無いフォルダでも表が1つできて、関係マップの「表が無い」(test_rel_7_no_tables_message)が落ちた。ノートが無ければ今の表は無い、のままにした。
- 範囲の表の中に今のフォルダがあるとき(印のワークスペースで Tasks のフォルダを開く)、前は根のフォルダが `.base` の表と同じなので足さずに済んでいた。今のフォルダで比べると Tasks の箱が2つになるので、「範囲の表のフォルダの下なら足さない」にした(test_rel_7_folder_inside_base_table_has_no_extra_table)。
- 同じ作業場所で別の作業が書きかけの変更(試験が先にあり実装がまだのもの)を持っていてビルドが通らないので、HEAD と自分の変更だけの写しで試験・clippy・書式を確かめた。

## 照合

`check.py --evidence` の行(bugfix で全部 checked のため検証役は起動しない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| REL-2 | checked | コード: src/ui/relations.rs:202 / テスト: src/ui/test_link_labels.rs::test_rel_2_group_heading_shows_link_names ほか2件(リストの見出し、`%` の件数の表) / 今: 通った / 前: 落ちた(見出しの直し src/ui/view.rs と件数の表の直し src/ui/freq.rs を外すと3件とも落ちる) |
| REL-4 | checked | コード: src/ui/relations.rs:485、src/base.rs:890 / テスト: src/ui/test_link_labels.rs::test_rel_4_open_link_from_base_to_registered_table ほか2件 / 今: 通った / 前: 落ちた(.base の全体の絞り込みの判定を外すと、登録した表へ・行き先のフォルダへの2件が落ちる。ビューで隠れたときの理由の1件は前も通る守りの試験) |
| REL-7 | checked | コード: src/ui/relations.rs:119、src/ui/relations.rs:135 / テスト: src/ui/test_link_labels.rs::test_rel_7_partial_workspace_has_no_extra_table ほか1件 / 今: 通った / 前: 落ちた(今の表のフォルダを読み込んだノートの根に戻すと、一部のフォルダの範囲の1件が落ちる。.base の中のフォルダの1件は前も通る守りの試験) |

見本でも確かめた: examples/studio で Tasks.base の By project の見出しが「project: Bakery Website Redesign (7)」、owner の `%` が「Sam Ortiz 6 (30%)」、owner のセルから「行き先を開く」で People の表が開き Hana Ito が選ばれる、`mdgrid Tasks Projects People --add-to Work` のあと `mdgrid -w Work` の関係マップの箱が Tasks・Projects・People の3つ、印のワークスペースで Tasks のフォルダを開いた関係マップの箱が5つ(余分な箱なし)。全体の試験(cargo test)・clippy(-D warnings)・触ったファイルの書式は、HEAD と自分の変更だけの写しで通った。

既存のテストの削除・skip・弱体化: なし(足したのは新しいファイルだけ)

確かめた: 3 / 3
