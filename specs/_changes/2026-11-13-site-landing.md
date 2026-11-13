---
type: Change
id: 01M794EKZVVHAANHYC1DNC8RYZ
title: サイトの入口を、見て触って「使いたい」と思える並びと見た目に作り直す
status: done
size: tiny
created: 2026-11-13
updated: 2026-11-13
---

# サイトの入口を、見て触って「使いたい」と思える並びと見た目に作り直す

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 製品の振る舞いでなく公開のサイトの入口の見た目と並び。SR-49・SR-50・SR-51 の意味は変えない | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、サイト(GitHub Pages)を、見た人が mdgrid を使いたいと思うデザイン・UI・誘導にするよう頼んだ(会話 2026-11-13)。作る向きは画面の写し(スクショ)で見せる。

今の入口は、見出しと録画1枚、4つの特徴の小さな画像、リンクのカードが並ぶだけで、「ノートのファイルが表の行になる」「直した行だけが書かれる」という肝が、読まないと伝わらない。入れるコマンドは1つだけで、試す道・自分の場面に当てはめる道が下に埋もれている。

終わりの条件: 手元で作った site/ の入口(ja・en)が、`docs/writing.md` の読む人の順(何が見えるか → 1分で試す → 自分の場面 → できること → 見た目 → 説明書へ)に並び、幅 390px でも横にはみ出さず、明るい・暗いの両方で読め、`scripts/check_site.py` が通る。SR-49・SR-51(入口から見せ方の一覧・できることの一覧へ行ける)と SR-50(文の決まり)はそのまま守る。

## 不明点と仮定

- 人の見方(会話 2026-11-13 の続き): mdgrid の一番の良さは、md や CSV をデータベースとして扱え、リレーションを付けられ、1枚を見る画面もあり、CLI なので軽くターミナルと相性がいいこと。入口の段をこの軸で組み直す。見出しの文(キャッチフレーズ)は人が書くので、説明だけの仮の文を置く。
- 仮定: 「プレビュー」は、今ある「1つのノートの詳細(K)」として出す。本文を読む画面は今の版に無いので、あるとは書かない。CSV はリレーションの範囲に入らない(説明書のとおり)ので、リレーションの段はノートの話として書く。外れたら: 文を直す。

- 仮定: 作り直すのは入口(`ja/`・`en/`)と、同じ CSS を使う画面の一覧・言語を選ぶ入口。説明書(mdBook)とカタログの中身は変えない(見た目の帯だけ揃えるのは次)。外れたら: 範囲を足す。
- 仮定: フォントや部品を外のサーバーから読まない(今と同じく1枚の HTML で完結。速さと、見る人の通信を外へ出さないため)。
- 仮定: ブラウザの中の「触れる図」は本物の画面の代わりにしない。本物の画面(撮った場面の SVG)を主役にし、図は仕組みの説明として添え、図だと分かる見た目にする。

## 設計

- `scripts/pages.py` の `HOME`・`landing()`・`nav()`・`page()` を書き直す。見た目は `scripts/site/site.css`、入口の動きは `scripts/site/landing.js` に分け、作るときに各ページへ埋める。`TEXT` の形(`views`・`features`・`features_desc` のキー)、`PAGES`、`landing()` の中の `views.html`・`features.html` へのリンクは残す(SR-49・SR-51 の試験が読む)。
- 入口の並び(読む人の順):
  1. 上の帯(貼り付く): 名前・説明書・見た目のカタログ・画面の一覧・GitHub・言語・「はじめかた」。
  2. 見出しと、入れるコマンド(写すボタン)と、本物の画面を端末の枠に入れた「開く → 直す → 差分を見て保存」の3場面の切り替え。
  3. 仕組みの図(触れる): 左にノート1枚の Markdown、右に表。行を選ぶと左のノートが替わり、status を選び直すと、左に書かれる1行の差分が出る。
  4. 1分で試す: 3行のコマンドと、できたと分かる目印。
  5. 自分の場面: 4つのスターター(タスク・読書・人・ブックマーク)をタブで。画面とコマンド。
  6. できること: 直す・保存・リンク・見せ方を、大きな画面と短い文で。
  7. 見た目: テーマの切り替え(本物の画面を差し替える)とカタログへ。
  8. 録画(あるときだけ)、説明書・一覧へのカード、最後にもう一度入れるコマンド。
- 見た目: 方眼(grid)を地の柄にし、見出しは等幅と本文の字の組み合わせ。明るい・暗いは `prefers-color-scheme` と `data-theme`。動きは `prefers-reduced-motion` で止める。
- JS は入口の中だけ(写す・タブ・場面の切り替え・図)。JS が無くても、全部の画面と文とコマンドが読める(タブは最初の1枚、図は静かな状態)。
- `scripts/check_site.py`: 入口の新しい部品(写すボタン・タブ・図)を確かめる行を足す。今の検査(`nav.top a[data-pick]` など)が通る形を保つ。
- `docs/design.md` の「説明書とサイトの並びと文」に、入口の並びと図の扱いを1段足す。
- 関わる束: `cargo test --test test_pages --test test_pages_images --test test_pages_check --test test_views_page --test test_views_page_more --test test_features_page --test test_docs_order` と、手元で `python3 scripts/pages.py` → `python3 scripts/check_site.py`
- 全体: `cargo test`・`cargo clippy --all-targets -- -D warnings`・`cargo fmt --check`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 入口の作り直し | SR-49, SR-50, SR-51 | ja・en の入口が上の並びで出て、390px ではみ出さず、明暗の両方で読める | scripts/pages.py scripts/site/site.css scripts/site/landing.js | test_sr_49_pages_py_builds_views_and_links_from_landing / test_sr_51_pages_py_builds_features_after_views_and_links_from_landing / test_sr_50_no_hype_words_ja / 手元で作って画面を写して見る | 済 |
| 2 | サイトの検査に入口の部品を足す | — | check_site.py が写すボタン・タブ・図を確かめて通る | scripts/check_site.py | 読んで判定(手元で check_site.py) | 済 |
| 3 | 設計の記録 | — | docs/design.md に入口の並びの段がある | docs/design.md | 読んで判定 | 済 |

## 実装の気づき

- 6回目: 大きい見出しを、人が候補から選んだ「Markdown と CSV を、ターミナルでデータベースに。」にした(会話 2026-11-13。英語は合わせて訳した)。1語だけ色を付ける強調は外した。

- 5回目: 人の決定で、左の pane を架空の AI エージェントの画面(名前は sample-agent。どの製品の画面・印・名前も写さない)に作り込んだ。出す結果は本物で確かめた(Tasks.base の --print は 9 行、--check は問題が無ければ何も出さず終了コード 0)。ターミナルの段にエディタ(e で $EDITOR。docs/config.ja.md の editor)を足した。

- 4回目: 人の見方「画面を割れる環境で、AI を隣の pane に開くと輝く」を段にした。左の pane はエージェントの図(特定の製品の画面を真似ない)、右は見本 ai-human の本物の画面。書いた事実は、外の変更の読み直し(poll_ms。docs/safety.ja.md)、外で変わったノートには書かない(WB-4)、エージェント向けのスキル(plugins/mdgrid)。

- 3回目: 人が「ターミナル内で完結」をどこかに入れると決めた。ターミナルの段の見出しにし、端末との相性を、リポで確かめられる事実だけで足した(tmux の中からのコピー = OUT-1、WezTerm・Ghostty の見分け = SR-36、マウス = はじめかた)。

- 2回目(同じ日): 段を、人の決めた軸(データベースとして・リレーション・ターミナルの道具として・書くのは直した値だけ)に組み直した。CSV を開く場面 `csv-table` を台本(`docs/manual-scenarios.toml`)と画面の一覧(`docs/manual/<言語>/shots.md`)に足した。詳細(K)の画面には本文も出るので、「1枚を見る」として出した。上の設計の「入口の並び」は1回目のもので、今の並びは docs/design.md にある。

- 手元の字(コンテナの日本語のフォント)は太さが1つで、写しの見出しは細く見える。見る人の端末ではその OS の太い字になる。
- 共有したときの絵(og:image)と題・説明のタグ、タブの印(favicon)を入口に足した(前の記録 2026-11-13-site-seo が「ページを整える」に回した分のうち、入口の分)。hreflang の x-default が相互でない件は、この変更でも触っていない。
- 録画(GIF)の段は、リリースから取ってきた分があるときだけ出る。手元では取っていないので、写しには無い。

## レビューの指摘

なし

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-49 | checked | テスト: tests/test_views_page.rs::test_sr_49_pages_py_builds_views_and_links_from_landing / コード: scripts/pages.py の landing() のカードと「できること」の1項が views.html を指す |
| SR-50 | checked | テスト: tests/test_docs_order.rs::test_sr_50_no_hype_words_ja・test_sr_50_no_hype_words_en・test_sr_50_pages_py_pages_in_reading_order / 読んで判定: 入口の段が docs/writing.md の読む人の順 |
| SR-51 | checked | テスト: tests/test_features_page.rs::test_sr_51_pages_py_builds_features_after_views_and_links_from_landing / コード: landing() のカードが features.html を指す |

要件の無いタスク2・3: checked。手元で全部の場面を撮り(ja・en)、`python3 scripts/pages.py <一時フォルダ>/site --shots target/shots` で作った site/ を `python3 scripts/check_site.py` で確かめ、65 件が全部通った(2回目の並びで)(入口の部品を触る検査を ja・en に足した分を含む。390px の入口は ja・en とも横にはみ出さない)。入口は明るい・暗い・スマホの幅で写して目で見た。

既存のテストの削除・skip・弱体化: なし

セキュリティの回: 対象外(tiny)

全体の走り: 未測定(Rust のコードは変えていない。関わる束 `cargo test --locked --test test_pages --test test_pages_images --test test_pages_check --test test_views_page --test test_views_page_more --test test_features_page --test test_docs_order` は全部通った)

確かめた: 3 / 3
