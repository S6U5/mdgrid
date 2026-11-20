---
type: Change
id: 01M4YVE469WMM6TWKC56FE2M2V
title: Windows の CI で落ちた設定の置き場とエディタの試験を直す(windows-tests)
status: done
size: tiny
created: 2026-10-15
updated: 2026-10-15
---

# Windows の CI で落ちた設定の置き場とエディタの試験を直す(windows-tests)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 条件に当たらない | なし |
| 仕様化 | 済 | [2026-10-15-windows-tests-locks](../_decisions/2026-10-15-windows-tests-locks.md)(錠の掛け直しだけ。要件の文は変えない) |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの PR #42 の CI(windows-latest)で、`tests/test_config.rs::test_cli_3_config_path_follows_xdg_config_home` が落ちた(Windows は `HOME` でなく `%APPDATA%` を見る)。同じ漏れで `tests/test_editor_setting.rs` の既定のエディタ `"vi"` も Windows では落ちる。人の決定 2026-10-15-windows-target の振る舞いに試験を合わせる。関係する要件: CLI-3・SR-8(文は変えない)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 2つの試験のファイルを OS ごとの既定に合わせる | CLI-3, SR-8 | Windows で決めた振る舞いと食い違う確かめが無く、macOS では今までどおり通る | tests/test_config.rs, tests/test_editor_setting.rs, specs/test-locks.json | test_cli_3_config_path_follows_xdg_config_home・test_sr_8_none_falls_back_to_vi ほか | 済 |

## 実装の気づき

Windows の試験は手元の Mac で組み立てられないので、公開リポの CI が初めての確かめになる。CI の windows の試験は最初に落ちた試験のファイルで止まるので、ほかにも落ちる所が残っているかもしれない。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-3 | checked | テスト: tests/test_config.rs::test_cli_3_config_path_follows_xdg_config_home(macOS で通る。Windows は CI で確かめる) |
| SR-8 | checked | テスト: tests/test_editor_setting.rs の11本(macOS で通る。Windows は CI で確かめる) |

既存のテストの削除・skip・弱体化: Windows で、`HOME` を見る後半を回さない(Windows の振る舞いは %APPDATA% で、src/test_windows_unit.rs が確かめる)。macOS と ubuntu は今までどおり

確かめた: 2 / 2
