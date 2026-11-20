---
type: Change
id: 01M479PEH4CW0DBV77VAADYV08
title: 隠しフォルダと node_modules を探さず、README の --pick の例を空白に強くする(skip-hidden-dirs)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 隠しフォルダと node_modules を探さず、README の --pick の例を空白に強くする(skip-hidden-dirs)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-skip-hidden-dirs.md |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の B-5(BV-23 のとおり)と B-3(README の `--pick path | xargs -o vi` が空白を含むパスで壊れる。見本の名前は全部空白入り)。

## 不明点と仮定

- 仮定: 渡したフォルダ(根)そのものは名前によらず探す(隠しフォルダを渡すのは利用者の意図)。飛ばすのは探していく途中の下のフォルダだけ。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 隠しフォルダと node_modules を飛ばす | BV-23 | 採択した確かめ方 | src/vault.rs, tests/test_skip_hidden.rs, docs/obsidian-bases(.ja).md | test_bv_23_*(tests/test_skip_hidden.rs) | 済 |
| 2 | README の --pick の例 | OUT-3 | 空白を含むパスでも1つの引数 | README.md, README.ja.md | 読んで判定 | 済 |

## 実装の気づき

- 点検の B-15(シンボリックリンクのノートを黙って飛ばす)は、BV-11(実体のパスで1行にする)のとおりの振る舞いだった(リンク先のノートが同じ保管庫にあり、もう1行になっている)。直さない。
- README の例は \`tr '\n' '\0' | xargs -0 -o vi\` にし、空白を含むパスが1つの引数になることを確かめた。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。探さないフォルダの決まりを1か所変えるだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-23 | checked | コード: src/vault.rs の skipped_dir と visit / テスト: tests/test_skip_hidden.rs::test_bv_23_skips_hidden_and_node_modules・test_bv_23_given_hidden_folder_is_searched / 今: 通った(全体 1255 passed) / 前: 落ちた(実装の前に、隠しフォルダと node_modules のノートが行になり落ちた) |
| OUT-3 | checked | 読んで判定: README.md・README.ja.md の --pick の例を、空白を含むパス(\`Tasks/Answer support emails.md\`)で printf に流し、1つの引数になった |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
