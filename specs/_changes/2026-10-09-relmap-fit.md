---
type: Change
id: 01M4EE9BBTJE5BJTJVJAG94Y51
title: 関係マップを幅100の端末に収める(道の幅・詳細の窓・種類の英語の文)(REL-9)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# 関係マップを幅100の端末に収める(道の幅・詳細の窓・種類の英語の文)(REL-9)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(REL-9 の「切れて読めなくなってはならない」に合わせる) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 対象外: bugfix | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

README の画面(幅100)で関係マップを撮ると、見本の3つの表(tasks・projects・members)のうち右の members の箱が切れ、右の詳細の種類(many-to-one (N:1))も切れた。道の幅と詳細の窓を詰めて、幅100で3つの箱と詳細が収まるようにする。

## 設計

- src/relmap.rs: 列の間の道の幅を 4 + 2k から 3 + 2k に(行き先の箱の前の `1▶` の2桁はそのまま)。
- src/ui/relmap.rs: 2つの段組みの詳細の窓を 30 から 26 に。詳細の項目の名前の欄を、いちばん長い名前 + 2 桁に。
- src/i18n.rs: 種類の英語を many-to-one N:1・many-to-many N:N に(括弧を外して短く)。
- examples/relations に印(.mdgrid/workspace.toml)を置き、設定なしで3つの表を1つのワークスペースにする(README の画面と try.sh のため)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| REL-9 | checked | コード: src/relmap.rs(gap_w)、src/ui/relmap.rs(Panes::Two の詳細の幅・detail_body の項目の欄) / テスト: 既存の src/ui/test_relmap.rs・src/test_relmap_unit.rs・src/ui/test_relmap_click.rs が通る。今: docs/assets/demo-relmap.svg(幅100)で3つの箱と詳細が切れずに出る |

既存のテストの削除・skip・弱体化: なし

メインが試験と、撮った画面を目で見て確かめた。

確かめた: 1 / 1
