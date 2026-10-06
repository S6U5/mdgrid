---
type: Change
id: 01M47A26JA057A89M172BMGCMZ
title: --help の折り返しと、table 以外のビューの理由の「版1」を直す(help-wording)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# --help の折り返しと、table 以外のビューの理由の「版1」を直す(help-wording)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 条件に当たらない | なし |
| 仕様化 | 対象外: 文言の直しで要件の意味を変えない | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 対象外: 条件に当たらない | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の B-14。cards などのビューの理由が「version 1 supports table only」で、実行ファイルの版(0.0.1)と食い違って読める。--help の説明の段落が途中で早く折り返す。

## 不明点と仮定

対象外

## 設計

対象外: 条件に当たらない

## タスク

対象外: 条件に当たらない

## 実装の気づき

なし

## 照合

自分で照合(文言だけ)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-7 | checked | 読んで判定: src/i18n.rs の BaseViewUnsupported を「mdgrid shows table views only」「mdgrid は table のビューだけ」に、docs/obsidian-bases(.ja).md と docs/manual-scenarios.toml の例も同じに。全体 1255 passed |
| CLI-13 | checked | 読んで判定: --help の説明の段落を英語で80桁に収めて折り返した(\`mdgrid --help\` の8〜11行目を見た) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
