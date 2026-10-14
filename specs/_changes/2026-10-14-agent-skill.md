---
type: Change
id: 01M4XF345A72W5G367P82HJ0YN
title: mdgrid を CLI で使う AI エージェント向けのスキルを、プラグインとして置く
status: done
size: full
created: 2026-10-14
updated: 2026-10-14
---

# mdgrid を CLI で使う AI エージェント向けのスキルを、プラグインとして置く

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [提案](../_decisions/2026-10-14-agent-skill.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-14): mdgrid は CLI としても使えるので、AI エージェント向けのスキル・プラグインとして置いておきたい。例えば、タスクの状態が ToDo や Doing のものを選んで Markdown か CSV のファイルに出し、Claude Code・Codex・チャット型の AI に、そのファイルを渡して読ませる使い方ができるようにする。

絞り込み・並べ替え・形の選択・書き戻しは、もう CLI にある(CLI-5・CLI-14・CLI-16・CLI-17)。足りないのは、エージェントがそれを知って正しく組み立てられる手引きと、それを入れる口。終わりは、リポに mdgrid のスキル(SKILL.md)があり、Claude Code と Codex のプラグインとして入れられ、スキルの例のコマンドが試験で実際に動くこと。関係する要件: 新しい CLI-22。

## 不明点と仮定

- 仮定: スキルとマニフェストは `plugins/mdgrid/` に置き、リポ直下の `.claude-plugin/marketplace.json` と `.agents/plugins/marketplace.json` から指す(decidespec と同じ形。`/plugin marketplace add S6U5/mdgrid` で入る)。外れたら: 置き場を移すだけ。
- 仮定: スキルの本文は英語で書く(公開のリポで、エージェントは英語の手引きを最もよく読む。日本語の例の値も含める)。外れたら: 日本語の版を足す。
- 仮定: プラグインの版は Cargo.toml の版と揃える(古いスキルが新しい旗を知らないずれを、版で見分けるため)。
- 仮定: 新しい CLI の旗は足さない(ファイルへの出力はシェルの `>` で足りる)。
- 人に聞くことはなし。

## 設計

- 置き場: スキルは `plugins/mdgrid/skills/mdgrid/SKILL.md`。マニフェストは `plugins/mdgrid/.claude-plugin/plugin.json`(Claude Code)・`plugins/mdgrid/.codex-plugin/plugin.json`(Codex)・`plugins/mdgrid/plugin.json`(共通の形)。索引はリポ直下の `.claude-plugin/marketplace.json` と `.agents/plugins/marketplace.json`。decidespec の配り方と同じ形にした(人が既に使っている入れ方に揃える)。
- 動く例の約束: SKILL.md の中で、情報の文字列が `sh` の囲みだけを「動く例」とする(`text` などは動かさない)。試験は一時のフォルダに見本を作り、`mdgrid` を PATH に置いて、`sh` の囲みを上から順に、同じフォルダで `sh -e` で動かす。見本は `tasks/` に `status`(todo・doing・done)・`due`(日付)・`priority`(数)を持つノートを3つと、`tasks.base`(ビュー `Open`: `status != "done"`)。例はこのキーとビューだけを使う。囲みの1つは `active.md` に、status が todo か doing の行を Markdown の表で出す。
- `sh` の囲みの試験は Unix だけで動かす(CI の Windows には sh が無い)。版の試験はどの OS でも動く。
- 却下した案: 例を Rust で字句に分けて動かす(シェルの `>` やパイプの扱いを作り直すことになる)。SKILL.md を生成する(手引きは人が読む文で、生成すると読みにくい)。
- CLI-22 の対応: タスク1(試験)・タスク2(スキルとマニフェスト)。
- 検証のコマンド: `cargo test --test test_agent_skill`、`python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | スキルの例が動くことと版が揃うことの試験を先に置く | CLI-22 | 試験がスキル無しで落ちる | tests/test_agent_skill.rs | test_cli_22_skill_examples_run, test_cli_22_plugin_versions_match_crate | 済 |
| 2 | SKILL.md・マニフェスト・索引を書く | CLI-22 | 試験が通り、README から入れ方が分かる | plugins/mdgrid/ .claude-plugin/marketplace.json .agents/plugins/marketplace.json README.md README.ja.md | test_cli_22_skill_examples_run, test_cli_22_plugin_versions_match_crate | 済 |

## 実装の気づき

- レビュー役の指摘で、スキルに注意を足した: 1つの `.md` とフォルダを混ぜると他のフォルダが出力から落ちる(CLI-15 の決まり)、出力を読むフォルダの中に置くと次の回でノートになる、`file.name` は `.md` を含む、`--apply` は書き出しと同じ場所と根から動かす。
- 残したこと: 試験が動かすのは `sh` の囲みだけで、式の箇条書きや `--yes` の行は動かさない(人が手で確かめた)。式の一覧は docs/obsidian-bases.md に任せ、スキルには代表だけを置く。
- Codex の `validate_plugin.py` はこの環境に無いので走らせていない。Claude Code の `claude plugin validate --strict` は、プラグインと索引の両方で通った。

## 照合

`check.py --evidence` は「前: 要件の手前で落ちた」の gap を出した(実装前の試験が、要件の置くべきファイルを読む所で panic したため、アサーションと見分けられない)。full のため、書込なしのフレッシュ文脈の検証役が前後のコミットで走らせ直して判定した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-22 | checked | コード: plugins/mdgrid/skills/mdgrid/SKILL.md / テスト: tests/test_agent_skill.rs::test_cli_22_skill_examples_run, tests/test_agent_skill.rs::test_cli_22_plugin_versions_match_crate / 今: 通った(cargo test --test test_agent_skill、2 passed) / 前: 落ちた(342e603 で、要件の置く SKILL.md と plugin.json が無いため2件とも落ちる。見本と実行ファイルの前置きの確かめは通っており、要件の手前の失敗ではない) |

既存のテストの削除・skip・弱体化: なし(tests/ は test_agent_skill.rs の新規だけ。錠は2件の追加だけ)

確かめた: 1 / 1
