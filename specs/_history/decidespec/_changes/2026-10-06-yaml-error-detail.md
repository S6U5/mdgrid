---
type: Change
id: 01M46RWKEZYECG1YT98F374DRJ
title: 読めない YAML の理由に行と誤りを添える(yaml-error-detail)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# 読めない YAML の理由に行と誤りを添える(yaml-error-detail)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 要件の意味を変えない(WB-5 の「理由を表示」の理由を詳しくするだけ) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-9)が、閉じていないリスト(`tags: [a, b`)のノートの理由が「unreadable format (read-only)」だけで、どこが悪いか分からないと指摘した。

終わりの条件: YAML として読めないノートのセルを選ぶと、理由に「何行目(ファイルの行)」と YAML の読み手の誤りの文が出る(例: `unreadable YAML at line 3: …(read-only)`)。mdgrid の独自の検査で読むだけにした形(YAML としては読めるもの)は今の理由のまま。関係する要件: WB-5(意味は変えない)・SR-23。

## 不明点と仮定

- 仮定: `ReadOnly::InvalidYaml` は値を持たないまま(錠のある多くの試験が等しさで比べる)。理由を見せるときにだけ、ノートのバイトからフロントマターを取り出して saphyr_parser で読み直し、最初の誤りの行と文を得る。
- 仮定: `--print` で読めないノートを標準エラーに知らせるのは、CLI-5 の出力の試験に響くので、この変更に入れない(docs/todo.md に残す)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | YAML の誤りの行と文 | WB-5, SR-23 | 上の終わりの条件のとおり | src/frontmatter.rs, src/source/markdown.rs, src/i18n.rs, src/test_yaml_error_unit.rs | test_wb_5_yaml_error_*(src/test_yaml_error_unit.rs) | 済 |

## 実装の気づき

- saphyr_parser は閉じていないフローのリストの誤りを、次の行の頭で見つける。行は「その近く」になる(`tags: [a, b` が3行目なら4行目)。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。理由の文を詳しくするだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-5 | checked | コード: src/frontmatter.rs の yaml_error、src/source/markdown.rs の read_only_reason / テスト: src/test_yaml_error_unit.rs::test_wb_5_yaml_error_unclosed_flow_list・_none_for_valid_yaml、錠のある WB-5 の試験(ReadOnly::InvalidYaml の等しさ)はそのまま通る / 今: 通った(全体 1110 passed) / 前: 落ちた(実装の前は yaml_error が無くコンパイルで落ちた。要件の手前)。実物: 閉じていないリストのノートを選ぶと `unreadable YAML at line 4: while parsing a flow sequence, expected ',' or ']'` |
| SR-23 | checked | 読んで判定: src/i18n.rs の LockInvalidYamlAt に英語と日本語 |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
