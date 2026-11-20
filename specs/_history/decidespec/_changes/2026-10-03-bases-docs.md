---
type: Change
id: 01M3ZW8S2T9QT897CBQCGSEN9N
title: .base の対応範囲の文書(bases-docs)
status: done
size: full
created: 2026-10-03
updated: 2026-10-03
---

# .base の対応範囲の文書(bases-docs)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [bases-docs](../_decisions/2026-10-03-bases-docs.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の D-3。mdgrid が解釈する `.base` の範囲(読む項目・ビューの型・演算子・関数とメソッド・`file.*`)と、解釈しないものを英語と日本語の文書にし、文書の関数の一覧と式の評価が受け付ける一覧のずれを試験で落とす。会話 2026-10-03 の目標による。

終わりの条件: docs/obsidian-bases.md と docs/obsidian-bases.ja.md があり、式の評価に関数を足して文書を直さないと試験が落ち、文書に評価できない関数を書いても落ちる。関係する要件: BV-21(新)・BV-7・BV-2・BV-4〜BV-6・SC-8。

## 不明点と仮定

- 仮定: 「受け付ける一覧」は、式の読み取り(src/expr.rs)が未対応(BV-7)にしない 関数・メソッド・`file.*` の名前。今は `match` の中に散らばっているので、一覧の定数にまとめ、読み取りもその定数を引く(一か所にして、文書の試験と食い違わないように)。
- 仮定: Obsidian の Bases の関数で mdgrid が解釈しないものの一覧は、文書に「主なもの」として書き、網羅は試験しない(Obsidian の側の一覧は変わりうる)。

## 設計

- src/expr.rs に `pub const FUNCTIONS: &[&str]`(`if`・`date`・`now`・`today`・`duration` など)・`pub const METHODS: &[&str]`(`contains`・`containsAll`・…)・`pub const FILE_FIELDS: &[&str]`(`name`・`basename`・…)を置き、読み取りの `match` の名前の判定をこの定数と食い違わない形にする(定数から引くか、`match` の腕と定数の対応を単体の試験で確かめる)。名前の正確な一覧は今の `match` を正とする。
- 文書: docs/obsidian-bases.md(英)・docs/obsidian-bases.ja.md(日)。節: 読むもの(filters・formulas・properties・views の table・order・sort・groupBy など今の src/base.rs が読む項目)、ビューの型(table だけ。他は BV-7)、演算子、関数、メソッド、`file.*`、解釈しないもの(`this`・table 以外のビュー・summaries など)。
- 却下: 文書を定数から生成する(説明を手で書けない)。
- 検証: `cargo test test_bv_21`(tests/ の新しいファイル)、`./ci.sh`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 対応の一覧の定数と文書(英・日) | BV-21 | test_bv_21_* が通る | src/expr.rs, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, README.md, tests/test_bases_docs.rs | test_bv_21_*(tests/test_bases_docs.rs) | 済 |

## 実装の気づき

- タスク1: 読み取りの名前の判定は定数で先に絞る形(func_spec・meth_spec・is_length・file_func_spec・file_field)にした。定数に無い名前は `match` に腕があっても未対応になり、定数にあって腕が無ければ test_bv_21_*_are_accepted が落ちる。そのため関数を足すときは、定数・`match`・文書(英・日)の3か所を直す。`file.hasTag`・`inFolder`・`hasProperty` は FILE_FIELDS に置き、`length` は METHODS に置いた。ci.sh の `cargo test` は test_print(CLI-5)で止まるので、その後ろの試験のファイルは `cargo test -- --skip test_cli_5` で確かめた。

## 照合

書込なしのフレッシュ文脈の照合役で照合した(2026-10-03、コミット済みの clone で)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-21 | checked | コード: src/expr.rs の FUNCTIONS・METHODS・FILE_FIELDS と func_spec・meth_spec・is_length・file_func_spec・file_field、docs/obsidian-bases.md・ja.md / テスト: tests/test_bases_docs.rs::test_bv_21_*(19本) / 今: 通った(test_bases_docs 19 passed、test_expr 24 passed) / 前: 落ちた(0724f90^ はコンパイルできない。定数だけを足すと文書を見る10本が「文書を読めない」で落ちた)。変異: 文書の関数を1つ消す・METHODS から lower を消す・match に無い名前を FUNCTIONS に足す・日本語の文書に upper を足す → それぞれ落ちた |

既存のテストの削除・skip・弱体化: なし(試験の変更は新しい tests/test_bases_docs.rs だけ)

確かめた: 1 / 1
