---
type: Proposal
id: 01M4FXKQJ9Y1CMMM8EB7S8EJEC
title: Windows でも通るように、Unix の前提を置いた2つの試験を直して錠を掛け直す(windows-test-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5cab76eb89168a07
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-09
touches: [WS-1]
created: 2026-10-09
updated: 2026-10-09
---

# Windows でも通るように、Unix の前提を置いた2つの試験を直して錠を掛け直す(windows-test-locks)

## きっかけ

PR #16 の CI で、Windows だけ WS-1 の試験が2つ落ちた。1つは環境変数 HOME があると決めていた(Windows の CI には無い)。もう1つは `/p` を絶対パスとして使っていた(Windows では相対で、設定のフォルダからのパスに読まれる)。AI が直してからマージする案を示し、人が直してからにすると決めた(会話 2026-10-09)。

試験の確かめる中身(`~` で書き直すこと、相対のパス、末尾の別の表と1行の形を壊さないこと)は変えない。変えるのは、HOME が無い環境では `~` の確かめを飛ばすことと、パスを一時フォルダの下の絶対パスにすること(TOML の文字列は引用して書く)だけ。

## 差分

- 錠: WS-1(src/test_workspace_more_unit.rs の test_ws_1_home_and_relative_paths。HOME が無ければ ~ の確かめを飛ばす)
- 錠: WS-1(src/test_toml_edit_safety_unit.rs の test_ws_1_trailing_table_and_inline_form_survive。パスを一時フォルダの下の絶対パスに)

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(Windows は落ちても止めない設定のまま) | 新しく足した試験が Windows で落ちるのを残すことになる |
| 試験を消す | 確かめる中身(~ と相対のパス・別の表を壊さない)を失う |

## 承認の記録

2026-10-09 人の承認(要約: Windows で落ちる試験を直してからマージすると決めた)
