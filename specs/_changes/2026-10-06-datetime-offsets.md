---
type: Change
id: 01M47FG3MC355Q4VXNYJ0N53HK
title: Z・時差・空白の区切りの日時を日時として読む(datetime-offsets)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# Z・時差・空白の区切りの日時を日時として読む(datetime-offsets)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 不具合の直し(CE-2・CV-2・BV-6 の「日時」を ISO 8601 の書き方の全部で読む。書く形は変えない) | なし |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

5回目の点検(docs/todo.md の C-1・C-2)。`2026-10-05T21:44:59Z`・`…+09:00`・`2026-10-06 10:00:00`・`…T10:00:00.123+02:00` が日時の列で型の合わない値になり(`!`)、カレンダーで日を選ぶと今日の月から始まって時刻を捨て、--apply は「YYYY-MM-DD で書く」と断る。式では `Z` を落として地域の時刻に読み、時差つきは null で、並べ替えで後ろに回る。

終わりの条件: これらを日時として読む(型が合う。カレンダーは元の日から始め、時刻の部分(時差を含む)を元のまま保つ)。式と並べ替えでは、時差の付いた値を地域の時刻に直して比べる。書くときの形(WB-18 の囲むか)は変えない。

## 不明点と仮定

- 仮定: 時差は `Z`・`±HH:MM`・`±HHMM`・`±HH`。時差の無い値は今までどおり地域の時刻(Obsidian と同じ)。
- 仮定: C-3(日付の型の列にある日時の値)は、Obsidian の扱いを確かめていないので、この記録では扱わない。

## 設計

- `types::parse_datetime(s) -> Option<i64>`(地域の時計の秒)を1つ置き、型の判定(valid_datetime・datetime_shape)、式(expr の parse_when)、ビューの並べ替え(settings の parse_datetime)で使う。WB-18 の `date_or_datetime` は今の厳しい形(`T` の区切り、時差なし)のまま。
- カレンダーは日の部分を `parse_date(&s[..10])`、時刻の部分は10文字目の後ろの文字をそのまま持ち越す(今の作り)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 日時の読み方を1か所に | CE-2, CV-2, BV-6, CE-21 | 上の終わりの条件 | src/types.rs, src/expr.rs, src/settings.rs, src/writeback.rs, src/ui/entry.rs, src/test_datetime_offsets_unit.rs, src/ui/test_datetime_offsets.rs, src/ui/mod.rs | test_cv_2_datetime_offsets_*(src/test_datetime_offsets_unit.rs)、test_ce_21_offset_time_kept・test_ce_21_space_separated_written_with_t(src/ui/test_datetime_offsets.rs) | 済 |

## 実装の気づき

- 錠のある src/test_writeback_unit.rs(date_shapes_and_quotes)が「書く日時に空白の区切りは使わない」を確かめていた。書き戻しの日時の形(\`writeback::is_date\`)を、読む形(fits)から分けて \`types::writable_datetime\`(日付か、\`T\` の区切りの日時。時差は付けてよい)にした。読んだ \`Z\`・\`+09:00\` は書き戻せ、空白の区切りは書かない。画面と --apply で空白の区切りを打ったときは \`T\` に直して書く(src/ui/entry.rs)。
- 型の決まらない列の日付を囲まずに書く判定(\`date_or_datetime\`。untyped-date)は今の狭い形のまま。
- 地域の時差は大域の値なので、試験は時差を渡す \`parse_datetime_at\` で確かめる(並んで動く試験に響かない)。
- 本物の実行ファイルで確かめた: TZ=Asia/Tokyo で \`2026-10-05T21:44:59Z\` の \`updated.hour\` は 6、\`+09:00\` の値と時刻の順に並ぶ。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。読み方を1か所にまとめる直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CV-2 | checked | コード: src/types.rs の parse_datetime・datetime_full・split_offset / テスト: src/test_datetime_offsets_unit.rs の3本 / 今: 通った(全体 1286 passed) / 前: 落ちた |
| CE-2 | checked | コード: src/types.rs の infer(datetime_shape が時差つき・空白の区切りも日時と見る) / テスト: src/test_datetime_offsets_unit.rs::test_cv_2_datetime_offsets_fit(\`2026-10-05T21:44:59Z\` の列は日時) / 今: 通った / 前: 落ちた |
| BV-6 | checked | コード: src/expr.rs の parse_when、src/settings.rs の parse_datetime / テスト: src/test_datetime_offsets_unit.rs::test_cv_2_datetime_offsets_convert_to_local / 今: 通った / 前: 落ちた(Z を落とし、+09:00 は null) |
| CE-21 | checked | コード: src/ui/calendar.rs(今の作り)、src/ui/entry.rs / テスト: src/ui/test_datetime_offsets.rs の2本 / 今: 通った / 前: 落ちた(型の合わない値で、時刻を捨てた) |
| WB-18 | checked | コード: src/writeback.rs の is_date、src/types.rs の writable_datetime / テスト: src/test_writeback_unit.rs::date_shapes_and_quotes(錠あり) / 今: 通った / 前: 通った(書く形を変えないことの確かめ) |

既存のテストの削除・skip・弱体化: なし

確かめた: 5 / 5
