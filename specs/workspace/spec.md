---
type: Spec
id: 01M4E5CQZKK9X940T902KWBQ8A
title: ワークスペース(workspace)
description: 表をまとめる範囲(ワークスペース)をアプリの側に持ち、画面とコマンドで管理し、.git と保管庫を検知する。
status: active
load_when: ワークスペースの持ち方・管理(画面とコマンド)・検知・範囲の決め方を作る・変えるとき
created: 2026-10-09
updated: 2026-10-10
---

# ワークスペース(workspace)

ワークスペースは、名前の付いた表(フォルダか `.base`)の集まりで、リンクの行き先・つながった行・表どうしのつながり・関係マップ([relations](../relations/spec.md))の範囲になる。設定のフォルダの `workspaces.toml` に持ち、画面のパレットとコマンドの引数で管理する。フォルダに `.mdgrid/workspace.toml` の印があれば、そちらを先に使う(WS-7)。書いたワークスペースが無いときは、全体の設定で選んだ検知(Obsidian の保管庫・git のリポ)で範囲を決め、それも無ければ登録した表([cli](../cli/spec.md) の CLI-18。よく開く場所のブックマーク)を使う。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — ワークスペース

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| WS-1 | ワークスペース(名前の付いた表の集まり)を、設定のフォルダの `workspaces.toml`(`[[workspace]]` の並び。`name` と、表の並び `table`(`name`・`path`(フォルダか `.base`)・`view`)と、ワークスペースと表それぞれの表のプロファイル(SR-44。`[workspace.look]`・`[workspace.table.look]` など))に持つべき。ノートのフォルダには何も書かない。`path` の先頭の `~` はホームのフォルダ。読めない行は警告にして飛ばし、起動を止めてはならない。 | `[[workspace]]` に name = "Product"、表を2つ → 2つの表のワークスペース。path の無い表 → 警告して飛ばす。`[workspace.look] theme = "nord"` → そのワークスペースの表が nord(`test_ws_1_*`) | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-2 | 画面のパレットから、ワークスペースを作る・今の表をワークスペースに足す(今のワークスペースから選ぶか新しく作る)・今の表をワークスペースから外す・ワークスペースを開く(一覧から選び、その表を開く)ができるべき。 | 「この表をワークスペースに足す」→ 名前を選ぶ → workspaces.toml にその表が入る。「ワークスペースを外す」→ 消える(`test_ws_2_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-3 | コマンドの引数で、ワークスペースの一覧を出す(`mdgrid --workspaces`)・表を足す(`mdgrid <パス> --add-to <名前> [--as <表の名前>]`。ワークスペースが無ければ作る)・表を外す(`mdgrid <パス> --remove-from <名前>`)・ワークスペースごと消す(`mdgrid --remove-workspace <名前>`)・ワークスペースで開く(`mdgrid -w <名前> [<パス>]`。パスが無ければ最初の表)ができるべき。操作は、開くパス(位置の引数)と重ならない旗で書くべき(同じ名前のフォルダを開けなくしないため)。画面を出さない操作は、標準出力がパイプでも動くべき。 | `mdgrid ~/notes/tasks --add-to Product` → `mdgrid --workspaces` に出る。`workspace` という名前のフォルダは `mdgrid workspace` で開ける。`mdgrid -w Product` → 最初の表で開く。無い名前 → 理由1行と 0 以外の終了コード(`test_ws_3_*`) | [2026-10-09](../_decisions/2026-10-09-workspace-flags.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-4 | 開いた表が属するワークスペース(`-w` で選んだもの、無ければ開いた表を含む最初のワークスペース)があれば、リンクの行き先を探す範囲・つながった行・表どうしのつながり・関係マップ・上の端のタブは、そのワークスペースの表(と今の表)だけを使うべき。 | ワークスペース Work に tasks と projects、Hobby に books → tasks で関係マップを開くと tasks と projects だけ。books を指すリンクは行き先の表に数えない(`test_ws_4_*`) | [2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-5 | 全体の設定 `[workspace]` の `detect`(`"vault"`・`"git"` の並び。既定は `["vault"]`)で、書いたワークスペースが無い表を開いたときに、開いたフォルダから上へたどって最初に見つかった Obsidian の保管庫の根(`.obsidian/` のあるフォルダ)か git のリポの根(`.git` のあるフォルダ)を、名前の無いワークスペースとみなすべき。その表は、根の直下の、ノートのあるフォルダにするべき(`.base` は入れない。`.base` の表は保管庫の根を指すので、自動で入れるとほかの表と重なるため。要れば書いて足す)。 | `.obsidian/` のある保管庫の tasks を開く → 保管庫の直下の projects も関係マップに出る。`[workspace] detect = []` → 検知しない(`test_ws_5_*`) | [2026-10-10](../_decisions/2026-10-10-config-v2.md)、[2026-10-09](../_decisions/2026-10-09-workspace-auto-folders.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-6 | 範囲の決め方は、`-w` で選んだワークスペース → 開いたフォルダから上へたどって最初に見つかった `.mdgrid/workspace.toml`(WS-7)→ 開いた表を含む `workspaces.toml` のワークスペース → 検知(WS-5)→ 登録した表(CLI-18)の順にするべき。今どの範囲かを、ヘッダーかメッセージ行に示すべき。ヘッダーの近道のボタン(SR-42)の「ワークスペース」の字は範囲を示す表示ではなく、ワークスペースが1つも無いときも出してよい(範囲は名前で示す)。 | 書いたワークスペースと保管庫の両方に入る表 → 書いたほう。どれにも入らない → 登録した表(今までどおり)。ヘッダーにワークスペースの名前(`test_ws_6_*`) | [2026-10-10](../_decisions/2026-10-10-ws-button.md)、[2026-10-09](../_decisions/2026-10-09-workspace-marker.md)、[2026-10-09](../_decisions/2026-10-09-workspaces.md) |
| WS-7 | フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対。ワークスペースの表のプロファイルは最上位に、表のものは `[table.look]` などに書く。SR-44)で、表を書かなければ根の直下の、ノートのあるフォルダを表にする(`.base` は WS-5 と同じ理由で入れない)。`mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。 | notes/.mdgrid/workspace.toml(表なし)→ notes の直下の tasks・projects がワークスペースの表。table を1つ書く → その表だけ。印に `[look] theme = "nord"` → その下の表が nord。`mdgrid notes --init-workspace` → notes/.mdgrid/workspace.toml ができる(`test_ws_7_*`) | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md)、[2026-10-09](../_decisions/2026-10-09-workspace-auto-folders.md)、[2026-10-09](../_decisions/2026-10-09-workspace-flags.md)、[2026-10-09](../_decisions/2026-10-09-workspace-marker.md) |

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
| 2026-10-10 | [表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)](../_decisions/2026-10-10-scope-overrides.md) | accepted | 人 | 人の発言 | SR-44, SR-43, WS-1 ほか 2 |
| 2026-10-10 | [設定の形を作り直し、アプリ全体の項目と表のプロファイルに分け、重なる項目をまとめる(config-v2)](../_decisions/2026-10-10-config-v2.md) | accepted | 人 | 人の発言 | CLI-3, CLI-11, CLI-12 ほか 16 |
| 2026-10-10 | [ワークスペースのボタンは、ワークスペースが無くても出す(ws-button)](../_decisions/2026-10-10-ws-button.md) | accepted | 人 | 人の発言 | WS-6 |
| 2026-10-09 | [ワークスペース(表をまとめる範囲)をアプリの側で持ち、画面とコマンドで管理し、.git と保管庫を検知できるようにする(workspaces)](../_decisions/2026-10-09-workspaces.md) | accepted | 人 | 人の発言 | WS-1, WS-2, WS-3 ほか 5 |
| 2026-10-09 | [フォルダの .mdgrid/workspace.toml をワークスペースの印として受け、アプリの側のワークスペースより先に使う(workspace-marker)](../_decisions/2026-10-09-workspace-marker.md) | accepted | 人 | 人の発言 | WS-6, WS-7 |
| 2026-10-09 | [ワークスペースの操作を、開くパスと重ならない旗にする(workspace-flags)](../_decisions/2026-10-09-workspace-flags.md) | accepted | 人 | 人の発言 | WS-3, WS-7 |
| 2026-10-09 | [自動の表に .base を入れない(workspace-auto-folders)](../_decisions/2026-10-09-workspace-auto-folders.md) | accepted | 人 | 検証の指摘 | WS-5, WS-7 |
| 2026-10-09 | [Windows でも通るように、Unix の前提を置いた2つの試験を直して錠を掛け直す(windows-test-locks)](../_decisions/2026-10-09-windows-test-locks.md) | accepted | 人 | 人の発言 | WS-1 |
<!-- decidespec:history:end -->
