---
type: Spec Index
id: 01M3S4GFYN946RRT31SAPFH5F0
title: mdgrid の仕様
description: mdgrid の仕様の正本の入口。
status: active
created: 2026-09-30
updated: 2026-09-30
---

# mdgrid の仕様

このリポの仕様の正本。ケイパビリティ(何ができるか)ごとに `<capability>/spec.md` を置く。変えるときは `_proposals/` に提案を書き、採択したら `_decisions/` へ移す。誤字・表現だけの直しは提案なしで直し、コミットを `specs: 字句` で始める。下の区画は `check.py --write` が書く。

<!-- decidespec:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

### ケイパビリティ

| ケイパビリティ | 内容 | 読むとき | 要件 | うち守られる |
|---|---|---|---|---|
| [base-view](base-view/spec.md) | 何を読んで、どの行とどの列の表を作るか。.base の table ビューの解釈、ビューの切り替え、解釈できないときの振る舞い。 | 起動の引数・.base の読み込み・絞り込み・並べ替え・グループ分け・式の評価・ビューの切り替えを作る・変えるとき | 23 | 9 |
| [cell-edit](cell-edit/spec.md) | 表のセルでどの値をどう直せるか。自由入力・リストからの選択・切り替え・日付・空にする・一括の設定と、読むだけにするセル。 | セルの編集・候補の出し方・入力ボックス・一括の編集を作る・変えるとき | 31 | 17 |
| [cell-view](cell-view/spec.md) | セルの値をどう見せるか。空・null・キーなしの区別、型と合わない値、改行、寄せ、列の幅、文字の幅、色。 | セルの描画・列の幅・文字の幅・色を作る・変えるとき | 9 | 0 |
| [cli](cli/spec.md) | 起動のしかた(引数・オプション)と、設定ファイルの置き場所と中身、起動できないときの振る舞い。 | 起動の引数・オプション・設定ファイル・終了コードを作る・変えるとき | 22 | 8 |
| [navigation](navigation/spec.md) | 表の中を動く・探す・絞る・並べ替える・列を扱う・行を選ぶ操作。どれも .base は変えず、画面の中だけで効く。 | 移動・検索・簡易の絞り込み・一時的な並べ替え・列の表示・行の選択・詳細の表示・ビューの設定を作る・変えるとき | 28 | 15 |
| [output](output/spec.md) | セルや行をクリップボードにコピーする方法と、表を外に出す方法。 | コピー・貼り付け・書き出し・標準出力への出力を作る・変えるとき | 5 | 1 |
| [relations](relations/spec.md) | ノートのフロントマターに書いたリンクを、表と表をつなぐリレーションとして見せ・入れ・たどる方法。 | リンクの値の見せ方・入れ方・行き先を開く・つながった行・表どうしのつながり・関係マップの画面を作る・変えるとき | 12 | 10 |
| [scope](scope/spec.md) | mdgrid が何のための道具で、最初の版に何を入れ、何を入れないか。 | 機能を足す・削るとき、最初の版の範囲を判断するとき、名前や配り方を決めるとき | 14 | 8 |
| [screen](screen/spec.md) | 画面の構成、キーとマウス、ヘルプとコマンドのパレット、見た目の状態の保存。セルの値・文字の幅・色の見せ方は cell-view。 | 画面の構成・キー・マウス・ヘルプ・パレット・見た目の状態を作る・変えるとき | 40 | 24 |
| [workspace](workspace/spec.md) | 表をまとめる範囲(ワークスペース)をアプリの側に持ち、画面とコマンドで管理し、.git と保管庫を検知する。 | ワークスペースの持ち方・管理(画面とコマンド)・検知・範囲の決め方を作る・変えるとき | 7 | 7 |
| [write-back](write-back/spec.md) | 直した値をためて、差分を見てから保存する流れと、ノートのファイルにどう書き戻すか。何を1バイトも変えず、いつ書かずに止めるか。 | 保存・取り消し・ファイルへの書き込みを作る・変えるとき、YAML の読み書きの部品を選ぶとき | 19 | 10 |

取り込みで決定者が未確認の要件: なし

### 審議中

なし

### 決定の一覧(新しい順)

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触る要件 |
|---|---|---|---|---|---|
| 2026-10-10 | [設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)](_decisions/2026-10-10-config-v2-locks.md) | accepted | 人 | 人の発言 | BV-17, BV-20, CE-16 ほか 34 |
| 2026-10-10 | [表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)](_decisions/2026-10-10-scope-overrides.md) | accepted | 人 | 人の発言 | SR-44, SR-43, WS-1 ほか 2 |
| 2026-10-10 | [設定の形を作り直し、アプリ全体の項目と表のプロファイルに分け、重なる項目をまとめる(config-v2)](_decisions/2026-10-10-config-v2.md) | accepted | 人 | 人の発言 | CLI-3, CLI-11, CLI-12 ほか 16 |
| 2026-10-10 | [ワークスペースのボタンは、ワークスペースが無くても出す(ws-button)](_decisions/2026-10-10-ws-button.md) | accepted | 人 | 人の発言 | WS-6 |
| 2026-10-10 | [親子で並べた表に WBS の番号と進み具合を出し、値の対応表を設定で作って保存する(wbs)](_decisions/2026-10-10-wbs.md) | accepted | 人 | 人の発言 | NV-28 |
| 2026-10-10 | [設定の画面に「ビュー」の区画を足し、タブの並び・出す隠す・既定・名前・削除をそこで変える(view-tabs)](_decisions/2026-10-10-view-tabs.md) | accepted | 人 | 人の発言 | NV-26 |
| 2026-10-10 | [mdgrid のビューの削除を確かめ、直後の取り消しで戻せるようにする(view-delete-undo)](_decisions/2026-10-10-view-delete-undo.md) | accepted | 人 | 人の発言 | BV-18 |
| 2026-10-10 | [ビューの削除の確かめに合わせて、削除を通る試験の錠を掛け直す(view-delete-undo-locks)](_decisions/2026-10-10-view-delete-undo-locks.md) | accepted | 人 | 人の発言 | BV-3, BV-13, BV-17 ほか 6 |
| 2026-10-10 | [親子のノートを字下げして並べ、畳めるようにする(tree)](_decisions/2026-10-10-tree.md) | accepted | 人 | 人の発言 | NV-27 |
| 2026-10-10 | [親子の畳むキーを z から Z にする(tree-key)](_decisions/2026-10-10-tree-key.md) | accepted | 人 | 検証の指摘 | NV-27 |
| 2026-10-10 | [部品の形の既定(sumi)に合わせて、SR-33・SR-35・SR-36 の例を直す(style-examples)](_decisions/2026-10-10-style-examples.md) | accepted | 人 | 検証の指摘 | SR-33, SR-35, SR-36 |
| 2026-10-10 | [並べ替えを選んで覚える(並べ替えの窓・見出しの並べ替えの保存・既定のビュー)(sort-views)](_decisions/2026-10-10-sort-views.md) | accepted | 人 | 人の発言 | NV-3, NV-20, SR-12 ほか 4 |
| 2026-10-10 | [ビューの設定の画面を、左に区画の一覧・右に中身・上に反映の形にする(settings-layout)](_decisions/2026-10-10-settings-layout.md) | accepted | 人 | 人の発言 | NV-18 |
| 2026-10-10 | [設定の画面の作り直しに合わせて SR-20・SR-23 の文を直す(settings-layout-follow)](_decisions/2026-10-10-settings-layout-follow.md) | accepted | 人 | 人の発言 | SR-20, SR-23 |
| 2026-10-10 | [設定の画面の作り直しに合わせて、関わる要件の例(確かめ方)の操作を直す(settings-layout-examples)](_decisions/2026-10-10-settings-layout-examples.md) | accepted | 人 | 人の発言 | NV-13, NV-16, NV-23 ほか 4 |
| 2026-10-10 | [アプリからノートの名前を変え、そのノートを指すリンクの書き換えをためる(note-rename)](_decisions/2026-10-10-note-rename.md) | accepted | 人 | 人の発言 | CE-34 |
| 2026-10-10 | [nerd_font = "auto" で、丸い端を自分で描く端末なら丸い札にする(nerd-auto)](_decisions/2026-10-10-nerd-auto.md) | accepted | 人 | 人の発言 | SR-36 |
| 2026-10-10 | [設定の画面に「見た目」の区画を足し、テーマ・組・丸い札の端を選んで look.toml に残し、組み合わせを名前で保存して選べるようにする(look-section)](_decisions/2026-10-10-look-section.md) | accepted | 人 | 人の発言 | SR-43 |
| 2026-10-10 | [組 dozy-pink を足す(dozy-preset)](_decisions/2026-10-10-dozy-preset.md) | accepted | 人 | 人の発言 | SR-36 |
| 2026-10-10 | [公開リポの CI に合わせて、整形と unix だけの指定を当てた試験の錠を掛け直す(ci-locks)](_decisions/2026-10-10-ci-locks.md) | accepted | 人 | 運用で起きた失敗 | SR-38, SR-39 |
| 2026-10-09 | [ワークスペース(表をまとめる範囲)をアプリの側で持ち、画面とコマンドで管理し、.git と保管庫を検知できるようにする(workspaces)](_decisions/2026-10-09-workspaces.md) | accepted | 人 | 人の発言 | WS-1, WS-2, WS-3 ほか 5 |
| 2026-10-09 | [フォルダの .mdgrid/workspace.toml をワークスペースの印として受け、アプリの側のワークスペースより先に使う(workspace-marker)](_decisions/2026-10-09-workspace-marker.md) | accepted | 人 | 人の発言 | WS-6, WS-7 |
| 2026-10-09 | [ワークスペースの操作を、開くパスと重ならない旗にする(workspace-flags)](_decisions/2026-10-09-workspace-flags.md) | accepted | 人 | 人の発言 | WS-3, WS-7 |
| 2026-10-09 | [自動の表に .base を入れない(workspace-auto-folders)](_decisions/2026-10-09-workspace-auto-folders.md) | accepted | 人 | 検証の指摘 | WS-5, WS-7 |
| 2026-10-09 | [Windows でも通るように、Unix の前提を置いた2つの試験を直して錠を掛け直す(windows-test-locks)](_decisions/2026-10-09-windows-test-locks.md) | accepted | 人 | 人の発言 | WS-1 |
| 2026-10-09 | [ビューが1つならタブの行を出さない設定(view-tabs-auto)](_decisions/2026-10-09-view-tabs-auto.md) | accepted | 人 | 人の発言 | SR-34 |
| 2026-10-09 | [テーマの名前の並びに、新しい組と auto を足す(theme-list)](_decisions/2026-10-09-theme-list.md) | accepted | 人 | 人の発言 | SR-26, SR-27 |
| 2026-10-09 | [部品の形とテーマを設定で選べるようにし、選ぶためのカタログをリポに置く(style-catalog)](_decisions/2026-10-09-style-catalog.md) | accepted | 人 | 人の発言 | SR-33, SR-35, SR-36, SR-37, SR-38, SR-39 |
| 2026-10-09 | [表のセルを値の型に合わせた部品で見せる(rich-cells)](_decisions/2026-10-09-rich-cells.md) | accepted | 人 | 人の発言 | SR-35 |
| 2026-10-09 | [セルの部品を、種類ごと・列ごとに設定で選べるようにする(rich-cells-config)](_decisions/2026-10-09-rich-cells-config.md) | accepted | 人 | 人の発言 | SR-35 |
| 2026-10-09 | [関係マップでも「+ 新規」を出し、選んでいる表に作る(relmap-new-note)](_decisions/2026-10-09-relmap-new-note.md) | accepted | 人 | 人の発言 | CE-25 |
| 2026-10-09 | [関係マップをクリックで操作する(relmap-click)](_decisions/2026-10-09-relmap-click.md) | accepted | 人 | 人の発言 | REL-12 |
| 2026-10-09 | [関係マップの画面・切り替えのタブ・幅に合わせた段組みを版1に入れる(relation-map-now)](_decisions/2026-10-09-relation-map-now.md) | accepted | 人 | 人の発言 | REL-7, REL-8, REL-9 |
| 2026-10-09 | [lazygit のようなモダンな見た目を既定にし、今の見た目は設定で選べるようにする(modern-look)](_decisions/2026-10-09-modern-look.md) | accepted | 人 | 人の発言 | SR-33 |
| 2026-10-09 | [モダンな見た目を既定にしたのに合わせて、色の試験の比べる元を今までの見た目にして錠を掛け直す(modern-look-locks)](_decisions/2026-10-09-modern-look-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-26, BV-19, NV-18 |
| 2026-10-09 | [窓の枠をつながった罫線にし、ASCII は設定で選べるようにする(modern-borders)](_decisions/2026-10-09-modern-borders.md) | accepted | 人 | 人の発言 | SR-32 |
| 2026-10-09 | [窓の枠をつながった罫線にしたのに合わせて、枠の文字を読む試験の錠を掛け直す(modern-borders-locks)](_decisions/2026-10-09-modern-borders-locks.md) | accepted | 人 | 人の発言 | CE-10, CE-20, CE-22 ほか 7 |
| 2026-10-09 | [ヘッダーにワークスペースと設定のボタン(header-buttons)](_decisions/2026-10-09-header-buttons.md) | accepted | 人 | 人の発言 | SR-42 |
| 2026-10-09 | [色の上書き [colors] と値の色、カタログの色の欄、透けない札 solid(colors)](_decisions/2026-10-09-colors.md) | accepted | 人 | 人の発言 | SR-36, SR-38, SR-40, SR-41 |
| 2026-10-08 | [ノートのリンクをリレーションとして扱う(relations)](_decisions/2026-10-08-relations.md) | accepted | 人 | 人の発言 | REL-1, REL-2, REL-3 ほか 6 |
| 2026-10-08 | [リレーションの細部: 行き先の無いリンクの印と、名前の変更でのリンクの書き直し(relations-details)](_decisions/2026-10-08-relations-details.md) | accepted | AI | AI の提案 | REL-10, REL-11 |
| 2026-10-08 | [よく使う表を名前と分類で登録し、一覧から開く(places)](_decisions/2026-10-08-places.md) | accepted | 人 | 人の発言 | CLI-1, CLI-18, CLI-19 |
| 2026-10-08 | [セルの入力欄と新しいノートの窓の見た目を良くする(edit-screen-design)](_decisions/2026-10-08-edit-screen-design.md) | accepted | AI | 人の発言 | SR-31 |
| 2026-10-08 | [新しいノートの窓の枠に合わせて、窓の欄を確かめる試験の錠を掛け直す(edit-screen-design-locks)](_decisions/2026-10-08-edit-screen-design-locks.md) | accepted | 人 | 人の発言 | CE-25, CE-26 |
| 2026-10-08 | [セルの入力欄は反転でなく地の色で示す(edit-screen-design-bg)](_decisions/2026-10-08-edit-screen-design-bg.md) | accepted | AI | 検証の指摘 | SR-31 |
| 2026-10-07 | [日時の列のカレンダーに時刻の欄を足す(time-picker)](_decisions/2026-10-07-time-picker.md) | accepted | 人 | 人の発言 | CE-30 |
| 2026-10-07 | [時刻の欄の細部(time-picker-details)](_decisions/2026-10-07-time-picker-details.md) | accepted | AI | AI の提案 | CE-31 |
| 2026-10-07 | [1つの値で書かれたリストを、付け外しでリストに書き換える(scalar-list-edit)](_decisions/2026-10-07-scalar-list-edit.md) | accepted | 人 | 人の発言 | CE-19, CE-16, CE-17 |
| 2026-10-07 | [新しいノートの雛形の変数と本文の雛形(note-templates)](_decisions/2026-10-07-note-templates.md) | accepted | 人 | 人の発言 | CE-32 |
| 2026-10-07 | [左のノートの欄から共通のフォルダと .md を除き、名前の列があれば名前を出さない(note-column)](_decisions/2026-10-07-note-column.md) | accepted | 人 | 人の発言 | SR-29 |
| 2026-10-07 | [左のノートの欄の名前に合わせて、画面の名前を見る試験の錠を掛け直す(note-column-locks)](_decisions/2026-10-07-note-column-locks.md) | accepted | 人 | 人の発言 | SR-21, BV-17, CE-16 ほか 9 |
| 2026-10-07 | [新しいノートをフォーム(1つの窓)で作る(new-note-form)](_decisions/2026-10-07-new-note-form.md) | accepted | 人 | 人の発言 | CE-26, CE-27 |
| 2026-10-07 | [フォームに合わせて、新しいノートの試験の錠を掛け直す(new-note-form-locks)](_decisions/2026-10-07-new-note-form-locks.md) | accepted | 人 | 人の発言 | CE-25, CE-26, BV-17, CE-20, WB-2, SR-23 |
| 2026-10-07 | [新しいノートをフォームかエディタかで作れるようにする(new-note-editor)](_decisions/2026-10-07-new-note-editor.md) | accepted | 人 | 人の発言 | CE-33 |
| 2026-10-07 | [file.backlinks と file.hasLink() を評価する(links-backlinks)](_decisions/2026-10-07-links-backlinks.md) | accepted | 人 | 人の発言 | BV-22, BV-7, SR-23, WB-3, WB-5, CE-22 |
| 2026-10-07 | [見本の .base の未対応の式の例を file.embeds に替える(links-backlinks-sample)](_decisions/2026-10-07-links-backlinks-sample.md) | accepted | 人 | 人の発言 | BV-3, BV-7 |
| 2026-10-07 | [まとまりの見出しの上を空ける表示の設定(group-gap)](_decisions/2026-10-07-group-gap.md) | accepted | 人 | 人の発言 | SR-30 |
| 2026-10-07 | [group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)](_decisions/2026-10-07-group-gap-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-21, NV-16 ほか 4 |
| 2026-10-07 | [画面の表をファイルに書き出し、--print に tsv を足す(export-table)](_decisions/2026-10-07-export-table.md) | accepted | 人 | 人の発言 | OUT-2, CLI-5 |
| 2026-10-07 | [表の書き出しの細部(export-table-details)](_decisions/2026-10-07-export-table-details.md) | accepted | AI | AI の提案 | OUT-5 |
| 2026-10-07 | [displayName の無い列の見出しを Obsidian の既定にする(default-headings)](_decisions/2026-10-07-default-headings.md) | accepted | 人 | 人の発言 | BV-24 |
| 2026-10-07 | [既定の見出しに合わせて、見出しを鍵に使う試験の錠を掛け直す(default-headings-locks)](_decisions/2026-10-07-default-headings-locks.md) | accepted | 人 | 人の発言 | BV-22, BV-7 |
| 2026-10-06 | [日付と数の入力でも、開いた直後に打つと今の値を置き換える(type-replaces-date-number)](_decisions/2026-10-06-type-replaces-date-number.md) | accepted | AI | 検証の指摘 | CE-5, CE-7 |
| 2026-10-06 | [TOML のフロントマター(+++)のノートを読むだけにする(toml-frontmatter)](_decisions/2026-10-06-toml-frontmatter.md) | accepted | AI | 検証の指摘 | WB-20 |
| 2026-10-06 | [画面のテーマを見本の7つから選べるようにする(themes)](_decisions/2026-10-06-themes.md) | accepted | 人 | 人の発言 | SR-26 |
| 2026-10-06 | [テーマの設定の名前と既定、色なしと 256 色の扱い(themes-details)](_decisions/2026-10-06-themes-details.md) | accepted | AI | AI の提案 | SR-27 |
| 2026-10-06 | [テーマで、ためる変更・差分・一致の強調の色もテーマの地で読める色にする(theme-meaning-colors)](_decisions/2026-10-06-theme-meaning-colors.md) | accepted | AI | 検証の指摘 | SR-28 |
| 2026-10-06 | [.base の組み込みの集計(summaries)を版1に入れる(summaries)](_decisions/2026-10-06-summaries.md) | accepted | AI | 検証の指摘 | BV-14 |
| 2026-10-06 | [SR-16 のモードの並びに頻度表を足す(sr16-freq-mode)](_decisions/2026-10-06-sr16-freq-mode.md) | accepted | AI | AI の提案 | SR-16 |
| 2026-10-06 | [隠しフォルダと node_modules の下のノートを探さない(skip-hidden-dirs)](_decisions/2026-10-06-skip-hidden-dirs.md) | accepted | AI | 検証の指摘 | BV-23 |
| 2026-10-06 | [--print に各行のノートのパスの列を足す --with-path(print-with-path)](_decisions/2026-10-06-print-with-path.md) | accepted | AI | 検証の指摘 | CLI-14 |
| 2026-10-06 | [.md を渡した --print は、渡したノートの行だけを出す(print-md-rows)](_decisions/2026-10-06-print-md-rows.md) | accepted | 人 | 人の発言 | CLI-15 |
| 2026-10-06 | [--print に --filter と --sort を足す(print-filter-sort)](_decisions/2026-10-06-print-filter-sort.md) | accepted | AI | 検証の指摘 | CLI-16 |
| 2026-10-06 | [大きな保管庫では1回の見回りで見るノートを限って順に回す(poll-round-robin)](_decisions/2026-10-06-poll-round-robin.md) | accepted | AI | 検証の指摘 | BV-9 |
| 2026-10-06 | [.md のファイルを渡したらそのフォルダを開いてその行を選ぶ(open-md-file)](_decisions/2026-10-06-open-md-file.md) | accepted | AI | 検証の指摘 | CLI-15 |
| 2026-10-06 | [日付と日時が混ざる列は日時と推定する(mixed-datetime)](_decisions/2026-10-06-mixed-datetime.md) | accepted | AI | 検証の指摘 | CE-2 |
| 2026-10-06 | [リストで文字を打つと今の値を置き換えて自由入力を始める(list-type-replaces)](_decisions/2026-10-06-list-type-replaces.md) | accepted | AI | 検証の指摘 | CE-3 |
| 2026-10-06 | [リストで打つと、打った文字で候補を絞って出し続ける(list-narrow)](_decisions/2026-10-06-list-narrow.md) | accepted | AI | 検証の指摘 | CE-3 |
| 2026-10-06 | [リンク・被リンク・.base を開いたときの this を評価する(links-this)](_decisions/2026-10-06-links-this.md) | accepted | AI | 検証の指摘 | BV-22 |
| 2026-10-06 | [BV-22 から file.backlinks・file.hasLink を外す(links-this-narrow)](_decisions/2026-10-06-links-this-narrow.md) | accepted | AI | 検証の指摘 | BV-22 |
| 2026-10-06 | [キーの名前の変更と削除を、全部のノートにまとめて行う(key-rename-delete)](_decisions/2026-10-06-key-rename-delete.md) | accepted | 人 | 人の発言 | WB-1, WB-2, CE-29 |
| 2026-10-06 | [ヘルプの最後にセルと行の印の意味を並べる(help-markers)](_decisions/2026-10-06-help-markers.md) | accepted | AI | 検証の指摘 | SR-5 |
| 2026-10-06 | [列の値の頻度表を版1に入れる(frequency-table)](_decisions/2026-10-06-frequency-table.md) | accepted | AI | 検証の指摘 | NV-9 |
| 2026-10-06 | [NV-9 の頻度表で絞った条件の出る場所を NV-8 と同じにそろえる(frequency-filter-place)](_decisions/2026-10-06-frequency-filter-place.md) | accepted | AI | 検証の指摘 | NV-9 |
| 2026-10-06 | [詳細の表示で本文の先頭を読むだけで見せる(detail-body)](_decisions/2026-10-06-detail-body.md) | accepted | AI | 検証の指摘 | NV-6 |
| 2026-10-06 | [チェックボックスの Enter を真と偽の切り替えにする(checkbox-two-state)](_decisions/2026-10-06-checkbox-two-state.md) | accepted | 人 | 人の発言 | CE-4 |
| 2026-10-06 | [チェックボックスの真と偽の切り替えを、一括と元に戻す例にも書く(checkbox-two-state-examples)](_decisions/2026-10-06-checkbox-two-state-examples.md) | accepted | 人 | 人の発言 | CE-4, CE-10, WB-17 |
| 2026-10-06 | [--print で出した CSV・JSON を直して戻す --apply(apply)](_decisions/2026-10-06-apply.md) | accepted | AI | 検証の指摘 | CLI-17 |
| 2026-10-06 | [どのノートにも無いキーの列を表に足す(add-column)](_decisions/2026-10-06-add-column.md) | accepted | AI | 検証の指摘 | CE-28 |
| 2026-10-05 | [SR-16 のモードの並びに操作の一覧を足す(sr16-menu-mode)](_decisions/2026-10-05-sr16-menu-mode.md) | accepted | AI | 検証の指摘 | SR-16 |
| 2026-10-05 | [その場の操作の一覧と、前置きのキーの続きの案内(action-menu)](_decisions/2026-10-05-action-menu.md) | accepted | 人 | 人の発言 | SR-24, SR-25 |
| 2026-10-05 | [操作の一覧の窓・前置きの Esc の直し(action-menu-fixes)](_decisions/2026-10-05-action-menu-fixes.md) | accepted | 人 | 検証の指摘 | SR-23, SR-24, SR-25, WB-5 |
| 2026-10-03 | [型の決まらない列に書く日付も囲まない(untyped-date)](_decisions/2026-10-03-untyped-date.md) | accepted | 人 | 人の発言 | WB-18 |
| 2026-10-03 | [書き戻しの安全を「しなければならない」に上げる(strict-write-safety)](_decisions/2026-10-03-strict-write-safety.md) | accepted | 人 | 人の発言 | WB-1, WB-2, WB-4 ほか 6 |
| 2026-10-03 | [仕様の古い記述を今の決定と実装に合わせる(stale-text)](_decisions/2026-10-03-stale-text.md) | accepted | AI | 検証の指摘 | SR-1, SR-16, CLI-2 ほか 4 |
| 2026-10-03 | [stale-text の SR-1 と WB-2 の書き落としを直す(stale-text-fix)](_decisions/2026-10-03-stale-text-fix.md) | accepted | AI | 検証の指摘 | SR-1, WB-2 |
| 2026-10-03 | [書き戻しの安全の文書(safety-docs)](_decisions/2026-10-03-safety-docs.md) | accepted | AI | AI の提案 | WB-19 |
| 2026-10-03 | [書き込みの権限が無いノートは読むだけにする(readonly-permission)](_decisions/2026-10-03-readonly-permission.md) | accepted | 人 | 人の発言 | WB-5 |
| 2026-10-03 | [--print と --pick を版1に入れる(print-pick)](_decisions/2026-10-03-print-pick.md) | accepted | AI | AI の提案 | CLI-5, OUT-3, SR-10 |
| 2026-10-03 | [--pick の列の値は素の文字、Esc は解くものが無いときだけ取りやめ(pick-plain)](_decisions/2026-10-03-pick-plain.md) | accepted | AI | 検証の指摘 | OUT-3 |
| 2026-10-03 | [空にするときは区切りの空白も値の範囲に含める(null-separator)](_decisions/2026-10-03-null-separator.md) | accepted | 人 | 人の発言 | WB-1 |
| 2026-10-03 | [フォルダを2つ以上開いたときは作る場所を選ぶ欄を出す(new-note-folders)](_decisions/2026-10-03-new-note-folders.md) | accepted | 人 | 人の発言 | CE-25 |
| 2026-10-03 | [作る場所を選ぶ欄の順と、そのときの「+ 新規」(new-note-folders-order)](_decisions/2026-10-03-new-note-folders-order.md) | accepted | 人 | 検証の指摘 | CE-25, CE-26 |
| 2026-10-03 | [リストの値を足すときの行と、消えた CE-6 への参照を直す(list-lines)](_decisions/2026-10-03-list-lines.md) | accepted | 人 | 検証の指摘 | WB-1, WB-3, CE-10 |
| 2026-10-03 | [画面の文言を英語と日本語で切り替える(language)](_decisions/2026-10-03-language.md) | accepted | 人 | 人の発言 | SR-23 |
| 2026-10-03 | [キーの割り当ての一覧の文書(keys-docs)](_decisions/2026-10-03-keys-docs.md) | accepted | AI | AI の提案 | SR-22 |
| 2026-10-03 | [シェルの補完と man ページを出す(completions-man)](_decisions/2026-10-03-completions-man.md) | accepted | AI | AI の提案 | CLI-13 |
| 2026-10-03 | [.base の対応範囲の文書(bases-docs)](_decisions/2026-10-03-bases-docs.md) | accepted | AI | AI の提案 | BV-21 |
| 2026-10-02 | [表の上に常に出る検索の欄(search-bar)](_decisions/2026-10-02-search-bar.md) | accepted | 人 | 人の発言 | NV-2, NV-23 |
| 2026-10-02 | [表から新しいノートを作る(new-note)](_decisions/2026-10-02-new-note.md) | accepted | 人 | 人の発言 | CE-14, CE-25, CE-26 |
| 2026-10-02 | [新しいノートの細部(キー・設定の形)(new-note-details)](_decisions/2026-10-02-new-note-details.md) | accepted | AI | AI の提案 | CE-27 |
| 2026-10-02 | [mdgrid 独自のビューの定義(native-views)](_decisions/2026-10-02-native-views.md) | accepted | 人 | 人の発言 | BV-17, BV-18, BV-19 |
| 2026-10-02 | [mdgrid のビューの定義の細部(native-views-details)](_decisions/2026-10-02-native-views-details.md) | accepted | AI | AI の提案 | BV-20 |
| 2026-10-02 | [リストの値(tags など)を候補から選んで付け外しする(list-select)](_decisions/2026-10-02-list-select.md) | accepted | 人 | 人の発言 | CE-6, CE-13, CE-16, CE-17, CE-18 |
| 2026-10-02 | [リストの読むだけの形を、元の書き方を保てない形まで広げる(list-select-readonly-forms)](_decisions/2026-10-02-list-select-readonly-forms.md) | accepted | AI | 検証の指摘 | CE-19 |
| 2026-10-02 | [リストの値の付け外しの細部(list-select-details)](_decisions/2026-10-02-list-select-details.md) | accepted | AI | AI の提案 | CE-19 |
| 2026-10-02 | [空のフロントマターにも書き、フロントマターを足すかを設定で選べる(empty-frontmatter-config)](_decisions/2026-10-02-empty-frontmatter-config.md) | accepted | 人 | 人の発言 | WB-3, WB-5 |
| 2026-10-02 | [ノートを開くエディタを設定で選べる(editor-setting)](_decisions/2026-10-02-editor-setting.md) | accepted | 人 | 人の発言 | SR-8 |
| 2026-10-02 | [行番号・一行おきの色・列の区切り線・上の帯の出し入れ(display-options)](_decisions/2026-10-02-display-options.md) | accepted | 人 | 人の発言 | SR-20 |
| 2026-10-02 | [表の見せ方の設定の名前と既定(display-options-details)](_decisions/2026-10-02-display-options-details.md) | accepted | AI | AI の提案 | SR-21 |
| 2026-10-02 | [日付の列は Obsidian と同じく囲まずに書く(date-unquoted)](_decisions/2026-10-02-date-unquoted.md) | accepted | 人 | 人の発言 | WB-18 |
| 2026-10-02 | [日付の列のカレンダーの入力(date-picker)](_decisions/2026-10-02-date-picker.md) | accepted | 人 | 人の発言 | CE-20 |
| 2026-10-02 | [カレンダーの入力の細部(date-picker-details)](_decisions/2026-10-02-date-picker-details.md) | accepted | AI | AI の提案 | CE-21 |
| 2026-10-02 | [日付の見せ方と打ち込みの形を設定で選ぶ(date-format)](_decisions/2026-10-02-date-format.md) | accepted | 人 | 人の発言 | CE-22 |
| 2026-10-02 | [設定の全項目の文書と、既定の設定の書き出し(config-docs)](_decisions/2026-10-02-config-docs.md) | accepted | 人 | 人の発言 | CLI-11, CLI-12 |
| 2026-10-02 | [列の区切り線は入れると │ を引き、既定は線なし(column-lines-fix)](_decisions/2026-10-02-column-lines-fix.md) | accepted | 人 | 検証の指摘 | SR-20, SR-21 |
| 2026-10-02 | [ファイル名のクリックは e と同じ振る舞いにする(click-name-same-as-e)](_decisions/2026-10-02-click-name-same-as-e.md) | accepted | 人 | 検証の指摘 | SR-19 |
| 2026-10-02 | [ファイル名をクリックしてノートを $EDITOR で開く(click-name-editor)](_decisions/2026-10-02-click-name-editor.md) | accepted | 人 | 人の発言 | SR-19 |
| 2026-10-02 | [CLI-2 の起動の引数に --print-config を並べる(cli2-print-config)](_decisions/2026-10-02-cli2-print-config.md) | accepted | AI | 検証の指摘 | CLI-2 |
| 2026-10-02 | [チェックボックスの列で空・真・偽の3つを選べるようにする(checkbox-three-state)](_decisions/2026-10-02-checkbox-three-state.md) | accepted | 人 | 人の発言 | CE-4 |
| 2026-10-02 | [カレンダーで1年を飛ばす(calendar-year-keys)](_decisions/2026-10-02-calendar-year-keys.md) | accepted | AI | AI の提案 | CE-24 |
| 2026-10-02 | [カレンダーで矢印のほかのキーで月を飛ばす(calendar-month-keys)](_decisions/2026-10-02-calendar-month-keys.md) | accepted | 人 | 人の発言 | CE-23 |
| 2026-10-02 | [チェックボックスの一括の切り替えで型の合わない行を飛ばす(bulk-skip-mismatch)](_decisions/2026-10-02-bulk-skip-mismatch.md) | accepted | 人 | 人の発言 | CE-10 |
| 2026-10-02 | [フロントマターの無いノートに入力したらフロントマターを足す(add-frontmatter)](_decisions/2026-10-02-add-frontmatter.md) | accepted | 人 | 人の発言 | WB-1, WB-3, CE-10 |
| 2026-10-01 | [保存の基準を「最初に変更をためた時点」に揃える(write-baseline)](_decisions/2026-10-01-write-baseline.md) | accepted | AI | 検証の指摘 | WB-4 |
| 2026-10-01 | [ビューの設定の画面と、表の上のフィルターの帯(view-settings)](_decisions/2026-10-01-view-settings.md) | accepted | 人 | 人の発言 | NV-13, NV-14, NV-15, NV-16, NV-17, NV-18 |
| 2026-10-01 | [ビューの設定の細部(フィルターの条件・重なり方・戻し方)(view-settings-details)](_decisions/2026-10-01-view-settings-details.md) | accepted | AI | AI の提案 | NV-19, NV-20, NV-21, NV-22 |
| 2026-10-01 | [表示をセッションとして保存し、名前で起動する(sessions)](_decisions/2026-10-01-sessions.md) | accepted | 人 | 人の発言 | CLI-6 |
| 2026-10-01 | [セッションが持つ絞り込みを、版1の画面の絞り込みに合わせる(sessions-filters)](_decisions/2026-10-01-sessions-filters.md) | accepted | AI | 検証の指摘 | CLI-7 |
| 2026-10-01 | [セッションの中身・置き場・一覧・失敗のとき(sessions-details)](_decisions/2026-10-01-sessions-details.md) | accepted | AI | AI の提案 | CLI-7, CLI-8, CLI-9, CLI-10 |
| 2026-10-01 | [範囲外の項目に「次の段:」「採らない:」の印を付ける(scope-priority-marks)](_decisions/2026-10-01-scope-priority-marks.md) | accepted | AI | 検証の指摘 | SC-13 |
| 2026-10-01 | [元の値に戻したセルを、ためる変更から外す(revert-to-original)](_decisions/2026-10-01-revert-to-original.md) | accepted | 人 | 人の発言 | WB-17 |
| 2026-10-01 | [保存の基準を「最後に読んだ内容」と明記し、外の変更の上に書く選択と WB-4 の止め方を両立させる(overwrite-baseline)](_decisions/2026-10-01-overwrite-baseline.md) | accepted | AI | 検証の指摘 | WB-4, WB-16 |
| 2026-10-01 | [既定のキーのぶつかりと抜けを決める(key-defaults)](_decisions/2026-10-01-key-defaults.md) | accepted | AI | 検証の指摘 | SR-16, WB-10, BV-16, SR-18 |
| 2026-10-01 | [日本語入力の読点と中黒、キーの無いセルの Backspace、未対応の印と本当の値の見分け(ime-keys-marks)](_decisions/2026-10-01-ime-keys-marks.md) | accepted | AI | 検証の指摘 | SR-17, SR-18, BV-7 |
| 2026-10-01 | [見せ方と編集の食い違いと抜けを直す(edit-view-gaps)](_decisions/2026-10-01-edit-view-gaps.md) | accepted | AI | 検証の指摘 | SR-10, BV-7, CE-3, CE-5, CE-8, CE-10 |
| 2026-09-30 | [仕様の点検の指摘を直す(v1-review-fixes)](_decisions/2026-09-30-v1-review-fixes.md) | accepted | AI | 検証の指摘 | SR-13, SR-10, SR-12 ほか 18 |
| 2026-09-30 | [最初の版の仕様を、表の TUI の調査に合わせて書き切る(v1-full-spec)](_decisions/2026-09-30-v1-full-spec.md) | accepted | AI | 人の発言 | BV-4, BV-9, CE-1 ほか 52 |
| 2026-09-30 | [最初の版の設計(v1-design)](_decisions/2026-09-30-v1-design.md) | accepted | AI | AI の提案 | BV-1, BV-2, BV-3 ほか 35 |
| 2026-09-30 | [TUI の名前を mdgrid にする(rename-mdgrid)](_decisions/2026-09-30-rename-mdgrid.md) | accepted | AI | 人の発言 | OUT-3, CLI-1, CLI-3 |
| 2026-09-30 | [形式ごとの読み込み口を差し替えられる形にする(pluggable-sources)](_decisions/2026-09-30-pluggable-sources.md) | accepted | AI | 人の発言 | SC-14 |
| 2026-09-30 | [取り込み: scope](_decisions/2026-09-30-import-scope.md) | accepted | 取り込み | 取り込み | SC-1, SC-2, SC-3 ほか 9 |
<!-- decidespec:end -->
