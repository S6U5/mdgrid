---
type: Change
id: 01M4GNXMB0FYPG7KCPGXPVH38F
title: 組 dozy-pink(SR-36)
status: done
size: tiny
created: 2026-10-10
updated: 2026-10-10
---

# 組 dozy-pink(SR-36)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 組の中身は採択の文のとおり | — |
| 仕様化 | 済 | specs/_decisions/2026-10-10-dozy-preset.md(人) |
| 設計 | 対象外: 組の表に1つ足すだけ | — |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した SR-36 の組 dozy-pink。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 組 | SR-36 | 試験が通る | src/style.rs, src/config_items.rs, docs/ | test_sr_36_dozy_pink_preset(src/test_dozy_unit.rs)、test_sr_38_catalog_presets_match(tests/test_catalog.rs) | 済 |

## 実装の気づき

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-36 | checked | テスト: src/test_dozy_unit.rs::test_sr_36_dozy_pink_preset、tests/test_catalog.rs::test_sr_38_catalog_presets_match(カタログの組の中身と Style::of が同じ) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
