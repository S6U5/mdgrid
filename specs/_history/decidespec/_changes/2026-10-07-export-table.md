---
type: Change
id: 01M4AZJ2JE8TB4NX94R19MA376
title: 画面の表をファイルに書き出し、--print に tsv を足す(export-table)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 画面の表をファイルに書き出し、--print に tsv を足す(export-table)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-export-table.md(人の承認)・2026-10-07-export-table-details.md(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した OUT-2・OUT-5・CLI-5 のとおり。

## 不明点と仮定

- 仮定: セルの値は画面と同じく、ためた変更があればその値にする(画面に出している表なので)。
- 仮定: `--apply` で戻せるのは CSV と JSON(TSV と Markdown は改行や区切りを空白にするので、元に戻せない)。

## 設計

- src/print.rs: `Format::Tsv` と `tsv`(見出しつき、セルのタブと改行は空白)。拡張子から形を決める `Format::from_extension`。
- src/main.rs: `--format tsv`。
- src/ui/export.rs(新): パレットの `export_table`。ファイル名を聞き(Ask)、既にあれば `y` で確かめ、今の `rows`・`cols`(隠した列を除く)・見出しから print::Table を組んで書く。先頭に path の列。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | tsv の形と、画面の表の書き出し | OUT-2, OUT-5, CLI-5 | 採択した確かめ方 | src/print.rs, src/main.rs, src/i18n.rs, src/ui/export.rs, src/ui/mod.rs, src/ui/keymap.rs, src/ui/native_io.rs, src/ui/app.rs, src/ui/test_export.rs, tests/test_print_tsv.rs, tests/golden/, docs/, README.md, README.ja.md | test_out_2_export_shown_table(src/ui/test_export.rs) | 済 |

## 実装の気づき

- 読むだけの起動では、パレットはどのコマンドも出さない(BV-19 の錠のある試験)ので、書き出しも出さない。OUT-5 の「読むだけの起動でも書き出せてよい」は「してよい」なので、今は使わない。
- パレットの名前が長いとヘルプの欄に入らないので「表をファイルに書き出す」にした(形は問いの文に出す)。
- ヘルプのゴールデン(sr_5)は、コマンドが1つ増えた分だけ作り直した。

## 照合

自分で照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| OUT-2 | checked | コード: src/ui/export.rs の export_table_to・shown_table / テスト: src/ui/test_export.rs::test_out_2_export_shown_table / 今: 通った(全体 1342 passed) / 前: 落ちた(書き出しが無かった) |
| OUT-5 | checked | コード: src/ui/export.rs の resolve・export_table_write / テスト: src/ui/test_export.rs::test_out_5_export_matches_print_with_path、src/ui/test_export.rs::test_out_5_overwrite_asks_and_home / 今: 通った / 前: 落ちた |
| CLI-5 | checked | コード: src/print.rs の tsv、src/main.rs の PrintFormat / テスト: tests/test_print_tsv.rs::test_cli_5_print_tsv / 今: 通った / 前: 落ちた(tsv の形が無かった) |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
