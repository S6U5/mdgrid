---
type: Change
id: 01M475VNM9H0AZZAZYFYAH4ASY
title: 大きな整数・未対応の集計・保管庫の根を黙らせない(trust-notices)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# 大きな整数・未対応の集計・保管庫の根を黙らせない(trust-notices)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 不具合の直し(在る値を null やマップの印で見せない・知らせを見える所に出すだけで、要件の意味は変えない) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

3回目の点検(docs/todo.md の A)で見つかった、利用者が黙って誤る3つ。

- A-1: 64ビットに入らない整数(例 `id: 123456789012345678901234`)が画面でマップの印、--print で null になる。在る値を無いように見せない。
- A-2: ビューの summaries の未対応の集計の知らせが画面でほぼ見えない。
- A-7: `.obsidian/` の無い場所の .base が、保管庫の根の取り違えで黙って0行になる。

終わりの条件: A-1 は元の文字のまま見せ(数の列なら型の合わない印 `!`)、--print でも元の文字。A-2 は開いたとき下の行に一度出す。A-7 は `.obsidian/` が見つからず根を推した時、下の行(と結果が0行の --print の stderr)に根の場所を出す。

## 不明点と仮定

- 仮定(A-1): 入らない整数は文字の値として読む(Obsidian は JS の数で丸めるが、丸めた値を見せるより元の文字の方が誤らない)。書き戻しはしない(触らなければファイルはそのまま)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 大きな整数を文字で読む | CV-2 | 上の A-1 | src/frontmatter.rs, src/writeback.rs, tests | test_cv_2_big_int_*(src/test_big_int_unit.rs・src/test_big_int_write_unit.rs・tests/test_trust_notices.rs) | 済 |
| 2 | 未対応の集計を開いたときに知らせる | BV-14 | 上の A-2 | src/base.rs, src/ui/grid.rs | test_bv_14_unsupported_summary_*(src/ui/test_trust_notices.rs) | 済 |
| 3 | 保管庫の根を知らせる | BV-2 | 上の A-7 | src/main.rs, src/ui/app.rs, docs/obsidian-bases(.ja).md | test_bv_2_vault_root_*(tests/test_trust_notices.rs・src/ui/test_trust_notices.rs) | 済 |

## 実装の気づき

- A-1: 書き戻しは `resolve_plain` が文字を返すかで引用符を外すかを決めていた。大きな整数を文字で読むようにすると、整数の形の文字を引用符なしで書いてほかの YAML の読み手が数に読むので、`is_int_form` で整数の形は引用符を残す(test_cv_2_big_int_text_stays_quoted。前の実装でも通る、戻りを防ぐ試験)。
- A-2: 開いたときに全部の理由を出すと、錠のある test_bv_7_unsupported_formula_column(列の `?` を選んだときの案内のゴールデン)を上書きした。列の理由はセルの `?` と凡例で見えるので、開いたときに出すのは画面に印の無い理由(集計・並べ替え)だけにした(`Grid.marked`)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。知らせを足すだけの不具合の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CV-2 | checked | コード: src/frontmatter.rs の resolve_plain・is_int_form、src/writeback.rs の plain_safe / テスト: src/test_big_int_unit.rs::test_cv_2_big_int_kept_as_text・test_cv_2_big_int_boundary_still_int、tests/test_trust_notices.rs::test_cv_2_big_int_prints_original_text / 今: 通った(全体 1237 passed) / 前: 落ちた(実装の前に回し、3本とも落ちた) |
| BV-14 | checked | コード: src/ui/grid.rs の refresh、src/base.rs の Grid.marked / テスト: src/ui/test_trust_notices.rs::test_bv_14_unsupported_summary_shown_on_open・test_bv_14_unsupported_summary_shown_once / 今: 通った(全体 1237 passed) / 前: 落ちた(実装の前は App に無い項目で組み立てられず、試験の束ごと落ちた) |
| BV-2 | checked | コード: src/main.rs の open_target・print、src/ui/app.rs の load_step / テスト: tests/test_trust_notices.rs::test_bv_2_vault_root_named_when_empty_without_obsidian・test_bv_2_vault_root_silent_with_obsidian_or_rows、src/ui/test_trust_notices.rs::test_bv_2_vault_root_shown_when_empty / 今: 通った(全体 1237 passed) / 前: 落ちた(実装の前に回し、知らせが無く落ちた) |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
