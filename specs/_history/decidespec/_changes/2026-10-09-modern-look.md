---
type: Change
id: 01M4E34PM7Y6X6AH6PEZP50PJ0
title: lazygit のようなモダンな見た目(SR-33)
status: active
size: full
created: 2026-10-09
updated: 2026-10-09
---

# lazygit のようなモダンな見た目(SR-33)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」(人が全体を lazygit 風にする案を選んだ) |
| 仕様化 | 済 | specs/_decisions/2026-10-09-modern-look.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 進行中 | コミット |
| 照合 | 未 | この記録の「照合」 |

## 要求

採択した SR-33。大きな変更なので、人の言葉なしに main へマージしない。

## 不明点と仮定

- 仮定: 設定の名前は `look`(`"modern"` が既定、`"classic"`)。知らない値は警告にして既定。
- 仮定: 既定のテーマ(色の表なし)のモダンの色は、アクセント = 端末の水色(Cyan)、薄い色 = 端末の暗い灰色(DarkGray)、選びの背景 = 暗い灰色(256色の 237 / RGB 58,58,70)。端末の地の色は変えない。
- 仮定: テーマ(色の表あり)のときのアクセントは列の見出しの色、薄い色は文字と地の中間、選びは選択の色。下の帯の地はテーマの帯の色のまま(SR-26 の確かめ方を保つ)。

## 設計

- src/ui/look.rs: `Look`(accent・dim・sel_bg・sel_fg)と、今の設定から選ぶ `look(app)`(classic か色なしなら None)。
- 重ねる窓: popup::blit で、縁の文字をアクセント、題を太字、反転の行を背景の色と太字に。カレンダー・新しいノートの窓も同じ。
- 下の帯(bands::footer): 反転をやめ、キーをアクセントの太字、説明を薄い色。
- タブ(bands::tabs): 選んでいるタブを札(アクセントの背景)、ほかを薄い色。ヘッダーの名前を太字。
- 列の見出し・表の選び(view.rs): 見出しはアクセントの太字(下線なし)、選んだセルは背景の色と太字。
- src/config.rs・docs: `look`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | モダンな見た目と設定 | SR-33 | 色のある表示で SR-33 の見た目、classic と色なしは今まで、文字は同じ | src/ui/look.rs, src/ui/popup.rs, src/ui/bands.rs, src/ui/view.rs, src/ui/calendar.rs, src/ui/new_note.rs, src/ui/list.rs, src/ui/mod.rs, src/ui/app.rs, src/ui/theme.rs, src/config.rs, src/config_items.rs, src/i18n.rs, src/ui/test_look.rs, docs/, README.md, README.ja.md, tests/, specs/test-locks.json | test_sr_33_modern_popup_uses_background(src/ui/test_look.rs) | 進行中 |

## 実装の気づき

## 照合
