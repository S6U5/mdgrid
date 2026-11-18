---
type: Change
id: 01M4791C20SM8VCZ1Y9DN1NY0G
title: TOML のフロントマターを読むだけに、起動のカーソルを先頭に、1% 未満の割合を小数で(audit4-first)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# TOML のフロントマターを読むだけに、起動のカーソルを先頭に、1% 未満の割合を小数で(audit4-first)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-toml-frontmatter.md(WB-20)。B-1・B-12 は今の要件(SR-3・NV-9)の中の不具合 |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

4回目の点検(docs/todo.md の B)の B-2・B-1・B-12。

- B-2: 1行目が `+++` のノートは読むだけ(WB-20)。
- B-1: 大きなフォルダで、起動したときのカーソルが表の途中になる。読み込みの途中の組み直しが、最初に読んだノートを選んだまま追うため。利用者が動かしていなければ、読み終えたとき先頭にいる。
- B-12: 頻度表の割合が 20,000 行で `81 (0%)`。1% 未満は小数1桁(0.1% 未満は `<0.1%`)。

## 不明点と仮定

- 仮定(B-1): 「動かしていない」は、読み込みの途中の組み直しの前に、選んでいる位置が先頭(行 0)で表の上端も 0 のとき。先頭のまま組み直す(先頭が見出しの行でも同じ)。起動の引数で渡したノートを選ぶ(CLI-15)のは読み終えたあとなので、そのまま効く。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | +++ を読むだけ | WB-20 | 採択した確かめ方 | src/frontmatter.rs, src/source/markdown.rs, src/writeback.rs, src/i18n.rs, src/ui/test_audit4.rs, docs/safety(.ja).md | test_wb_20_*(src/ui/test_audit4.rs) | 済 |
| 2 | 起動のカーソルを先頭に | SR-3 | 上の B-1 | src/ui/app.rs, src/ui/mod.rs, src/ui/test_audit4.rs | test_sr_3_load_keeps_top | 済 |
| 3 | 1% 未満の割合 | NV-9 | 上の B-12 | src/ui/freq.rs, src/ui/test_audit4.rs, cliff.toml | test_nv_9_small_percent | 済 |

## 実装の気づき

- WB-20: 初めは \`ReadOnly\` に腕(Toml)を足したが、錠のある tests/test_safety_docs.rs が \`ReadOnly\` の腕を網羅の match で数えていてコンパイルが通らなくなった。腕は足さず、\`frontmatter::is_toml\` で、読み込み口(src/source/markdown.rs の get。\`NoFrontmatter\` のうち \`+++\` は理由つきの lock)と書き戻し(src/writeback.rs。念のための止めで NotEditable)の2か所で読むだけにした。安全の文書(docs/safety.md・.ja.md)の \`no_frontmatter\` の節に一文(小見出しの名前は錠のある試験が決めるので、小見出しは足さない)。
- B-1: 20,000 ノートでなくても、3つのフォルダに 180 ノートを 7 ずつ読むと、読み終えたとき 121 行目にいた(実装の前に落ちた)。
- cliff.toml に \`試験:\` の書き出し(前のコミットの書き出しが、変更の履歴の試験 test_cliff_history_prefixes_all_covered に当たらなかった)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。3つとも数行の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-20 | checked | コード: src/frontmatter.rs の is_toml、src/source/markdown.rs の get、src/writeback.rs の apply_inner / テスト: src/ui/test_audit4.rs::test_wb_20_toml_frontmatter_is_read_only・test_wb_20_writeback_refuses_toml / 今: 通った(全体 1253 passed) / 前: 落ちた(実装の前は lock が無く落ちた) |
| SR-3 | checked | コード: src/ui/app.rs の load_step / テスト: src/ui/test_audit4.rs::test_sr_3_load_keeps_top / 今: 通った(全体 1253 passed) / 前: 落ちた(121 行目) |
| NV-9 | checked | コード: src/ui/freq.rs の pct_text / テスト: src/ui/test_audit4.rs::test_nv_9_small_percent、錠のある src/ui/test_freq.rs::test_nv_9_tags_counts_order_and_percent / 今: 通った(全体 1253 passed) / 前: 落ちた(\`(0%)\`) |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
