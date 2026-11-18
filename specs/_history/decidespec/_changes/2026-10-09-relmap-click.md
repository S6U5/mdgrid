---
type: Change
id: 01M4E8QFNBQ9G64V7FTAV4WTX9
title: 関係マップをクリックで操作する(REL-12)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# 関係マップをクリックで操作する(REL-12)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-relmap-click.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、関係マップの画面がクリックで操作できないと指摘した(会話 2026-10-09)。表・つながり・つながった行をクリックで選べ、開ければ終わり。REL-12。

## 不明点と仮定

- 仮定: クリックの振る舞いは表の画面(SR-6)に合わせる(1回目は選ぶ、選んだ表をもう一度で開く)。外れたら: 1回で開く形に変える。
- 仮定: つながった行のクリックは、その行の元のノート(矢印の左)を開く(つながった行の一覧(REL-5)で選んだときと同じ)。

## 設計

- src/ui/relmap.rs: 段組みの寸法を1つの関数(geometry)にまとめ、描画とクリックの判定の両方で使う(ずれないように)。盤のずらし(ox, oy)も関数にまとめる。`relmap_click(x, y)` で、一覧の行・盤のセル(Cell.table / Cell.link)・狭い画面の行・つながった行を判定する。
- src/ui/app.rs: Mode::Relations のクリックで、タブのあとに relmap_click を呼ぶ。ホイールは Up/Down。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 関係マップのクリックとホイール | REL-12 | クリックの試験が通る | src/ui/relmap.rs, src/ui/app.rs, src/ui/mod.rs, src/ui/test_relmap_click.rs, docs/, specs/test-locks.json | test_rel_12_click_selects_and_opens(src/ui/test_relmap_click.rs) | 済 |

## 実装の気づき

- 段組みの寸法を geo() にまとめ、描画とクリックで同じものを使う(描く位置と押せる位置がずれないように)。盤のずらしも map_offset() にまとめた。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| REL-12 | checked | コード: src/ui/relmap.rs(geo・map_offset・relmap_click)、src/ui/app.rs(関係マップのクリックとホイール) / テスト: src/ui/test_relmap_click.rs::test_rel_12_click_selects_and_opens・test_rel_12_click_link_and_linked_row・test_rel_12_narrow_list_and_wheel |

既存のテストの削除・skip・弱体化: なし

照合は書込なしの検証役(2026-10-09)。指摘: 窓の左の端が幅2の文字の右半分のとき描く桁が1つずれる → 空白で埋めて直した。狭い画面(1つの段組み)で窓より下の行は描かず押せない → 残した(一覧のスクロールは別の改善)。

確かめた: 1 / 1
