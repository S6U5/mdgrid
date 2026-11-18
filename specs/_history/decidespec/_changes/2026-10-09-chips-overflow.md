---
type: Change
id: 01M4ECYGGHV9C69JQPEC0CTEK6
title: 入りきらない札を途中で切らず、残りの数を +N で示す(SR-35)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# 入りきらない札を途中で切らず、残りの数を +N で示す(SR-35)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(SR-35 の「どの値も文字はそのまま見せる」に寄せる) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 対象外: bugfix(1か所) | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

README の画面を今の見た目で撮ると、右端の tags の列で札が途中で切れ(code-…・writi…)、読めない札が並んだ。入る札だけを丸ごと見せ、残りの数を +N で示す。

## 設計

- src/ui/view.rs の chip_spans: 札ごとに、この札と(残りがあれば)+N の幅が入るかを見て、入らなければ +N を出して止める。1つ目の札は、単独で入れば +N の幅が取れなくても丸ごと見せ、単独でも入らなければ列の幅いっぱいに … で切る(+N は出さない)。README の画面を撮り直して、`wr… +1` のような短すぎる札が出ないことを目で確かめた。幅は今までどおり列の幅ちょうど。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-35 | checked | コード: src/ui/view.rs(chip_spans) / テスト: src/ui/test_rich_cells_fit.rs::test_sr_35_chips_overflow_counts_the_rest・test_sr_35_chips_all_fit・test_sr_35_first_chip_too_wide_is_cut、src/ui/test_rich_cells_first.rs::test_sr_35_first_chip_whole_before_count |

既存のテストの削除・skip・弱体化: なし

1か所の bugfix のため、メインが試験で確かめた。

確かめた: 1 / 1
