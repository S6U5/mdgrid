---
type: Change
id: 01M46N6G59C3BKFNRWAKP5FFYG
title: 今日の日付と日時を地域の時刻で決める(local-today)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# 今日の日付と日時を地域の時刻で決める(local-today)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 不具合の直し(BV-6 の `today()`・`now()` は Obsidian Bases と同じ意味で、Obsidian は地域の時刻を使う) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(サブエージェント)が見つけた。日本時間の 10月6日 03:21 に、見本の Due のビューで期限 2026-10-05 の仕事が `0s`・`upcoming` と出た(期限切れのはず)。カレンダーも 5日を今日として印を付けた。今日の日付を UTC の時計の日で決めているため(src/print.rs の `today_now`)。UTC より東の人は毎朝、西の人は毎晩、期限切れの判定がずれる。

終わりの条件: UTC+9 で UTC の 10月5日 18:21(地域の 10月6日 03:21)のとき、今日は 10月6日。UTC−7 で UTC の 10月6日 03:00(地域の 10月5日 20:00)のとき、今日は 10月5日。`MDGRID_TODAY` があればそれ(今と同じ)。関係する要件: BV-6(式の関数)・CE-21(カレンダーの今日)。

## 不明点と仮定

- 仮定: 地域の時差は起動の最初(main の先頭、まだ糸が1本のとき)に1回だけ `time` の `UtcOffset::current_local_offset` で求め、取れなければ UTC(0)。`time` は ratatui を通して既に入っている依存で、機能 `local-offset` を足すだけ。
- 仮定: 時刻の付いた値(`YYYY-MM-DDTHH:MM`)は、Obsidian と同じく地域の時刻として読む。そのため「今」も地域の時計の秒(UNIX 秒 + 時差)で持ち、式の中ではどちらも同じ物差しで比べる。docs/obsidian-bases.md の「(UTC)」を直す。
- 仮定: 試験は時差を渡せる純関数で確かめる(端末の時差に依らないため)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 地域の時差で今日と今を決める | BV-6, CE-21 | 上の2つの例のとおりの今日になり、MDGRID_TODAY は今と同じ | src/print.rs, src/main.rs, src/expr.rs, src/test_today_unit.rs, Cargo.toml, Cargo.lock, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, docs/design.md | test_bv_6_today_uses_local_offset_*(src/test_today_unit.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。小さな不具合の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-6 | checked | コード: src/print.rs の today_now_at と set_local_offset、src/main.rs の main の先頭 / テスト: src/test_today_unit.rs::test_bv_6_today_uses_local_offset_east・_west・test_bv_6_today_utc_without_offset・test_bv_6_mdgrid_today_wins_over_clock / 今: 通った(cargo test --lib test_bv_6、4 passed) / 前: 落ちた(実装の前は today_now_at が無くコンパイルで落ちた(要件の手前)。時差を無視する変異で east と west の2本が「今日が10月5日」で落ちた) |
| CE-21 | checked | コード: カレンダーの今日は App::new の today_now(src/ui/app.rs)で、同じ today_now_at を通る / テスト: BV-6 と同じ純関数の試験 / 今: 通った / 前: 落ちた(同じ変異)。実物: 03:29 JST に release 版で Due のビューを出し、期限 2026-10-05 が `-1d`・`overdue`。TZ=UTC と TZ=America/Los_Angeles では `0s`・`upcoming`(どちらも地域の日付は 10月5日) |

既存のテストの削除・skip・弱体化: なし(全体 1081 passed)

確かめた: 2 / 2
