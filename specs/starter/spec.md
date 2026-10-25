---
type: Spec
id: 01M5PHAAPGZMYFKA351A8DFKQD
title: 台帳のスターター(starter)
description: 台帳の見せ方(印・新しいノートの雛形・見本)のひとまとまりを組み込みと設定の置き場に持ち、そこから台帳を作る。
status: active
load_when: スターターの形・組み込みのスターター・ユーザーのスターターの置き場・スターターから台帳を作る操作(CLI と画面)を作る・変えるとき
created: 2026-10-24
updated: 2026-10-24
---

# 台帳のスターター(starter)

タスク・ブックマーク・読書の記録・人と会議のように、よくある台帳の「見せ方」(ワークスペースの印 `.mdgrid/workspace.toml`・新しいノートの本文の雛形・見本のノート)をひとまとまりにして持ち、そこから新しい台帳を作る仕組み。組み込みのスターターは実行ファイルの中にあり、自分のスターターは設定の置き場の `starters/` に同じ形で置く。CLI(`--init-workspace --starter <名前>`)と画面のコマンドのパレットの両方から作れる。スターターは見せ方だけを持ち、ノートの形の決まり(スキーマ・lint)は別の道具の役目で、ここでは扱わない。版1 はスターターから作るだけで、今の表からスターターを書き出すのは次の段。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — スターターの形と組み込み

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| ST-1 | 台帳のスターターは、ワークスペースの印 `.mdgrid/workspace.toml`(WS-7・WS-9・WS-10・TR-2 の中身)と、新しいノートの本文の雛形(CE-32 の `body` のファイル)と、見本のノートからなる、台帳の見せ方のひとまとまりとするべき。スターターは見せ方だけを持ち、ノートの形の決まり(スキーマ・lint)は持たないべき(ノートの形を検査する道具の役目で、mdgrid が台帳ごとの知識を持たないため)。名前と1行の説明は言語ごとに `starter.toml`(`[name]`・`[description]` に、文言の台帳と同じ言語の符号の鍵)に持ち、無ければフォルダの名前と空の説明にするべき。印と雛形は `ledger/`、見本は `samples/` に、台帳の根からの相対パスのまま置き、`ledger.<言語>/`・`samples.<言語>/` があれば、その言語のときはそれを丸ごと代わりに使うべき(重ねない)。 | `starters/tasks/` に `starter.toml`・`ledger/.mdgrid/workspace.toml`・`ledger/.mdgrid/templates/task.md`・`samples/tasks/Write the plan.md` → スターターとして読め、名前は ja で「タスク」、en で「Tasks」。`starter.toml` の無いユーザーのスターター → 名前はフォルダの名前、説明は空。`ledger.ja/` があり言語が ja → `ledger/` は使わず `ledger.ja/` の中身だけ(`test_st_1_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |
| ST-2 | mdgrid は組み込みのスターター `tasks`(タスク)・`bookmarks`(ブックマーク)・`reading`(読書・視聴の記録)・`people`(人・連絡先・会議メモ)を実行ファイルの中に持つべき。組み込みはユーザーのスターター(ST-3)と同じ形・同じ読み方の見本で、組み込みだけに効く特例を作らないべき。組み込みのスターターは、どの言語の変種でも、印が警告なしに読め(印の読み込み・木の設定・`open` の警告が空)、`open` が印の表を指し、見本のノートが `--check` で問題 0 になるべき。 | 4つの組み込みから見本つきで作る → `--print-config --resolved` の標準エラーに `mdgrid:` の行が無く、`--print` が見本の行を出し、`--check` の問題が 0。`LANG=en_US.UTF-8` でも同じ(`test_st_2_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |
| ST-3 | ユーザーのスターターは、設定の置き場(CLI-3)の `starters/<名前>/` に、組み込みと同じ形で置けるべき。一覧はユーザーのスターターを名前の順に先に、組み込みをその後に並べ、同じ名前はユーザーのものを使って組み込みを出さないべき。`mdgrid --starters` は、スターターの名前と今の言語の1行の説明を1行ずつ標準出力に出して終了コード 0 で終わり、設定の置き場が無くても組み込みを出すべき。読めないスターター(壊れた `starter.toml`・`ledger/.mdgrid/workspace.toml` の無いもの)は CLI-3 の形で警告して飛ばすべき。画面を開かず、ファイルは書かないべき。 | `starters/kanban/` を置く → `--starters` の先頭に kanban。`starters/tasks/` を置く → tasks は1つで、ユーザーのもの。置き場なし → 組み込みの4つ。壊れた `starter.toml` → `starters/kanban/starter.toml:` で始まる警告1行で、ほかは出る(`test_st_3_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |

## 要件: 版1 — 作る(CLI と画面)

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| ST-4 | `mdgrid [<フォルダ>] --init-workspace --starter <名前> [--with-samples]` は、そのフォルダ(無ければ今のフォルダ。フォルダが無ければ作る)に、スターターの `ledger/` の中身を同じ相対のパスで書き、`--with-samples` のときは見本のノートも書くべき。見本のノートの中の `{date}`・`{date+N}`・`{now}` などは CE-32 と同じ変数として書くときに埋め、`ledger/` の中身(印と本文の雛形)はそのまま写すべき(雛形の変数は新しいノートを作るときに埋まるため)。書く先に同じパスのファイルが1つでもあれば、どれも書かずに理由を出して終了コード 2 にするべき(WS-7 と同じく上書きしない)。無い名前・2つ以上のフォルダ・`--init-workspace` の無い `--starter`・`--starter` の無い `--with-samples` は、CLI-4 と同じく理由1行と終了コード 2 にするべき。作ったファイルを1行ずつ標準出力に出し、画面は出さないべき。 | `mdgrid books --init-workspace --starter reading` → `books/.mdgrid/workspace.toml` と `books/.mdgrid/templates/` の雛形ができ、見本は無い。`--with-samples` と `MDGRID_TODAY=2026-10-06` → 見本の `{date-10}` が `2026-09-26` で書かれ、雛形の `{date}` は文字のまま。もう一度 → 理由1行と終了コード 2 で何も変わらない。`--starter nope` → 理由1行(`--starters` で一覧を見られることを添える)と終了コード 2(`test_st_4_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |
| ST-5 | コマンドのパレット(SR-14)の「スターターから台帳を作る」で、スターターを一覧(名前と説明。打った字で絞る)から選び、作る場所(開いたフォルダからの相対・`~`・絶対のパス。既定は開いたフォルダの下のスターターの名前)を打ち、見本のノートを入れるかを `y`/`n` で答えると、ST-4 と同じに作って、その台帳の印の `open` の表(無ければ根)を開くべき。ためた変更があれば CLI-19 と同じく保存か取りやめを確かめるべき。Esc はどの段でも取りやめにし、何も作らないべき。読むだけの起動(WB-15)ではパレットに出さないべき。 | `:` → 「スターター」→ 一覧に組み込みの4つ → reading を Enter → 「作る場所」の欄に `reading` が入っているのを `~/notes/books` に直して Enter → 「見本のノートも入れる?(y/n)」→ `y` → books の Reading の表が開き、パンくずに books。途中の Esc → 「やめた」で何も作らない。`--readonly` → パレットに無い(`test_st_5_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |
| ST-6 | スターターから作った台帳は、ふだんの印のあるフォルダと同じに扱い、開いて見るだけでは印・雛形・見本のバイトを変えないべき。スターターの中身は、文書にある印の項目(CLI-12)と雛形の変数(CE-32)の範囲に収め、スターターだけに効く項目を作らないべき(スターターから作った台帳と手で書いた台帳の間に差を作らないため)。 | 作った直後と、開いて閉じたあとの印のバイトが同じ。組み込みの印に知らない項目が無い(ST-2 の警告なしで確かめる)(`test_st_6_*`) | [2026-10-24](../_decisions/2026-10-24-starter.md) |

## 範囲外

- 次の段: 今の表(印・ビュー)からスターターを書き出すこと(逆の向き)。版1 はスターターから作るだけ。
- 採らない: スキーマ・lint・ノートの形の検査(ノートの形を検査する道具の役目。ST-1)。
- 採らない: ネットからスターターを取ってくること(置き場は組み込みと設定のフォルダだけ。ST-3)。

## 前提(AI の仮定)

- 組み込みのスターターの印には `name` を書かない(作った先のフォルダの名前がワークスペースの名前になる。WS-7)。
- 言語の変種(`ledger.ja/` など)は丸ごとの差し替えで、`ledger/` に重ねない(見本のノートの名前が英語と日本語で混ざらないため)。英語(接尾なし)が正本。
- 呼び名: 「型」は列の型・画面の型に、「テンプレート」は CLI-3 の `[templates]` と CE-32 の雛形に使っているので、この機能は「スターター」(`starter`)と呼ぶ。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-24 | [台帳のスターター(組み込みの印・雛形・見本)から表を作れるようにする(starter)](../_decisions/2026-10-24-starter.md) | accepted | 人 | 人の発言 | ST-1, ST-2, ST-3, ST-4, ST-5, ST-6 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| ST-1 | [2026-10-24](../_decisions/2026-10-24-starter.md) | src/test_starter_more_unit.rs::test_st_1_ledger_variant_without_marker_is_refused, src/test_starter_unit.rs::test_st_1_lang_variant_replaces_whole_ledger_and_samples, src/test_starter_unit.rs::test_st_1_missing_language_falls_back_to_en_then_folder_name ほか 3 | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
| ST-2 | [2026-10-24](../_decisions/2026-10-24-starter.md) | src/test_starter_unit.rs::test_st_2_builtin_every_language_creates_a_ledger_that_reads_without_warnings, src/test_starter_unit.rs::test_st_2_builtin_four_in_name_order_with_every_language, src/test_starter_unit.rs::test_st_2_builtin_marker_has_no_name_so_the_folder_names_the_workspace ほか 2 | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
| ST-3 | [2026-10-24](../_decisions/2026-10-24-starter.md) | src/test_starter_more_unit.rs::test_st_1_ledger_variant_without_marker_is_refused, src/test_starter_more_unit.rs::test_st_3_a_linked_starter_folder_is_read, src/test_starter_more_unit.rs::test_st_3_hidden_folders_in_starters_are_not_starters ほか 12 | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
| ST-4 | [2026-10-24](../_decisions/2026-10-24-starter.md) | src/test_starter_more_unit.rs::test_st_4_create_refuses_when_a_folder_on_the_way_is_a_link, src/test_starter_unit.rs::test_st_4_create_copies_ledger_as_is_without_samples, src/test_starter_unit.rs::test_st_4_create_makes_a_missing_root ほか 10 | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
| ST-5 | [2026-10-24](../_decisions/2026-10-24-starter.md) | src/ui/test_starter.rs::test_st_5_empty_folder_is_refused_and_absolute_path_is_used_as_is, src/ui/test_starter.rs::test_st_5_esc_at_any_step_cancels_and_writes_nothing, src/ui/test_starter.rs::test_st_5_n_makes_the_ledger_without_samples ほか 6 | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
| ST-6 | [2026-10-24](../_decisions/2026-10-24-starter.md) | tests/test_starter_e2e.rs::test_st_6_opening_a_made_ledger_changes_no_bytes_e2e | [2026-10-24-starter](../_changes/2026-10-24-starter.md) 2026-10-25 |
<!-- decidespec:history:end -->
