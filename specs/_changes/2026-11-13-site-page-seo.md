---
type: Change
id: 01M7992MHJC9X5S030S8M37HC5
title: サイトのページを検索と共有に向けて整える(正規の URL・共有のカード・説明書の説明文・入口の題・hreflang の指し合い・構造化データ)
status: done
size: tiny
created: 2026-11-13
updated: 2026-11-13
---

# サイトのページを検索と共有に向けて整える

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 進め方は人と決めた(会話 2026-11-13 の「mdgrid 内でやっていけるものはやりましょう」) | — |
| 仕様化 | 対象外: 製品の振る舞いでなく公開のサイトの作り | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、サイトの検索への出やすさで、まだできることを尋ねた(会話 2026-11-13)。`specs/_changes/2026-11-13-site-seo.md` で人と決めた順の4「ページを整える」に当たる穴を挙げ、人は mdgrid の中でできるものを進めると決めた。

サイトを作る `scripts/pages.py` を読んで見つけた穴:

- どのページにも正規の URL(canonical)が無く、説明書の入口が `manual/en/` と `manual/en/index.html` の2つの形で並ぶ。
- どのページにも共有のカードのタグ(OGP・Twitter Card)が無く、X・Slack・Discord に貼っても画像も文も出ない。
- 説明書(mdBook)のページに説明文(description)が無い。
- 根の入口の題が `mdgrid` だけで、探す語が入っていない。
- hreflang の x-default が指し合っていない(前の変更のレビューの指摘5)。
- 構造化データが無い。

終わりの条件: 手元で作った site/ で、サイトの地図に載るページのどれにも、地図と同じ URL の canonical・空でない description・共有のカードのタグ(画像は site/ の中にある)があり、hreflang で示した先のページが同じ組を示し返す。これを `scripts/check_site.py` が公開の前に確かめる。

## 不明点と仮定

なし

## 設計

- `scripts/pages.py`:
  - `social(lang, title, desc, url)`: canonical と、OGP(`og:type`・`og:url`・`og:title`・`og:description`・`og:image` と幅・高さ・alt、`og:site_name`、`og:locale`)と Twitter Card(`summary_large_image`)のタグを作る。`page()` は URL を受けてこれを head に足す(言語の入口・画面の一覧・根の入口・前の画面の一覧の URL)。
  - 説明書(`decorate_book`): 各章に同じタグを書き足す。hreflang も章のページだけにする(レビューの指摘5)。説明文は、作った HTML の本文の最初の段落から取る(タグを外し、長ければ文の切れ目か字数で切る)。mdBook が空の description を出していれば置き換える。印刷の版・目次・404 には足さない。
  - カタログ: 作るときに head へ canonical(`catalog/`)と共有のカードのタグを足す。`docs/catalog/index.html` 自体は変えない(SR-38 の「1ファイル・外の資源なし」のまま)。
  - URL の形を1つにする: 説明書の入口も `manual/<言語>/`(フォルダの形)。サイトの中のリンク・README・Cargo.toml の documentation がこの形なので、canonical・hreflang・サイトの地図をこれに合わせる(前の変更のレビューの指摘3の2つの形を無くす)。
  - hreflang の x-default: 画面の一覧は、言語を決めて移る `gallery.html`(これも同じ組を示す)を指す。説明書は英語の版を指す(言語を決めて移るページが無いため)。入口は今のまま根。
  - 根の入口の題: `mdgrid — Markdown frontmatter as a table in the terminal`。
  - 言語の入口に `SoftwareApplication` の JSON-LD(名前・説明・URL・OS・分類・無料・リポと crates.io)。`<` を `\u003c` にして埋める。
  - 共有のカードの画像: `docs/assets/og.png`・`docs/assets/ja/og.png`(1200×630。題・ひとことと本物の画面)を site/assets/<言語>/og.png に写す。今の hero.png は 2024×608 で、カードの比(1.91:1)に合わず左右が切れるため使わない。画像は `scripts/og_image.py`(Playwright。check_site.py と同じ道具)で作り直せる。文は pages.py の入口の文(HOME の tagline)から取る。
- `scripts/check_site.py`: ブラウザを使わない検査を足す。サイトの地図に載るページごとに、canonical が地図の URL と同じ・description が空でない・`og:title`・`og:url`(canonical と同じ)・`og:image`(site/ の中にある)・`twitter:card` がある。hreflang を持つページは、示した先(サイトの中)のページが同じ組を示し返す。`site_pages()` の説明書の入口の形も pages.py に合わせる。
- 却下:
  - サイトの地図の `lastmod`: Pages の workflow の checkout は履歴が1件だけで、ファイルごとの日付を git から取れない。履歴を全部取ると毎回の公開が重くなる。
  - カタログの英語の版を別の URL にする: `docs/catalog/index.html` の作り(言語を JS の状態で持つ)を変えるので、別の変更にする。この変更では `?lang=en` も `catalog/` に正規化される。
- 関わる束: `cargo test --locked --test test_pages --test test_pages_images --test test_pages_check --test test_views_page --test test_docs_order` と、手元で `python3 scripts/pages.py` → `python3 scripts/check_site.py`
- 全体: `cargo test`・`cargo clippy --all-targets -- -D warnings`・`cargo fmt --check`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | ページの head を整える | — | 地図に載るページに canonical・description・共有のカードのタグがあり、hreflang が指し合う | scripts/pages.py, scripts/og_image.py, docs/assets/og.png, docs/assets/ja/og.png | 読んで判定(手元で作って check_site.py) | 済 |
| 2 | 公開の前の検査 | — | check_site.py が上の条件を確かめ、欠けたら落ちる | scripts/check_site.py | 読んで判定(タグを消して落ちることを見る) | 済 |

## 実装の気づき

- mdBook は空の description(`content=""`)をどのページにも出すので、足さずに置き換える。章の本文の最初の段落は「English · 目次」のようなリンクだけの行なので、リンクを除いた字が10字に満たない段落は飛ばす。
- 手元に日本語の字(Noto Sans JP)が無く、カードの画像は npm の @fontsource/noto-sans-jp から入れた字で描いた(リポには入れない)。
- レビューの版: ce1ecc2e07c32a268d14d59b5cfde7c7111611e9(記録・scripts/og_image.py・docs/assets/og.png・docs/assets/ja/og.png はまだ追跡していないので作業ツリーで読む)

## レビューの指摘

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 細部 | scripts/check_site.py:123 | hreflang の検査が「示した先が同じ組を示し返す」だけを見て、ページ自身が組に入るかを見ない。ある章が別の章の組を持っていても通る。直し方: 示した先のファイルに自分が入ることを足す | 直した |
| 2 | レビュー | 細部 | scripts/check_site.py:128 | hreflang の無いページを黙って飛ばすので、入口や画面の一覧から hreflang が全部消えても通る。直し方: 地図に載るページ(カタログを除く)に組を必須にする | 直した |
| 3 | レビュー | 細部 | scripts/pages.py:535 | JSON-LD の license が MIT だけで、Cargo.toml の `MIT OR Apache-2.0` と違う。直し方: 2つの URL の配列にする | 直した |
| 4 | レビュー | 細部 | scripts/pages.py:613 | 最初の段落が説明文として弱い章がある(設定の章は「mdgrid reads one TOML file:」、作業の手引きは例の前提の注記)。`:` で終わる段落を飛ばす案は、設定の章で次の段落が文の途中で切れてかえって悪くなったので入れない。章ごとの説明文は人が書く文なので、別の変更にする | 見送り |
| 5 | レビュー | 前から | scripts/pages.py:579 | 検索に出さない印刷の版・目次・404 にも hreflang が付く。直し方: hreflang も章のページだけにする | 直した |

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1・2(サイトの作り)の照合: checked。手元で mdgrid を作って全部の場面を撮り(ja・en)、`python3 scripts/pages.py <一時フォルダ>/site --shots target/shots` で作った site/(地図は 31 件)に `python3 scripts/check_site.py`(ブラウザの検査も。Playwright 1.63.0)を回して 43 項目が全部通った。地図に載る 31 ページの canonical は地図の URL と同じで、説明書の入口は `manual/<言語>/` の形に揃った。壊した版では落ちた: 章の canonical を消す・画面の一覧の description を空にする・入口の og:image を無い先にする・カタログの twitter:card を消す・章の x-default を根に戻す・画面の一覧の hreflang を全部消す・章に別の章の組を持たせる。カードの画像2枚(1200×630)は目で見て、題・ひとこと・画面が読めた。関わる束 `cargo test --locked --test test_pages --test test_pages_images --test test_pages_check --test test_views_page --test test_docs_order` は 15 passed。

既存のテストの削除・skip・弱体化: なし

セキュリティの回: 対象外(tiny)

全体の走り: 1 分(`cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`cargo test`。69 秒)。`tests/test_cliff.rs` の `test_cliff_history_prefixes_all_covered` が1件落ちる(公開リポの PR の題 `docs:` が cliff.toml の commit_parsers に無い。この変更の前の main でも同じに落ちる。この変更では直さない)。ほかは 1405 件が通った。

確かめた: 0 / 0(要件の無い変更。タスク1・2は上の文のとおり checked)
