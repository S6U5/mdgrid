---
type: Proposal
id: 01M3ZWY3KK7NYXHZHXYNWCNHW4
title: シェルの補完と man ページを出す(completions-man)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: docs/todo.md
touches: [CLI-13]
created: 2026-10-03
updated: 2026-10-03
---

# シェルの補完と man ページを出す(completions-man)

## きっかけ

docs/todo.md の D-5。会話 2026-10-03 の判断の束で、人が引数の読み取りに clap(clap_complete・clap_mangen とともに)を入れる案を選んだ(部品の選定なので設計の判断として docs/design.md に置く)。それに合わせ、補完と man の出し方の細部を AI が決める(守られる要件に触らない)。配る人(Homebrew・パッケージ)がファイルを作れるよう、実行ファイルから出す形にする。

## 差分

cli の spec.md の「要件: 版1 — 起動と設定」に足す。

- 追加: CLI-13「`mdgrid --completions <シェル>`(bash・zsh・fish・elvish・powershell)はそのシェルの補完の定義を、`mdgrid --man` は man ページ(roff)を、標準出力に出して終了コード 0 で終わってよい。知らないシェルの名前は CLI-4 と同じく理由1行と終了コード 2 にするべき。補完と man は、受け付けるオプション(CLI-2)と食い違わないよう、引数の定義から作るべき。」(確かめ方: `--completions zsh` → `#compdef mdgrid` で始まり `--readonly` を含む。`--completions nushell` → 理由1行と終了コード 2。`--man` → `.TH` を含み `--print-config` を含む(`test_CLI_13`))

## 却下した案

| 案 | 理由 |
|---|---|
| build.rs で補完と man をファイルに作る | 配る人が実行ファイルだけで作れる方が手間が少ない。cargo install の利用者にも届く |
| 補完をサブコマンドにする(`mdgrid completions zsh`) | 位置の引数はフォルダと `.base` のパス(CLI-1)で、`completions` という名前のフォルダと区別できない |
