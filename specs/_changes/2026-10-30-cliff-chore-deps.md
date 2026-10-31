---
type: Change
id: 01M66C7E4K99WCNF9KAA2R5DGX
title: 公開リポから取り込んだ Dependabot の「chore(deps):」のコミットを変更履歴の決まりに足す(cliff-chore-deps)
status: done
size: tiny
created: 2026-10-30
updated: 2026-10-30
---

# 公開リポから取り込んだ Dependabot の「chore(deps):」のコミットを変更履歴の決まりに足す(cliff-chore-deps)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny | — |
| 仕様化 | 対象外: tiny(要件の意味を変えない) | — |
| 設計 | 対象外: tiny | — |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「設定: …」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの Dependabot の PR #47(serde_json 1.0.154)を 2026-10-30 に取り込んだところ、題が「chore(deps): …」の形(前の「Bump …」から Dependabot の設定が変わった)で、変更履歴の決まりの試験 `test_cliff_history_prefixes_all_covered` が落ちた(cliff.toml のどの parser にも当たらない)。何を: cliff.toml に `^chore\(deps\)[:：]` を Dependencies の節として足す(2026-10-25 の `^Bump ` と同じ扱い)。何ができたら終わり: `cargo test --test test_cliff` が通る。関係する要件: CLI-1(版と配布。変更履歴の決まりは開発の道具で、2026-10-25-release-0-4-0 と同じく CLI-1 に紐づける)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | cliff.toml に chore(deps) の parser を足し、試験の「prefix の形」が逃がした括弧(`\\(`・`\\)`)を許すようにする | CLI-1 | `cargo test --test test_cliff` が通る | cliff.toml tests/test_cliff.rs | test_cliff_history_prefixes_all_covered | 済 |

## 照合

自分で照合(フレッシュ文脈でない。tiny)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-1 | checked | コード: cliff.toml:65(`^chore\\(deps\\)[:：]` を Dependencies に)、tests/test_cliff.rs の `prefix_of`(逃がした括弧を語に含める) / テスト: tests/test_cliff.rs::test_cliff_history_prefixes_all_covered・test_cliff_parser_messages_are_prefix_form / 今: 通った(`cargo test --test test_cliff` 6 件) / 前: 落ちた(history_prefixes が「chore(deps)」を当たらない題として挙げた。逃がした括弧を許す前は parser の形の試験も落ちた) |

既存のテストの削除・skip・弱体化: なし(`prefix_of` は逃がした括弧を足して広げ、逃がさない括弧と他のメタ文字は今までどおり退ける)

セキュリティの回: 対象外(tiny)

全体の走り: 未測定(次の「実装:」(features-page)の直前に走らせる)

確かめた: 1 / 1
