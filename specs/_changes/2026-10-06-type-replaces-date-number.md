---
type: Change
id: 01M474MYGEMNHPRTHB42PKQ2K7
title: 日付と数の入力でも、開いた直後に打つと今の値を置き換える(type-replaces-date-number)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 日付と数の入力でも、開いた直後に打つと今の値を置き換える(type-replaces-date-number)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [type-replaces-date-number](../_decisions/2026-10-06-type-replaces-date-number.md)(AI) |
| 設計 | 対象外: 条件に当たらない(list-type-replaces の fresh を日付・日時・数の入力にも広げる) | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、3回目の点検(docs/todo.md の A-3・A-4)が、日付と数の入力で打った文字が今の値の後ろに足されると指摘した。CE-5・CE-7 に「開いた直後に打つと置き換える」を足した。

終わりの条件: CE-5・CE-7 の例のとおり。カレンダーの矢印(CE-21)と ← → BS のあとは続きを直す。関係する要件: CE-5・CE-7・CE-21・CE-3。

## 不明点と仮定

- 仮定: 日付・日時・数の入力(Entry の Date・DateTime・Number)は、開いたときに fresh を立てる。カレンダーの ↑↓ ←→(日を動かす)は値を変える操作なので fresh を下ろす(そのあと打つと続き)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 日付と数で置き換える | CE-5, CE-7 | 上の終わりの条件のとおり | src/ui/input.rs, src/ui/calendar.rs, src/ui/test_type_replaces_more.rs, src/ui/mod.rs, docs/manual-scenarios.toml | test_ce_5_type_replaces_*・test_ce_7_type_replaces_*(src/ui/test_type_replaces_more.rs) | 済 |

## 実装の気づき

- カレンダーの → ← PageUp・PageDown は入力の動作(input_action)を通らずに日を動かすので、日を動かす所(src/ui/calendar.rs で touched を立てる2か所)でも fresh を下ろした。錠のある test_ce_21_datetime_keeps_time(→ のあとに打った時刻を保つ)がこれで通る。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-5 | checked | コード: src/ui/input.rs の fresh と insert、src/ui/calendar.rs の日を動かす所 / テスト: src/ui/test_type_replaces_more.rs::test_ce_5_type_replaces_plus_days・_continues_after_backspace、錠のある src/ui/test_calendar.rs::test_ce_21_datetime_keeps_time / 今: 通った(全体 1225 passed) / 前: 落ちた(実装の前に plus_days が `2026-10-05+3` で落ちた) |
| CE-7 | checked | コード: src/ui/input.rs の fresh / テスト: src/ui/test_type_replaces_more.rs::test_ce_7_type_replaces_number / 今: 通った / 前: 落ちた(`15` になって落ちた) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
