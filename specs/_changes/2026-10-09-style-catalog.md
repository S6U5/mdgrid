---
type: Change
id: 01M4G2TK6TVKYAMPWCH3J5Q4YJ
title: 部品の形とテーマを設定で選べるようにし、カタログをリポに置く(SR-33・SR-35〜SR-39)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# 部品の形とテーマを設定で選べるようにし、カタログをリポに置く(SR-33・SR-35〜SR-39)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-style-catalog.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した SR-36(部品の形の設定と組)・SR-37(新しいテーマ)・SR-38(カタログ)・SR-39(端末の明るさに従うテーマ)と、SR-33・SR-35 の変更。大きな変更なので、人の言葉なしに main へマージしない。

## 不明点と仮定

- 仮定: 既定の組は sumi(人が選んだ)。色を使わない表示では、部品の形のうち文字で意味が分かるもの(点・形・括弧)はそのまま使い、色だけのもの(text の色)は文字のまま。
- 仮定: classic の組は今までの見た目(chip の札・塗る選び・丸い札のタブ)に当たり、look = "classic" と合わせて今までの画面になる。
- 仮定: 端末への問い合わせ(OSC 11)は unix の /dev/tty で、答えを短く待つ。Windows と答えの無い端末では COLORFGBG、無ければ暗い地。

## 設計

- src/style.rs(新): Style(部品ごとの enum)・組(preset)・[style] の読み取り。config.rs に style と theme_light・theme_dark・nerd_font。borders = "ascii" は frames = ascii。
- src/theme.rs: sumi・slate・saas・saas-dark・paper のテーマと Auto。
- src/ui/cell.rs・view.rs・chips.rs: status・tags・check の形。select・rules の描き方。
- src/ui/bands.rs: tabs・band の形。src/ui/popup.rs: frames。
- src/main.rs か src/ui: theme = auto の解決(COLORFGBG・OSC 11)。
- docs/catalog/index.html と、カタログと本体の値を突き合わせる試験。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 設定と組とテーマ | SR-36, SR-37 | 読み取りの試験が通る | src/style.rs, src/lib.rs, src/config.rs, src/config_items.rs, src/theme.rs, src/i18n.rs, src/test_style_unit.rs, docs/ | test_sr_36_style_presets_and_overrides(src/test_style_unit.rs) | 済 |
| 2 | 画面の部品 | SR-33, SR-35, SR-36 | 画面の試験が通る | src/ui/, tests/golden/, docs/ | test_sr_36_default_sumi_screen(src/ui/test_style_screen.rs) | 済 |
| 3 | 端末に従うテーマ | SR-39 | 試験が通る | src/theme.rs, src/main.rs, src/ui/, tests/ | test_sr_39_auto_theme(src/test_theme_auto_unit.rs)、test_sr_39_e2e_auto_theme_without_answer_starts(tests/test_theme_auto_e2e.rs) | 済 |
| 4 | カタログ | SR-38 | 突き合わせの試験が通る | docs/catalog/, tests/ | test_sr_38_catalog_matches_config・test_sr_38_catalog_presets_match・test_sr_38_catalog_is_self_contained(tests/test_catalog.rs) | 済 |

## 実装の気づき

- 人の指摘(会話 2026-10-09 の要約: 丸い札にしても、押したあとや候補の窓で四角くなるのは違和感)を受けて、候補から選んでいる間(まだ打っていない)のセルを部品の形のまま見せ、候補の窓の候補も札の形のまま(選んでいる候補も。塗る・反転の選びは今まで)にした。丸い端は Nerd Font のときだけで、pill に加えて tint も両端を丸い端の字にする(幅は同じ)。窓は丸い端の分だけ1桁広げる。テスト: src/ui/test_style_list_pill.rs。カタログも端末と同じに(tint の丸みは Nerd Font のときだけ)。

- select = "bar" の左の印: SR-36 の例は `▌` だが、SR-33 の「どの見た目でも画面の文字は同じ」(と錠つきの試験)とぶつかるので、印の文字は `>` のまま、アクセントの色の太字と行の背景の色で線に見せた。例の `▌` とのずれは、人に確かめる(gap)。
- 状態の語の意味の色(done=緑・doing=黄・blocked=赤・todo=薄い)は英語の語だけ。日本語の語は SR-23(コードに日本語のリテラルを置かない)のため外し、値の文字から決まる色にした。
- 色を使わない表示と look = "classic" では、選び・線・タブ・帯の形は今まで(SR-33)。値の部品(点・区切り)は色のある表示だけ(SR-35)。窓の枠(frames)はどの表示でも効く。
- 錠つきの試験のうち、今までの見た目(chip の札・塗る選び・keys の帯)を確かめるもの(test_look の2つ、test_rich_cells・test_rich_cells_more)は、設定に preset = "classic" を足し、この決定で掛け直した。

- カタログ(タスク4)は、検索の欄と設定の帯の形を「案(まだ設定に無い)」として並べ、設定の文には出さない(採るなら別の提案で要件にする)。
- カタログ(タスク4)は、人の依頼(会話 2026-10-09 の要約: HTML で UI の全体をお試しで見られるように)により、表と関係マップだけでなく、画面の全体(タブ・表・下の帯・詳細・候補の窓・検索と絞り込み・ヘルプ)をクリックで試せる形にする。

## 照合

書込なしの検証役が、要件の条件ごとに確かめた(42 条件。cargo test は 1508 通過・0 失敗)。指摘のうち実装で直せるものは直した(下の「直した」)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-33 | checked | 枠と題のアクセントの色は、frames のどの形でも(直した: src/ui/popup.rs の modern_row。テスト: src/ui/test_style_frames.rs::test_sr_33_frame_accent_in_every_frame_shape)。反転しない選び・classic と色なし・文字は同じ(テスト: src/ui/test_look.rs の各試験) |
| SR-35 | checked | plain・色なし・部品ごと・列ごと・知らない値・値を変えない(テスト: src/ui/test_rich_cells.rs、src/ui/test_rich_cells_more.rs、src/ui/test_style_screen.rs::test_sr_36_no_color_keeps_text) |
| SR-36 | checked | 名前・組と上書き・既定 sumi・`● doing`・`ui · web`・saas・chip・pill と nerd_font・知らない値・rules・cross・frames・窓の値の色(テスト: src/test_style_unit.rs、src/ui/test_style_screen.rs、src/ui/test_style_overlay.rs)。borders = "ascii" と frames = "ascii" の列の区切りの差は直した(src/ui/display.rs の col_sep) |
| SR-37 | checked | テスト: src/ui/test_themes.rs::test_sr_27_theme_names、src/test_style_unit.rs、src/ui/test_themes.rs の全テーマを回す試験 |
| SR-38 | checked | テスト: tests/test_catalog.rs の3つ。カードの値が名前の表の外から出る穴は直した(カードを NAMES から並べ、覚えていた上書きも NAMES で確かめる) |
| SR-39 | checked | テスト: src/test_theme_auto_unit.rs::test_sr_39_auto_theme・test_sr_39_osc11_reply、tests/test_theme_auto_e2e.rs::test_sr_39_e2e_auto_theme_without_answer_starts |

既存のテストの削除・skip・弱体化: なし(錠つきの試験のうち、今までの見た目を確かめるものに preset = "classic" を足し、この決定で掛け直した。確かめる中身は同じ)

残した軽い点: termbg の問い合わせの間(最大 200ms)に打ったキーは捨てることがある。`rgba:` で答える端末は「分からない」(暗い地)になる。

例と既定の食い違い(SR-33・SR-35・SR-36 の例)は、人の決定(specs/_decisions/2026-10-10-style-examples.md)で例を既定の sumi に合わせて解いた。

確かめた: 6 / 6

