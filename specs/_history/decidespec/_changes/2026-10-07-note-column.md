---
type: Change
id: 01M4909VE4JPXGQA4A7GNH2MR1
title: 左のノートの欄から共通のフォルダと .md を除き、名前の列があれば名前を出さない(note-column)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 左のノートの欄から共通のフォルダと .md を除き、名前の列があれば名前を出さない(note-column)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-note-column.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した SR-29 のとおり(人の判断待ちの表の8)。

## 不明点と仮定

- 仮定: 共通のフォルダは、読み込んだ全部の行(絞り込みの前)のパスのフォルダの部分の、共通の先頭の部分(フォルダの区切りの単位)。スクロールや絞り込みで欄の名前が変わらないように。

## 設計

- src/ui/view.rs の label が、App に持たせた共通の前置き(行を読み直すたびに計算)を除き、`.md` を除く。cols に `file.name` があれば名前を空にして印だけにし、見出しも出さない。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 欄の名前を短くし、名前の列があれば出さない | SR-29 | 採択した確かめ方 | src/ui/, tests/, specs/test-locks.json, specs/_decisions/, docs/ | test_sr_29_note_column(src/ui/test_note_column.rs) | 済 |

## 実装の気づき

- 名前の列があるとき、欄の幅は印の幅(印が無ければ 0)になる。選択の印 \`>\` と区切りの1桁は残る。
- 画面の名前を探していた試験(錠のある 11 の要件の試験)は、補いの決定 2026-10-07-note-column-locks の「錠:」の行で掛け直した。ゴールデン 18 本は UPDATE_GOLDEN=1 で作り直した。
- README・説明書の場面の画像は、次に場面を作り直すときに新しい欄で撮り直す(今の画像は古い欄のまま)。

## 照合

自分で照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-29 | checked | コード: src/ui/view.rs の label と name_column_shown、src/ui/grid.rs の common_folder / テスト: src/ui/test_note_column.rs::test_sr_29_common_folder、src/ui/test_note_column.rs::test_sr_29_note_column_drops_common_folder_and_md、src/ui/test_note_column.rs::test_sr_29_name_column_hides_names_but_keeps_marks / 今: 通った(全体 1325 passed) / 前: 落ちた(実装の前は欄が Tasks/a.md のまま) |

既存のテストの削除・skip・弱体化: なし(名前の探し方を新しい欄に合わせた。錠のある試験は人の決定で掛け直した)

確かめた: 1 / 1
