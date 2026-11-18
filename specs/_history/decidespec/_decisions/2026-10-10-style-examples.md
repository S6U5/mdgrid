---
type: Proposal
id: 01M4GRM1WQRQZ29JC7QR5372GB
title: 部品の形の既定(sumi)に合わせて、SR-33・SR-35・SR-36 の例を直す(style-examples)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: d5fcab98a2ae1626
decided-by: 人
trigger: 検証の指摘
trigger-link: specs/_changes/2026-10-09-style-catalog.md
touches: [SR-33, SR-35, SR-36]
created: 2026-10-10
updated: 2026-10-10
---

# 部品の形の既定(sumi)に合わせて、SR-33・SR-35・SR-36 の例を直す(style-examples)

## きっかけ

照合(specs/_changes/2026-10-09-style-catalog.md)で、SR-33・SR-35・SR-36 の例(確かめ方)が、既定の組 sumi と SR-33 の「画面の文字を変えない」と食い違っていた。AI が、選んだ行の印は `>` のまま色で見せ、帯と部品の例を sumi に合わせる案を示し、人がおすすめでよいと言った(会話 2026-10-10 の要約)。要件の文は変えず、例だけを直す。

## 差分

- 変更: SR-33 旧「色を使える表示では、設定の `look` が `"modern"`(既定)のとき、今どきの道具のような見た目にするべき: 窓の枠は薄い色で、今操作している窓の枠と題はアクセントの色。選んでいる行・候補・項目は、反転でなく背景の色と太字(形は SR-36 の `select`)。下の帯・ビューのタブ・列の見出しの形は SR-36 の `band`・`tabs`・`icons` に従う。色はテーマ(SR-26)のものを使い、テーマが既定(色の表が無い)なら端末の地の色を変えない控えめな組を使う。`look` が `"classic"` のときと色を使わない表示(SR-10)では今までの見た目(反転)にする。どちらでも、値の文字・印・構成は変えない(SR-26)。」→ 新「色を使える表示では、設定の `look` が `"modern"`(既定)のとき、今どきの道具のような見た目にするべき: 窓の枠は薄い色で、今操作している窓の枠と題はアクセントの色。選んでいる行・候補・項目は、反転でなく背景の色と太字(形は SR-36 の `select`)。下の帯・ビューのタブ・列の見出しの形は SR-36 の `band`・`tabs`・`icons` に従う。色はテーマ(SR-26)のものを使い、テーマが既定(色の表が無い)なら端末の地の色を変えない控えめな組を使う。`look` が `"classic"` のときと色を使わない表示(SR-10)では今までの見た目(反転)にする。どちらでも、値の文字・印・構成は変えない(SR-26)。」(確かめ方: 色のある表示で候補の窓を開く → 選んだ候補のセルに反転が無く背景の色が付き、枠の線は薄い色。下の帯のキーは組の帯の形(SR-36 の `band`)で、説明は薄い色(既定の sumi は `quiet` でキーは太字、`classic` の組ではアクセントの色の太字)。`look = "classic"` → 今の見た目(反転)。`--no-color` → 色の指定が無い。どの look でも画面の文字は同じ(`test_sr_33_*`))
- 変更: SR-35 旧「色を使う表示(SR-10)では、設定の `cells` が `"rich"`(既定)のとき、表のセルを値の型に合わせた部品で見せるべき: 真偽の値、リストの要素、種類の少ない短い文字の列(status など)の値、リンク(REL-2)、列の見出しの型の印を、SR-36 の形(`check`・`tags`・`status`・`icons`)で見せる。部品の色は値ごとに決まった色にし、どの値も文字はそのまま見せる(色だけで値を伝えない。SR-15)。部品は設定で選べるべき: `cells` は文字列(`"rich"`・`"plain"`)か表で、表なら `style`(全体)と、部品の種類ごとの真偽(`checkbox`・`chips`・`select`・`links`・`icons`)と、列ごとの見せ方 `[cells.columns]`(`"rich"`・`"plain"`・`"chip"`(自動で部品にならない列も部品に))を書ける。列ごとの設定は種類ごとの設定より優先する。`"plain"` と色を使わない表示では今の見た目にする。ファイルに書く値と、編集・検索・出力(`--print`)の値は変えない。」→ 新「色を使う表示(SR-10)では、設定の `cells` が `"rich"`(既定)のとき、表のセルを値の型に合わせた部品で見せるべき: 真偽の値、リストの要素、種類の少ない短い文字の列(status など)の値、リンク(REL-2)、列の見出しの型の印を、SR-36 の形(`check`・`tags`・`status`・`icons`)で見せる。部品の色は値ごとに決まった色にし、どの値も文字はそのまま見せる(色だけで値を伝えない。SR-15)。部品は設定で選べるべき: `cells` は文字列(`"rich"`・`"plain"`)か表で、表なら `style`(全体)と、部品の種類ごとの真偽(`checkbox`・`chips`・`select`・`links`・`icons`)と、列ごとの見せ方 `[cells.columns]`(`"rich"`・`"plain"`・`"chip"`(自動で部品にならない列も部品に))を書ける。列ごとの設定は種類ごとの設定より優先する。`"plain"` と色を使わない表示では今の見た目にする。ファイルに書く値と、編集・検索・出力(`--print`)の値は変えない。」(確かめ方: 色のある表示で、既定の組(sumi)では done(真偽)の列 → `✓`/`·`、tags `[a, b]` → `a · b`、status(doing・todo・done が何行も)→ `● doing`、見出しに型の印なし。組 classic では `☑`/`☐`・札の `a` と `b`・札の `doing`・`◷ due`。`cells = "plain"` か `--no-color` → 今の見た目(`true`・`[a, b]`)。`[cells]` で `checkbox = false` → 真偽だけ文字のまま。`[cells.columns]` で `status = "plain"` → その列だけ文字のまま、`owner = "chip"` → owner の値が札。知らない値 → 警告して rich(`test_sr_35_*`))
- 変更: SR-36 旧「設定の `[style]` で、表と窓の部品の形を選べるべき: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、列の見出しの型の印(`icons`)。`[style]` の `preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`・`dozy-pink`。`dozy-pink` はテーマ dozy-pink に合う柔らかい組: 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ)を書けば部品をまとめて選び、ほかの項目はそれを上書きする。既定は `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)。`pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき。丸い端を描けるかは設定の `nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける。`"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき(どの端末でも、描けなければ字が化けない形に落とすため)。今までの `borders = "ascii"` は `frames = "ascii"` として受けるべき。知らない値は警告して既定にするべき。」→ 新「設定の `[style]` で、表と窓の部品の形を選べるべき: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、列の見出しの型の印(`icons`)。`[style]` の `preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`・`dozy-pink`。`dozy-pink` はテーマ dozy-pink に合う柔らかい組: 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ)を書けば部品をまとめて選び、ほかの項目はそれを上書きする。既定は `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)。`pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき。丸い端を描けるかは設定の `nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける。`"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき(どの端末でも、描けなければ字が化けない形に落とすため)。今までの `borders = "ascii"` は `frames = "ascii"` として受けるべき。知らない値は警告して既定にするべき。」(確かめ方: 既定(設定なし)で色のある表示 → status の値は `● doing`(点だけに色)、tags は `ui · web`、選んだ行の左の印 `>` はアクセントの色の太字(印の文字は変えない。SR-33)。`[style] preset = "saas"` → 淡い地の丸い札と切り替えの枠のタブ。`status = "chip"` → 今までの札。`pill` で `nerd_font` なし → `soft`。`borders = "ascii"` → 枠が ASCII。知らない値 → 警告(`test_sr_36_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| 選んだ行の印を `▌` にする | SR-33 の「どの見た目でも画面の文字は同じ」とぶつかり、`>` を確かめる多くの試験を変えることになる |

## 承認の記録

2026-10-10 人の承認(要約: 選んだ行の印と例の直し方は、AI のおすすめでよいと言った)
