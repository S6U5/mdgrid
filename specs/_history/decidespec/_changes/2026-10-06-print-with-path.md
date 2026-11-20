---
type: Change
id: 01M46PE5MHCE1KMTADD37YNHW1
title: --print に各行のノートのパスの列を足す --with-path(print-with-path)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# --print に各行のノートのパスの列を足す --with-path(print-with-path)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [print-with-path](../_decisions/2026-10-06-print-with-path.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-3)が、`--print` の行がどのノートのものか分からないと指摘した。CLI-14 で `--with-path` を足した。

終わりの条件: CLI-14 の例のとおり。今の `--print` の出力は変わらない。関係する要件: CLI-14・CLI-5・CLI-4・OUT-3・CLI-13(補完と man は引数の定義から作るので、新しい旗も出る)。

## 不明点と仮定

- 仮定: パスの作り方は `--pick path` の `pick_path`(src/main.rs)をそのまま使う。
- 仮定: md の形でも先頭の列に path を出す。

## 設計

- src/print.rs の `Table` に行の鍵(`RowId`。ノートの実体のパス)を持たせ、`table` で埋める。`with_path` のとき、呼ぶ側(src/main.rs の print の道)で先頭に `Column { id: "path", title: "path" }` と、各行に `pick_path` のパスの文字のセル(PrintCell::Prop(Some(Value::Str)))を差し込む(書き出しの関数は変えない。json の鍵の重なりは今の決まりのまま)。
- src/main.rs の clap の `Cli` に `with_path: bool`(`requires = "print"`)、`Command::Print` に旗を渡す。`rest` と使い方の文(src/i18n.rs の Usage、英語と日本語)、docs の該当(README の「Use it from scripts」)に足す。
- 却下: print.rs の `table` の中で path の列を作る(パスの形は起動の引数で決まり、核は引数を知らない)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | --with-path | CLI-14 | CLI-14 の例のとおりで、今の --print の試験と --help の試験が通る | src/print.rs, src/main.rs, src/i18n.rs, tests/test_print_with_path.rs, README.md, README.ja.md, tests/golden/ | test_cli_14_*(tests/test_print_with_path.rs) | 済 |

## 実装の気づき

- clap の MissingRequiredArgument は要る側(`--print`)しか名指さないので、渡された旗を引数から探して `{旗} には --print が要る` の1行にした(`needs_print`・Msg::CliRequiredBy)。同じ道を通る `--format` だけのときの文もこの形に変わった(錠のある試験はこの文を見ていない)。

## 照合

自分で照合(フレッシュ文脈でない。試験は実装を見ていない別の役が先に書き、実装は別の役が入れた)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-14 | checked | コード: src/main.rs の Cli の with_path・print_view・needs_print、src/print.rs の Table の row_ids / テスト: tests/test_print_with_path.rs::test_cli_14_*(7本) / 今: 通った(cargo test、全体 1095 passed) / 前: 落ちた(実装の前に回し、6本が「知らないオプション: --with-path」と終了コード 2 で落ちた。--print なしの1本は前から理由1行と終了コード 2 で通る)。実物: `examples/demo/Tasks.base --print --with-path` の見出しが `path,status,…`、行の先頭が `examples/demo/Tasks/Fix login redirect bug.md`。json の最初の鍵が path。`--with-path` だけ → `--with-path requires --print (see --help)` と終了コード 2 |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
