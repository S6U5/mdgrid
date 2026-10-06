---
type: Change
id: 01M47EW2N2EP0EPZZ3B32T9NND
title: --sort の綴り、--apply の捨てた直しの知らせと文言(cli-polish)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# --sort の綴り、--apply の捨てた直しの知らせと文言(cli-polish)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: CLI-16・CLI-17 の中の直し(知らせと文言。書く値は変えない) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

5回目の点検(docs/todo.md の C-8・C-9・C-10)。

- C-9: `--sort` の知らない列(ビューの列にもノートのキーにも `file.*`・`formula.*` の列にも無い)を黙って受ける。理由1行と終了コード 2。
- C-8: `--apply` が `file.*`・`formula.*` の列の直しを黙って捨てる。捨てた件数と列を標準エラーに1行。
- C-10: `1 files would change` の単数・複数。重なった行の理由に、最初に当てた行を出す。

## 不明点と仮定

- 仮定(C-9): 知っている列は、開いたビューの列の id、ノートのキー(BV-1 の範囲)、`file.` で始まる名前、`.base` の formulas の名前(`formula.x`)。`file.` の名前の綴りまでは確かめない(式の評価が理由を出す)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 3つの直し | CLI-16, CLI-17 | 上の要求のとおり | src/main.rs, src/apply.rs, src/i18n.rs, tests/test_cli_polish.rs | test_cli_16_sort_unknown_column, test_cli_17_ignored_columns_noted, test_cli_17_wording(tests/test_cli_polish.rs) | 済 |

## 実装の気づき

- 書けない列の直しは、今の \`--print\` の表の文字(式の列を含む)と比べ、表に無い \`file.*\` の文字の列(name・basename・ext・path・folder)はノートのファイルの属性と比べる。出したまま戻した列では知らせない。
- 式の名前は \`Base::formula_names\`(BV-19 の書き出しのもの)をそのまま使った。

## 照合

自分で照合(知らせと文言の直し。書く値は変えない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-16 | checked | コード: src/main.rs の print_view の綴りの確かめ / テスト: tests/test_cli_polish.rs::test_cli_16_sort_unknown_column / 今: 通った(全体 1281 passed) / 前: 落ちた |
| CLI-17 | checked | コード: src/apply.rs の plan(ignored・最初の行)、src/main.rs の apply_file / テスト: tests/test_cli_polish.rs::test_cli_17_ignored_columns_noted・test_cli_17_wording、tests/test_apply.rs・tests/test_apply_review.rs / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
