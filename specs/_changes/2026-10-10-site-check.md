---
type: Change
id: 01M4J8FT6ZD8WK86YHK2CJJMEY
title: Pages のサイトを公開の前にブラウザで確かめる(リンク・振り分け・切り替え・画像・カタログ・スマホの幅・コンソールのエラー)
status: done
size: tiny
created: 2026-10-10
updated: 2026-10-10
---

# Pages のサイトを公開の前にブラウザで確かめる

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 確かめることは人と決めた | — |
| 仕様化 | 対象外: 公開のサイトの確かめ方。製品の振る舞いは変えない | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開したカタログが JS の文法の誤りで何も出なかった(specs/_changes/2026-10-10-catalog-quote-fix.md)。人と決めたとおり、Pages の workflow で、作ったサイトを公開の前に Playwright のブラウザで確かめ、落ちたら公開しない(会話 2026-10-10)。

## 設計

- scripts/check_site.py: site/ を手元の HTTP の口で出し、(1) 全部のページの相対のリンクと画像があること、(2) Chromium で、入口の振り分け(ja・en・ほかの言語・覚えた選択)、説明書の言語の切り替えと hreflang、画面の一覧と入口の画像と録画が読めること・絞り込み、カタログの見本の画面が描かれること・幅 390px で縮んで横にはみ出さないこと・拡大の切り替え、入口が 390px で横にはみ出さないこと、どのページにもコンソールのエラーが無いことを確かめる。
- .github/workflows/pages.yml: サイトを作ったあと Playwright(版を固定)と Chromium を入れて scripts/check_site.py を動かし、通ったときだけ上げる。
- 手元でも `python3 scripts/check_site.py site` で同じものを動かせる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | サイトの確かめ | SR-38 | 今のサイトで全部通り、カタログを壊すと落ちる | scripts/check_site.py, scripts/pages.py, docs/catalog/index.html, .github/workflows/pages.yml, tests/test_pages_check.rs | test_sr_38_pages_workflow_checks_site(tests/test_pages_check.rs) | 済 |

## 実装の気づき

- 人がスマホ(iPhone の Safari)の画面の写しで、縮めたカタログの見本の画面の半角の字だけが大きく出ると示した。Safari の字の自動の拡大(text autosizing)なので、カタログとサイトのページに `text-size-adjust: 100%` を付け、確かめにも足した(Playwright の Chromium・WebKit は自動の拡大をまねないので、指定があることを確かめる)。
- 前に公開を止めたカタログの不具合(二重引用符)をサイトの写しに戻すと、確かめがコンソールのエラーと見本の画面が無いことで落ちるのを確かめた。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-38 | checked | テスト: tests/test_pages_check.rs::test_sr_38_pages_workflow_checks_site。読んで判定: 手元で `python3 scripts/check_site.py site` が全部通り、カタログに前の不具合を戻すと落ちることを確かめた |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
