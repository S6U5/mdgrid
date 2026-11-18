---
type: Change
id: 01M47GZG1XJ0WN4NGD03Q3YGVM
title: JSON・フローの形のフロントマターの値を、読むだけのまま見せる(flow-frontmatter)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# JSON・フローの形のフロントマターの値を、読むだけのまま見せる(flow-frontmatter)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: BV-1 のとおりに値を見せる不具合の直し。WB-5 の「YAML として読めない(書き方が読めない)ノートは書かず理由を出す」は変えない | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

5回目の点検(docs/todo.md の C-4)。`---\n{"tags": ["json"], "status": "todo"}\n---` のノートが、画面では全部 `#`、--print では黙って空、hasTag にも当たらない。Obsidian は JSON のフロントマターを読む。値を見せ(列・file.tags にも)、書き込みは今までどおりしない(WB-5)。

## 不明点と仮定

- 仮定: 区切りの間の中身が `{` で始まる(フローのマップ。JSON も含む)ときだけ、YAML の読み手(saphyr。.base と同じ、別名の展開の上限つき)で読む。ほかの読めない YAML は今までどおり。
- 仮定: 値は第1階層だけ(文字・数・真偽・null・スカラーのリスト)。入れ子はほかと同じく `{…}`。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | フローのフロントマターの値を見せる | BV-1, WB-5, BV-6 | 上の要求のとおり | src/frontmatter.rs, src/base.rs, src/source/markdown.rs, tests/test_flow_frontmatter.rs, docs/safety(.ja).md | test_wb_5_flow_frontmatter_*(tests/test_flow_frontmatter.rs) | 済 |

## 実装の気づき

- 見せる値(Parsed.shown)は BOM と同じ仕組みに足した。タグとリンクは、ファイルのバイトの位置が合うフロントマター(読めたものか、フローの形)から拾う(BOM の見せる値は3バイトずれるので、今までどおり使わない)。
- 別名の展開の上限は .base の読み取りの check_aliases をそのまま使う(pub(crate) にした)。

## 照合

自分で照合(見せる値を足すだけで、書き込みの道は変えない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-1 | checked | コード: src/frontmatter.rs の flow_frontmatter、src/source/markdown.rs の parse_note / テスト: tests/test_flow_frontmatter.rs::test_wb_5_flow_frontmatter_values_shown / 今: 通った(全体 1289 passed) / 前: 落ちた(値が空) |
| WB-5 | checked | コード: 書き込みの道は変えない(parse は InvalidYaml のまま) / テスト: tests/test_flow_frontmatter.rs::test_wb_5_flow_frontmatter_stays_read_only / 今: 通った / 前: 通った(読むだけのまま) |
| BV-6 | checked | コード: src/source/markdown.rs の parse_note の tags / テスト: tests/test_flow_frontmatter.rs::test_wb_5_flow_frontmatter_values_shown(hasTag の .base) / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
