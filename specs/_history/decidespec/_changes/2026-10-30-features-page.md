---
type: Change
id: 01M6697QZJ4H4K0CHJW2VZ8TZK
title: 説明書とサイトに「できることの一覧」のページを置く(features-page)
status: done
size: full
created: 2026-10-30
updated: 2026-10-30
---

# 説明書とサイトに「できることの一覧」のページを置く(features-page)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [SR-51](../_decisions/2026-10-30-features-page.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「テスト: [SR-51] …」(6a0f5cb)と「実装: [SR-51] …」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-30): GitHub Pages のサイトに、画面の一覧(見せ方)とは別に、mdgrid にどんな機能があるかを見渡せるページを置く。なぜ: 見せ方の一覧は画面の型だけで、何ができるかは README の段落と手引きに散らばっている。何ができたら終わり: 説明書(ja・en)に「できることの一覧」のページがあり、機能ごとに本物の画面1枚・説明・手引きへのリンクが並ぶ。見せ方の一覧の次の章にあり、入口から行ける。スターターの場面が台本にある(今は無い)。関係する要件: SR-51(新)、SR-49(前例。見せ方の一覧)、SR-50(章の並び。views.md の直後に入れるので6つの相対の順は変わらない)。

## 不明点と仮定

- 載せる機能(仮定): 値をその場で直す / 差分を見て保存する / まとめて直す / 探す・絞る・並べる / 新しいノートを作る / ビューを決めて保存する / `.base` を開く / リンクで表をつなぐ / スターターから台帳を作る / ほかのコマンドと組む / 読むだけで開く / 見た目を選ぶ、の 12 に、キーとパレットの1項を添える。並びは手引き(tasks.md)の順に合わせる(やりたいことの順)。外れたら: 項を足す・減らすだけ。
- スターターの場面(仮定): パレットでスターターの一覧が開いた画面(`:` → starter → Enter)。待ちの文字は手元で撮って決める(止める: 撮れないと画像が無い)。
- 章の位置: SR-50 の 6 つの相対の順(views → getting-started → tasks → concepts → appearance → faq)は保ち、features.md は views.md の直後に置く(錠のある試験 test_sr_50_pages_py_pages_in_reading_order は PAGES の2つ目が views.md であることを見る)。
- CSV・TSV を開く機能と AI エージェントのスキルは、場面が無いので「ほかのコマンドと組む」の文で触れるだけにする(画像は付けない)。

## 設計

- 変える場所: `docs/manual/ja/features.md`・`docs/manual/en/features.md`(新)、`docs/manual/ja/index.md`・`en/index.md`(目次。views の次に1行)、`docs/manual-scenarios.toml`(場面 `starter-pick`)、`docs/manual/{ja,en}/shots.md`(台本から作り直す)、`scripts/pages.py`(`PAGES` の views.md の直後に features.md、`TEXT` に `features`・`features_desc`、入口のカード)、`README.ja.md`・`README.md`(「文書」の節のリンクに1つ)、`docs/design.md`(1行)、`AGENTS.md`(一覧に足す決まりの1句)、`specs/screen/spec.md`(SR-51)、`tests/test_features_page.rs`(新)。
- 変えない場所: 見せ方の一覧(views.md)、本体(src/)、ほかの場面、錠のある試験。
- 画像は場面の id で指す(views.md と同じ)。各項は 見出し + 画像1枚 + 2〜3行 + 手引きの節へのリンク。
- 「場面で見る」の画像は、組み込みのスターターから `--init-workspace --starter <名前> --with-samples`(英語・見本つき)で作った見本の台帳 `examples/tasks`・`examples/reading`・`examples/people` を開いて撮る(`examples/bookmarks` と同じ作り方。場面 starter-tasks・starter-reading・starter-people)。見本の制作会社の画面(demo-table・demo-relmap)をスターターの説明に使わない(レビュー #2)。
- 選んだ案: 説明書の1ページ(SR-49 と同じ理由)。却下: README の「できること」を写すだけ(画像とリンクが無い)。
- 検証のコマンド(「関わる束: …」と「全体: …」の2行):
  - 関わる束: `cargo test --test test_features_page --test test_views_page --test test_views_page_more --test test_docs_order --test test_pages --test test_pages_images` と、場面の撮り `python3.12 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang ja --only starter-pick --env LANG=ja_JP.UTF-8`(en は `--lang en --env LANG=en_US.UTF-8`)
  - 全体: `cargo test` ・ `cargo clippy --all-targets -- -D warnings` ・ `cargo fmt --check` ・ `python3 .decidespec/scripts/check.py .`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | SR-51 の受け入れの試験(テスト役) | SR-51 | 実装の前に落ち、後に通る | tests/test_features_page.rs | test_sr_51_* | 済 |
| 2 | スターターの場面を台本に足し、手元で撮り、shots.md を作り直す | SR-51 | `--only starter-pick` で ja・en の画像ができる | docs/manual-scenarios.toml docs/manual/ja/shots.md docs/manual/en/shots.md | test_sr_51_* | 済 |
| 3 | できることの一覧のページ(ja・en)と目次 | SR-51 | 12 の機能の節に画像・説明・リンクが揃い、リンク先が全部ある | docs/manual/ja/features.md docs/manual/en/features.md docs/manual/ja/index.md docs/manual/en/index.md | test_sr_51_* | 済 |
| 4 | サイトの組み立て(PAGES・文言・入口のカード)と README のリンク | SR-51 | 説明書の目次の views の次に載り、入口から行ける | scripts/pages.py README.ja.md README.md | test_sr_51_* | 済 |
| 5 | 機能を足すときの決まり | SR-51 | AGENTS.md と design.md に1句 | AGENTS.md docs/design.md | 読んで判定 | 済 |
| 6 | スターターの本物の画面のため、組み込みのスターターから見本の台帳(examples/tasks・reading・people)を作り、場面を足す(レビュー #2・#5) | SR-51 | 「場面で見る」の4つがスターターの画面で、見本の README に載る | examples/tasks examples/reading examples/people examples/README.md docs/manual-scenarios.toml | test_sr_51_features_pages_images_are_scenes | 済 |

## 実装の気づき

- レビューに渡した版: 79feeee04eb514399556d9f9c0596f2bde50a075(`git stash create`。HEAD は「テスト:」の 6a0f5cb。未追跡の新しいファイル docs/manual/ja/features.md・docs/manual/en/features.md はこの版に入らないので、レビュー役には作業ツリーで読ませる)。
- スターターの場面(`starter-pick`)の待ちの文字は、ja は「スターター:」、en は「starter:」(パレットの問いの文。locales の AskStarterPick)。
- ガントチャートは本体に無い(仕様にも実装にも無く、scope の範囲外にある)。ブックマークの機能(スターター bookmarks・木の画面・木の画面の `o` で URL)は README と手引きにしか無かったので、「場面で見る」の節で先頭に出した。
- 「実装:」の直前の全体の走り: `cargo test --no-fail-fast` 107 束 2090 件 0 失敗、41 分(21:30〜22:12)。clippy(`--all-targets -D warnings`)と `cargo fmt --check` も通った。
- スターターの本物の画面を撮るために、見本の台帳 examples/tasks・examples/reading・examples/people を `--init-workspace --starter <名前> --with-samples`(英語、MDGRID_TODAY=2026-10-06)で作った(examples/bookmarks と同じ作り方。同期の試験は bookmarks だけで、3つには無い)。場面の `ready` はパンくずの文字(`Tasks`・`Reading`・`people`)。
- 試験全体の1回目は、取り込んだ Dependabot の「chore(deps):」の題で変更履歴の試験が落ちて 17 束で止まった(別の記録 2026-10-30-cliff-chore-deps で直し、`--no-fail-fast` で走らせ直した)。
- Pages の配信が 10/22 から落ちていた(別の記録 2026-10-30-site-check-crumb で直した)。配信が通るまで、公開サイトには見せ方の一覧も載っていない。

## レビューの指摘

実装のレビュー(書込なしのレビュー役。版 79feeee と HEAD 6a0f5cb の差分と、作業ツリーの新しいファイル)の指摘を1行ずつ。区分は 重要 / 細部 / 前から、行き先は 直した / 間違い / 意図どおり / 見送り。

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 重要 | docs/manual/ja/features.md:61、docs/manual/en/features.md:61 | 「`n` でフォームが開き」が誤り。新しいノートの既定のキーは `a`(docs/keys.ja.md:63。`n` は search_next)。手引きも場面 new-note も `a` |→ 確かめる回: 本当(keymap.rs:716 が `a`)。`a`(か「+ 新規」)に直した | 直した |
| 2 | レビュー | 重要 | docs/manual/ja/features.md:13、en:13(「場面で見る」のタスク) | tasks スターターでなく見本の制作会社(examples/studio)を描いている。スターターの列は status・due・priority・tags・created・done_at だけで、project・owner も、つながる表も無い。画像 demo-table も studio の .base で、スターターのタブ(未完了・進行中・待ち・完了)でない |→ 確かめる回: 本当。スターターから作った見本の台帳(examples/tasks・reading・people)を開いた場面 starter-tasks・starter-reading・starter-people を足し、4つのスターターそれぞれの画像と、印の設定どおりの説明にした | 直した |
| 3 | レビュー | 細部 | ja:17、en:17(ブックマーク) | 「`o` で URL をブラウザで開きます」は木の画面でだけ(表の `o` はビューの設定)。→ 「木の画面で `o`」と書いた | 直した |
| 4 | レビュー | 細部 | en:13・:21 | en の語がスターターの語と合わない(タブは Open・Doing・Waiting・Done、読書は Queue・Now・Done、organization)。→ 直した | 直した |
| 5 | レビュー | 細部 | ja:21、en:21 | 読書のスターターだけ画像が無く、人と会議の段落の末尾に付け足し。→ 「場面で見る」を4つのスターターそれぞれの段落にし、スターターの本物の画面を場面に足して並べた(#2 の直しと一緒に) | 直した |
| 6 | レビュー | 細部 | ja:69、en:69(ビュー) | 「次に同じフォルダを開いたとき、その表のまま開きます」は言い過ぎ。保存したビューはタブになるだけで、次に開くのは既定の表(「既定のビューにする」が要る)。→ 直した | 直した |
| 7 | レビュー | 細部 | tests/test_features_page.rs:372・:446・:550・:576 | 節ごとの確かめが画像とリンクの数だけで説明の文を見ない。スターターの節が starter-pick の画像を使うことは見ない。landing の features.html がカードの中にあることは見ない。「12 本以上」に先頭の言語と目次のリンクが入る。→ 錠を掛けた試験は変えない。要件の確かめ方(画像が場面・節に画像とリンク・PAGES・入口の文言)は満たしているので、強める試験は次に一覧を触る変更で足す | 見送り |
| 8 | レビュー | 細部 | docs/writing.md に照らした文 | 癖は見つからず | 意図どおり |
| 9 | 確かめる回 | 前から | starters/*/ledger*/.mdgrid/workspace.toml のコメント(例: tasks:25「New note (n)」) | スターターの印のコメントが新しいノートのキーを `n` と書いている(正しくは `a`)。このページの外で、見本 examples/bookmarks と同期の試験(ST-2)にも関わるので、別の小さな変更で直す | 見送り |
| 10 | セキュリティ | 細部 | specs/_changes/2026-10-30-features-page.md:64 | 「実装の気づき」に人の理解の状態を述べる文があり、記録は公開リポへ書き出される。→ 事実だけの文に直した | 直した |
| 11 | セキュリティ | 前から | specs/_decisions/2026-10-30-features-page.md:6 | approval-evidence にセッションの ID が入る(decidespec の決まり)。公開の書き出しで public_sync.py が伏せ字にすることを確かめた | 意図どおり |

## 照合

`check.py --evidence` の行(SR-51。「前」は「テスト:」のコミット 6a0f5cb の親に今の試験を重ねた版)。規模が full なので、書込なしの検証役(フレッシュ文脈)が 6a0f5cb → d845dcb の差分と実物で読んで判定の部分を確かめ、セキュリティの役が de09ede → d845dcb の差分を読んだ。検証役が走らせたのは関わる束(`cargo test --test test_features_page --test test_views_page --test test_views_page_more --test test_docs_order`、22 件)だけ。照合のあとの直し(検証役の補足: docs/writing.md に機能の一覧の一言、ja の語尾1か所)は文だけで、関わる束をもう一度通した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-51 | checked | コード: docs/manual/ja/features.md:7-131・docs/manual/en/features.md:7-131(「場面で見る」+ 12 の機能の節 + キーとパレット。各節に画像1枚・説明・手引きへのリンク)、scripts/pages.py:47-48・:69-70(`TEXT` の features)・:376(入口のカード)・:436(`PAGES` の views.md の直後)、docs/manual/ja/index.md:17・:45、docs/manual/en/index.md:17・:45、docs/manual-scenarios.toml:783-808(場面 starter-pick・starter-tasks・starter-reading・starter-people)、examples/tasks・reading・people(スターターから作った見本の台帳)、AGENTS.md:7、docs/design.md:697、README.ja.md:233・README.md:233 / テスト: tests/test_features_page.rs::test_sr_51_features_pages_images_are_scenes・test_sr_51_features_sections_have_image_and_link・test_sr_51_features_headings_name_each_feature_exactly・test_sr_51_features_links_resolve・test_sr_51_features_anchors_point_at_headings・test_sr_51_scenarios_have_starter_pick_scene・test_sr_51_pages_py_builds_features_after_views_and_links_from_landing・test_sr_51_manual_index_links_features_page_after_views / 今: 通った(`cargo test test_sr_51_… -- --exact` 8 件、検証役の束の走り 22 件 0 失敗) / 前: 落ちた(`--test test_features_page` が失敗。features.md が無く、PAGES・目次・台本に無い) / 読んで判定: (a) 12 の節の説明のキーと機能は keys.ja.md・tasks.md・config.ja.md・safety.ja.md と合う(検証役が1つずつ照らした)。(b) 「場面で見る」の4つの説明はスターターの印と starter.toml に合い、画像の場面は examples/tasks・bookmarks・reading・people を開いている(手元の撮りの文字で確認)。(c) AGENTS.md:7 と docs/design.md:697 に「機能を足したら一覧に画面1枚と1項」の決まりがある。(d) docs/writing.md の癖の語は無く、見出しは文の形、太字なし |

既存のテストの削除・skip・弱体化: なし(実装のコミット d845dcb は試験のファイルに触れていない。間にある 18a2f70(別の記録 cliff-chore-deps)の tests/test_cliff.rs は逃がした括弧を許す広げ方で、弱体化ではない)

セキュリティの回: 指摘 2 件(細部 1 は直した、前から 1 は意図どおり。「レビューの指摘」#10・#11)

全体の走り: 41 分(cargo test --no-fail-fast。107 束 2090 件 0 失敗、21:30〜22:12)

確かめた: 1 / 1
