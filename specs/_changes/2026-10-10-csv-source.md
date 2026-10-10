---
type: Change
id: 01M4JG30YR93KCHN4NDPRDV6KE
title: CSV・TSV のファイルを表として開いて直す(SC-15〜SC-17)
status: done
size: full
created: 2026-10-10
updated: 2026-10-10
---

# CSV・TSV のファイルを表として開いて直す(SC-15〜SC-17)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-10-csv-source.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット(ブランチ feature/csv。大きな変更なのでマージは人の許可を待つ) |
| 照合 | 済 | この記録の「照合」 |

## 要求

`mdgrid 台帳.csv` で CSV・TSV を表として開き、直したセルだけを書き戻す(SC-15〜SC-17)。CSV とノートの間のリレーションは未定(人)なので入れない。

## 不明点と仮定

- 仮定: 行の鍵は「ファイルの実体のパス#行の番号」。番号は開いたときのデータの何行目か(1から)で、そのあと変わらない(行を足すと新しい番号)。外でファイルが変わったら、1文字も変わっていない行だけが番号を引き継ぎ、中身の変わった・消えた行の直しはどの行にも書かない(位置で引くと、外で行が足されたときに別の行に書く。コミットの検査の指摘)。`--with-path` の `#n` は今の何行目か。
- 仮定: 値は文字として持ち、列の型は値から推し量る(数・日付・真偽)。リストは扱わない(`a, b` は1つの文字)。
- 仮定: 全部の行が同じファイルなので、保存は書く単位(ファイル)ごとにまとめて1回で書く(設計の「書く単位」。初めの「基準を覚える」案はやめた)。
- 仮定: `a` で足す行はすぐファイルの末尾に書く(新しいノートをすぐ作るのと同じ)。ためた直しは、自分が末尾に足しただけなので基準を新しい内容に進める。

## 設計

- 核 `mdgrid::csvfile`: バイトを読んで、行ごと・値ごとのバイトの位置(引用符の有無を含む)と改行の形を持つ。値を書き換えるときは、その値のバイトの範囲だけを置き換え、区切り・引用符・改行を含む値だけを引用符で囲む。
- 読み込み口 `source::csv::Csv`(Source の実装)。label は1列目の値(空なら行の番号)。file はファイルの属性。リンクの索引は持たない。
- 書く単位: 保存の流れ(ためる変更・保存の確認・外の変更の上に書く・捨てる)は、行ではなく読み込み口の `unit(row)`(行が書かれる先)ごとにまとめる。Markdown は行そのもの(今までと同じ)、CSV はファイル1つ。差分と保存は `preview_unit`・`save_unit` で単位ごとに1回(全部の行の直しを1つのバイトに当てて、1回書く)。基準(WB-4)は単位の中の行で同じでなければ止める。行ごとに同じファイルを書き直したり、自分の書いた基準を覚えて受けたりしない。
- main の `open_target`: `.csv`・`.tsv` のファイルを渡されたら Csv で開く。
- 画面: ノートだけの機能は、読み込み口が CSV のときに出さないか理由を出す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 読み書きの芯 | SC-15, SC-16 | 引用符・改行・BOM・CRLF を読み、1つの値だけを書き換えるとほかのバイトが変わらない | src/csvfile.rs | test_sc_15_*・test_sc_16_*(src/test_csvfile_unit.rs) | 済 |
| 2 | 読み込み口と開く口 | SC-15, SC-16 | `mdgrid 台帳.csv` で開いて直して保存できる | src/source/csv.rs, src/source.rs, src/source/markdown.rs, src/main.rs, src/i18n.rs, src/ui/mod.rs, src/changes.rs, src/ui/review.rs, docs/design.md | test_sc_15_*・test_sc_16_*(src/ui/test_csv_source.rs) | 済 |
| 3 | ノートだけの機能を出さない | SC-17 | e・R・名前の変更・キーの操作で理由。一覧とパレットに出さない。左のノートの欄に名前を重ねない | src/source.rs, src/source/csv.rs, src/ui/app.rs, src/ui/keymap.rs, src/ui/menu.rs, src/ui/help.rs, src/ui/view.rs, src/i18n.rs, src/ui/native_views.rs, src/ui/places.rs, src/ui/test_view_tabs_auto.rs, docs/catalog/index.html, docs/config.md, docs/manual-scenarios.toml, docs/manual/en/ | test_sc_17_*(src/ui/test_csv_note_only.rs)、test_sr_23_default_tab_english_name | 済 |
| 4 | 行を足す | SC-17 | a で末尾に1行 | src/ui/add_row.rs, src/ui/mod.rs, src/ui/grid.rs, src/source/csv.rs, src/csvfile.rs, src/changes.rs | test_sc_17_add_row*・test_sc_17_single_column_empty_row_kept | 済 |
| 5 | 見本・説明書・本物の実行ファイル | SC-15 | examples の台帳、説明書、e2e、録画の台本 | examples/, docs/manual/, README.md, README.ja.md, demos/, src/main.rs, src/i18n.rs, tests/ | tests/test_csv_e2e.rs | 済 |

## 実装の気づき

- 保存の流れが「1行=1ファイル」を前提にしていたので、書く単位(`Source::unit`)を足して作り直した(設計の「書く単位」)。
- 直した行を留める(NV-12)の組み立てで、留める行のうち前の表に無かった行(足した行・取り消しで戻った行)を抜いたまま戻さず、表から消えていた。前の表にあった行だけを抜くように直した(録画で見つけた)。
- 英語の既定の表の名前「All notes」はノートでない表に合わないので、人の決定で「Default table」に変えた([default-tab-name-locks](../_decisions/2026-10-10-default-tab-name-locks.md))。前の名前は別名として受ける。
- 録画の一覧(scripts/pages.py の GIF)には、次のリリースで demo-csv.gif ができてから足す(今足すとサイトの検査が落ちる)。

## 照合

照合は書込なしの検証役(2026-10-10)。指摘(高1・中3・低5)は全部直し、直した点ごとに試験を足した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SC-15 | checked | test_sc_15_open_csv_as_table・test_sc_15_unreadable_csv_reason・test_sc_15_parse_*(src/test_csvfile_unit.rs)・test_sc_15_lone_cr_refused・test_sc_15_duplicate_and_empty_headers(src/test_csv_edges_unit.rs)・test_sc_15_e2e_print_and_unreadable。直した: CR だけの改行は理由を出して読まない、同じ名前・名前の無い列を別の列にする |
| SC-16 | checked | test_sc_16_save_only_edited_cells・test_sc_16_external_change_*・test_sc_16_set_changes_only_that_value・test_sc_16_single_column_clear_keeps_rows・test_sc_16_many_edits_use_original_positions・test_sc_16_quote_only_when_needed・test_sc_16_float_stays_decimal・test_sc_16_preview_against_changed_file_matches_write・test_sc_16_e2e_edit_and_save_csv。直した: 1列の表で値を空にすると行が消えて後ろの直しが別の行に書かれた(高)、空白だけで引用符を付けていた、2.0 を 2 と書いていた |
| SC-17 | checked | 使える: test_sc_17_typed_input_date_and_checkbox・test_sc_17_bulk_edit_and_undo・test_sc_17_filter_and_sort・test_sc_17_saved_view_with_group・test_sc_17_with_path_and_apply_roundtrip・test_sc_17_add_row*。出さない・断る: test_sc_17_note_only_actions_refused・test_sc_17_note_only_actions_hidden・test_sc_17_settings_without_tree_section・test_sc_17_relation_map_and_new_note_paths・test_sc_17_csv_tables_left_out_of_links_and_map・test_sc_17_no_note_label_column。直した: 設定の画面の親子の区画、関係マップのタブのクリック、--with-path と --apply の行の位置、ワークスペースの CSV を関係マップとリンクの範囲から外す |

既存のテストの削除・skip・弱体化: なし(test_sr_23_default_tab_english_name は人の決定 default-tab-name-locks で掛け直し、前の名前の確かめを増やした)

確かめた: 3 / 3
