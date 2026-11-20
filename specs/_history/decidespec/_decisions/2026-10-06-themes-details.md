---
type: Proposal
id: 01M46DQTAAPKX9YAVR8J78CKJ8
title: テーマの設定の名前と既定、色なしと 256 色の扱い(themes-details)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: specs/_decisions/2026-10-06-themes.md
touches: [SR-27]
created: 2026-10-06
updated: 2026-10-06
---

# テーマの設定の名前と既定、色なしと 256 色の扱い(themes-details)

## きっかけ

SR-26(themes)を実装するのに、設定の名前・値・既定と、色を使わない表示・256 色の端末での扱いを決める必要がある。人は推しの順で最後まで進めるよう委ねたので、AI が細部を足す。

## 差分

screen の spec.md の「要件: 版1 — 設定と見た目の状態」に足す。

- 追加: SR-27「SR-26 のテーマは、設定の最上位の `theme` に `"default"`(既定。今の見た目で、端末の既定の色と反転・太字・薄い表示だけを使う)・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` のどれかを書いて選んでよく、知らない名前は警告にとどめて `default` で起動するべき(CLI-3)。色を使わない表示(SR-10・`--no-color`・設定の `color = false`)ではテーマを使わず、トゥルーカラーに対応しない端末(SR-15)ではテーマの色を近い 256 色に落とすべき。」(確かめ方: 設定なし → 今と同じ色。`theme = "neon"` → 警告に `theme` と出て既定の見た目で起動。`theme = "dracula"` と `NO_COLOR=1` → 色の指定が無い。`COLORTERM` なしで `theme = "nord"` → 256 色の番号で塗る(`test_SR_27`))

## 却下した案

| 案 | 理由 |
|---|---|
| 既定を Nord など色のあるテーマにする | 今の利用者の画面が、何もしないのに変わる |
| 知らない名前で起動を止める | CLI-3 は知らない項目を警告にとどめる。値の誤りも同じ扱いにそろえる |
| 起動の引数 `--theme` も足す | 一度決めれば変えないものなので、設定だけで足りる。引数は増やさない |

## 承認の記録

AI 自己採択(人が推しの順で最後まで進めるよう委ねた細部。守られる要件に触らない。予防でなく機能の細部なので「するべき」と「してよい」で書く)
