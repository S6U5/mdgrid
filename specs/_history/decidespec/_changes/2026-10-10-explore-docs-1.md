---
type: Change
id: 01M4JB12N0VWPC5P044E1KA00V
title: 点検で見つかった説明書とサイトの直し(1): テーマの頁、丸い札の説明、画面の一覧、カタログ、見本で試す
status: done
size: tiny
created: 2026-10-10
updated: 2026-10-10
---

# 点検で見つかった説明書とサイトの直し(1)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 文書と見せ方の直し | — |
| 仕様化 | 対象外: 要件の意味を変えない(SR-26・SR-37・SR-38 の説明を今の実装に合わせる) | — |
| 設計 | 対象外: 文書の直し | — |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼の点検(会話 2026-10-10。docs/todo.md の F-10)で見つかった、説明書とサイトの古い・分かりにくい所を直す。

- テーマの頁が7つのまま(sumi・slate・saas・saas-dark・paper と auto が無い)。見本の設定と画面も足す。
- 見た目の頁の丸い札の説明が `nerd_font = true` のときだけ、のまま(既定は auto)。
- 画面の一覧に、README の動きの GIF を作るための「Demo frame」の場面(英語の見出し)が混ざる。
- カタログの見出しが大文字になる(NERD_FONT・CONFIG.TOML)、サイトへ戻るリンクが無い、前置きに開発者向けの文(試験が落ちる)。
- 入口の「見本で試す」が clone を前提にしているのに clone の行が無い。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 文書とサイトの直し | SR-38 | 上の5つが直り、サイトの確かめが通る | docs/manual/*/themes.md, docs/manual/*/appearance.md, docs/manual-scenarios.toml, examples/themes/, docs/catalog/index.html, scripts/pages.py | scripts/check_site.py と tests/test_pages*.rs | 済 |

## 実装の気づき

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-38 | checked | テスト: tests/test_pages.rs・test_pages_images.rs(新しいテーマの場面も撮る場面にある)。読んで判定: 手元で新しい5つのテーマの場面を日英で撮り、scripts/check_site.py が全部通った |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
