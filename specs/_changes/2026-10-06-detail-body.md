---
type: Change
id: 01M47381F85AWA54S85WM8JFCA
title: 詳細の表示で本文の先頭を読むだけで見せる(detail-body)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 詳細の表示で本文の先頭を読むだけで見せる(detail-body)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [detail-body](../_decisions/2026-10-06-detail-body.md)(AI) |
| 設計 | 対象外: 条件に当たらない(詳細の表示の描き方に節を1つ足す) | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-14)が、詳細の表示で本文が見えないと指摘した。NV-6 に「プロパティの下に本文の先頭の最大 20 行を読むだけで」を足した。

終わりの条件: NV-6 の例のとおり。英語と日本語で見出しが出る。どの行も画面の幅以下(SR-9)。関係する要件: NV-6・SR-9・SR-23。

## 不明点と仮定

- 仮定: 本文の行は折り返さず幅で切る(先頭を確かめる目的)。20 行より長ければ最後に「…」の行。本文の先頭と末尾の空行は除く。フロントマターの無いノートは全体が本文。
- 仮定: 本文の行は選べない(↑↓はプロパティの間だけを動く)。画面に入らないときは、詳細の表示の今の流し方(下へ流す)で見えるところまで。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 本文のちょい見 | NV-6, SR-9, SR-23 | 上の終わりの条件のとおり | src/ui/detail.rs, src/i18n.rs, src/source.rs, src/source/markdown.rs, src/ui/test_detail_body.rs, src/ui/mod.rs | test_nv_6_body_*(src/ui/test_detail_body.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。試験と実装は同じ実装役が、試験を先に置いて落ちることを確かめてから書いた。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-6 | checked | コード: src/ui/detail.rs の detail_body の本文の節、src/source/markdown.rs の body / テスト: src/ui/test_detail_body.rs::test_nv_6_body_heading_and_first_lines・_whole_note_without_frontmatter・_capped_at_20_lines・_none_without_body・_arrows_stay_on_properties / 今: 通った(全体 1222 passed) / 前: 落ちた(実装の前に5本が落ちた。本文の無いノートと ↑↓ の2本は今の振る舞いを守る側で前から通る) |
| SR-9 | checked | コード: src/ui/detail.rs の sanitize と fit / テスト: src/ui/test_detail_body.rs::test_nv_6_body_no_control_chars_and_fits / 今: 通った / 前: 落ちた(本文の節が無い) |
| SR-23 | checked | コード: src/i18n.rs の DetailBody / テスト: src/ui/test_detail_body.rs::test_nv_6_body_heading_english / 今: 通った / 前: 落ちた(見出しが無い) |

既存のテストの削除・skip・弱体化: なし(golden nv_6 も同じ)

確かめた: 3 / 3
