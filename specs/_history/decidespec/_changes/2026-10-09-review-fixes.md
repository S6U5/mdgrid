---
type: Change
id: 01M4EJR2BFM9Q5YDRV2HZDJHCM
title: ブランチ全体の確認の指摘を直す(文字のまま書き換える所の安全・印の外のパス・自動の表・関係マップ・札の判定)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# ブランチ全体の確認の指摘を直す

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(WS-1・WS-7・CLI-18・REL-7・REL-9・SR-35 に合わせる) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 対象外: bugfix | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

書込なしの確認役に main からの差分全体を見てもらい、13件の指摘を受けた(2026-10-09)。直したもの:

1. 見出しの行にコメント(`[[workspace]] # x`・`[[place]] # x`)や括弧の中の空白があると見出しと見なさず、隣の項目を消す・取り違える(重い)。
2. 末尾の別の表(`[meta]` など)を区画に巻き込んで消す。1行の形(`workspace = [{...}]`)のワークスペースに足すと壊す。書き換えられなくても「消した」と出す。
3. コマンドのワークスペースの名前の前後の空白・空の名前で、以後の書き換えが全部断られる。画面で外せなかったときも「外した」と出す。
4. 自動の表がシンボリックリンクをたどり、node_modules・target なども表にする(起動が重い)。
5. 同じ名前の表(フォルダの Tasks と Tasks.base)を関係マップで1つに混ぜる。
6. 狭い端末(幅2未満)の関係マップで引き算があふれて落ちる。
7. 関係マップに文字の向きを変える書式の文字(U+202E など)をそのまま出す。
8. フォルダの印(配られうる)の表が根の外(`../`・外の絶対パス)を指しても使う。
9. 札の列の判定の使い回しが、ビューの切り替え・外での書き換えの読み直しで古いまま。

残したもの: 関係マップを開くとき全部のノートを一度に読む。測ると2万ノートの1つの表で約0.9秒(release)、数千なら0.2秒ほどなので、今は直さない(少しずつ読むのは別の改善)。

## 設計

- src/config.rs: toml_header(コメント・括弧の中の空白を除いた見出し)と toml_blocks(区画は別の見出しで終わる)。
- src/workspace.rs・src/places.rs: 区画は toml_blocks で探し、書き換えた結果を読み直して狙った並びと違えば並びから書き直す(コメントは消えるが、ほかの項目を壊さない)。印の表は根の中だけ(外は警告)。自動の表はリンクをたどらず、依存・ビルドの置き場を外し、見る項目に上限。
- src/ws_cli.rs: 名前の前後の空白を落とし、空は断る。src/ui/workspace.rs: 外せなかったら WsNotIn。
- src/ui/relmap.rs: 同じ名前の表は番号で分ける。幅の引き算は saturating。src/relmap.rs の clean で向きの書式の文字も ?。
- src/ui/grid.rs・app.rs・external.rs・new_note.rs: 札の判定の目印にビューと読み直しの回数(data_gen)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| WS-1 | checked | テスト: src/test_toml_edit_safety_unit.rs::test_ws_1_header_with_comment_keeps_neighbour・test_ws_1_trailing_table_and_inline_form_survive |
| CLI-18 | checked | テスト: src/test_toml_edit_safety_unit.rs::test_cli_18_place_header_with_comment_keeps_neighbour |
| WS-7 | checked | テスト: src/test_toml_edit_safety_unit.rs::test_ws_7_marker_paths_stay_inside |

既存のテストの削除・skip・弱体化: なし

照合の元は書込なしの確認役の指摘(2026-10-09)。直したあとはメインが試験で確かめた(全部の試験と clippy -D warnings が通る)。

確かめた: 3 / 3
