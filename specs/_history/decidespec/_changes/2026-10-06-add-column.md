---
type: Change
id: 01M47BTJRC4X50DJNE1WXC53GV
title: どのノートにも無いキーの列を表に足す(add-column)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# どのノートにも無いキーの列を表に足す(add-column)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-add-column.md |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の B-7 のうち列の追加。採択した CE-28 のとおり。名前の変更・削除は WB-1(人の決定)に触るので人の判断を待つ。

## 不明点と仮定

- 仮定: 名前の頭の `note.` は外す(.base の書き方と同じ)。前後の空白は除く。
- 仮定: 足した列は今のビュー(`.base`・mdgrid のビュー・既定の表のどれでも)の右に出す。ビューを切り替えても起動の間は出す。

## 設計

- 入力は .base の書き出しの名前と同じ「パレットの続きの入力」(`Ask`)に `NewColumn` を足す。
- `App` に起動の間だけの `extra_cols` を持ち、`build_grid` の結果に、まだ無い列として右に足す(既定の表・.base・mdgrid のビュー共通)。refresh は表の列に無い列を落とすので、組み立ての側で足す。
- 動作 `add_column`(既定 `A`)をキーの表・パレット・操作の一覧の列の節に。docs/keys(.ja).md にも(試験が突き合わせる)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 列を足す | CE-28 | 採択した確かめ方 | src/ui/keymap.rs, src/ui/columns.rs, src/ui/native_io.rs, src/ui/grid.rs, src/ui/app.rs, src/ui/menu.rs, src/ui/mod.rs, src/i18n.rs, src/ui/test_add_column.rs, docs/keys.md, docs/keys.ja.md, README.md, README.ja.md, tests/golden/sr_5.txt | test_ce_28_*(src/ui/test_add_column.rs) | 済 |

## 実装の気づき

- ヘルプのゴールデン tests/golden/sr_5.txt は、キーの表に1行増えて行数と並びが変わったので UPDATE_GOLDEN で作り直した(錠は試験のファイルに掛かり、ゴールデンには掛かっていない。前にキーを足したときと同じ)。
- 書き込みは今の道(キーの無いノートにキーの行を足す WB-3)のまま。足した列のセルの型は値が無いのでテキスト(候補のリストは出ない)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。書き込みの道は変えず、列の組み立てに足すだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-28 | checked | コード: src/ui/native_io.rs の start_add_column・add_extra_column、src/ui/grid.rs の build_grid、src/ui/keymap.rs・src/ui/menu.rs / テスト: src/ui/test_add_column.rs の4本(足して1行に入れて保存 → そのノートにだけ \`reviewer: ken\`、ほかは同じバイト。書けない名前・もうある名前・読むだけ)、src/ui/test_keys_doc.rs(文書とキーの表の突き合わせ) / 今: 通った(全体 1266 passed) / 前: 落ちた(実装の前は 4本とも落ちた) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
