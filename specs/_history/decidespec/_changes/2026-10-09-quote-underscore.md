---
type: Change
id: 01M4G0XVM2Q5S5J7KWTAQX95S1
title: _ を除くと数に読める文字の値を囲んで書く(WB-7)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# _ を除くと数に読める文字の値を囲んで書く(WB-7)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(WB-7 の「読み直して同じ文字列」に合わせる) | — |
| 設計 | 対象外: 1か所 | — |
| タスク | 対象外: 1か所 | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

PR #17 の CI(Rust 1.90)で、乱数で値を作る試験 test_wb_7_props_text_reads_back_as_same_string が、文字の値 `_0` を囲まずに書くのを見つけた(`_` を除くと数として読む道具がある)。src/writeback.rs の plain_safe で、`_` を除くと数に読める形(数字を含むもの)は囲む。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-7 | checked | コード: src/writeback.rs(plain_safe) / テスト: src/test_writeback_underscore_unit.rs::test_wb_7_underscore_numbers_are_quoted、tests/test_writeback_props.rs |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
