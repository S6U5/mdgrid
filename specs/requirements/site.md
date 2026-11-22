# サイト(site) の要求

## 目的

mdgrid の公開のサイト(GitHub Pages)のうち、説明書の外に置くもの。今はブログ(記事)だけ。記事では、そのときの話題から mdgrid を使う理由と使い方を伝える。記事は1記事1ファイルの Markdown で書き、記事の置き場を mdgrid で表として開いて管理できるようにする。書く側も「Markdown のファイルで管理する」形を自分で使う。

説明書の章・見せ方の一覧・機能の一覧・文の決まりの要求は [screen](screen.md) にある(`screen.views-page-*`・`screen.features-page-*`・`screen.docs-order-*`)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-23 に足した。

## 範囲外

- 採らない: 記事へのコメント・いいね・メールでの購読。外のサービスに頼ることになるため。記事を追う人には Atom のフィードを出す。
- 採らない: 記事を日本語と英語の外の言語で書くこと(screen の範囲外と同じ)。
- 次の段: タグや分類ごとの記事の一覧。記事が少ないうちは、言語ごとの一覧1枚で足りる。
- サイトを組む道具は今の `scripts/pages.py` に足す。サイトを作る枠組み(静的サイトの生成器)は入れない。

## 要求

### 記事を書く

#### 記事は `docs/blog/<言語>/<名前>.md` に1記事1ファイルで置き、フロントマターに `title`・`date`・`description` を持つべき
`req~site.blog-post-file~1` P1
- Rationale: 記事の置き場を mdgrid で開き、題・日付・説明・下書きかどうかを表で並べて管理するため。名前は記事の URL になる
- Acceptance:
  - Given: `docs/blog/ja/` と `docs/blog/en/` の記事
  - When:
  - Then: どの記事のフロントマターにも、空でない `title`・`description` と、`YYYY-MM-DD` の形の `date` がある。
- Acceptance:
  - Given: `date` の無い記事
  - When: `scripts/pages.py site` でサイトを組む
  - Then: 記事のパスと足りない鍵の名前を出して、終了コード 0 以外で止まる。
- Acceptance:
  - Given: 名前が `My Post.md`(大文字と空白を含む)の記事
  - When: サイトを組む
  - Then: 名前は英小文字・数字・`-` だけ、という理由を出して止まる。
- Needs: utest

#### `draft: true` の記事は、サイトのどこにも出さないべき
`req~site.blog-draft~1` P1
- Rationale: 書きかけの記事をリポに置いたまま、公開は書き終えてからにするため
- Acceptance:
  - Given: `draft: true` の記事と、`draft` の無い記事
  - When: サイトを組む
  - Then: 下書きの記事は、記事のページ・一覧・フィード・サイトの地図のどこにも無く、もう一方は全部にある。
- Needs: utest

#### 記事の Markdown の見出し・段落・箇条書き・表・コードのブロック・引用・リンク・画像・強調・コードの字は、HTML の同じ働きの要素にするべき
`req~site.blog-markdown~1` P1
- Rationale: 記事を説明書と同じ Markdown で書き、手で HTML を書かずに済ませるため
- Acceptance:
  - Given: 本文に `## 見出し`・`- 項目`・2列の表・`yaml` のコードのブロックがある記事
  - When: サイトを組む
  - Then: 記事のページに `<h2>`・`<ul>`・`<table>`・`<pre><code>` がある。
- Acceptance:
  - Given: コードのブロックの中に `<script>` と書いた記事
  - When: サイトを組む
  - Then: 記事のページでは `&lt;script&gt;` の文字として出て、スクリプトとしては動かない。
- Acceptance:
  - Given: `![見本の表](images/tasks.svg)` を書いた記事と、`docs/blog/images/tasks.svg`
  - When: サイトを組む
  - Then: 記事のページの画像が読め、代わりの文字が「見本の表」。
- Needs: utest

### 一覧と記事のページ

#### 言語ごとに記事の一覧のページを作り、`date` の新しい順に、題・日付・説明と記事へのリンクを並べるべき
`req~site.blog-index~1` P1
- Rationale: 最近の記事から読めるようにするため
- Acceptance:
  - Given: `date` が 2026-11-01 と 2026-11-22 の日本語の記事
  - When: サイトを組む
  - Then: 日本語の一覧で 2026-11-22 の記事が先に並び、どちらにも題・日付・説明とその記事へのリンクがある。
- Acceptance:
  - Given: 英語の記事が無い(下書きだけを含む)
  - When: サイトを組む
  - Then: 英語の一覧は、まだ記事が無いことを1文で伝える。
- Needs: utest

#### 記事のページには、題・日付・本文と、記事の一覧とサイトの入口へのリンクを出すべき
`req~site.blog-post-page~1` P1
- Rationale: 記事から一覧へ戻り、mdgrid そのものの入口へも行けるようにするため
- Acceptance:
  - Given: `date: 2026-11-22` の記事
  - When: サイトを組む
  - Then: 記事のページに題と 2026-11-22 と本文があり、同じ言語の記事の一覧と入口へのリンクがある。
- Acceptance:
  - Given: `updated: 2026-12-10` も持つ記事
  - When: サイトを組む
  - Then: 記事のページに、書いた日と並べて更新した日 2026-12-10 が出る。
- Needs: utest

#### 同じ名前の記事がもう一方の言語にあれば、言語の切り替えはその記事を指し、無ければもう一方の言語の記事の一覧を指すべき
`req~site.blog-lang-switch~1` P1
- Rationale: 訳のある記事は訳へ、片方の言語だけの記事でも行き止まりにしないため
- Acceptance:
  - Given: `docs/blog/ja/a.md` と `docs/blog/en/a.md`
  - When: サイトを組む
  - Then: 日本語の a の切り替えは英語の a を指し、どちらも hreflang で互いを示す。
- Acceptance:
  - Given: 日本語にだけある記事 b
  - When: サイトを組む
  - Then: 日本語の b の切り替えは英語の記事の一覧を指し、英語の一覧に b は無い。
- Needs: utest

#### サイトの入口(日本語と英語)から、同じ言語の記事の一覧へ行けるべき
`req~site.blog-link~1` P1
- Rationale: 記事の置き場を、サイトに来た人が見つけられるようにするため
- Acceptance:
  - Given:
  - When: サイトを組む
  - Then: `ja/` の入口に日本語の記事の一覧へ、`en/` の入口に英語の記事の一覧へのリンクがある。
- Needs: utest

#### 言語を決めずに記事の一覧の URL を開いたときは、入口と同じく、覚えた言語かブラウザの言語で日本語か英語の一覧へ移るべき
`req~site.blog-root~1` P1
- Rationale: 言語を含まない短い URL を、記事の紹介に使えるようにするため
- Acceptance:
  - Given: 日本語のブラウザ
  - When: 言語を含まない記事の一覧の URL を開く
  - Then: 日本語の記事の一覧に移る。
- Acceptance:
  - Given: 前に入口で English を選んだブラウザ
  - When: 言語を含まない記事の一覧の URL を開く
  - Then: 英語の記事の一覧に移る。

#### 記事のページと一覧には、正規の URL・説明文・貼ったときのカードのタグを付け、サイトの地図に載せるべき
`req~site.blog-meta~1` P1
- Rationale: 記事を SNS やチャットに貼ったときに題と説明が出て、検索にも載るようにするため
- Acceptance:
  - Given: `title`・`description` を持つ記事
  - When: サイトを組む
  - Then: 記事のページの head に、その記事の URL の canonical と、`description` と同じ説明文と、og:title・og:url・og:image・twitter:card があり、サイトの地図にその URL がある。
- Needs: utest

#### 言語ごとに記事の Atom のフィードを出し、記事の一覧の head からそのフィードを示すべき
`req~site.blog-feed~1` P1
- Rationale: 新しい記事を、フィードの読み手で追えるようにするため
- Acceptance:
  - Given: 日本語の記事が2つ
  - When: サイトを組む
  - Then: 日本語のフィードが Atom の XML として読め、2つの entry が新しい順に、題・記事の URL・`date`(`updated` があればその日)・説明を持つ。
- Acceptance:
  - Given:
  - When: サイトを組む
  - Then: 日本語の記事の一覧の head に、日本語のフィードを指す `<link rel="alternate" type="application/atom+xml">` がある。
- Needs: utest

#### 公開の前のブラウザでの確かめ(`scripts/check_site.py`)は、記事の一覧・記事のページ・言語を決めて移るページも、ほかのページと同じ項目で確かめるべき
`req~site.blog-check~1` P1
- Rationale: 記事のリンク切れ・読めない画像・言語の切り替えの誤りを、公開する前に止めるため。言語を決めて移る動き(site.blog-root)はブラウザの中で動くので、ここで確かめる
- Acceptance:
  - Given: 記事の中に、行き先の無い相対のリンク
  - When: `scripts/check_site.py` でサイトを確かめる
  - Then: その記事のパスとリンクを挙げて、終了コード 1 で落ちる。
- Acceptance:
  - Given: 記事のページ
  - When: 幅 390px で開く
  - Then: 横にはみ出さない(表とコードのブロックは、その中だけで横に流れる)。

### 記事の文

#### 記事の文は `docs/writing.md` の決まりで書くべき
`req~site.blog-style~1` P1
- Rationale: 説明書と同じく、人が読んで温度のある文にし、AI の文の癖を抜くため(screen.docs-order-style と同じ決まり)
- Acceptance:
  - Given: `docs/blog/` の記事
  - When:
  - Then: 大げさな語(screen.docs-order-style の一覧)が無い。文の決まりに合うかは、`docs/writing.md` の一覧に照らして読んで判定。
- Needs: utest

#### 記事でほかの製品やサービスに触れるときは、事実(日付・範囲など)に出典のリンクを付け、その製品やサービスをけなさないべき
`req~site.blog-fair~1` P1
- Rationale: 記事の主張を、確かめられる事実の上に置くため。ほかの道具を下げて mdgrid を上げる書き方をしない(2026-11-22 に人が決めた)
- Acceptance:
  - Given: ほかのサービスの終了の日付を書いた記事
  - When:
  - Then: その日付の文の近くに、発表か報道へのリンクがある(読んで判定)。
- Acceptance:
  - Given: ほかの製品の名前を出した記事
  - When:
  - Then: その製品の欠点を並べる文が無く、mdgrid と同じファイルを開ける道具として紹介している(読んで判定)。

## 未確定事項

### 記事の URL の形
`tbd~site.blog-url`
- Decider: 人
- Provisional: 一覧は `ja/blog/`・`en/blog/`、記事は `ja/blog/<名前>.html`、フィードは `ja/blog/feed.xml`。言語を含まない `blog/` は、言語を決めて移るページにする(今の `gallery.html` と同じ形)
