---
type: Change
id: 01M4B5XZV2BZYPNQBHSGNYRSP0
title: 雛形の time・now の変数 に時差を2度足していたのを直す(template-time-offset)
status: done
size: bugfix
created: 2026-10-07
updated: 2026-10-07
---

# 雛形の time・now の変数 に時差を2度足していたのを直す(template-time-offset)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | なし |
| 仕様化 | 対象外: bugfix(CE-32 の「今の時刻」のとおりに直す) | なし |
| 設計 | 対象外: bugfix | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

リファクタリング(時計を clock.rs に分ける)の途中で見つけた。App の now は today_now が返す「地域の時計の秒」(UNIX 秒 + 時差)なのに、新しいノートの雛形の変数(note_vars)はそこへさらに local_offset() を足していた。日本(+9 時間)では time・now の変数 が9時間ずれる。now の1日の中の秒から分を出す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 分を now から出す | CE-32 | time の変数 が now の地域の時刻になる | src/ui/new_note.rs, src/ui/test_note_time.rs, src/ui/mod.rs | test_ce_32_time_variable_uses_local_now(src/ui/test_note_time.rs) | 済 |

## 実装の気づき

- 試験の環境は時差 0(local_offset を決めない)なので、直す前のコードでもこの試験は通る。時差は全体の静的な値で、決めると並べて走るほかの試験(式の日時)に響くので、試験では決めない。直したことは、コードを読んで(now に時差を足さない)確かめた。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-32 | checked | コード: src/ui/new_note.rs の note_vars / テスト: src/ui/test_note_time.rs::test_ce_32_time_variable_uses_local_now / 今: 通った(全体 1355 passed) / 前: 通った(試験の環境は時差 0 で、時差を2度足す誤りが試験では出ない) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
