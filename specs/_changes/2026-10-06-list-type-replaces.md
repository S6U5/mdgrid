---
type: Change
id: 01M46P1ZJFGEV97PM1ASP8GZ64
title: リストで文字を打つと今の値を置き換える(list-type-replaces)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# リストで文字を打つと今の値を置き換える(list-type-replaces)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [list-type-replaces](../_decisions/2026-10-06-list-type-replaces.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-1)が見つけた。status のリスト(今の値 `doing`)で `blocked` と打つと `doingblocked` になる。CE-3 を「リストを開いた直後(選び直したあとも)に打つと、今の値を消して打った文字から自由入力を始める」に変えた。

終わりの条件: CE-3 の例のとおり。錠のある test_ce_3_none_and_free_input(Ctrl+R のあとは続き)も通る。関係する要件: CE-3。

## 不明点と仮定

- 仮定: リストの無い入力ボックス(候補が多い列・数・日付)は今のまま(続きを直す)。
- 仮定: ←→・Home・End・BS・Del・Ctrl+R のあとは置き換えない(続きを直すつもりの操作)。リストの ↑↓ は置き換えのまま。

## 設計

- src/ui/input.rs の Input に `fresh: bool`(リストを開いた直後で、まだ文字も ↑↓ 以外の操作も無い)。開くときにリストがあれば真。`insert` で fresh かつリストを出していれば文字を消してから入れ、fresh を偽にする。`input_action` は ListUp・ListDown のほかは fresh を偽にする。新しいノートの欄(src/ui/new_note.rs)の Input は偽。
- 却下: `touched` を使う(Ctrl+R が touched を偽に戻すので、戻したあとに打つと置き換わり、錠のある試験と食い違う)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | リストで打つと置き換える | CE-3 | CE-3 の例のとおりで、既存の試験が通る | src/ui/input.rs, src/ui/new_note.rs, src/ui/test_list_type.rs, src/ui/mod.rs, src/ui/test_gaps.rs, docs/manual-scenarios.toml, docs/design.md, docs/todo.md | test_ce_3_type_replaces_*(src/ui/test_list_type.rs) | 済 |

## 実装の気づき

- 錠の無い src/ui/test_gaps.rs::test_sr_17_terminal_cursor_after_draw_is_at_input が落ちた。ノート1つの title はリスト(候補1つ)になり、打った `d` が `abc` を置き換えてカーソルが左へ動いたため。この試験が見るのは「打つとカーソルが動く」(SR-17)なので、打つ前に End を1つ足して続きを直す形にした(確かめの強さは変えていない)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-3 | checked | コード: src/ui/input.rs の Input の fresh、insert、input_action / テスト: src/ui/test_list_type.rs::test_ce_3_type_replaces_current_value・_after_moving_in_list・test_ce_3_type_continues_after_cursor_key・_after_revert、錠のある src/ui/test_input.rs::test_ce_3_none_and_free_input / 今: 通った(全体 1088 passed) / 前: 落ちた(実装の前に試験を置いて回し、replaces_current_value と after_moving_in_list の2本が `doingblocked`・`doingx` で落ちた)。実物: 説明書の場面 edit-free-text を Enter → `waiting` に変えて撮り、status が `waiting` に置き換わった |

既存のテストの削除・skip・弱体化: なし(test_gaps の1本に End を1つ足した。上の気づき)

確かめた: 1 / 1
