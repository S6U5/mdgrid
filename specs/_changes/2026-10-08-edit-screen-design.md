---
type: Change
id: 01M4CKJ4BN3GK8N16SP1ZFQQB0
title: セルの入力欄と新しいノートの窓の見た目を良くする(edit-screen-design)
status: done
size: full
created: 2026-10-08
updated: 2026-10-08
---

# セルの入力欄と新しいノートの窓の見た目を良くする(edit-screen-design)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-08-edit-screen-design.md(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した SR-31 のとおり(人が段階1を選んだ)。

## 不明点と仮定

- 仮定: 窓の幅は、画面の幅から左右2桁ずつを引いた幅と、見出し+40桁の狭い方。左右の真ん中に置く。
- 仮定: 型の手がかりの文字は、数は「数」(英語 number)、リストは「[リスト]」([list])、チェックは「[ ]」(あいまいな幅の字を使わない)。

## 設計

- src/ui/view.rs: 入力欄の見た目を反転と太字に。src/ui/input.rs: 入力欄の最小の幅を 16 に。
- src/ui/new_note.rs: 窓の形(geom)に左の桁と幅を持たせ、枠(上の縁に題と場所、欄、区切り、キー、下の縁)を描く。入力ボックスの位置も窓の中に。型の手がかりは列の型(src.kind)から。
- 枠の文字は ambiguous_wide なら ASCII。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 入力欄と新しいノートの窓 | SR-31 | 採択した確かめ方 | src/ui/view.rs, src/ui/input.rs, src/ui/new_note.rs, docs/assets/, src/i18n.rs, src/ui/mod.rs, src/ui/test_edit_design.rs, src/ui/test_*.rs, tests/golden/, docs/manual/, specs/test-locks.json, specs/_decisions/ | test_sr_31_note_form_box_and_hints(src/ui/test_edit_design.rs) | 済 |

## 実装の気づき

- 入力欄を反転にすると、錠のある SR-26 の試験(テーマで入力欄の太字と下線が強い色になる)とぶつかるので、地の色(テーマなら選んだセルの地の色、テーマなしなら灰色)で示し、太字と下線は残した。SR-31 の文は edit-screen-design-bg(AI)で直した。
- 窓の左の縁が付いたので、行の頭を確かめる錠のある2つの試験を「含む」に直し、edit-screen-design-locks(人の承認)で掛け直した。
- 窓の左右は1桁ずつ空け、表の文字が縁に付いて見えないようにした。
- 画像は人が選んだ範囲 A(新しいノートの4場面と、README の編集の3つ)だけを撮り直した(変わったのは6場面 × 2つの言葉。新しいノートの作ったあとと保存の画像は同じだった)。
- 全部の試験を回した1回目で1件が落ちたが(機械が混んでいて10分を超えた)、続けて2回回すと全部通った。疑似端末の試験の時間切れと見る。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-31 | checked | コード: src/ui/view.rs の input_style、src/ui/new_note.rs の overlay・box_chars・type_hint、src/ui/input.rs の INPUT_MIN / テスト: src/ui/test_edit_design.rs::test_sr_31_input_box_is_wide_and_filled、src/ui/test_edit_design.rs::test_sr_31_note_form_box_and_hints / 今: 通った(全体 1357 passed) / 前: 落ちた(枠と手がかりが無かった) |

既存のテストの削除・skip・弱体化: なし(錠のある2つの試験の行の確かめを「始まる」から「含む」に。人の承認で掛け直した)

確かめた: 1 / 1
