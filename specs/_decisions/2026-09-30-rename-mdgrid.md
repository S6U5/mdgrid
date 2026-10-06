---
type: Proposal
id: 01M3SC8N3NZ9FH92Z3Y13XDP5Q
title: TUI の名前を mdgrid にする(rename-mdgrid)
status: accepted
decided-by: AI
trigger: 人の発言
trigger-link: 会話 2026-09-30
touches: [OUT-3, CLI-1, CLI-3]
created: 2026-09-30
updated: 2026-09-30
---

# TUI の名前を mdgrid にする(rename-mdgrid)

## きっかけ

依頼の要約(会話 2026-09-30): TUI 本体の名前は汎用的なものにする。cellops は自作の TUI の上に載せるプラグイン(タスクの画面)としてならよい。候補(mdgrid・fmgrid・frontgrid・yamlgrid。crates.io はどれも未使用)から、人が mdgrid を選んだ(選択肢への答えで、打ち込んだ発言としては残らない形式なので承認の照合はしていない。次の束で見直す)。リポのフォルダ・パッケージ・仕様の本文の名前を変えた。決定記録の中の旧い名前は履歴なので変えない。

## 差分

- 変更: OUT-3 旧「パイプで使えるように、印を付けた行のノートのパスか、指定の列の値を、標準出力に出して終わってよい。」→ 新「パイプで使えるように、印を付けた行のノートのパスか、指定の列の値を、標準出力に出して終わってよい。」(確かめ方: `mdgrid --pick path` で2行に印を付けて Enter → 2つのパスが出て終わる(読んで判定))
- 変更: CLI-1 旧「`fmgrid [<.base のパス \| フォルダ> …]` で起動し、引数が無ければ今のフォルダを開くべき。」→ 新「`mdgrid [<.base のパス \| フォルダ> …]` で起動し、引数が無ければ今のフォルダを開くべき。」
- 変更: CLI-3 旧「設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/fmgrid/config.toml`)に1つ置き、キーの割り当て・色・候補の数(CE-3)・読み直しの間隔(BV-9)・East Asian Ambiguous の幅(CV-6)を持つべき。設定が無くても既定で動くべきで、知らない設定の項目は警告にとどめるべき。」→ 新「設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`)に1つ置き、キーの割り当て・色・候補の数(CE-3)・読み直しの間隔(BV-9)・East Asian Ambiguous の幅(CV-6)を持つべき。設定が無くても既定で動くべきで、知らない設定の項目は警告にとどめるべき。」

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(fmgrid のまま) | 人が名前を変えると決めた |
| mdbase | 既存の仕様(mdbase.dev)と同名 |

## 承認の記録

AI 自己採択(人の選択にもとづく名前の置き換えだけ。次の決めるモードの束の末尾で事後確認)
