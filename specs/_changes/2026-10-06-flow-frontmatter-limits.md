---
type: Change
id: 01M47HHN1WFBS2PJZB1676FPB6
title: フローの形のフロントマターを読む前に、大きさと入れ子の深さで止める(flow-frontmatter-limits)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# フローの形のフロントマターを読む前に、大きさと入れ子の深さで止める(flow-frontmatter-limits)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 安全の直し(読めないときは今までどおり読むだけで値を見せない) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

コミット 7f6dd74(flow-frontmatter)の自動の安全の点検が、src/frontmatter.rs のサービス妨害を指摘した。保管庫のどのノートでも、区切りの間が `{` で始まれば YAML の読み手(saphyr)で読むので、深く入れ子にしたフローの値(`{a: [[[[…`)で読み手の再帰がスタックを使い切って落ちるか、大きな中身で読み込みが遅くなる。共有された保管庫のノートは信用できない入力。

終わりの条件: 中身が 64 KiB を超えるか、括弧の入れ子が 32 段を超えるときは読まずに None(今までどおり読むだけで値を見せない)。深い入れ子のノートを開いても落ちない。

## 不明点と仮定

- 仮定: 入れ子の深さは `{` と `[` を数え、引用符の中は数えない(引用符の扱いは粗くてよい。多く数える向きにずれても読まないだけ)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 大きさと深さの上限 | WB-5, BV-1 | 上の終わりの条件 | src/frontmatter.rs, src/base.rs, src/i18n.rs, src/test_flow_limits_unit.rs | test_wb_5_flow_limits_*(src/test_flow_limits_unit.rs) | 済 |

## 実装の気づき

- 安全の点検の知らせには場所(src/frontmatter.rs)と種類(サービス妨害)だけがあった。読み手に信用できない中身を渡す道は2つ: ノートのフローの形のフロントマター(7f6dd74 で足した)と .base の読み取り。両方を止めた。
  - flow_frontmatter: 中身が 64 KiB を超えるか、括弧の入れ子が 32 段を超えたら、読み手に渡さず None(文字を1回なめるだけ)。
  - check_aliases(.base とフローの形の両方が通る): イベントで数えたシーケンス・マップの入れ子が 64 段を超えたら Err(ブロックの入れ子も数える)。
- 読めない YAML の理由(yaml_error)は、再帰しないイベントの読み手を1回なめるだけなので、そのまま。
- 照合の前の試験で、10 万段の入れ子でも今の saphyr は落ちなかった(返す値が誤った)。読み手の版で振る舞いが変わりうるので、上限は読み手の外に置く。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。読む前に止める上限を足すだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-5 | checked | コード: src/frontmatter.rs の flow_frontmatter・flow_depth、src/base.rs の check_aliases / テスト: src/test_flow_limits_unit.rs::test_wb_5_flow_limits_deep_nesting_is_refused・test_wb_5_flow_limits_large_is_refused・test_wb_5_flow_limits_deep_base_is_refused / 今: 通った(全体 1293 passed) / 前: 落ちた(上限が無く、読み手に渡していた) |
| BV-1 | checked | コード: 同上 / テスト: src/test_flow_limits_unit.rs::test_wb_5_flow_limits_normal_still_read、tests/test_flow_frontmatter.rs / 今: 通った / 前: 通った(ふつうの大きさは今までどおり読む) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
