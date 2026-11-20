---
type: Change
id: 01M47JDB58JTTP70JVBRHQNRG9
title: 日付の .format() に Moment の書き方を足す(Do・hh・A・WW・gggg・Q など)(moment-tokens)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# 日付の .format() に Moment の書き方を足す(Do・hh・A・WW・gggg・Q など)(moment-tokens)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: BV-6 の関数を Obsidian(Moment)と同じ結果にする不具合の直し | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

5回目の点検(docs/todo.md の C-7)。`.format("Do MMMM")` が `5o October`(Obsidian は `5th October`)、`"hh:mm A"` が `hh:04 A`、週ごとのノートの `"YYYY-[W]WW"`・`"gggg-[W]ww"` が `2026-WWW`・`gggg-Www`。Moment の書き方の記号を文字のまま出す。

終わりの条件: Moment(英語の既定のロケール)の `Do`・`hh`・`h`・`A`・`a`・`Q`・`dd`・`d`・`E`・`e`・`DDDD`・`DDD`・`WW`・`W`・`GGGG`・`ww`・`w`・`gggg`・`X`・`x`・`SSS` を Moment と同じに出す。知らない文字は今までどおりそのまま(Moment も同じ)。

## 不明点と仮定

- 仮定: 週(`ww`・`gggg`)は Moment の英語の既定(日曜始まり、1月1日を含む週が第1週)。ISO の週(`WW`・`GGGG`)は月曜始まり、木曜日を含む年。
- 仮定: `X`・`x` は地域の時刻から UTC に直した UNIX の秒・ミリ秒。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 書き方を足す | BV-6 | 上の終わりの条件 | src/expr.rs, src/test_moment_tokens_unit.rs, docs/obsidian-bases(.ja).md | test_bv_6_moment_tokens_*(src/test_moment_tokens_unit.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(書式の記号を足すだけ。期待する値は Moment の決まりから手で計算した)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-6 | checked | コード: src/expr.rs の format_when / テスト: src/test_moment_tokens_unit.rs::test_bv_6_moment_tokens_ordinal_and_12h・test_bv_6_moment_tokens_weeks・test_bv_6_moment_tokens_misc(年の境の週: 2027-01-01 は ISO で 2026-W53、日曜始まりで 2027-W01) / 今: 通った(全体 1297 passed) / 前: 落ちた(記号を文字のまま出した) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
