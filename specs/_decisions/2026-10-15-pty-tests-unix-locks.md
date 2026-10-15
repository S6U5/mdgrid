---
type: Proposal
id: 01M4YTET91AK7MYD2AG0YB4M4Q
title: 疑似端末の試験のファイルを Windows で組み立てないようにし、錠を掛け直す(pty-tests-unix-locks)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: https://github.com/S6U5/mdgrid/pull/42
touches: [WS-8]
created: 2026-10-15
updated: 2026-10-15
---

# 疑似端末の試験のファイルを Windows で組み立てないようにし、錠を掛け直す(pty-tests-unix-locks)

## きっかけ

公開リポの PR #42 の CI で、windows-latest の試験が組み立てで落ちた。疑似端末の道具 `tests/pty/mod.rs` は `#![cfg(unix)]` なので、Windows では中身が無く、`mod pty;` を使う試験のファイルが `use pty::…` を解決できない。既存の疑似端末の試験(`tests/test_e2e.rs` など)は、ファイルの頭に `#![cfg(unix)]` を置いて Windows で組み立てない。新しく足された `tests/test_marker_base_switch.rs` にはそれが無かった。頭に1行足すと、錠の指紋が変わる。

| ファイル | 試験 | 直し |
|---|---|---|
| `tests/test_marker_base_switch.rs` | test_ws_9_switch_back_to_marker_base_table | ファイルの頭に `#![cfg(unix)]` を足す(関数の中は変えない) |

## 差分

- 錠: WS-8(tests/test_marker_base_switch.rs::test_ws_9_switch_back_to_marker_base_table。ファイルの頭の `#![cfg(unix)]` だけ)

## 却下した案

| 案 | 理由 |
|---|---|
| 疑似端末の道具を Windows でも組み立てる | 道具はもともと unix だけで作ってあり、Windows の疑似端末(ConPTY)で同じに動くかは別の作業になる |
| CI の windows の試験から外す | 組み立ての失敗は試験の選び方で避けられず、ほかの試験の Windows の確かめも止まる |

## 錨との照合

- 試験の関数の中身(手順・assert・期待値)は1文字も変わらない。macOS と ubuntu では今までどおり回り、確かめる強さは同じ。Windows では、疑似端末の道具がもともと無いので回らない(ほかの疑似端末の試験と同じ扱い)。
- specs に principles.md は無く、照らす P 番号も gated も無い。WS-8 の要件の文と例は変わらない。
- 前例: 疑似端末の試験はすべてファイルの頭に `#![cfg(unix)]` を置く(tests/test_e2e.rs・tests/test_pick.rs ほか)。整形だけで錠を掛け直した前例に 2026-10-15-examples-fmt-locks があり、同じ向き。

判定: 向きに沿う

## 承認の記録

AI 自己採択(ファイルの頭の属性だけで、確かめる中身と強さは変えない。錨との照合あり)
