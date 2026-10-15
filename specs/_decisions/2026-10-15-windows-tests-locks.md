---
type: Proposal
id: 01M4YVBYS98CVW37Q4JZZWBYCH
title: Windows で OS ごとの既定と違う所を確かめていた2つの試験のファイルを直し、錠を掛け直す(windows-tests-locks)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: https://github.com/S6U5/mdgrid/pull/42
touches: [CLI-3, SR-8]
created: 2026-10-15
updated: 2026-10-15
---

# Windows で OS ごとの既定と違う所を確かめていた2つの試験のファイルを直し、錠を掛け直す(windows-tests-locks)

## きっかけ

人の決定 2026-10-15-windows-target で、Windows の設定の置き場を `XDG_CONFIG_HOME` が無ければ `%APPDATA%\mdgrid`、既定のエディタを `notepad` にした。そのとき錠を掛け直した試験から、次の2つのファイルが漏れていた。Windows の試験は手元の Mac では組み立てられず、公開リポの PR #42 の CI(windows-latest)で初めて回って落ちた(`test_cli_3_config_path_follows_xdg_config_home` の `HOME` の半分。`tests/test_editor_setting.rs` の `"vi"` は、その先で落ちる)。

| ファイル | 試験 | 直し |
|---|---|---|
| `tests/test_config.rs` | test_cli_3_config_path_follows_xdg_config_home | `XDG_CONFIG_HOME` が無いとき `HOME` を見る後半を、Windows 以外だけで確かめる(前半の `XDG_CONFIG_HOME` はどの OS でも確かめる) |
| `tests/test_editor_setting.rs` | test_sr_8_none_falls_back_to_vi・test_sr_8_empty_is_same_as_absent・test_sr_8_config_empty_editor_no_warning_and_absent | どれも無いときの `"vi"` を、OS ごとの既定 `DEFAULT_EDITOR` と比べる(2026-10-15-windows-target が src/test_config_unit.rs で取ったのと同じ形。名前の値 `"vi"`・`"notepad"` は src/test_windows_unit.rs が確かめる)。`use` の行が変わるので、同じファイルのほかの8つの試験の指紋も変わる(中身は変えない) |

## 差分

- 錠: CLI-3(tests/test_config.rs::test_cli_3_config_path_follows_xdg_config_home。`HOME` の後半を Windows 以外だけに)
- 錠: SR-8(tests/test_editor_setting.rs の11の試験。3つは `"vi"` を `DEFAULT_EDITOR` に、ほかはファイルの `use` の行が変わっただけ)
- 錠: CLI-3(tests/test_editor_setting.rs::test_sr_8_config_editor_keeps_other_items。`use` の行だけ)

## 却下した案

| 案 | 理由 |
|---|---|
| 2つのファイルを Windows で組み立てない | `XDG_CONFIG_HOME` と、設定と環境変数の順の確かめは Windows でも効かせたい |
| 試験を消す | 確かめる強さが落ちる |

## 錨との照合

- macOS と ubuntu では、確かめる値と強さは今までどおり(`DEFAULT_EDITOR` はこの2つの OS で `"vi"`、`HOME` の後半も回る)。Windows では、人の決定 2026-10-15-windows-target が決めた振る舞い(`%APPDATA%`・`notepad`)と食い違う確かめだけを外す。その振る舞いは src/test_windows_unit.rs の `test_cli_3_windows_*` と `default_editor_for` の試験が確かめる。
- 前例: 2026-10-15-windows-target 自身が、SR-8 の `vi` を OS ごとの既定と比べる形にした。ここはその漏れを同じ形で埋める。
- specs に principles.md は無く、照らす P 番号も gated も無い。CLI-3・SR-8 の要件の文と例は変わらない。

判定: 向きに沿う

## 承認の記録

AI 自己採択(人の決定 2026-10-15-windows-target の漏れを同じ形で埋めるだけ。錨との照合あり)
