---
type: Change
id: 01M46TNHTVC7S94Y8BVG3JTSM3
title: 日付と日時が混ざる列は日時と推定する(mixed-datetime)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 日付と日時が混ざる列は日時と推定する(mixed-datetime)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [mixed-datetime](../_decisions/2026-10-06-mixed-datetime.md)(AI) |
| 設計 | 対象外: 条件に当たらない(推定の関数1つの直し) | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-7)が、日付の列の日時の値が `!` になると指摘した。CE-2 に「推定で日付になった列に日時の値もあれば日時の列」を足した。

終わりの条件: CE-2 の例のとおり。関係する要件: CE-2・CV-2・CE-21(日時の列のカレンダーは時刻を保つ)。

## 不明点と仮定

- 仮定: 推定の関数 `types::infer` だけを直す(types.json の型は使う側で先に決まり、推定を通らない)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 混ざる列を日時に | CE-2 | CE-2 の例のとおり | src/types.rs, src/test_mixed_datetime_unit.rs | test_ce_2_mixed_date_datetime_*(src/test_mixed_datetime_unit.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。推定の関数1つの直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-2 | checked | コード: src/types.rs の infer / テスト: src/test_mixed_datetime_unit.rs::test_ce_2_mixed_date_datetime_is_datetime・_dates_only_stay_date・_first_value_rule_otherwise、錠のある CE-2 の試験 / 今: 通った(全体 1137 passed) / 前: 落ちた(実装の前に試験を置き、is_datetime の1本が Date で落ちた)。実物: 2026-08-01 と 2026-08-03T08:15:00 が混ざる when の列で、日付だけの値に `!` が付かない |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
