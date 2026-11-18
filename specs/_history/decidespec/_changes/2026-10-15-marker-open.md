---
type: Change
id: 01M4XK1XVVN6SX4HVESEVTS5JF
title: 印のあるフォルダを開いたら、印に書いた表を開く(marker-open)
status: active
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 印のあるフォルダを開いたら、印に書いた表を開く(marker-open)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [2026-10-15-marker-open](../_decisions/2026-10-15-marker-open.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 進行中 | (実装のコミット) |
| 照合 | 未 | この記録の「照合」 |

## 要求

タスクの台帳のフォルダ(根に `tasks.base` と、台帳でない AGENTS.md・CLAUDE.md・README.md)で mdgrid を開くと、台帳でないファイルまで表に入る。フォルダの印 `.mdgrid/workspace.toml` に開く表(`open`)を書けるようにし、印の根のフォルダを開いたら(引数なし・`mdgrid .`・パスで渡す)その表を開く。素の表は `--raw` で開く(会話 2026-10-15。決定 [2026-10-15-marker-open](../_decisions/2026-10-15-marker-open.md))。終わりは、CLI-1・WS-7 の新しい例が試験で通り、台帳のフォルダに印を置いて `mdgrid` で tasks.base の表が開くこと。要件: CLI-1・WS-7。

## 不明点と仮定

- 仮定: 表を切り替えたとき(登録した表の一覧・ワークスペースの引き出し)には `open` を使わない(CLI-1 は起動の引数のフォルダだけを言う)。外れたら: 切り替えでも使う提案を起こす。
- 仮定: `-w <名前>` を引数のパスなしで使ったとき(ワークスペースの最初の表を開く)は `open` を使わない。パスを渡したときは使う。外れたら: 起動の分岐を直す。
- 仮定: `--apply` には使わない(要件が `--print`・`--check`・`--print-config --resolved` だけを挙げる。`--apply` はフォルダの下のどのノートの path も受けられるほうがよい)。
- 仮定: `--print --with-path` と `--pick path` のパスは、`open` で開いた表のパスを起動の引数として作る(`.base` を直接渡したときと同じ形)。

## 設計

- 変える場所: `src/workspace.rs` に印の `open` を読む関数(`marker_open(root) -> (Option<WsTable>, Vec<String>)`)を足し、`read_marker` が最上位の `open` を知らない項目として警告しないようにする。`src/main.rs` に `--raw` の旗(Options に `raw`)と、起動の引数を印の表に置き換える関数を足し、`Run`・`Pick`・`Print`・`Check`・`PrintResolved` の分岐で呼ぶ。置き換えたときは、引数なしでも登録した表の一覧(CLI-19)を重ねない。印の `view` は `--view` が無いときだけ使う。警告は画面では起動の警告(Startup の warnings)に、画面を出さない起動では標準エラーに出す。
- 変えない場所: 表を切り替える道(run_switching)、`--apply`、印を上へたどる範囲の決め方(WS-6)。
- 文言: `locales/{ja,en}.toml` の使い方に `--raw`、`open` の警告の理由。docs/config.md と docs/config.ja.md の印の説明に `open`。
- 対応: CLI-1 → main.rs の置き換えと `--raw`、WS-7 → workspace.rs の `marker_open` と警告。
- 却下: 印を `open_app` の中で読む案(表を切り替える道にも効いてしまい、`--pick` のパスの組み立てと食い違う)。
- 検証: `cargo test`、`./ci.sh`(あれば)、`python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く | CLI-1, WS-7 | 新しい試験が実装前に落ちる | `tests/test_marker_open.rs` | test_cli_1_marker_open_* / test_ws_7_marker_open_* | 済 |
| 2 | 印の `open` を読む | WS-7 | `open` の表が引け、無い名前は `.mdgrid/workspace.toml: open:` の警告 | `src/workspace.rs` `locales/ja.toml` `locales/en.toml` | test_ws_7_marker_open_* | 済 |
| 3 | 起動で印の表に置き換え、`--raw` を足す | CLI-1 | 引数なし・`.`・パスで印の表、`--raw` で素の表、一覧を重ねない | `src/main.rs` `locales/ja.toml` `locales/en.toml` | test_cli_1_marker_open_* | 済 |
| 4 | 文書に `open` と `--raw` を書く | CLI-1, WS-7 | docs の印の節と使い方に載る | `docs/config.md` `docs/config.ja.md` `docs/manual/ja/tasks.md` `docs/manual/en/tasks.md` `examples/studio/.mdgrid/workspace.toml` | 読んで判定 | 済 |

## 実装の気づき

- 錠のある src/test_main.rs が Options を構造体の字で作るので、`--raw` は Options に入れず main の先頭で読む。
- レビューの指摘: 印の表の `view` は open_app に渡すと mdgrid のビューで起動できない(.base でない表)ので、開いてから select_view_named で選ぶ。`-w` だけかは位置の引数で見る(`--print` などの旗が残るため)。
- 範囲外で見つけた前からの不具合: `-w X --print`(パスなし)は X の最初の表ではなく今のフォルダを出す(workspace_paths に渡す no_args が旗を数える)。この変更では直さない。
- 見本 examples/studio の印に open = "Tasks" を足した(サンプル刷新のスレッドの依頼)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-1 | unassessed | |
| WS-7 | unassessed | |

既存のテストの削除・skip・弱体化: なし

確かめた: 0 / 2
