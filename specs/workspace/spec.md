---
type: Spec
id: 01M4E5CQZKK9X940T902KWBQ8A
title: ワークスペース(workspace)
description: 表をまとめる範囲(ワークスペース)をアプリの側に持ち、画面とコマンドで管理し、.git と .obsidian/ のあるルートフォルダを検知する。
status: active
load_when: ワークスペースの持ち方・管理(画面とコマンド)・検知・範囲の決め方を作る・変えるとき
created: 2026-10-09
updated: 2026-10-15
---

# ワークスペース(workspace)

ワークスペースは、名前の付いた表(フォルダか `.base`)の集まりで、リンクの行き先・つながった行・表どうしのつながり・関係マップ([relations](../relations/spec.md))の範囲になる。設定のフォルダの `workspaces.toml` に持ち、画面のパレットとコマンドの引数で管理する。フォルダに `.mdgrid/workspace.toml` の印があれば、そちらを先に使う(WS-7)。書いたワークスペースが無いときは、全体の設定で選んだ検知(.obsidian/ のあるルートフォルダ・git のリポ)で範囲を決め、それも無ければ登録した表([cli](../cli/spec.md) の CLI-18。よく開く場所のブックマーク)を使う。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — ワークスペース

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| WS-1 | ワークスペース(名前の付いた表の集まり)を、設定のフォルダの `workspaces.toml`(`[[workspace]]` の並び。`name` と、表の並び `table`(`name`・`path`(フォルダか `.base`)・`view`)と、ワークスペースと表それぞれの表のプロファイル(SR-44。`[workspace.look]`・`[workspace.table.look]` など))に持つべき。ノートのフォルダには何も書かない。`path` の先頭の `~` はホームのフォルダ。読めない行は警告にして飛ばし、起動を止めてはならない。 | `[[workspace]]` に name = "Product"、表を2つ → 2つの表のワークスペース。path の無い表 → 警告して飛ばす。`[workspace.look] theme = "nord"` → そのワークスペースの表が nord(`test_ws_1_*`) | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-2 | 画面のパレットから、ワークスペースを作る・今の表をワークスペースに足す(今のワークスペースから選ぶか新しく作る)・今の表をワークスペースから外す・ワークスペースを開く(一覧から選び、その表を開く)ができるべき。 | 「この表をワークスペースに足す」→ 名前を選ぶ → workspaces.toml にその表が入る。「ワークスペースを外す」→ 消える(`test_ws_2_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-3 | コマンドの引数で、ワークスペースの一覧を出す(`mdgrid --workspaces`)・表を足す(`mdgrid <パス> --add-to <名前> [--as <表の名前>]`。ワークスペースが無ければ作る)・表を外す(`mdgrid <パス> --remove-from <名前>`)・ワークスペースごと消す(`mdgrid --remove-workspace <名前>`)・ワークスペースで開く(`mdgrid -w <名前> [<パス>]`。パスが無ければ最初の表)ができるべき。操作は、開くパス(位置の引数)と重ならない旗で書くべき(同じ名前のフォルダを開けなくしないため)。画面を出さない操作は、標準出力がパイプでも動くべき。 | `mdgrid ~/notes/tasks --add-to Product` → `mdgrid --workspaces` に出る。`workspace` という名前のフォルダは `mdgrid workspace` で開ける。`mdgrid -w Product` → 最初の表で開く。無い名前 → 理由1行と 0 以外の終了コード(`test_ws_3_*`) | [2026-10-09](../_decisions/2026-10-09-workspace-flags.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-4 | 開いた表が属するワークスペース(`-w` で選んだもの、無ければ開いた表を含む最初のワークスペース)があれば、リンクの行き先を探す範囲・つながった行・表どうしのつながり・関係マップ・上の端のタブは、そのワークスペースの表(と今の表)だけを使うべき。 | ワークスペース Work に tasks と projects、Hobby に books → tasks で関係マップを開くと tasks と projects だけ。books を指すリンクは行き先の表に数えない(`test_ws_4_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-5 | 全体の設定 `[workspace]` の `detect`(`".obsidian"`・`".git"` の並び(目印のフォルダの名前。旧い値 `"vault"`・`"git"` も同じ意味で読み、移ったと警告する)。既定は `[".obsidian"]`)で、書いたワークスペースが無い表を開いたときに、開いたフォルダから上へたどって最初に見つかったルートフォルダ(`.obsidian/` のあるフォルダ)か git のリポの根(`.git` のあるフォルダ)を、名前の無いワークスペースとみなすべき。その表は、根の直下の、ノートのあるフォルダにするべき(`.base` は入れない。`.base` の表はルートフォルダを指すので、自動で入れるとほかの表と重なるため。要れば書いて足す)。 | `.obsidian/` のあるルートフォルダの tasks を開く → ルートフォルダの直下の projects も関係マップに出る。`[workspace] detect = []` → 検知しない(`test_ws_5_*`) | [2026-10-13](../_decisions/2026-10-13-root-folder.md)、[2026-10-10](../_decisions/2026-10-10-config-v2.md)、[2026-10-09](../_decisions/2026-10-09-workspace-auto-folders.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-6 | 範囲の決め方は、`-w` で選んだワークスペース → 開いたフォルダから上へたどって最初に見つかった `.mdgrid/workspace.toml`(WS-7)→ 開いた表を含む `workspaces.toml` のワークスペース → 検知(WS-5)→ 登録した表(CLI-18)の順にするべき。今どの範囲かを、ヘッダーのパンくず(SR-42)に範囲の名前で示すべき(検知した範囲は、関係マップの画面でだけ示してよい)。 | 書いたワークスペースとルートフォルダの両方に入る表 → 書いたほう。どれにも入らない → 登録した表(今までどおり)。ヘッダーにワークスペースの名前(`test_ws_6_*`) | [2026-10-13](../_decisions/2026-10-13-root-folder.md)、[2026-10-11](../_decisions/2026-10-11-view-bar.md)、[2026-10-10](../_decisions/2026-10-10-ws-button.md)、[2026-10-09](../_decisions/2026-10-09-workspace-marker.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-7 | フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対。ワークスペースの表のプロファイルは最上位に、表のものは `[table.look]` などに書く。SR-44)で、表を書かなければ根の直下の、ノートのあるフォルダを表にする(`.base` は WS-5 と同じ理由で入れない)。印の最上位の `open` に印の表の名前(書いた表なら `name`、自動の表ならフォルダの名前)を書けば、根のフォルダを開いたときに開く表になり(CLI-1)、その表に `view` があればそのビューで開くべき。`open` が印の表のどれも指さないときは、CLI-3 の形で警告して、`open` が無いときと同じに開くべき。`mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。 | notes/.mdgrid/workspace.toml(表なし)→ notes の直下の tasks・projects がワークスペースの表。table を1つ書く → その表だけ。印に `[look] theme = "nord"` → その下の表が nord。`open = "tasks"` と `[[table]] name = "tasks"`・`path = "tasks.base"`・`view = "人だけ"` → `mdgrid notes` が tasks.base の「人だけ」のビュー。表なしの印に `open = "projects"` → projects の表。`open = "無い"` → `.mdgrid/workspace.toml: open:` で始まる警告と、notes の既定の表。`mdgrid notes --init-workspace` → notes/.mdgrid/workspace.toml ができる(`test_ws_7_*`) | [2026-10-15](../_decisions/2026-10-15-marker-open.md)、[2026-10-10](../_decisions/2026-10-10-scope-overrides.md)、[2026-10-09](../_decisions/2026-10-09-workspace-auto-folders.md)、[2026-10-09](../_decisions/2026-10-09-workspace-flags.md)、[2026-10-09](../_decisions/2026-10-09-workspace-marker.md) |
| WS-8 | 今の範囲(WS-6)の表の一覧を、画面の左から表の上に重ねて出す引き出しで見せ、選んだ表へ移れるようにするべき。引き出しは普段は出さず、表の画面と関係マップのどちらでも、ヘッダーのパンくずの範囲か表の名前(SR-42)・表の画面のキー `w`(割り当て直せる。SR-13)・パレットで開き、表を選ぶ(クリックか Enter)・Esc・引き出しの外のクリック・開いたのと同じボタンかキーで閉じるべき(出しっぱなしにすると、左に自分の一覧を持つ端末の道具の中で動かしたとき左に帯が2本並び、表の幅も減るため)。引き出しには今の範囲の表を並べて今の表に印を付け、その下にほかのワークスペースを名前の見出しだけ畳んで並べ、見出しを開くとその表が並ぶようにするべき。範囲がワークスペースでない(検知(WS-5)か登録した表(CLI-18))ときは、その範囲の表を並べるべき。ためた変更があるときに別の表へ移るなら、登録した表へ移るとき(CLI-19)と同じく保存か取りやめを確かめるべき。関係マップで開いたら、表の画面に戻ってから開くべき。 | ワークスペース「仕事」に tasks・projects、「趣味」に books → tasks で開き `w` → 左に重ねて tasks(印付き)・projects、その下に畳んだ「趣味」。projects を Enter → projects の表になり、引き出しは閉じる。「趣味」を開いて books をクリック → books の表で範囲は「趣味」。開いて Esc → 何も変わらずに閉じる。未保存の変更があるときに projects を選ぶ → 保存の確かめ。ワークスペースが無く登録した表だけ → 登録した表が並ぶ。普段の画面に引き出しの帯が無い(`test_ws_8_*`) | [2026-10-11](../_decisions/2026-10-11-view-bar.md)、[2026-10-11](../_decisions/2026-10-11-ws-drawer.md) |
| WS-9 | 印の `[[table]]` には、`path` の `.base` のファイルの代わりに、`.base` と同じ中身(`filters`・`formulas`・`properties`・`summaries`・`views`。キーの名前と式は `.base` と同じ)を TOML の表 `base` に書いてよく、そのときその表は、印の根に置いた同じ中身の `.base` を開いたのと同じに振る舞うべき(ビューのタブ・絞り込み・計算の列・表示名・グループ分け・並べ替え・`--print`・`--check`・`--view`)。ノートを探す範囲は印の根からにし(BV-2)、書いた中身は mdgrid が書き換えないべき(BV-3 と同じ理由)。`base` を書いた表には `path` は要らず、両方を書いたときは `base` を使って警告するべき(CLI-3 の形)。中身が `.base` として読めないとき(BV-1 の読めない `.base`)は、`.base` のときと同じく理由を出して開かないべき。 | 根に `tasks.base` を置かず、印に `open = "台帳"`、`[[table]] name = "台帳"` と `[table.base]` に `filters = { or = ['file.inFolder("todo")'] }`・`[table.base.formulas] 担当 = 'if(actor == "ai", "AI", "人")'`・`[table.base.properties."note.title"] displayName = "題名"`・`[[table.base.views]]` を2つ(`name = "未完了"` で `status != "done"`、`name = "終わった"` で `status == "done"`、どちらも `groupBy = { property = "formula.担当", direction = "ASC" }`)→ 根で `mdgrid --print` が未完了のビューの行で、見出しに「題名」。`--view 終わった` → done の行だけ。画面でタブに「未完了」「終わった」があり、担当でまとまりに分かれる。AGENTS.md の行は無い。開く前後で印のバイトが同じ。`base` と `path = "x.base"` を両方書く → `.mdgrid/workspace.toml: table[1].path:` で始まる警告で `base` を使う。`[table.base] views = "x"` のような読めない中身 → 理由1行と 0 以外の終了コード(`test_ws_9_*`) | [2026-10-15](../_decisions/2026-10-15-marker-base.md) |
| WS-10 | 印の表の `base`(WS-9)では、最上位とビューに `where`(列の名前を鍵に、値の配列を値にした表)を書いてよく、その列の値が配列のどれかと同じ行だけを残すべき(リストの列は、要素のどれかが当たれば残す)。鍵を2つ以上書いたときは全部を満たす行だけにし、同じ所の `filters` と両方を書いたときは両方を満たす行だけにするべき。値の配列の要素は文字列・数・真偽で、空の配列や配列でない値は、その鍵を CLI-3 の形で警告して使わないべき。 | `[[table.base.views]]` に `name = "未完了"`・`where = { status = ["todo", "doing"] }` → todo と doing の行だけ。`where = { status = ["done"], repo = ["tasks"] }` → done で repo が tasks の行だけ。最上位の `where = { kind = ["todo"] }` → どのビューも kind が todo の行だけ。tags が `[a, b]` のノートに `where = { tags = ["b"] }` → 残る。`where = { status = [] }` → `.mdgrid/workspace.toml: table[1].base.views[1].where.status:` で始まる警告と、絞らない表(`test_ws_10_*`) | [2026-10-15](../_decisions/2026-10-15-marker-where.md) |

## 範囲外

- 採らない: 根の `mdgrid.toml` の形の印。印は `.mdgrid/workspace.toml` だけ(WS-7)。
- 採らない: 登録した表(CLI-18)の分類をワークスペースにする形。登録はブックマークとして別に残す。

## 前提(AI の仮定)

- 表がいくつのワークスペースに入っていてもよい。`-w` が無ければ、`workspaces.toml` の順で最初のものを使う。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-15 | [印のあるフォルダを開いたら、印に書いた表を開く(marker-open)](../_decisions/2026-10-15-marker-open.md) | accepted | 人 | 人の発言 | CLI-1, WS-7 |
| 2026-10-15 | [印の表に、.base と同じ中身(絞り込み・計算の列・表示名・ビュー)を TOML で書けるようにする(marker-base)](../_decisions/2026-10-15-marker-base.md) | accepted | AI | 人の発言 | WS-9 |
| 2026-10-13 | [「保管庫」をルートフォルダと言い換え、検知の値を目印のフォルダの名前にする(root-folder)](../_decisions/2026-10-13-root-folder.md) | accepted | 人 | 人の発言 | BV-2, BV-12, BV-22 ほか 6 |
| 2026-10-12 | [ヘッダーのワークスペースの名前がパンくずに移ったので、-w で開く試験の錠を掛け直す(view-bar-locks-3)](../_decisions/2026-10-12-view-bar-locks-3.md) | accepted | 人 | AI の提案 | WS-3 |
| 2026-10-12 | [右上のメニュー・3行のビューの帯で行の位置が変わる試験の錠を、まとめて掛け直す(header-menu-locks)](../_decisions/2026-10-12-header-menu-locks.md) | accepted | 人 | AI の提案 | BV-3, BV-7, BV-13 ほか 26 |
| 2026-10-12 | [余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)](../_decisions/2026-10-12-half-row-locks.md) | accepted | 人 | AI の提案 | BV-7, BV-13, BV-17 ほか 28 |
| 2026-10-12 | [パンくずのボタンでヘッダーの字が変わる試験の錠を掛け直す(crumb-button-locks)](../_decisions/2026-10-12-crumb-button-locks.md) | accepted | 人 | AI の提案 | SR-1, BV-13, NV-15, SR-42, WS-6 |
| 2026-10-11 | [ワークスペースの表を、左から重ねて出す引き出しで切り替える(ws-drawer)](../_decisions/2026-10-11-ws-drawer.md) | accepted | 人 | 人の発言 | WS-8, SR-42 |
| 2026-10-11 | [表の上を、パンくずのヘッダーと、ビューの操作を1列に集めたビューの帯の2行にする(view-bar)](../_decisions/2026-10-11-view-bar.md) | accepted | 人 | 人の発言 | SR-1, SR-42, NV-23 ほか 6 |
| 2026-10-10 | [ワークスペースのボタンは、ワークスペースが無くても出す(ws-button)](../_decisions/2026-10-10-ws-button.md) | accepted | 人 | 人の発言 | WS-6 |
| 2026-10-10 | [表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)](../_decisions/2026-10-10-scope-overrides.md) | accepted | 人 | 人の発言 | SR-44, SR-43, WS-1, WS-7, CLI-21 |
| 2026-10-10 | [設定の形を作り直し、アプリ全体の項目と表のプロファイルに分け、重なる項目をまとめる(config-v2)](../_decisions/2026-10-10-config-v2.md) | accepted | 人 | 人の発言 | CLI-3, CLI-11, CLI-12 ほか 16 |
| 2026-10-10 | [公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)](../_decisions/2026-10-10-config-v2-relock.md) | accepted | 人 | 人の発言 | BV-17, BV-19, BV-20 ほか 42 |
| 2026-10-10 | [設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)](../_decisions/2026-10-10-config-v2-locks.md) | accepted | 人 | 人の発言 | BV-17, BV-20, CE-16 ほか 34 |
| 2026-10-09 | [ワークスペース(表をまとめる範囲)をアプリの側で持ち、画面とコマンドで管理し、.git と保管庫を検知できるようにする(workspaces)](../_decisions/2026-10-09-workspaces.md) | accepted | 人 | 人の発言 | WS-1, WS-2, WS-3 ほか 5 |
| 2026-10-09 | [フォルダの .mdgrid/workspace.toml をワークスペースの印として受け、アプリの側のワークスペースより先に使う(workspace-marker)](../_decisions/2026-10-09-workspace-marker.md) | accepted | 人 | 人の発言 | WS-6, WS-7 |
| 2026-10-09 | [ワークスペースの操作を、開くパスと重ならない旗にする(workspace-flags)](../_decisions/2026-10-09-workspace-flags.md) | accepted | 人 | 人の発言 | WS-3, WS-7 |
| 2026-10-09 | [自動の表に .base を入れない(workspace-auto-folders)](../_decisions/2026-10-09-workspace-auto-folders.md) | accepted | 人 | 検証の指摘 | WS-5, WS-7 |
| 2026-10-09 | [Windows でも通るように、Unix の前提を置いた2つの試験を直して錠を掛け直す(windows-test-locks)](../_decisions/2026-10-09-windows-test-locks.md) | accepted | 人 | 人の発言 | WS-1 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| WS-1 | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md) | src/test_toml_edit_safety_unit.rs::test_ws_1_header_with_comment_keeps_neighbour, src/test_toml_edit_safety_unit.rs::test_ws_1_trailing_table_and_inline_form_survive, src/test_workspace_keep_unit.rs::test_ws_1_add_keeps_hand_written_text ほか 6 | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| WS-2 | [2026-10-09](../_decisions/2026-10-09-workspaces.md) | src/test_workspace_profile_unit.rs::test_ws_2_add_keeps_table_profile, src/ui/test_workspace.rs::test_ws_2_add_and_remove_from_palette, src/ui/test_workspace.rs::test_ws_2_open_none_says_how ほか 1 | なし |
| WS-3 | [2026-10-09](../_decisions/2026-10-09-workspace-flags.md) | tests/test_workspace_cli.rs::test_ws_3_add_list_remove_on_pipe, tests/test_workspace_cli.rs::test_ws_3_bad_input_one_line_reason, tests/test_workspace_cli.rs::test_ws_3_folder_named_workspace_opens ほか 2 | [2026-10-10-explore-fixes-3](../_changes/2026-10-10-explore-fixes-3.md) 2026-10-10 |
| WS-4 | [2026-10-09](../_decisions/2026-10-09-workspaces.md) | src/ui/test_workspace.rs::test_ws_4_linked_rows_follow_scope, src/ui/test_workspace.rs::test_ws_4_scope_limits_tables_and_relmap | なし |
| WS-5 | [2026-10-13](../_decisions/2026-10-13-root-folder.md) | src/test_workspace_unit.rs::test_ws_5_detect_vault_and_git, tests/test_root_folder.rs::test_ws_5_detect_old_names_read_with_warning, tests/test_root_folder.rs::test_ws_5_detect_reads_marker_names ほか 3 | [2026-10-13-root-folder](../_changes/2026-10-13-root-folder.md) 2026-10-13 |
| WS-6 | [2026-10-13](../_decisions/2026-10-13-root-folder.md) | src/test_workspace_chosen_unit.rs::test_ws_6_chosen_without_the_table_warns, src/test_workspace_unit.rs::test_ws_6_resolve_order, src/ui/test_view_bar.rs::test_sr_42_crumb_scope_and_table ほか 1 | [2026-10-13-root-folder](../_changes/2026-10-13-root-folder.md) 2026-10-13 |
| WS-7 | [2026-10-15](../_decisions/2026-10-15-marker-open.md) | src/test_toml_edit_safety_unit.rs::test_ws_7_marker_paths_stay_inside, src/test_workspace_more_unit.rs::test_ws_7_broken_marker_warns, src/test_workspace_profile_unit.rs::test_ws_7_marker_profiles ほか 7 | [2026-10-15-examples-studio](../_changes/2026-10-15-examples-studio.md) 2026-10-15 |
| WS-8 | [2026-10-11](../_decisions/2026-10-11-view-bar.md) | src/ui/test_crumb_button.rs::test_sr_42_crumb_button_click_key_and_arrow, src/ui/test_view_bar.rs::test_sr_42_crumb_click_toggles_drawer, src/ui/test_view_bar.rs::test_ws_8_crumb_drawer_switches_table ほか 17 | [2026-10-11-view-bar](../_changes/2026-10-11-view-bar.md) 2026-10-12 |
| WS-9 | [2026-10-15](../_decisions/2026-10-15-marker-base.md) | なし | なし |
<!-- decidespec:history:end -->
