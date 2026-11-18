---
type: Change
id: 01M477R5QF3JM5S69JGY20TKKX
title: テーマで、ためる変更・差分・一致の強調の色もテーマの地で読める色にする(theme-meaning-colors)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# テーマで、ためる変更・差分・一致の強調の色もテーマの地で読める色にする(theme-meaning-colors)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-theme-meaning-colors.md |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の A-6。採択した SR-28 のとおり。

## 不明点と仮定

- 仮定: 色は各テーマの元の配色(Nord・Solarized・Dracula・Gruvbox の公式の色)の緑・赤・黄から取り、Pink Monster と Dozy Pink は地に合わせて決める。

## 設計

- `Palette` に add・del・pending・hl_bg の4つの役割を足す(表の行は12から16の値)。
- 決まった色を返していた ui/view.rs の pending_color・highlight と ui/review.rs の diff_color に、テーマを渡し、パレットがあればその色(256 色の端末では to_indexed)を返す。後段の塗り替え(ui/theme.rs)は、自分の色を持つセルを塗り替えないので、そのまま残る。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 意味の色をテーマから | SR-28 | 採択した確かめ方のとおり | src/theme.rs, src/ui/view.rs, src/ui/review.rs, src/ui/theme.rs, src/ui/mod.rs, src/ui/test_theme_meaning.rs | test_sr_28_*(src/ui/test_theme_meaning.rs) | 済 |

## 実装の気づき

- 各テーマの意味の色は、地との明るさの差(相対輝度の近似で 60 より大)と、強調の地と文字の差(80 より大)を試験で確かめる(test_sr_28_every_theme_has_readable_meaning_colors)。Nord の赤 #bf616a が 66 で最も小さい(Nord の公式の赤)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。色の表と、決まった色を返す3つの関数にテーマを渡すだけの変更のため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-28 | checked | コード: src/theme.rs の Palette(add・del・pending・hl_bg)、src/ui/theme.rs の meaning、src/ui/view.rs・src/ui/review.rs の呼び出し / テスト: src/ui/test_theme_meaning.rs::test_sr_28_pending_cell_uses_theme_color・test_sr_28_diff_lines_use_theme_colors・test_sr_28_highlight_background_uses_theme_color・test_sr_28_default_theme_keeps_colors・test_sr_28_every_theme_has_readable_meaning_colors、錠のある src/ui/test_themes.rs::test_sr_26_every_theme_keeps_the_text / 今: 通った(全体 1248 passed) / 前: 落ちた(実装の前は Palette に役割が無く組み立てられなかった) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
