---
type: Change
id: 01M3XCTKC5NTYSKS866QG910NG
title: 日付の列は Obsidian と同じく囲まずに書く(date-unquoted)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# 日付の列は Obsidian と同じく囲まずに書く(date-unquoted)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [date-unquoted](../_decisions/2026-10-02-date-unquoted.md)(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): 日付を書き戻すときの二重引用符について、人はどちらでもよいと答えたうえで、囲まなくてよいと決めた。今の書き戻しはクオートの無い日付にも `due: "2026-11-03"` と二重引用符を付けて書き、write-back の未決の問いと、date-picker の照合の CE-22 の gap になっていた。

終わりの条件: 日付・日時の列に書く値は、元の値が囲んでいなければ囲まずに書き(Obsidian と同じ)、元が囲んであれば元のクオートを保つ。write-back の未決の問いを閉じ、CE-22 の gap を checked にする。関係する要件: WB-7・CE-22・CE-5・CE-20・WB-6。

## 不明点と仮定

- 問い → 答え(要約、会話 2026-10-02): 日付を囲むか → どちらでもよい、囲まなくてよい。
- 仮定: 日付の列かどうかは、列の型(Source::kind が Date か DateTime)で決める。テキストの列に `2026-11-03` と打った値は、これまでどおり WB-7 で囲む(書くと型が変わるため)。
- 仮定: ラボの別のリポの日付のクオートの決まり(harness の cross-repo の未決の問い)は、mdgrid の書き方とは別に決まる。この変更は mdgrid の中だけ。

## 設計

核の `NewValue` に `Date(String)`(`YYYY-MM-DD` か日時の形の文字列)を足す。`writeback::apply` は Date を、元の値が引用符で囲んであれば元の引用符で、そうでなければ(素の値・キーなし・空・Null)囲まずに書き、文字が日付・日時の形に合わなければ EditError にする。読み直しの検査(WB-6)は Date を、読み直した値の文字列(素の日付は Str で読まれる)と比べる。画面は、列の型(Source::kind)が Date か DateTime のセルに書くとき(入力・カレンダー・一括)に NewValue::Date を使い、ほかの列はこれまでどおり Str。Changes の same_value は Date と Str を文字列で比べる(WB-17)。

選ばなかった案: 文字の形だけで判断して囲まない(テキストの列で型が変わる。WB-7)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 日付の列を囲まずに書く | WB-18, CE-22 | 素の日付は囲まず、囲んだ日付は元の引用符、キーなしは囲まない、テキストの列は囲む、読み直しの検査を通る、画面の日付の列から書くと囲まない | src/writeback.rs, src/changes.rs, src/test_writeback_unit.rs, src/test_changes_unit.rs, src/ui, tests/test_date_unquoted.rs | test_WB_18 | 済 |

受け入れの試験(tests/test_date_unquoted.rs)は実装を見ていない役が先に書く。

## 実装の気づき

- 画面の試験 src/ui/test_calendar.rs(助けの `day()` の60行が `NewValue::Str`)と src/ui/test_input.rs の319・327行が、日付の列のためた値を `NewValue::Str` と求めて12件落ちる(クオートでなく変種の違い。Date に直せば通るはず)。入力の「同じ値なら書かない」(input.rs)は Date と Str を文字で比べる `same_new` を足した。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。WB-18 は checked、CE-22 は「画面の試験がファイルに囲んだ形も正としていた」ため gap で、main がその試験を囲まない形だけを正とするよう締めた(前の date-picker の照合で残した CE-22 の gap も、これで閉じる)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-18 | checked | コード: src/writeback.rs:124 / テスト: tests/test_date_unquoted.rs::test_wb_18_plain_date_stays_unquoted / 今: 通った(cargo test) / 前: 落ちた(変異: Date をいつも二重引用符で書く → 6件落ちた) |
| CE-22 | checked | コード: src/ui/calendar.rs:153 / テスト: src/ui/test_calendar.rs::test_ce_22_date_format_in_table_and_input / 今: 通った(cargo test) / 前: 落ちた(WB-18 の前の 7ffa909 では `due: "2026-11-03"` と囲んで書き、締めた試験で落ちる) |

既存のテストの削除・skip・弱体化: なし(画面の試験の2か所は日付の列にためる値の変種を Str から Date に合わせて厳しくした。test_ce_22 の「どちらの書き方でもよい」を囲まない形だけに締めた)

確かめた: 2 / 2
