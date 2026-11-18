---
type: Change
id: 01M48Y1XVXRTS76DFMA90WEBDV
title: 1つの値で書かれたリストを、付け外しでリストに書き換える(scalar-list-edit)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 1つの値で書かれたリストを、付け外しでリストに書き換える(scalar-list-edit)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-scalar-list-edit.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CE-19 のとおり(人の判断待ちの表の4)。

## 不明点と仮定

- 仮定: 文字列でない1つの値(`tags: 2024`)とブロックの文字(`tags: |`)は読むだけのまま(書くと型か書き方が変わる)。理由は「文字列でない要素を含むリスト」と「1つの値で書かれたリスト」。

## 設計

- 読む側(src/source/markdown.rs の scalar_list_lock): Plain・引用符の空でない文字列は lock なし。文字列でない値は ListNonString、それ以外(ブロックの文字)は LockScalarList。
- 画面(src/ui/listpick.rs): wants_pick と list_of が、リストの列の空でない文字列を1つの要素のリストとして扱う。
- 書く側(src/writeback.rs の write_list): Plain・引用符の文字列の値の範囲を `[..]` に置き換える。元の要素は元の書き方(フローで安全なら)を使う。全部外したら CE-9 と同じく `key:`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 読む・開く・書くを変え、錠のある試験を掛け直す | CE-19, CE-16, CE-17 | 採択した確かめ方 | src/source/markdown.rs, src/source/test_markdown_unit.rs, src/ui/listpick.rs, src/ui/test_listpick.rs, src/writeback.rs, src/test_scalar_list_edit_unit.rs, src/lib.rs, tests/test_list_write.rs, tests/test_scalar_list_edit.rs, docs/safety.md, docs/safety.ja.md, docs/design.md, docs/todo.md, specs/test-locks.json | test_ce_19_scalar_list_is_rewritten_as_flow_list(tests/test_scalar_list_edit.rs) | 済 |

## 実装の気づき

- ブロックの文字(`tags: |`)は value_lock が先に「複数行の値」で止めるので、「1つの値で書かれたリスト」の理由は使われなくなり消した。文字列でない1つの値の理由は「文字列でない要素を含むリスト」。
- 錠のある試験(CE-16・CE-17・CE-19)は、決定の「錠:」の行で掛け直した。CE-17 の試験は、今の行が開けない例を `tags: 2024` に替えた。
- 確かめ方の `tags: [x, y]` の y は、WB-7 により実際には `"y"` と囲まれる(YAML 1.1 の真偽の語)。試験は w を使った。

## 照合

自分で照合(変更は3か所の小さな分岐)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-19 | checked | コード: src/writeback.rs の write_list、src/source/markdown.rs の scalar_list_lock、src/ui/listpick.rs / テスト: tests/test_scalar_list_edit.rs、src/ui/test_scalar_list_pick.rs、src/ui/test_listpick.rs::test_ce_19_read_only_shapes_have_reasons / 今: 通った(全体 1320 passed) / 前: 落ちた(書き換えの試験は実装の前に NotEditable で落ちた) |
| CE-16 | checked | コード: src/source/markdown.rs の scalar_list_lock / テスト: src/source/test_markdown_unit.rs::scalar_in_list_column_is_locked_and_counted / 今: 通った / 前: 落ちた(lock があった) |
| CE-17 | checked | コード: src/ui/listpick.rs の wants_pick / テスト: src/ui/test_listpick.rs::test_ce_17_bulk_when_current_row_cannot_open / 今: 通った / 前: 通った(例を `tags: 2024` に替えた) |

既存のテストの削除・skip・弱体化: なし(錠のある試験は人の決定で掛け直した)

確かめた: 3 / 3
