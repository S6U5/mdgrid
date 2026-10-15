---
type: Proposal
id: 01M4XF3460KFP6HXDZRA134VVD
title: mdgrid を CLI で使う AI エージェント向けのスキルを、プラグインとして置く(agent-skill)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: ec76f408167ce486
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-14
touches: [CLI-22]
created: 2026-10-14
updated: 2026-10-14
---

# mdgrid を CLI で使う AI エージェント向けのスキルを、プラグインとして置く(agent-skill)

## きっかけ

人が、mdgrid は CLI としても使えるのだから、AI エージェント向けのスキル・プラグインとして置いておきたい、と頼んだ(会話 2026-10-14)。例は、状態が ToDo や Doing のタスクを選んで Markdown や CSV のファイルに出し、チャット型の AI にファイルとして渡して読ませる使い方。人の依頼を AI が要件の文に起こした。

## 差分

- 追加: CLI-22「リポは、AI エージェント(Claude Code・Codex)が mdgrid を CLI で使うための手引きのスキル(`SKILL.md`)を、プラグインとして入れられる形(Claude Code と Codex のマーケットプレイスの索引とマニフェスト)で置くべき。スキルは、頼まれた条件を `--print` の `--filter`・`--sort`・`--view`・`--format`・`--with-path` に組み立てて表をファイルに出す手順と、`--apply` で書き戻すときは差分を人に見せてから `--yes` で書く手順を書く。スキルに書いた例のコマンドは、そのまま動かして通るべき(古い旗や書けない式を AI に教えないため)。マニフェストの版は Cargo.toml の版と揃えるべき。」(確かめ方: SKILL.md の例のコマンドを見本のフォルダ(status が todo・doing・done のノート)で動かす → 全部が終了コード 0 で、todo と doing の絞り込みの例は2行だけを出す(`test_cli_22_skill_examples_run`)。マニフェストの版が Cargo.toml の版と違う → 落ちる(`test_cli_22_plugin_versions_match_crate`))

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | エージェントは mdgrid の旗と式の書き方を知らず、毎回 `--help` から組み立て直すか、Markdown を自前で解析してしまう |
| MCP サーバーを先に作る | 計画の束3の大きな案。CLI とスキルなら今の機能だけで同じ使い方ができ、どのエージェントでも動く。MCP は別の提案で決める |
| 新しい旗 `--output <ファイル>` を足す | シェルの `>` で足りる。旗を増やすと CLI-2 の一覧と補完も変わる |

## 承認の記録

2026-10-14 人の承認(要約: mdgrid を CLI として使えるスキル・プラグインとして置いてほしいという依頼)
