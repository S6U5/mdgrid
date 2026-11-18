---
type: Change
id: 01M46YD5PS69EJ18N9BN803S00
title: 式によく使うメソッドと関数を足す(common-methods)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 式によく使うメソッドと関数を足す(common-methods)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: BV-6 の「よく使う関数を評価する」の範囲で数を増やすだけ。評価できないものは今までどおり BV-7 | なし |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、Obsidian を使う人の目の点検(docs/todo.md の O-6)が、絞り込みに1つでも未対応のメソッド(`status.upper() == "ACTIVE"` など)があるとビュー全体が開かないと指摘した。共有された .base でよく使うものから足す。名前と意味は Obsidian のヘルプ(https://obsidian.md/help/bases/functions、2026-10-06 に読んだ)のとおりにする。

足すもの:
- 文字: `lower()`・`upper()`・`title()`・`trim()`・`startsWith(q)`・`endsWith(q)`・`slice(start, end?)`・`replace(pattern, replacement)`(pattern は文字だけ。正規表現は未対応のまま)・`split(separator)`(文字だけ)
- 数: `round(digits?)`・`floor()`・`ceil()`・`abs()`(`toFixed` は既にあれば保つ)
- 日付・日時: フィールド `year`・`month`・`day`・`hour`・`minute`・`second`、`date()`(時刻を落とす)、`time()`(時刻の文字 `HH:mm:ss`)、`format(fmt)`(Moment の書き方のうち `YYYY`・`YY`・`MM`・`M`・`DD`・`D`・`HH`・`H`・`mm`・`ss`・`ddd`・`dddd`・`MMM`・`MMMM` と、ほかの文字はそのまま。`[...]` の中もそのまま)
- リスト: `join(sep)`・`unique()`・`sort()`・`reverse()`・`flat()`・`slice(start, end?)`
- 関数: `max(...)`・`min(...)`・`list(x)`

終わりの条件: 上の各々に Obsidian のヘルプの意味どおりの小さな例の試験があり通る。`status.upper() == "ACTIVE"` の絞り込みのビューが開く。docs/obsidian-bases.md(英日)の対応の一覧と未対応の一覧が食い違わない(tests/test_bases_docs.rs)。関係する要件: BV-6・BV-7。

## 不明点と仮定

- 仮定: 型の合わない受け手・引数は null(今の式の決まり)。`relative()` は今の時刻に依り言い回しも Obsidian の言語の設定に依るので入れない(未対応のまま)。
- 仮定: `format` の曜日と月の名前は英語(Obsidian の既定の英語)。

## 設計

- src/expr.rs のメソッド・フィールド・関数の表(名前の一覧と評価)に足す。評価は型ごとの純関数。未対応の判定(BV-7)の一覧から足したものを外す。
- docs/obsidian-bases.md・obsidian-bases.ja.md の対応の一覧に足し、未対応の一覧から外す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | メソッドと関数 | BV-6, BV-7 | 上の終わりの条件のとおり | src/expr.rs, src/test_methods_unit.rs, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, tests/test_bases_docs.rs | test_bv_6_method_*(src/test_methods_unit.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。試験と実装は同じ実装役が、試験を先に置いて落ちることを確かめてから書いた。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-6 | checked | コード: src/expr.rs のメソッド・フィールド・関数の表と評価 / テスト: src/test_methods_unit.rs::test_bv_6_method_*(23本) / 今: 通った(全体 1164 passed) / 前: 落ちた(実装の前に回し 21本が落ちた。既存の lower と未対応の名前の2本は前から通る)。実物: `status.upper() == "ACTIVE"` の絞り込みのビューが開き、`due.format("ddd, MMM D YYYY")` が `Tue, Oct 6 2026`、`tags.unique().sort().join(" / ")` が `a / b` |
| BV-7 | checked | 読んで判定: 未対応の名前(relative など)は今までどおり未対応の印になることを src/test_methods_unit.rs::test_bv_6_method_still_unsupported_names で、文書の対応と未対応の一覧の食い違いが無いことを錠のある tests/test_bases_docs.rs で確かめた(どちらも通った) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
