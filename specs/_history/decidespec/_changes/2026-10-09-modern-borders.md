---
type: Change
id: 01M4E26C08RGP2FACJATJAB6QP
title: 窓の枠をつながった罫線にし、ASCII を設定で選べるようにする(SR-32)
status: active
size: full
created: 2026-10-09
updated: 2026-10-09
---

# 窓の枠をつながった罫線にし、ASCII を設定で選べるようにする(SR-32)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」(人が案 A を選んだ) |
| 仕様化 | 済 | specs/_decisions/2026-10-09-modern-borders.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 進行中 | コミット |
| 照合 | 未 | この記録の「照合」 |

## 要求

採択した SR-32。人は、今ある窓もそろえる案 A を選び、錠の掛かった試験の掛け直しも承知した(会話 2026-10-08)。大きな変更なので、人の言葉なしに main へマージしない。

## 不明点と仮定

- 仮定: 設定の名前は `borders`(`"rounded"` が既定、`"ascii"`)。知らない値は警告にして既定。
- 仮定: 列の区切り線(SR-20 の column_lines)も、ASCII の設定では `|` にする(今の ambiguous_wide と同じ)。
- 仮定: 枠の形(題の位置・幅)は今のまま、文字だけを替える。

## 設計

- src/ui/popup.rs: 枠の文字の組(Frame)と、今の設定から選ぶ関数。top_edge・下の縁・左右の縁をそれから作る。
- 窓ごと(list・listpick・menu・freq・calendar・new_note)の `+ - |` を Frame に替える。display の列の区切りも設定を見る。
- src/config.rs と docs/config(.ja).md・--print-config: `borders` の項目。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 枠の文字をそろえ、設定を足す | SR-32 | 既定で罫線、設定と ambiguous_wide で ASCII。試験が通る | src/ui/popup.rs, src/ui/list.rs, src/ui/listpick.rs, src/ui/menu.rs, src/ui/freq.rs, src/ui/calendar.rs, src/ui/new_note.rs, src/ui/display.rs, src/ui/app.rs, src/ui/startup.rs, src/config.rs, src/i18n.rs, src/ui/test_borders.rs, src/ui/, tests/, docs/, README.md, README.ja.md, specs/test-locks.json, specs/_decisions/ | test_sr_32_rounded_by_default(src/ui/test_borders.rs) | 進行中 |

## 実装の気づき

## 照合
