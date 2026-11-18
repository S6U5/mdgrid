---
type: Proposal
id: 01M4E75P33DCCKS8DTJ2XHB7SF
title: ワークスペースの操作を、開くパスと重ならない旗にする(workspace-flags)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 2c0007255ec33578
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-09
touches: [WS-3, WS-7]
created: 2026-10-09
updated: 2026-10-09
---

# ワークスペースの操作を、開くパスと重ならない旗にする(workspace-flags)

## きっかけ

`mdgrid workspace list` の形では、`workspace` という名前のフォルダを開く `mdgrid workspace` と見分けられない。人が、フォルダのパスと重なるならよくないと言い、AI が出した旗の案(`--workspaces`・`--add-to`・`--remove-from`・`--remove-workspace`・`--init-workspace`)について、解決の形は一般的なやり方に任せて進めてよいとした(会話 2026-10-09)。

人の依頼を AI が要件の文に起こした。名前「ワークスペース」は変えない(一般的な呼び名のため)。確かめ方の列も、新しい形に合わせて変える(WS-3: `mdgrid ~/notes/tasks --add-to Product` → `mdgrid --workspaces` に出る。`workspace` という名前のフォルダは `mdgrid workspace` で開ける。WS-7: `mdgrid notes --init-workspace` → notes/.mdgrid/workspace.toml ができる)。

## 差分

- 変更: WS-3 旧「コマンドの引数で、ワークスペースの一覧を出す(`mdgrid workspace list`)・表を足す(`mdgrid workspace add <名前> <パス> [--as <表の名前>]`。ワークスペースが無ければ作る)・外す(`mdgrid workspace remove <名前> [<パス>]`。パスが無ければワークスペースごと)・ワークスペースで開く(`mdgrid -w <名前> [<パス>]`。パスが無ければ最初の表)ができるべき。画面を出さない操作は、標準出力がパイプでも動くべき。」→ 新「コマンドの引数で、ワークスペースの一覧を出す(`mdgrid --workspaces`)・表を足す(`mdgrid <パス> --add-to <名前> [--as <表の名前>]`。ワークスペースが無ければ作る)・表を外す(`mdgrid <パス> --remove-from <名前>`)・ワークスペースごと消す(`mdgrid --remove-workspace <名前>`)・ワークスペースで開く(`mdgrid -w <名前> [<パス>]`。パスが無ければ最初の表)ができるべき。操作は、開くパス(位置の引数)と重ならない旗で書くべき(同じ名前のフォルダを開けなくしないため)。画面を出さない操作は、標準出力がパイプでも動くべき。」
- 変更: WS-7 旧「フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対)で、表を書かなければ根の直下の、ノートのあるフォルダと `.base` のファイルを表にする。`mdgrid workspace init [<フォルダ>]` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。」→ 新「フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対)で、表を書かなければ根の直下の、ノートのあるフォルダと `.base` のファイルを表にする。`mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。」

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(`mdgrid workspace …`) | `workspace` という名前のフォルダを、`./workspace` と書かないと開けない |
| 別の実行ファイル(`mdgrid-workspace`) | 入れるものが増え、補完と `--help` に並ばない。ほかの機能(`--print`・`--apply`)は旗でそろっている |

## 承認の記録

2026-10-09 人の承認(要約: フォルダのパスと重なる形はよくないとし、解決の形は一般的なやり方に任せて進めてよいとした)
