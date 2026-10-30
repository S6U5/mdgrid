---
type: Change
id: 01M5SR3ARJSVWPKZDFRH229TJG
title: サイトと説明書に「見せ方の一覧」を置き、読む人の順に並べ、文を人が読むものに直す(views-page)
status: done
size: full
created: 2026-10-25
updated: 2026-10-30
---

# サイトと説明書に「見せ方の一覧」を置き、読む人の順に並べ、文を人が読むものに直す(views-page)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [SR-49](../_decisions/2026-10-25-views-page.md)(見せ方の一覧)、[SR-50](../_decisions/2026-10-26-docs-reading-order.md)(読む順と文の決まり) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「テスト: [SR-49][SR-50] …」(6a1cdaa)と「実装: [SR-49][SR-50] …」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-25): GitHub Pages のサイトに、mdgrid でノートがどんなふうに見られるか(画面の型と見せ方)の一覧を、画面つきで載せる。今のサイトは操作の場面が 95 枚並ぶ「画面の一覧」と、ビューの考え方の節しか無く、見せ方が一目で分からない。なぜ: 見せ方が並んでいれば魅力が伝わり、人もどんな図があるかを把握できる。何ができたら終わり: 説明書(ja・en)に「見せ方の一覧」のページがあり、1つの見せ方につき本物の画面1枚・説明・くわしい節へのリンクが並ぶ。サイトの入口から行ける。木の画面の場面が台本にあり、サイトの組み立て(CI の Pages)が通る。今後、画面の型を足すときはこの一覧に1項足す決まりにする。関係する要件: SR-49(新。見せ方の一覧)、SR-38(カタログ。前例の置き場)、TR-2(木の画面の場面)、REL(関係マップ)。

2つ目の依頼(会話 2026-10-25、作業の途中): 説明書全体について、使う前の人に自分がどう使えるかが伝わる順(魅力 → 詳しさ)に並べ直し、文の AI の書き方の癖を世間の知見を参照して抜く。サイトは docs サイトの形(「mdgrid docs」)に寄せる。何ができたら終わり: 目次と章の並びが読む人の順(どう見えるか → 見せ方の一覧 → はじめかた → 手引き → 考え方 → 見た目 → 困ったとき → 参照)になり、入口・index・はじめかた・考え方の文が `docs/writing.md` の決まりで書き直され、大げさな語が機械の検査で止まる。関係する要件: SR-50(新)。

## 不明点と仮定

- 載せる見せ方 → 人の案の承認(会話 2026-10-25): 表 → まとまり(グループ) → 木 → 関係マップ → 値の分布 → ビューのタブ → テーマ。仮定: これに「絞り込みの帯」「行番号・しましま・列の線」「親子の字下げと WBS」「日付の形と週の始まり」を足し、見る → 絞る・まとめる → 整える の順に並べる(理由: 見せ方として今ある場面を漏らさない)。外れたら: 並びと項目を直すだけ。
- 親子の字下げ: 見本(examples/)に `parent` の列を持つノートが無く、場面を撮れない。仮定: リリースの録画(demo-tree.gif)と手引きの節へのリンクで載せ、画像は付けない(見本に親子のノートを足すのは別の変更)。外れたら: 見本に親子を足す変更を起こし、場面を足す。
- 木の場面: `examples/bookmarks` を開いて `T`。画面に出る待ちの文字(ready・wait)は手元で撮って決める(止める: 撮れないと画像が無く、サイトの組み立てが落ちる)。
- 置き場: 説明書の1ページ(`docs/manual/<言語>/views.md`)にし、入口からはカードで行く。仮定: 別の HTML のページにしない(説明書なら mdBook の目次と検索に入り、リポで読む人にも効く)。
- スターターの画面は「見せ方」でないので、この変更では場面を足さない。
- 木の場面の待ちの文字 → 撮って決めた: 右の欄の題は「行: bar > Dev > Web」とフォルダの道筋で出る。`T` のあと `sleep:0.5` を挟んで `j` を3回で Web(3行)を選ぶ。
- サイトを docs サイトの形に寄せるという依頼の読み(2026-10-26 時点で答え待ち): A = サイト全体を1つの docs サイト(入口も説明書の一部、左に目次)にする / B = 今の形のまま入口と中身を読む順に直す。仮定: A で進め、入口の文と目次の並びは A・B で共通なので先に直す(止めない)。外れたら: 入口を別のページのまま残し、題だけ寄せる。
- 読む順は「手引き → 考え方」(手引きが先)。docs サイトの一般の並びでは考え方を先に置くこともあるが、人の指摘は使い方が先に見えないことなので、やりたいことから引ける手引きを先にする。

## 設計

- 変える場所: `docs/manual/ja/views.md`・`docs/manual/en/views.md`(新。1つの見せ方 = 見出し + 画像1枚 + 2〜3行 + リンク)、`docs/manual/ja/index.md`・`en/index.md`(目次に1行)、`docs/manual-scenarios.toml`(場面 `tree`: bookmarks を開いて `T`)、`docs/manual/ja・en の shots.md`(台本から `tui_shot.py` で作り直す)、`scripts/pages.py`(`PAGES` に `views.md`、`TEXT` に `views`・`views_desc`、入口の「見て選ぶ」のカードを3つに)、`docs/design.md`(画面の型を足すときの決まりを1行)、`specs/screen/spec.md`(SR-49)、`tests/test_views_page.rs`(新)。
- 変えない場所: 画面の一覧(gallery)の作り、カタログ、本体(src/)、ほかの場面。
- 要件ごとの対応: SR-49 → views.md(ja・en)・pages.py・試験。TR-2 → 場面 `tree` だけ(本体は触らない)。
- 選んだ案: 説明書のページ(git に残り、サイトでは mdBook の1章)。却下: 入口と同じ形の別の HTML(リポで読めず、文言が pages.py に埋まる)、画面の一覧に「見せ方」の札で絞る(読む順と説明が無く、一覧にならない)。
- 画像は場面の id で指す(`images/<id>.svg`。SR-38 の試験 `test_sr_38_pages_manual_images_are_scenes` がそのまま効く)。
- 文の決まり(SR-50): `docs/writing.md`(新。並べ方の順と、AI の癖の一覧と、書いたあとの読み直し)。AGENTS.md から1行で指し、`docs/design.md` に判断を書く。大げさな語の一覧は試験(`test_sr_50_*`)が入口(`scripts/pages.py` の文言)と説明書(`docs/manual/*/*.md`)と README を見る。
- 書き直す文(SR-50): `docs/manual/ja・en の index.md`(誰のため・どう見えるか・1分で試す・自分の場面・目次)、`getting-started.md`(1本の道と成功の目印)、`concepts.md`(導入の段落)、`scripts/pages.py` の `HOME`(入口の文言)。`tasks.md`・`appearance.md`・`faq.md` は導入の段落と大げさな語だけ直す(手引きの手順の文は変えない)。README は大げさな語だけ直す。
- サイトの形(A のとき): 入口を説明書の index(mdBook)にし、帯を「mdgrid docs」、画面の一覧とカタログは説明書の目次から行く。pages.py の `landing()` は index の内容を mdBook のページへ移す。
- 検証のコマンド(「関わる束: …」と「全体: …」の2行):
  - 関わる束: `cargo test --test test_views_page --test test_pages --test test_pages_images` と、場面の撮り `python3 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang ja --only tree --env LANG=ja_JP.UTF-8`(en は `--lang en --env LANG=en_US.UTF-8`)
  - 全体: `cargo test` ・ `cargo clippy --all-targets -- -D warnings` ・ `cargo fmt --check` ・ `python3 .decidespec/scripts/check.py .`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | SR-49 の受け入れの試験(テスト役) | SR-49 | 実装の前に落ち、後に通る | tests/test_views_page.rs | test_sr_49_* | 済 |
| 2 | 木の場面を台本に足し、手元で撮れることを確かめ、shots.md を作り直す | SR-49 | `--only tree` で ja・en の画像ができる | docs/manual-scenarios.toml docs/manual/ja/shots.md docs/manual/en/shots.md | test_sr_49_tree_scene_exists | 済 |
| 3 | 見せ方の一覧のページ(ja・en)と目次 | SR-49 | 8つの見せ方の見出し・画像・リンクが揃い、リンク先が全部ある | docs/manual/ja/views.md docs/manual/en/views.md docs/manual/ja/index.md docs/manual/en/index.md | test_sr_49_views_page_* | 済 |
| 4 | サイトの組み立て(PAGES と入口のカード) | SR-49 | 説明書の目次に載り、入口から行ける | scripts/pages.py | test_sr_49_pages_py_builds_views_and_links_from_landing | 済 |
| 5 | 画面の型を足すときの決まりと文の決まり | SR-49, SR-50 | design.md に節、writing.md があり AGENTS.md が指す | docs/design.md docs/writing.md AGENTS.md | 読んで判定 | 済 |
| 6 | SR-50 の受け入れの試験(目次と PAGES の順、大げさな語) | SR-50 | 実装の前に落ち、後に通る | tests/test_docs_order.rs | test_sr_50_* | 済 |
| 7 | 入口と index を読む人の順に書き直す(ja・en) | SR-50 | index.md が 誰のため → どう見えるか → 1分で試す → 自分の場面 → 目次 の順で、大げさな語が無い | docs/manual/ja/index.md docs/manual/en/index.md scripts/pages.py | test_sr_50_* 読んで判定 | 済 |
| 8 | はじめかた・考え方・手引きの導入を writing.md に照らして直す(ja・en) | SR-50 | 大げさな語が無く、語尾と長さが揃いすぎていない | docs/manual/ja/*.md docs/manual/en/*.md README.md README.ja.md | test_sr_50_* 読んで判定 | 済 |

## 実装の気づき

- レビューに渡した版: b03c0bf54799b21114211f25ea85e3a04334c728(`git stash create`。HEAD は「テスト:」の 1bf1722。追跡していない新しいファイル: docs/manual/ja/views.md・docs/manual/en/views.md・docs/writing.md)。
- 2回目のレビューに渡した版: 444cfb746f756b03ed77087026ca09d767f0b4b4(`git stash create`。HEAD は b0bd297。未追跡の views.md ja/en と writing.md はこの版にも入らず、作業ツリーで読ませた)。
- clippy(`--all-targets -D warnings`)が錠を掛けた試験 `tests/test_views_page.rs` の `iter().any()` を止めていたのを、1回目の出力で見落とした(grep の結果の有無でなく、コマンドの終了コードで判定する)。テストのコミットはまだ手元だけだったので、直して錠を掛け直し、2つのテストのコミットを 6a1cdaa に作り直した。
- 「実装:」の直前の全体の走り: `cargo test` 106 束 2082 件 0 失敗、68 分(15:30〜16:38)。clippy(`--all-targets -D warnings`)と `cargo fmt --check` も通った。走りのあとに直したのは README と views.md の文だけで、それを見る3つの束(test_views_page・test_views_page_more・test_docs_order)は直したあとにもう一度通した。
- 場面を撮る道具 `scripts/tui_shot.py` は tomllib が要るので python 3.11 以上(手元は `python3.12`。CI は 3.12)。全部の場面(97)の撮りは ja・en それぞれ 2.5 分。
- 木の画面では `T` の直後のキーが落ちるので `sleep:0.5` を挟む。右の欄の題はフォルダの道筋(`行: bar > Dev > Web`)。
- `docs/manual/ja・en の shots.md` は台本から作ったものを写す。撮り直すと、前からのずれが1か所出た(ある場面のキーが `Enter Down` → `Enter Up`。台本が先に変わっていて shots.md が古かった)。今回いっしょに直る。
- 大げさな語(強力・シームレス・直感的・画期的・最適・豊富 / seamless・robust…)と「することができます」は、説明書・README・入口のどこにも無かった。人が感じた AI っぽさは、語より並び(仕組みの説明が先、使う場面が無い)と箇条書きの多さにある。

## レビューの指摘

実装のレビュー(書込なしのレビュー役。版 b03c0bf と HEAD 1bf1722 の差分と、作業ツリーの新しいファイル)の指摘を1行ずつ。区分は 重要 / 細部 / 前から、行き先は 直した / 間違い / 意図どおり / 見送り。

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 重要 | docs/manual/ja/views.md:5・:93、docs/manual/en/views.md:5・:93 | 「どの画面も `Esc` か `q` で表に戻ります」が誤り。docs/keys.ja.md で `q` が戻るのは tree・relations・help だけで、detail(`K`)・freq(`%`)・chips(`f`)に `q` は無く、table モードの見せ方(まとまり・タブ・列の固定・行番号)では `q` は mdgrid を終える。→ 確かめる回: 本当。先頭は「`Esc` で表に戻ります」、末尾の節で `q`・`K`・`f` の違いと、表のままの見せ方では `q` が終了と書いた | 直した |
| 2 | レビュー | 重要 | docs/manual/ja/views.md:33、docs/manual/en/views.md:33 | 「階層を書いた列があれば `T` で木になる」は誤り。印 `.mdgrid/workspace.toml` の `[[table]]` に `tree = { path = "folder" }` がある表でだけ出る(keys.ja.md の tree の節、config.ja.md:65、場面 tree の text)。→ 確かめる回: 本当。印の表に `tree = { path = "folder" }` を書くと `T` で木、と書き直した | 直した |
| 3 | レビュー | 重要 | docs/manual/ja・en の views.md の「まとまり」(:23-27)と「値の分布」(:51-55) | SR-49 は1つの見せ方につき画像・説明・リンクの3つを求めるが、この2項にくわしい節へのリンクが無い(詳細と行番号も無いが8つの外)。→ 確かめる回: 本当。まとまり・行番号は手引きの「見せ方を決めて保存する」、値の分布・詳細はキーの割り当ての節へリンク | 直した |
| 4 | レビュー | 重要 | docs/manual/en/views.md:49 | `concepts.md#relation-map` の錨が外れている(見出しは「The relation map」、id は `#the-relation-map`)。試験は `#` より前しか見ないので通る。→ 確かめる回: 本当。`#the-relation-map` | 直した |
| 5 | レビュー | 重要 | docs/manual/ja・en の index.md の「1分で試す」→「自分の場面で」 | crates.io の mdgrid は 0.3.0 で、`cargo install mdgrid --locked` で入る版に starter(src/starter.rs)と木(src/folder_tree.rs)が無い。続く `--init-workspace --starter` と views.md の `T` は、0.4.0 を公開するまで動かない。→ 確かめる回: 本当。入口の「自分の場面で」と views の木に「0.4.0 から」を添え、入口の入れ方は Releases のアーカイブを先にした。v0.4.0 のタグと crates.io への公開は人の手順として残る | 直した |
| 6 | レビュー | 細部 | docs/manual/ja/index.md:16、en/index.md:16 | 「どれもキー1つで切り替わり」は、まとまりがキーでなくビューの設定(`o`)か `.base` のビューから、なので言い過ぎ。→ 木・関係マップ・値の分布・詳細はキー1つ、まとまりはビューの設定から、と書いた | 直した |
| 7 | レビュー | 細部 | docs/manual/en/index.md:9、README.md | "too good to move into a spreadsheet" は「ノートが上等すぎる」の意味になる。→ "and a shame to flatten into a spreadsheet" | 直した |
| 8 | レビュー | 細部 | scripts/pages.py:42・:62 | 入口の `manual_desc` が古い並び(考え方 → 手引き)のままで、見せ方の一覧が無い | 直した |
| 9 | レビュー | 細部 | docs/manual/ja・en の views.md:35 | `../../config.ja.md` が錨なし。木の設定は「ワークスペース・表・ビューごとの上書き」の節にある。→ 参照のページは pages.py が別の名前に写すので、錨の形が手元で確かめられない。ページの先頭でも設定の目次からたどれる | 見送り |
| 10 | レビュー | 細部 | docs/manual/ja・en の index.md の「1分で試す」 | `cargo install` はソースからのビルドで1分では終わらない。1分で終わるのは Releases のアーカイブの道。→ 見出しを「まず試す」に変え、`cargo install` は数分かかると書き、Releases の道を先に置いた | 直した |
| 11 | レビュー | 細部 | docs/manual/ja/getting-started.md:5、en:5 | 「道は1本」と言うが、同じページに入れ方が3通りと「見本を全部試す」「--readonly」が並ぶ。→ 「入れ方は3つから1つ選ぶ。見本を開いて最初のセルを直すまでがこのページの道」に | 直した |
| 12 | レビュー | 細部 | docs/writing.md:7-12 | 番号の一覧(入口の段、6段)と SR-50 の章の順(8段)が別の一覧で、どちらを指すか迷う。→ 入口の段と章の順の違いを1行で書いた | 直した |
| 13 | レビュー | 細部 | tests/test_views_page.rs:112-144・:192-225 | 画像とリンクを「ページ全体で8本以上」と数えるので、他言語・index・themes へのリンクが混ざり、まとまりと値の分布にリンクが無い版が通る(#3 の原因)。→ 錠のあるファイルは変えず、新しいファイル tests/test_views_page_more.rs で節ごとに画像1枚とリンク1本を求める試験をテスト役に足させた | 直した |
| 14 | レビュー | 細部 | tests/test_views_page.rs:147-189 | 見出しの語の部分一致(`contains`)なので、「表」は「表示」を含む見出しでも通る。→ 同じ新しいファイルで、括弧を除いた見出しの完全一致か先頭一致にした | 直した |
| 15 | レビュー | 細部 | tests/test_views_page.rs:192-225 | `#` より後(錨)を見ないので #4 を捕まえない。→ 同じ新しいファイルで、錨を見出しの id の形に直して照らす | 直した |
| 16 | レビュー | 細部 | tests/test_docs_order.rs:191-221 | 大げさな語の検査が shots.md(写し)を読み、台本 docs/manual-scenarios.toml の text を読まない。→ 同じ新しいファイルで台本の text も見る | 直した |
| 17 | レビュー(2回目) | 重要 | README.ja.md:22、README.md:22 | 「表のほかに…どれもキー1つで切り替わります」/ "Each is one key away." が #6 と同じ言い過ぎ(まとまりはビューの設定 `o` か `.base` から。docs/keys.ja.md:108) |。→ 確かめる回: 本当。index と同じ言い方に揃え、`Esc` で戻る1文も足した | 直した |
| 18 | レビュー(2回目) | 前から | README.ja.md:233、README.md:233 | 「文書」の節が「初めてなら、まず考え方を」と誘い、SR-50 の読む順(見せ方の一覧 → はじめかた → 手引き → 考え方)と逆。→ 「見せ方の一覧」と「はじめかた」を指すように直した | 直した |
| 19 | レビュー(2回目) | 細部 | tests/test_views_page_more.rs:185-195(`is_detail_link`) | 「くわしい節へのリンク」の判定が、ページ自身(`views.md#…`)と shots.md を除いていないので、自分の別の節だけを指す節が通る。今の views.md に自分へのリンクは無い。→ 錠を掛けたファイルなので変えず、自分へのリンクを足すときに新しい試験で塞ぐ | 見送り |
| 20 | レビュー(2回目) | 細部 | 記録の「レビューに渡した版」(444cfb7・b03c0bf) | `git stash create` の版は未追跡のファイル(views.md ja/en・writing.md)を含まないので、2つの版の差分に views.md の直しが無く、レビュー役は作業ツリーを読んだ。→ 記録に2回目の版も同じ注記を書き、以後は未追跡のファイルをレビュー役に名指しで渡す(implement.md の手順どおり) | 直した |
| 21 | セキュリティ | 細部 | specs/_changes/2026-10-25-views-page.md:41・:42 | 「不明点と仮定」に、人の言い回しと読める括弧つきの短い句が2つある(specs/ は公開リポへ書き出される)。→ 括弧なしの要約の文に直した | 直した |
| 22 | セキュリティ | 前から | specs/_changes/2026-10-25-views-page.md の「実装の気づき」 | 手元の履歴にしか無いコミットと stash の SHA が記録に残り、公開リポでは指す先が無い(秘密ではなく、前の記録と同じ型) | 見送り |

## 照合

`check.py --evidence` の行(SR-49・SR-50。「前」は「テスト:」のコミット 6a1cdaa の親に今の試験を重ねた版)。規模が full なので、書込なしの検証役(フレッシュ文脈)が 6a1cdaa → eba371b の差分と実物のファイルで読んで判定の部分を確かめ、セキュリティの役が bc9ce18 → eba371b の差分を読んだ。検証役が走らせたのは関わる束(`cargo test --test test_views_page --test test_views_page_more --test test_docs_order`)だけ。照合のあとの直し(検証役の指摘の「列の固定」のリンク、補足の語尾の単調4か所と README の段落頭の太字、セキュリティの回の #21)は文だけで、関わる束の3つをもう一度通した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-49 | checked | コード: docs/manual/ja/views.md:7-97・docs/manual/en/views.md:7-97(8つの見せ方の見出しに、場面の画像1枚・説明・くわしい節へのリンク)、scripts/pages.py:431(`PAGES` に views.md)・:371(入口「見て選ぶ」のカード)・:44-45・:64-65(`TEXT` の views)、docs/manual-scenarios.toml:773-780(場面 `tree`。examples/bookmarks/.mdgrid/workspace.toml:32 の木の設定が前提)、AGENTS.md:7、docs/design.md:695-697 / テスト: tests/test_views_page.rs::test_sr_49_views_pages_images_are_scenes・test_sr_49_views_pages_headings_cover_eight_views・test_sr_49_views_pages_links_resolve・test_sr_49_scenarios_have_tree_scene・test_sr_49_pages_py_builds_views_and_links_from_landing・test_sr_49_manual_index_links_views_page、tests/test_views_page_more.rs::test_sr_49_views_sections_have_image_and_link・test_sr_49_views_headings_name_each_view_exactly・test_sr_49_views_anchors_point_at_headings / 今: 通った(`cargo test test_sr_49_… -- --exact` 9 件、検証役の束の走り 14 件 0 失敗) / 前: 落ちた(`--test test_views_page` が失敗。views.md が無く、PAGES と台本に無い) / 読んで判定: 「画面の型や見せ方を足したら一覧に画面1枚と1項を足す」が AGENTS.md:7 と docs/design.md:697 の両方に決まりとして書かれている。リンクの錨は実物の見出しにある(ja: tasks.md:50・133・269、concepts.md:84、getting-started.md:61。en も同様)。「0.4.0 から」は Cargo.toml の版と合う |
| SR-50 | checked | コード: docs/manual/ja/index.md:44-49・docs/manual/en/index.md:44-49(目次が views → getting-started → tasks → concepts → appearance → faq)、scripts/pages.py:431(`PAGES` が同じ順)、docs/writing.md(新)、AGENTS.md:7、README.md・README.ja.md:8-26(冒頭と「できること」) / テスト: tests/test_docs_order.rs::test_sr_50_manual_index_toc_in_reading_order・test_sr_50_pages_py_pages_in_reading_order・test_sr_50_no_hype_words_ja・test_sr_50_no_hype_words_en、tests/test_views_page_more.rs::test_sr_50_scenarios_no_hype_words / 今: 通った(`cargo test test_sr_50_… -- --exact` 5 件、検証役の束の走り) / 前: 落ちた(`--test test_docs_order` が失敗。前の `PAGES` は index → getting-started → concepts → tasks で views 無し) / 読んで判定: 入口の index(ja・en)は「こんなとき → どう見えるか → まず試す(成功の目印あり)→ 自分の場面で → 目次 → 参照」で docs/writing.md の段に合う。index・views の ja/en で太字・文頭の接続詞・対比の型・締めの定型句・「することができ」は 0(検証役が grep で数えた)。en の見出しは sentence case。README の冒頭と「できること」は太字と絵文字が外れて段落になった。検証役が残ると見た語尾の単調4か所(views.md ja :19・:27・:57、README.ja.md:24)は照合のあとに直した |
| TR-2 | gap | 前でも通った(`check.py --evidence` が、タスク 2 の要件の欄にあった TR-2 の既存の試験 src/test_folder_tree_unit.rs::test_tr_2_* 10 件を今と前で走らせ、どちらも通った)。→ 直した: この変更は木の振る舞いを変えず、木の場面を台本に足しただけなので、タスク 2 の要件の欄から TR-2 を外した(場面の前提は検証役が examples/bookmarks の印で確かめた) |

既存のテストの削除・skip・弱体化: なし(bc9ce18 → eba371b の tests/・src/ の差分は新しい3ファイルの追加だけで削除 0。`#[ignore]`・早期 return も無く、錠は 6a1cdaa で 14 件足されたまま実装のコミットでは変わっていない)

セキュリティの回: 指摘 2 件(細部 1 は直した、前から 1 は見送り。「レビューの指摘」#21・#22)

全体の走り: 68 分(cargo test。106 束 2082 件 0 失敗、15:30〜16:38)

確かめた: 2 / 3(TR-2 は触った要件でなく、タスクの欄から外した。gap の行き先は上の表)
