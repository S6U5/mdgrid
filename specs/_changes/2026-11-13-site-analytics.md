---
type: Change
id: 01M794EBHMQ9CR45W6KEFHXPM4
title: サイトに Google Analytics 4 の計測のタグを、公開リポの Actions の変数から入れられるようにする
status: done
size: tiny
created: 2026-11-13
updated: 2026-11-13
---

# サイトに Google Analytics 4 の計測のタグを、公開リポの Actions の変数から入れられるようにする

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 製品の振る舞いでなく公開のサイトの作り(2026-11-13-site-seo と同じ扱い) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、サイト(GitHub Pages)の検索の管理(Search Console)とアクセス解析を設定したいと言った(会話 2026-11-13)。検索の管理の確認のタグは 2026-11-13-site-seo で入れてあり、残りは人が自分のアカウントで登録し、公開リポの変数 `GOOGLE_SITE_VERIFICATION` に値を入れること。アクセス解析は何も無いので、Google Analytics 4 を使う(人が選んだ。会話 2026-11-13)。

終わりの条件: 公開リポの Actions の変数 `GA_MEASUREMENT_ID` があるときだけ、サイトの全部の HTML(入口・言語の入口・画面の一覧・カタログ・説明書)に GA4 の計測のタグが付く。無ければ何も足さない。計測は公開のホスト(`s6u5.github.io`)でだけ動き、手元やサイトの検査(Playwright)では Google へ何も送らない。

## 不明点と仮定

- 解析の道具: GA4(人が選んだ。GoatCounter・Cloudflare とくらべて、検索の管理とつなげられる)。
- 同意の表示(Cookie のバナー)は付けない(仮定。人に伝える)。

## 設計

- `scripts/pages.py`:
  - `analytics()`: 環境変数 `GA_MEASUREMENT_ID` を読む。形は `G-` と英大文字・数字だけを受け、ほかは作り始める前に止める(確認のタグと同じ)。無ければ空。
  - タグは gtag.js を、`location.hostname` が `s6u5.github.io` のときだけ動的に読み込む小さなスクリプト。手元・CI の検査では読み込まないので、検査は Google へ通信せず、数も汚さず、外の読み込みの失敗でコンソールのエラーにもならない。
  - 付ける所: `page()` で作るページ(入口・前の URL・言語の入口・画面の一覧)の head、写したカタログ(`catalog/index.html`)と mdBook の各ページ(`decorate_book`)の `</head>` の前。
- `.github/workflows/pages.yml`: 「Build the site」の env に `GA_MEASUREMENT_ID: ${{ vars.GA_MEASUREMENT_ID }}`。ページに出る値なので secrets でなく vars。値はリポに書かない(手元のリポと公開リポの中身を同じに保つ)。
- `scripts/check_site.py`: 変数があるとき、サイトの全部の HTML にその ID のタグがあることを確かめる。
- 関わる束: `cargo test --test test_pages --test test_pages_images --test test_pages_check` と、手元で `python3 scripts/pages.py` → `python3 scripts/check_site.py`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | GA4 の計測のタグ | — | 変数のあるときだけ全部の HTML にタグ、公開のホストでだけ動く | scripts/pages.py, .github/workflows/pages.yml, scripts/check_site.py | 読んで判定(手元で作って check_site.py) | 済 |

## 実装の気づき

- 確認のファイルの方法(コミット 82be840)は取り下げた。人が、値をリポに書く(ハードコード)より変数のほうがよいと決め、タグの方法(変数 `GOOGLE_SITE_VERIFICATION`)を使う(会話 2026-11-13)。docs/pages-root と、pages.py・check_site.py の確認のファイルの扱いを戻した。

- 計測のスクリプトは即時の関数で包み、ページの変数(`s`)を外に漏らさない。
- 変数は Build の段だけでなく、検査の段(Check the site in a browser)の env にも渡す(検査がタグの有無を見るため)。
- レビューの版: 53ab0f3de50a36ea37b48ab89fbacb0c67c46172(記録はまだ追跡していないので作業ツリーで読む)

## レビューの指摘

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 細部 | scripts/check_site.py:106 | タグの ID の文字列しか見ず、ホストの判定が消えても検査が通り、CI の検査が Google へ送る回帰を捕まえない。直し方: head にホストの判定の文もあるかを見る | 直した |
| 2 | レビュー | 細部 | scripts/check_site.py:108 | 変数が無いときは飛ばすだけで、タグが付く回帰を捕まえない。直し方: 変数が無ければどのページにもタグが無いことを見る | 直した |

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1(サイトの作り)の照合: checked。手元で ja・en の全部の場面を撮り、`python3 scripts/pages.py` で3通り作った。変数なしでは googletagmanager を含むファイルが 0。`GA_MEASUREMENT_ID=G-TEST123ABC` では HTML 39 件の全部(入口・前の URL・言語の入口・画面の一覧・カタログ・説明書 ja・en の全ページ)の head にタグが付いた。`G-x"><script>` は作り始める前に止まった(終了コード 1)。check_site.py は変数ありで「全部通った」(タグの検査も ok、コンソールのエラーなし)、変数なしでもタグの検査を飛ばして全部通った。1ページからタグを消すと、タグの検査が落ちた。レビューの直しのあと: 1ページからホストの判定の文を消すと落ち、変数なしの検査を変数ありで作った site に当てると 39 件で落ち、正しく作った site ではどちらも ok。Playwright で `s6u5.github.io/mdgrid/` を手元のファイルに向けると、3ページで gtag.js を読み込み、file:// のほかのホストでは0回だった。関わる束 `cargo test --locked --test test_pages --test test_pages_images --test test_pages_check` は 5 passed。

既存のテストの削除・skip・弱体化: なし

セキュリティの回: 対象外(tiny)。値は形を `G-[A-Z0-9]+` に限り、スクリプトに変な字を入れない。

全体の走り: 未測定(Rust のコードは変えていない)

確かめた: 0 / 0(要件の無い変更。タスク1は上の文のとおり checked)
