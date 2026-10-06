---
type: Change
id: 01M47AE04B326K1VM9BXMFGF1W
title: BOM つきのノートの値を、読むだけのまま見せる(bom-values)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# BOM つきのノートの値を、読むだけのまま見せる(bom-values)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: BV-1(フロントマターのキーを列に)のとおりに値を見せる不具合の直し。WB-5 の「書き込まず、読むだけにして理由を表示」は変えない | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

4回目の点検(docs/todo.md の B-4)。BOM で始まるノートは、YAML が読めても行の全部のセルが空に見え、--print でも空。Windows で書いたノートの多い保管庫が、値の無い表に見える。BOM を除いて読めるなら値を見せ(列にも入れ)、書き込みは今までどおりしない(WB-5)。

## 不明点と仮定

- 仮定: parse は今までどおり BOM を Err(Bom) で返す(錠のある試験が確かめる)。読み込み口で、BOM を除いたバイトを別に読み、見せる値・列・式の値にだけ使う。lock(理由)は今までどおり。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | BOM のノートの値を見せる | BV-1, WB-5 | 上の要求のとおり | src/source/markdown.rs, src/ui/mod.rs, src/ui/test_bom_values.rs, docs/safety(.ja).md | test_wb_5_bom_values_shown_read_only(src/ui/test_bom_values.rs) | 済 |

## 実装の気づき

- \`Parsed\` に \`shown\`(BOM を除いて読んだフロントマター)を足し、見せる値(get)・列(add_columns)・候補の値(values)に使う。lock は今までどおり Err(Bom) から理由を出す。tags と本文のリンク・本文の始まりは今までどおり読めたフロントマターだけから(BOM の3バイトで位置がずれるため使わない)。
- 点検の B-4 の後半(アンカー・タグつきの値が \`{…}\`)は CE-8 で読むだけの形で、値を推し量らない今の扱いのまま(直さない)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。見せる値を足すだけで、書き込みの道は変えないため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-5 | checked | コード: src/source/markdown.rs の parse_note・get(lock は今までどおり) / テスト: src/ui/test_bom_values.rs::test_wb_5_bom_values_shown_read_only(編集が開かず、保存してもバイトが同じ)、錠のある tests/test_empty_frontmatter.rs・tests/test_language_lib.rs・src/ui/test_action_menu.rs の BOM の形 / 今: 通った(全体 1256 passed) / 前: 落ちた(値が None) |
| BV-1 | checked | コード: Parsed::readable と add_columns の3か所 / テスト: 同上(BOM のノートにしか無いキー owner が列になる) / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
