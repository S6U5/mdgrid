//! サイトのブログ(site.blog-*)。`scripts/pages.py <出力> --blog-only --blog-src <記事の置き場>` で、ブログと言語の入口と
//! サイトの地図だけを組み(説明書と撮った画面は要らない)、作ったファイルを確かめる。記事は試験の中に直に書く。
//! Python 3.11 以上(pages.py が tomllib を使う)が無い環境では、理由を出して飛ばす(CI の test の job で回る)。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// 3.11 以上の Python の名前。`python3`・`python` を順に試す。
fn python() -> Option<&'static str> {
    ["python3", "python"].into_iter().find(|cand| {
        Command::new(cand)
            .args([
                "-c",
                "import sys, tomllib; sys.exit(0 if sys.version_info >= (3, 11) else 1)",
            ])
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

/// 試験ごとの作業のフォルダ(終わったら消す)。
struct Work(PathBuf);

impl Work {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-blog-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Work(dir)
    }
}

impl Drop for Work {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 組んだ結果。
struct Site {
    _work: Work,
    out: PathBuf,
    ok: bool,
    stderr: String,
}

impl Site {
    /// 出力のフォルダの中のファイルの中身(無ければ None)。
    fn file(&self, rel: &str) -> Option<String> {
        std::fs::read_to_string(self.out.join(rel)).ok()
    }

    fn page(&self, rel: &str) -> String {
        self.file(rel)
            .unwrap_or_else(|| panic!("{rel} が無い。stderr: {}", self.stderr))
    }
}

/// `posts` は記事の置き場の中の (道筋, 中身)。記事の置き場 `src` で組む。Python が無ければ None。
fn build_from(src: Option<&Path>, posts: &[(&str, &str)]) -> Option<Site> {
    let Some(py) = python() else {
        eprintln!("Python 3.11 以上が無いので飛ばす(CI の test の job で回る)");
        return None;
    };
    let work = Work::new();
    let blog = work.0.join("blog");
    for (rel, text) in posts {
        let p = blog.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
    }
    let out = work.0.join("site");
    let src = src.map_or(blog, Path::to_path_buf);
    let o = Command::new(py)
        .arg(root().join("scripts/pages.py"))
        .arg(&out)
        .arg("--blog-only")
        .arg("--blog-src")
        .arg(&src)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env_remove("GA_MEASUREMENT_ID")
        .env_remove("GOOGLE_SITE_VERIFICATION")
        .output()
        .unwrap();
    Some(Site {
        _work: work,
        out,
        ok: o.status.success(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    })
}

fn build(posts: &[(&str, &str)]) -> Option<Site> {
    build_from(None, posts)
}

/// 組めたことを確かめた結果。
fn built(posts: &[(&str, &str)]) -> Option<Site> {
    let s = build(posts)?;
    assert!(s.ok, "組めなかった: {}", s.stderr);
    Some(s)
}

/// フロントマターつきの記事。`extra` はフロントマターに足す行。
fn post(title: &str, date: &str, desc: &str, extra: &str, body: &str) -> String {
    format!("---\ntitle: {title}\ndate: {date}\ndescription: {desc}\n{extra}---\n\n{body}\n")
}

const SITE: &str = "https://s6u5.github.io/mdgrid/";

#[test]
fn test_blog_repo_posts_have_frontmatter() {
    // [utest -> req~site.blog-post-file~1] リポの記事(docs/blog/ja・en)は、どれも空でない title・description と YYYY-MM-DD の date を持ち、組める。
    let Some(s) = build_from(Some(&root().join("docs/blog")), &[]) else {
        return;
    };
    assert!(s.ok, "リポの記事で組めない: {}", s.stderr);
}

#[test]
fn test_blog_missing_date_stops() {
    // [utest -> req~site.blog-post-file~1] date の無い記事があると、記事のパスと足りない鍵の名前を出して、終了コード 0 以外で止まる。
    let Some(s) = build(&[(
        "ja/no-date.md",
        "---\ntitle: 日付の無い記事\ndescription: 説明\n---\n\n本文\n",
    )]) else {
        return;
    };
    assert!(!s.ok, "止まらなかった");
    assert!(s.stderr.contains("no-date.md"), "{}", s.stderr);
    assert!(s.stderr.contains("date"), "{}", s.stderr);
    assert!(
        !s.stderr.contains("title"),
        "足りているキーまで挙げた: {}",
        s.stderr
    );
}

#[test]
fn test_blog_bad_name_stops() {
    // [utest -> req~site.blog-post-file~1] 名前が My Post.md(大文字と空白)の記事があると、名前は英小文字・数字・- だけという理由を出して止まる。
    let Some(s) = build(&[(
        "ja/My Post.md",
        &post("題", "2026-11-22", "説明", "", "本文"),
    )]) else {
        return;
    };
    assert!(!s.ok, "止まらなかった");
    assert!(s.stderr.contains("My Post.md"), "{}", s.stderr);
    assert!(s.stderr.contains("英小文字・数字・-"), "{}", s.stderr);
}

#[test]
fn test_blog_draft_is_nowhere() {
    // [utest -> req~site.blog-draft~1] draft: true の記事は、記事のページ・一覧・フィード・サイトの地図のどこにも無く、もう一方は全部にある。
    let Some(s) = built(&[
        (
            "ja/secret-draft.md",
            &post("書きかけ", "2026-11-20", "下書き", "draft: true\n", "本文"),
        ),
        (
            "ja/shown.md",
            &post("公開する記事", "2026-11-21", "公開", "", "本文"),
        ),
    ]) else {
        return;
    };
    let index = s.page("ja/blog/index.html");
    let feed = s.page("ja/blog/feed.xml");
    let map = s.page("sitemap.xml");
    assert!(
        s.file("ja/blog/secret-draft.html").is_none(),
        "下書きのページがある"
    );
    for (what, text) in [("一覧", &index), ("フィード", &feed), ("地図", &map)] {
        assert!(!text.contains("secret-draft"), "{what}に下書きがある");
        assert!(!text.contains("書きかけ"), "{what}に下書きの題がある");
    }
    assert!(
        s.file("ja/blog/shown.html").is_some(),
        "公開する記事のページが無い"
    );
    assert!(index.contains("shown.html"), "一覧に公開する記事が無い");
    assert!(
        feed.contains("ja/blog/shown.html"),
        "フィードに公開する記事が無い"
    );
    assert!(
        map.contains(&format!("{SITE}ja/blog/shown.html")),
        "地図に公開する記事が無い"
    );
}

#[test]
fn test_blog_markdown_elements() {
    // [utest -> req~site.blog-markdown~1] 本文の ## 見出し・- 項目・2列の表・yaml のコードのブロックは、記事のページで h2・ul・table・pre と code になる。
    let body = "## 見出し\n\n- 項目\n- もう1つ\n\n| 鍵 | 意味 |\n|---|---|\n| due | 期限 |\n\n```yaml\nstatus: todo\n```";
    let Some(s) = built(&[("ja/md.md", &post("題", "2026-11-22", "説明", "", body))]) else {
        return;
    };
    let page = s.page("ja/blog/md.html");
    assert!(page.contains("<h2>見出し</h2>"), "{page}");
    assert!(page.contains("<ul>\n<li>項目</li>"), "{page}");
    assert!(page.contains("<table>"), "{page}");
    assert!(page.contains("<th>鍵</th>"), "{page}");
    assert!(page.contains("<td>期限</td>"), "{page}");
    assert!(
        page.contains("<pre><code class=\"language-yaml\">status: todo</code></pre>"),
        "{page}"
    );
}

#[test]
fn test_blog_markdown_code_is_text() {
    // [utest -> req~site.blog-markdown~1] コードのブロックの中の <script> は、記事のページで &lt;script&gt; の文字として出て、スクリプトの要素にならない。
    let body = "```html\n<script>alert(1)</script>\n```";
    let Some(s) = built(&[("ja/code.md", &post("題", "2026-11-22", "説明", "", body))]) else {
        return;
    };
    let page = s.page("ja/blog/code.html");
    assert!(
        page.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{page}"
    );
    assert!(!page.contains("<script>alert"), "{page}");
}

#[test]
fn test_blog_markdown_image() {
    // [utest -> req~site.blog-markdown~1] ![見本の表](images/tasks.svg) と docs/blog/images/tasks.svg があると、記事のページの画像が読め、代わりの文字が「見本の表」。
    let body = "![見本の表](images/tasks.svg)";
    let Some(s) = built(&[
        ("ja/img.md", &post("題", "2026-11-22", "説明", "", body)),
        (
            "images/tasks.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
        ),
    ]) else {
        return;
    };
    let page = s.page("ja/blog/img.html");
    assert!(
        page.contains("<img src=\"images/tasks.svg\" alt=\"見本の表\""),
        "{page}"
    );
    assert!(
        s.out.join("ja/blog/images/tasks.svg").is_file(),
        "画像の行き先が無い"
    );
}

#[test]
fn test_blog_index_newest_first() {
    // [utest -> req~site.blog-index~1] 日本語の一覧で 2026-11-22 の記事が 2026-11-01 の記事より先に並び、どちらにも題・日付・説明と記事へのリンクがある。
    let Some(s) = built(&[
        (
            "ja/old.md",
            &post("古い記事", "2026-11-01", "古い説明", "", "本文"),
        ),
        (
            "ja/new.md",
            &post("新しい記事", "2026-11-22", "新しい説明", "", "本文"),
        ),
    ]) else {
        return;
    };
    let index = s.page("ja/blog/index.html");
    let at = |w: &str| index.find(w).unwrap_or_else(|| panic!("一覧に {w} が無い"));
    assert!(at("新しい記事") < at("古い記事"), "新しい順でない");
    for w in [
        "href=\"new.html\"",
        "2026-11-22",
        "新しい説明",
        "href=\"old.html\"",
        "2026-11-01",
        "古い説明",
    ] {
        at(w);
    }
}

#[test]
fn test_blog_index_empty_language() {
    // [utest -> req~site.blog-index~1] 英語の記事が下書きだけなら、英語の一覧は記事を並べず、まだ記事が無いことを1文で伝える。
    let Some(s) = built(&[
        ("ja/a.md", &post("記事", "2026-11-22", "説明", "", "本文")),
        (
            "en/a.md",
            &post("Draft", "2026-11-22", "Draft", "draft: true\n", "Body"),
        ),
    ]) else {
        return;
    };
    let index = s.page("en/blog/index.html");
    assert!(index.contains("<p>No posts in English yet.</p>"), "{index}");
    assert!(!index.contains("class=\"posts\""), "{index}");
}

#[test]
fn test_blog_post_page() {
    // [utest -> req~site.blog-post-page~1] date: 2026-11-22 の記事のページに、題と 2026-11-22 と本文があり、同じ言語の一覧と入口へのリンクがある。
    let Some(s) = built(&[(
        "ja/p.md",
        &post("記事の題", "2026-11-22", "説明", "", "本文の段落。"),
    )]) else {
        return;
    };
    let page = s.page("ja/blog/p.html");
    assert!(page.contains("<h1>記事の題</h1>"), "{page}");
    assert!(
        page.contains("<time datetime=\"2026-11-22\">2026-11-22</time>"),
        "{page}"
    );
    assert!(page.contains("<p>本文の段落。</p>"), "{page}");
    assert!(page.contains("<a href=\"./\">記事の一覧へ</a>"), "{page}");
    assert!(
        page.contains("<a href=\"../\">mdgrid の入口へ</a>"),
        "{page}"
    );
}

#[test]
fn test_blog_post_page_updated() {
    // [utest -> req~site.blog-post-page~1] updated: 2026-12-10 も持つ記事のページには、書いた日と並べて更新した日 2026-12-10 が出る。
    let Some(s) = built(&[(
        "ja/u.md",
        &post("題", "2026-11-22", "説明", "updated: 2026-12-10\n", "本文"),
    )]) else {
        return;
    };
    let page = s.page("ja/blog/u.html");
    let wrote = page
        .find("<time datetime=\"2026-11-22\">")
        .expect("書いた日が無い");
    let updated = page
        .find("更新 <time datetime=\"2026-12-10\">2026-12-10</time>")
        .expect("更新した日が無い");
    assert!(wrote < updated, "書いた日と並んでいない");
}

#[test]
fn test_blog_lang_switch_to_translation() {
    // [utest -> req~site.blog-lang-switch~1] ja/a.md と en/a.md があると、日本語の a の切り替えは英語の a を指し、どちらも hreflang で互いを示す。
    let Some(s) = built(&[
        ("ja/a.md", &post("題", "2026-11-22", "説明", "", "本文")),
        ("en/a.md", &post("Title", "2026-11-22", "Desc", "", "Body")),
    ]) else {
        return;
    };
    let ja = s.page("ja/blog/a.html");
    let en = s.page("en/blog/a.html");
    assert!(
        ja.contains("href=\"../../en/blog/a.html\" hreflang=\"en\""),
        "{ja}"
    );
    assert!(
        en.contains("href=\"../../ja/blog/a.html\" hreflang=\"ja\""),
        "{en}"
    );
    for page in [&ja, &en] {
        for l in ["ja", "en"] {
            let want =
                format!("<link rel=\"alternate\" hreflang=\"{l}\" href=\"{SITE}{l}/blog/a.html\">");
            assert!(page.contains(&want), "{want}: {page}");
        }
    }
}

#[test]
fn test_blog_lang_switch_to_index() {
    // [utest -> req~site.blog-lang-switch~1] 日本語にだけある記事 b の切り替えは英語の記事の一覧を指し、英語の一覧に b は無い。
    let Some(s) = built(&[("ja/b.md", &post("題", "2026-11-22", "説明", "", "本文"))]) else {
        return;
    };
    let ja = s.page("ja/blog/b.html");
    assert!(
        ja.contains("href=\"../../en/blog/\" hreflang=\"en\""),
        "{ja}"
    );
    assert!(
        !ja.contains("hreflang=\"en\" href="),
        "訳の無い記事が hreflang を示す"
    );
    assert!(s.file("en/blog/b.html").is_none());
    assert!(!s.page("en/blog/index.html").contains("b.html"));
}

#[test]
fn test_blog_link_from_landing() {
    // [utest -> req~site.blog-link~1] 公開する記事が1つ以上あると、ja/ の入口に日本語の記事の一覧へ、en/ の入口に英語の記事の一覧へのリンクがある。
    let Some(s) = built(&[("ja/a.md", &post("題", "2026-11-22", "説明", "", "本文"))]) else {
        return;
    };
    assert!(
        s.page("ja/index.html")
            .contains("<a href=\"blog/\">ブログ</a>"),
        "日本語の入口にリンクが無い"
    );
    assert!(
        s.page("en/index.html")
            .contains("<a href=\"blog/\">Blog</a>"),
        "英語の入口にリンクが無い"
    );
}

#[test]
fn test_blog_hidden_when_only_drafts() {
    // [utest -> req~site.blog-hidden-when-empty~1] 下書きの記事だけなら、記事の一覧・言語を決めて移るページ・フィードが無く、入口にリンクが無く、地図にブログの URL が無い。
    let Some(s) = built(&[(
        "ja/d.md",
        &post("題", "2026-11-22", "説明", "draft: true\n", "本文"),
    )]) else {
        return;
    };
    for rel in [
        "ja/blog/index.html",
        "en/blog/index.html",
        "blog/index.html",
        "ja/blog/feed.xml",
        "en/blog/feed.xml",
    ] {
        assert!(s.file(rel).is_none(), "{rel} がある");
    }
    for l in ["ja", "en"] {
        let landing = s.page(&format!("{l}/index.html"));
        assert!(
            !landing.contains("href=\"blog/\""),
            "{l} の入口にリンクがある"
        );
    }
    assert!(
        !s.page("ja/index.html").contains("ブログ"),
        "日本語の入口にブログの字がある"
    );
    assert!(
        !s.page("sitemap.xml").contains("blog/"),
        "地図にブログがある"
    );
}

#[test]
fn test_blog_hidden_when_no_posts() {
    // [utest -> req~site.blog-hidden-when-empty~1] 記事が1つも無ければ、ブログのページもフィードも作らない。
    let Some(s) = built(&[]) else {
        return;
    };
    assert!(!s.out.join("ja/blog").exists());
    assert!(!s.out.join("blog").exists());
    assert!(!s.page("ja/index.html").contains("href=\"blog/\""));
}

#[test]
fn test_blog_shown_with_one_post() {
    // [utest -> req~site.blog-hidden-when-empty~1] 日本語の記事が1つだけ公開になると、日本語と英語の記事の一覧と、両方の入口のリンクが出る。
    let Some(s) = built(&[("ja/one.md", &post("題", "2026-11-22", "説明", "", "本文"))])
    else {
        return;
    };
    assert!(s.file("ja/blog/index.html").is_some());
    assert!(s.file("en/blog/index.html").is_some());
    assert!(s.page("ja/index.html").contains("href=\"blog/\""));
    assert!(s.page("en/index.html").contains("href=\"blog/\""));
}

#[test]
fn test_blog_meta_tags() {
    // [utest -> req~site.blog-meta~1] 記事のページの head に、その記事の URL の canonical・description と同じ説明文・og:title・og:url・og:image・twitter:card があり、地図にその URL がある。
    let Some(s) = built(&[(
        "ja/m.md",
        &post("貼る記事", "2026-11-22", "貼ったときの説明", "", "本文"),
    )]) else {
        return;
    };
    let page = s.page("ja/blog/m.html");
    let head = page.split("</head>").next().unwrap();
    let url = format!("{SITE}ja/blog/m.html");
    for want in [
        format!("<link rel=\"canonical\" href=\"{url}\">"),
        "<meta name=\"description\" content=\"貼ったときの説明\">".to_string(),
        "<meta property=\"og:title\" content=\"貼る記事 · mdgrid\">".to_string(),
        format!("<meta property=\"og:url\" content=\"{url}\">"),
        format!("<meta property=\"og:image\" content=\"{SITE}assets/ja/og.png\">"),
        "<meta name=\"twitter:card\" content=\"summary_large_image\">".to_string(),
    ] {
        assert!(head.contains(&want), "{want}: {head}");
    }
    assert!(
        s.page("sitemap.xml").contains(&format!("<loc>{url}</loc>")),
        "地図に記事が無い"
    );
}

#[test]
fn test_blog_feed_entries() {
    // [utest -> req~site.blog-feed~1] 日本語の記事が2つあると、日本語のフィードが Atom の XML として読め、2つの entry が新しい順に、題・記事の URL・日付(updated があればその日)・説明を持つ。
    let Some(s) = built(&[
        (
            "ja/first.md",
            &post(
                "一つ目",
                "2026-11-01",
                "一つ目の説明",
                "updated: 2026-11-25\n",
                "本文",
            ),
        ),
        (
            "ja/second.md",
            &post("二つ目", "2026-11-22", "二つ目の説明", "", "本文"),
        ),
    ]) else {
        return;
    };
    let feed_path = s.out.join("ja/blog/feed.xml");
    let parsed = Command::new(python().unwrap())
        .args([
            "-c",
            "import sys, xml.etree.ElementTree as E; r = E.parse(sys.argv[1]).getroot(); \
             sys.exit(0 if r.tag == '{http://www.w3.org/2005/Atom}feed' else 1)",
        ])
        .arg(&feed_path)
        .output()
        .unwrap();
    assert!(
        parsed.status.success(),
        "Atom の XML として読めない: {}",
        String::from_utf8_lossy(&parsed.stderr)
    );
    let feed = s.page("ja/blog/feed.xml");
    let entries: Vec<&str> = feed.split("<entry>").skip(1).collect();
    assert_eq!(entries.len(), 2, "{feed}");
    let (a, b) = (entries[0], entries[1]);
    assert!(
        a.contains("<title>二つ目</title>"),
        "新しい順でない: {feed}"
    );
    assert!(
        a.contains(&format!("<link href=\"{SITE}ja/blog/second.html\"/>")),
        "{a}"
    );
    assert!(a.contains("<updated>2026-11-22T00:00:00Z</updated>"), "{a}");
    assert!(a.contains("<summary>二つ目の説明</summary>"), "{a}");
    assert!(b.contains("<title>一つ目</title>"), "{b}");
    assert!(
        b.contains(&format!("<link href=\"{SITE}ja/blog/first.html\"/>")),
        "{b}"
    );
    assert!(
        b.contains("<updated>2026-11-25T00:00:00Z</updated>"),
        "updated の日でない: {b}"
    );
    assert!(b.contains("<summary>一つ目の説明</summary>"), "{b}");
}

#[test]
fn test_blog_index_points_to_feed() {
    // [utest -> req~site.blog-feed~1] 日本語の記事の一覧の head に、日本語のフィードを指す Atom の alternate のリンクがある。
    let Some(s) = built(&[("ja/a.md", &post("題", "2026-11-22", "説明", "", "本文"))]) else {
        return;
    };
    let index = s.page("ja/blog/index.html");
    let head = index.split("</head>").next().unwrap();
    assert!(
        head.contains(&format!(
            "<link rel=\"alternate\" type=\"application/atom+xml\" title=\"mdgrid のブログ\" href=\"{SITE}ja/blog/feed.xml\">"
        )),
        "{head}"
    );
}
