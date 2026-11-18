---
type: Change
id: 01M494A4YQCP2ZFPZR7AJTKZ0R
title: 公開リポの CI(macOS・Windows)で落ちた試験の、一時フォルダとパスの書き方を直す(ci-portability)
status: done
size: bugfix
created: 2026-10-07
updated: 2026-10-07
---

# 公開リポの CI(macOS・Windows)で落ちた試験の、一時フォルダとパスの書き方を直す(ci-portability)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | なし |
| 仕様化 | 対象外: bugfix(試験の書き方の誤り。要件は変えない) | なし |
| 設計 | 対象外: bugfix | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

- macOS: tests/test_default_headings.rs の2つの試験が、同じ一時フォルダの名前(プロセスの番号と時刻だけ。macOS の時刻はマイクロ秒の細かさ)を並列で使い、片方の .base を上書きした。試験ごとの名前を入れる。
- Windows: tests/test_config.rs の XDG の試験が `/tmp/...` を絶対パスとして渡していた(Windows では絶対パスでなく、XDG の決まりどおり無視される)。OS の一時フォルダの絶対パスを使う。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 一時フォルダの名前とパスを直す | BV-24, CLI-3 | 公開リポの CI の全部の OS で通る | tests/test_default_headings.rs, tests/test_config.rs, specs/test-locks.json | test_bv_24_display_name_wins(tests/test_default_headings.rs) | 済 |

## 実装の気づき

- tests/test_default_headings.rs は BV-24 の錠があるので、その試験を足した人の決定 2026-10-07-default-headings で掛け直した。確かめる中身(期待する見出し)は変えず、一時フォルダの名前だけを変えた。
- tests/test_config.rs は錠が無い。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-24 | checked | コード: tests/test_default_headings.rs の header / テスト: tests/test_default_headings.rs::test_bv_24_display_name_wins / 今: 通った / 前: 落ちた(公開リポの CI の macOS で、並列の試験と一時フォルダが重なった) |
| CLI-3 | checked | コード: tests/test_config.rs の XDG の値 / テスト: tests/test_config.rs::test_cli_3_config_path_follows_xdg_config_home / 今: 通った / 前: 落ちた(公開リポの CI の Windows で /tmp が絶対パスでなかった) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
