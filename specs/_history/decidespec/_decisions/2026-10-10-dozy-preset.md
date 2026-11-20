---
type: Proposal
id: 01M4GNHR40YAQJ9FGA1W0W4AS5
title: 組 dozy-pink を足す(dozy-preset)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: d11c0d37408b6beb
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [SR-36]
created: 2026-10-10
updated: 2026-10-10
---

# 組 dozy-pink を足す(dozy-preset)

## きっかけ

人が、dozy-pink の組も作るよう言った(会話 2026-10-10 の要約)。テーマ dozy-pink(SR-27)に合う部品の形の組を足す。人の依頼を AI が要件の文に起こした。

## 差分

- 変更: SR-36 旧「設定の `[style]` で、表と窓の部品の形を選べるべき: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、列の見出しの型の印(`icons`)。`[style]` の `preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`)を書けば部品をまとめて選び、ほかの項目はそれを上書きする。既定は `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)。`pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき。丸い端を描けるかは設定の `nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける。`"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき(どの端末でも、描けなければ字が化けない形に落とすため)。今までの `borders = "ascii"` は `frames = "ascii"` として受けるべき。知らない値は警告して既定にするべき。」→ 新「設定の `[style]` で、表と窓の部品の形を選べるべき: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、列の見出しの型の印(`icons`)。`[style]` の `preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`・`dozy-pink`。`dozy-pink` はテーマ dozy-pink に合う柔らかい組: 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ)を書けば部品をまとめて選び、ほかの項目はそれを上書きする。既定は `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)。`pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき。丸い端を描けるかは設定の `nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける。`"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき(どの端末でも、描けなければ字が化けない形に落とすため)。今までの `borders = "ascii"` は `frames = "ascii"` として受けるべき。知らない値は警告して既定にするべき。」

## 却下した案

| 案 | 理由 |
|---|---|
| 組がテーマも選ぶ | 組は形、テーマは色で、今の組もテーマを選ばない。カタログでは組とテーマを並べて選べる |

## 承認の記録

2026-10-10 人の承認(要約: dozy-pink の組も作るよう言った)
