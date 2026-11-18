---
type: Change
id: 01M4G7C1PFBKG58TXAP0V1Z69M
title: 関係マップでも「+ 新規」、選んだセルでも部品の形を保つ、カタログの見出しで並べ替え(CE-25・SR-36・NV-3)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# 関係マップでも「+ 新規」、選んだセルでも部品の形を保つ、カタログの見出しで並べ替え(CE-25・SR-36・NV-3)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-relmap-new-note.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

- 人の依頼(会話 2026-10-09 の要約): 「+ 新規」は画面によって出たり消えたりせず、関係マップでも選んでいる表に新しいノートを作れるように(CE-25 の変更)。
- 人の指摘(同): 丸い札の形にしても、選んだセルで四角い塗りに変わるのはおかしい。選んだセルでも部品の形を保つ(SR-36 の文の中の振る舞い)。
- 人の問い(同): 見出しを押すと並べ替え、3回で元に戻るか。本体は NV-3 で既にそう動く。カタログの見本に足す。

## 不明点と仮定

- 仮定: 関係マップで選んでいる表が今の表なら表の画面に戻ってすぐ名前の欄、ほかの表ならその表を開き直してから名前の欄。
- 仮定: 選んだセルの部品は形をそのまま描き、選びの印(太字と下線、bar・outline はアクセントの色)を重ねる。fill(classic)と reverse は今までどおり塗る・反転する。

## 設計

- src/ui/relmap.rs: 関係マップで NewNote の操作(キー・ボタン)を受け、選んでいる表を開く処理に「開いたら名前の欄」を足す。App に switch_new_note。src/main.rs で開き直したあとに start_new_note。
- src/ui/bands.rs: 関係マップでもボタンを描く。src/ui/app.rs: 関係マップのクリックでもボタンを見る。
- src/ui/view.rs: 選んだセルも chip_spans で描き、選びの見た目を重ねる。
- docs/catalog/index.html: 同じ振る舞いと、見出しのクリックの並べ替え。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 関係マップの新規 | CE-25 | 試験が通る | src/ui/, src/main.rs, docs/ | test_ce_25_relmap_new_note(src/ui/test_relmap_new_note.rs) | 済 |
| 2 | 選んだセルの部品 | SR-36 | 試験が通る | src/ui/view.rs | test_sr_36_selected_cell_keeps_parts(src/ui/test_style_selected.rs) | 済 |
| 3 | カタログ | SR-38, NV-3 | 見本で確かめる | docs/catalog/ | test_sr_38_catalog_matches_config(tests/test_catalog.rs) | 済 |

## 実装の気づき

- 関係マップの `a`(new_note)をキーの表に足し、docs/keys.md・docs/keys.ja.md とヘルプのゴールデン(sr_5。行の数が1つ増えた)を合わせた。
- 新しいノートの名前の欄は編集のモード(Mode::Edit)で開く。

## 照合

書込なしの検証役が条件ごとに確かめた(cargo test 1512 通過・0 失敗)。指摘は直した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-25 | checked | 同じ位置にいつも出す・押すと今の表で名前の欄(テスト: src/ui/test_relmap_new_note.rs::test_ce_25_relmap_new_note)。ほかの表へ移る(同::test_ce_25_relmap_new_note_other_table)。移った表では読み込みとビューが決まったあとに一度だけ名前の欄(直した: src/ui/app.rs の select_pending。テスト: src/ui/test_relmap_new_note_load.rs::test_ce_25_relmap_new_note_after_load)。確認から戻ったら頼みを消す(同::test_ce_25_relmap_new_note_cancel_clears)。読むだけではボタンも a も出ない(読んで判定: new_note::button_shown と set_readonly)。残した小さい差: 関係マップの `a` は表の new_note の割り当て直しに付いてこない(関係マップのモードとして別に割り当て直せる) |
| SR-36 | checked | 選んだセルも部品の形(テスト: src/ui/test_style_selected.rs::test_sr_36_selected_cell_keeps_parts)。fill・reverse は今まで(読んで判定: src/ui/view.rs の keep_parts) |
| SR-38 | checked | テスト: tests/test_catalog.rs の3つ |
| NV-3 | checked | カタログの見出しのクリックの巡りと文が本体(src/ui/columns.rs の cycle_sort と i18n の SortedAsc 等)と同じ(読んで判定) |

既存のテストの削除・skip・弱体化: なし(ヘルプのゴールデン sr_5 の行の数が1つ増えた)

確かめた: 4 / 4

