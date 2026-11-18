---
type: Proposal
id: 01M4XS9VQB56M3HVZHC3Z8F3FF
title: 見本の試験を cargo fmt の形にそろえ、錠を掛け直す(examples-fmt-locks)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: ci.sh
touches: [REL-5, CE-16, CE-22, CLI-3]
created: 2026-10-15
updated: 2026-10-15
---

# 見本の試験を cargo fmt の形にそろえ、錠を掛け直す(examples-fmt-locks)

## きっかけ

人の決定 2026-10-15-examples-studio-locks で書き直した `tests/test_examples.rs` と、そのとき足した `tests/test_example_studio.rs` が、rustfmt の形になっていなかった。ci.sh と CI は `cargo fmt --check` を回すので、このままでは落ちる。rustfmt をかけると、関数の中の行の折り返しだけが変わり、錠の指紋が変わる。

直した試験(行の折り返しだけ。確かめる中身は変えない。同じファイルの test_examples_open_folder・test_example_studio_marker_tables も折り返されたが、錠の指紋は空白を落として取るので変わらない):

| ファイル | 試験 | 直し |
|---|---|---|
| `tests/test_examples.rs` | test_examples_showcase | 長い行(ビューの名前の配列・`b.build`・`b.cell` の呼び出し)を rustfmt が折り返した |
| `tests/test_example_studio.rs` | test_example_studio_linked_rows_of_a_project | 数える閉包を rustfmt が折り返した |

## 差分

- 錠: REL-5(tests/test_example_studio.rs::test_example_studio_linked_rows_of_a_project。rustfmt の折り返しだけ)
- 錠: CE-16(tests/test_examples.rs::test_examples_showcase。同じ理由)
- 錠: CE-22(tests/test_examples.rs::test_examples_showcase。同じ理由)
- 錠: CLI-3(tests/test_examples.rs::test_examples_showcase。同じ理由)

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | cargo fmt --check が落ち、CI が通らない |
| rustfmt の対象から外す(#[rustfmt::skip]) | 整形を避けるための印が試験に残り、ほかの試験と書き方がそろわない |

## 錨との照合

検証役(新しい文脈の読むだけのサブエージェント、2026-10-15)の照合:

- 差分は rustfmt による改行と字下げだけ。ほかに `b.build(...)` と `b.cell(...)` の引数の末尾にカンマが付き、`from` の閉包の本体が `{ }` でくくられた。どれも意味は変わらない。assert の対象・期待値・比べ方は1つも変わらず、確かめる強さも同じ。
- 錠の指紋は空白を落として取るので、指紋が変わるのは末尾のカンマと `{ }` が入った test_examples_showcase と test_example_studio_linked_rows_of_a_project の2本だけ。test_examples_open_folder と test_example_studio_marker_tables も折り返されたが、錠は今のままで一致する。
- specs に principles.md は無く、照らす P 番号も gated も無い。触る4つの要件は E15 のブレーキの対象にならない。
- 人の決定 2026-10-15-examples-studio-locks は「確かめる要件と強さは変えない」で試験を書き直した。今回はその中身をそのまま保つ。整形だけで錠を掛け直した前例に 2026-10-10-ci-locks と 2026-10-12-padding-default-one-locks があり、同じ向き。
- REL-5(2026-10-08-relations・2026-10-09-workspaces)、CE-16(2026-10-02-list-select)、CE-22(2026-10-02-date-format)、CLI-3(2026-10-10-config-v2)の人の決定とぶつかる点は無い。要件の文と例は変わらない。

判定: 向きに沿う

## 承認の記録

AI 自己採択(整形だけで、確かめる中身と強さは変えない。錨との照合あり)
