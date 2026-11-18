---
type: Proposal
id: 01M4XTRKA6T1NEKS9YG49VDFSG
title: [display] の項目の数を 12 に直した2つの試験の錠を掛け直す(note-path-locks)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: specs/_decisions/2026-10-15-note-path.md
touches: [SR-21, CLI-11, CV-8]
created: 2026-10-15
updated: 2026-10-15
---

# [display] の項目の数を 12 に直した2つの試験の錠を掛け直す(note-path-locks)

## きっかけ

決定 2026-10-15-note-path(SR-21 の `[display]` の項目に `note_path` を足し、例の項目の数を 12 にした)の実装で、`--print-config` の `[display]` の項目の数を 11 と決め打ちしていた2つの試験が落ちた。数を 12 に直し、直した理由をコメントに書いた。確かめるほかの中身(各項目の名前と既定値、読み直して警告が無いこと)は変えない。

| ファイル | 試験 | 直し |
|---|---|---|
| `tests/test_display_options.rs` | test_sr_21_print_config_has_display | `display.len()` の期待を 11 から 12 に、コメントの「11項目」を「12項目」に |
| `tests/test_row_lines_config.rs` | test_sr_21_row_extra_lines_in_print_config | 同じ(ファイルの先頭の説明の数も) |

## 差分

- 錠: SR-21(tests/test_display_options.rs::test_sr_21_print_config_has_display と tests/test_row_lines_config.rs::test_sr_21_row_extra_lines_in_print_config。項目の数が 11 から 12 になった)
- 錠: CLI-11(tests/test_display_options.rs::test_sr_21_print_config_has_display。同じ理由)
- 錠: CV-8(tests/test_row_lines_config.rs::test_sr_21_row_extra_lines_in_print_config。同じ理由)

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | 2つの試験が落ちたまま |
| 数を確かめるのをやめる | 項目が増えたのに文書と試験が追いつかないのを見つけられなくなる |

## 錨との照合

新しい文脈の検証役(書込なし、2026-10-15)の照合の要約。`specs/principles.md` は無い。

- 要件の変更に合わせただけ: 決定 2026-10-15-note-path は SR-21 の `[display]` に `note_path` を足し、例を12項目にした。差分は `display.len()` の期待を 11 から 12 にし、コメントの数を合わせただけで、新しい例と同じ。
- 確かめる中身は弱まっていない: assert の数・比べる相手・比べ方は変わらず、項目の数は「ちょうど12」の完全一致のまま。`note_path = "common"` が出ることは tests/test_note_path.rs::test_sr_21_note_path_in_print_config が確かめる。
- 人の決定とぶつからない: 2026-10-12-row-lines は一覧と数の例を決めたもので、同じ形で1つ足すだけ。CV-8 と CLI-11 の文と例は変わらない。
- 前例: 2026-10-07-group-gap-locks・2026-10-13-row-lines-locks(どちらも人の承認)。元の 2026-10-15-note-path は AI 自己採択なので、人が見直すときはこの錠も一緒に見直す。
- 判定: 向きに沿う

## 承認の記録

2026-10-15 AI 自己採択(錨との照合あり)。
