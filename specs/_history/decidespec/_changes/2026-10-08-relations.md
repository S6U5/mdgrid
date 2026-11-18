---
type: Change
id: 01M4DY54MGE7C5R4MHRQY1FVEZ
title: リレーションの最初の段(表の中のリレーション。REL-1〜REL-6・REL-10)
status: done
size: full
created: 2026-10-08
updated: 2026-10-08
---

# リレーションの最初の段(表の中のリレーション。REL-1〜REL-6・REL-10)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」(人と話して決めた) |
| 仕様化 | 済 | specs/_decisions/2026-10-08-relations.md(人)・2026-10-08-relations-details.md(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した REL-1〜REL-6・REL-10(specs/relations/spec.md の版1の節)。大きな変更なので、人の言葉なしに main へマージしない。

## 不明点と仮定

- 仮定: ただのパスの形は、`/` を含むか `.md` で終わる値だけ(`done` のような1語は、同じ名前のノートがあってもリンクにしない)。REL-1 の「普通の値を誤ってリンクにしない」に寄せた。
- 仮定: リンクの列は、その列の空でない値の過半数がリンク(仕様の前提のとおり)。
- 仮定: 行き先の無いリンクの印は `?`(名前の前。評価できない式の `?` は1字だけで薄いので見分けられる)。
- 仮定: 行き先を開く(REL-4)・つながった行(REL-5)は、操作の一覧とパレットのコマンド(既定のキーなし)。読むだけの起動でも使える(ノートを書かない)。
- 仮定: 行き先が今開いている表の中なら、開き直さずにその行へ移る。

## 設計

- src/relations.rs(ライブラリ): リンクの読み取り(`[[…]]`・`[…](…)`・パス)、行き先の解決(ノートのフォルダからの相対 → 根からのパス → 名前。名前は links::Index)、見せる名前、書く形(列の形に合わせる)、リンクの列の判定、つながった行(フォルダを読んでフロントマターから)、表どうしのつながり(REL-6)。
- src/ui/cell.rs: リンクの値を名前で見せ、行き先が無いリンクに印。解決の結果は App に覚える(読み直しで捨てる)。
- src/ui/input.rs・listpick.rs: リンクの列の候補を行き先のフォルダのノートにする(見せるのは名前、書くのはリンク)。
- src/ui/relations.rs: 行き先を開く・つながった行(パレットの続きの入力で選ぶ)。別の表へは places の切り替え(switch_to)に、選ぶ行(switch_select)を足して使う。
- src/main.rs: 開き直したあと、選ぶ行を当てる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | リレーションの核 | REL-1, REL-6 | 読み取り・解決・書く形・つながりの試験が通る | src/relations.rs, src/lib.rs, src/test_relations_unit.rs | test_rel_1_forms_and_resolution(src/test_relations_unit.rs) | 済 |
| 2 | 見せ方と入れ方 | REL-2, REL-10, REL-3 | 画面の試験が通る | src/ui/cell.rs, src/ui/input.rs, src/ui/listpick.rs, src/ui/list.rs, src/ui/app.rs, src/ui/test_relations.rs | test_rel_2_names_in_cells(src/ui/test_relations.rs) | 済 |
| 3 | 行き先を開く・つながった行 | REL-4, REL-5 | 画面と本物の実行ファイルの試験が通る | src/ui/relations.rs, src/ui/app.rs, src/ui/native_io.rs, src/ui/keymap.rs, src/ui/menu.rs, src/ui/help.rs, src/ui/mod.rs, src/ui/places.rs, src/ui/review.rs, src/main.rs, src/i18n.rs, src/ui/test_relations.rs, src/ui/test_relations_open.rs, tests/test_relations_e2e.rs, tests/golden/, docs/, README.md, README.ja.md, demos/, examples/, specs/test-locks.json | test_rel_4_open_target_in_other_table(src/ui/test_relations_open.rs)、test_rel_4_open_target_switches_table_and_selects_row(tests/test_relations_e2e.rs) | 済 |

## 実装の気づき

- 背景の安全の点検が、Markdown のリンクの `%xx` を戻すときに、`%` のあとが多バイトの文字だと落ちることを見つけた。バイトで読むように直し、試験を足した。
- 照合で見つかった gap と不具合を直した:
  - 登録した `.base` の表は、開いたときと同じく保管庫の根(`.obsidian` のある上のフォルダ)を探す(REL-1)。
  - 名前が探す表の全部で1つでなければ、`[[…]]` をノートのフォルダからの相対のパスで書く(REL-3。別の表の同じ名前に取り違えない)。
  - 描くたびに全部の行を作り直さない。今の表は読み込みが進んだときと世代(外の変更・保存・登録の変更)が進んだときだけ、登録した表は登録か世代が変わったときだけ作り直す。
  - 行き先が今の表で隠れていれば、開き直さずに理由を出す(REL-4)。
  - Markdown のリンクは表示の文字でなく行き先の名前で見せ、行き先の無いリンクは書いた行き先のまま見せる(REL-2・REL-10 の文言どおり)。
  - 操作の一覧の「つながった行」は、登録した表があれば出す(ほかの表からの被リンクは安く確かめられないため)。
- 残した点: 登録した表のフォルダの外での変更は、今の表の変更・保存・登録の変更・起動し直しのときに読み直す(常に見張らない)。編集中の入力の欄は、リンクを書いたままの文字で見せる(候補は名前)。

## 照合

書込なしの検証役が、版1の要件を節ごとに照合した(22 件の試験が通ることも確かめた)。gap 2 つと不具合 5 つを上の「実装の気づき」のとおり直し、試験を足した(src/ui/test_relations_open.rs の4つと、src/test_relations_unit.rs の .base・多対多・表示)。

| 要件 | 判定 | 根拠 |
|---|---|---|
| REL-1 | checked | コード: src/relations.rs の parse・resolve・Table::new、src/ui/relations.rs の link_target / テスト: src/test_relations_unit.rs::test_rel_1_forms_and_resolution、src/test_relations_unit.rs::test_rel_1_plain_words_are_not_links、src/test_relations_unit.rs::test_rel_1_target_outside_root、src/test_relations_unit.rs::test_rel_1_registered_base_uses_vault_root / 今: 3つの書き方を読み、相対 → 根 → 名前の順で解く。登録した .base は保管庫の根を探す / 前: 値は文字のまま |
| REL-2 | checked | コード: src/relations.rs の display、src/ui/cell.rs の shown / テスト: src/ui/test_relations.rs::test_rel_2_names_in_cells、src/test_relations_unit.rs::test_rel_2_display_names / 今: 行き先の名前で見せる(リストは並べる) / 前: 書いた文字のまま |
| REL-10 | checked | コード: src/ui/relations.rs の link_word / テスト: src/ui/test_relations.rs::test_rel_10_missing_target_marked、src/ui/test_relations_open.rs::test_rel_10_resolves_again_after_save / 今: 行き先の無いリンクに `?` / 前: 書いた文字のまま |
| REL-3 | checked | コード: src/relations.rs の format・link_column・count_name、src/ui/relations.rs の link_list・link_cands / テスト: src/ui/test_relations.rs::test_rel_3_pick_from_target_folder、src/ui/test_relations.rs::test_rel_3_list_column_many_to_many、src/ui/test_relations_open.rs::test_rel_3_same_name_in_other_table_written_as_path、src/test_relations_unit.rs::test_rel_3_format_matches_form / 今: 行き先のフォルダのノートを名前で出し、列の形で書く / 前: 列のほかの値が候補 |
| REL-4 | checked | コード: src/ui/relations.rs の start_open_link・open_note、src/main.rs の run_switching / テスト: src/ui/test_relations_open.rs::test_rel_4_open_target_in_other_table、src/ui/test_relations_open.rs::test_rel_4_pending_changes_ask_first、src/ui/test_relations_open.rs::test_rel_4_hidden_row_in_same_table_gives_reason、tests/test_relations_e2e.rs::test_rel_4_open_target_switches_table_and_selects_row / 今: 行き先の表で行を選ぶ / 前: 無し |
| REL-5 | checked | コード: src/relations.rs の backlinks、src/ui/relations.rs の start_linked_rows / テスト: src/ui/test_relations_open.rs::test_rel_5_linked_rows_lists_table_and_column、src/test_relations_unit.rs::test_rel_5_backlinks / 今: 表と列と一緒に並べて開ける / 前: 無し |
| REL-6 | checked | コード: src/relations.rs の edges / テスト: src/test_relations_unit.rs::test_rel_6_edges / 今: 設定なしで多対一・多対多を見つけ、形のそろわないノートでも止まらない / 前: 無し |

確かめた: 7 / 7。

