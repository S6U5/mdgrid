---
type: Proposal
id: 01M3XDKG70NBBPDR6Q5JSDCB97
title: mdgrid のビューの定義の細部(native-views-details)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: specs/_decisions/2026-10-02-native-views.md
touches: [BV-20]
created: 2026-10-02
updated: 2026-10-02
---

# mdgrid のビューの定義の細部(native-views-details)

## きっかけ

人の依頼の提案 native-views(BV-17〜BV-19)を実装に移すには、定義のファイルの名前と形、壊れたときの扱い、`.base` のビューとの並び、見た目の状態(SR-12)との関係が決まっていない。AI が細部を足す。

## 差分

base-view の spec.md の「要件: 版1 — mdgrid のビュー」に足す。

- 追加: BV-20「mdgrid のビューの定義は、設定の置き場の `views.toml` に、開いた対象の実体のパスごとに、ビューの名前・列の並びと隠す列・ビューの設定(NV-13〜NV-22 のフィルター・並べ替え・グループ)を持つべきで、知らない項目は警告にとどめ(CLI-3)、壊れた `views.toml` は警告して読まずに起動するべき。`.base` で開いたときのタブは `.base` のビューのあとに mdgrid のビューを並べ、起動時は BV-13 のとおり先頭のビューを開くべき。保存は一時ファイル → 名前の変更で書き、`--readonly`(WB-15)では書かないべき。見た目の状態(SR-12)の列の幅と畳んだまとまりは、mdgrid のビューにも名前ごとに当てるべき。」(確かめ方: `views.toml` を手で壊す → 起動して警告、mdgrid のビューのタブは出ない。知らない項目 `color = 1` → 警告して読む。`.base` と mdgrid のビューの両方がある対象 → タブは `.base` のビューが先。`--readonly` で保存 → 何も書かれず「読むだけ」(`test_BV_20`))

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | ファイルの形・壊れたとき・並び・読むだけのときが実装ごとにぶれる |
| `config.toml` に混ぜる | 画面から書き換えるファイルと、人が書く設定を分ける(sessions の決定と同じ考え) |

## 承認の記録

2026-10-02 AI 自己採択(理由: 守られる要件に触れず、人の依頼の要件を実装に移すための細部で、強さは「するべき」以下。人の依頼の提案の採択と同じコミットに入れる)
