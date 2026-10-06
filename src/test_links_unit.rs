//! links の純関数(BV-22): リンクの拾い方と行き先の解き方。

use super::*;

fn targets(text: &str) -> Vec<(String, bool)> {
    extract(text)
        .into_iter()
        .map(|r| (r.target, r.md))
        .collect()
}

fn wiki(t: &str) -> (String, bool) {
    (t.to_string(), false)
}

fn md(t: &str) -> (String, bool) {
    (t.to_string(), true)
}

#[test]
fn test_bv_22_extract_wikilink_forms() {
    // [BV-22] [[名前]]・[[名前|表示]]・[[名前#見出し]]・![[名前]] の行き先は名前
    assert_eq!(
        targets("[[b]] [[c|表示]] [[d#見出し]] ![[e]] [[f#^blk|x]]"),
        vec![wiki("b"), wiki("c"), wiki("d"), wiki("e"), wiki("f")]
    );
}

#[test]
fn test_bv_22_extract_markdown_links() {
    // [BV-22] [文字](相対のパス.md)。外のアドレスと見出しだけのリンクは拾わない。%20 は空白に戻す
    assert_eq!(
        targets(
            "[x](sub/c.md) [y](https://e.com/a.md) [z](#h) [w](<my note.md>) [v](a%20b.md#h \"t\")"
        ),
        vec![md("sub/c.md"), md("my note.md"), md("a b.md")]
    );
}

#[test]
fn test_bv_22_extract_skips_code() {
    // [BV-22] コードの区画とインラインのコードの中は拾わない
    let text = "```\n[[x]]\n```\n`[[y]]` [[b]]\n~~~\n[z](z.md)\n~~~\n``[[q]]``\n";
    assert_eq!(targets(text), vec![wiki("b")]);
}

#[test]
fn test_bv_22_extract_ignores_empty_and_unclosed() {
    // [BV-22] [[]]・閉じない [[・自分の見出し [[#h]] は拾わない
    assert_eq!(targets("[[]] [[#h]] [[open"), Vec::<(String, bool)>::new());
}

fn index(notes: &[(&str, &str)]) -> Index {
    Index::build(
        notes
            .iter()
            .map(|(p, t)| (p.to_string(), extract(t)))
            .collect(),
    )
}

#[test]
fn test_bv_22_resolve_path_then_nearest_name() {
    // [BV-22] 根からのパス → 無ければ同じ名前のうち根に近い・パスの短いもの → 無ければ文字のまま
    let ix = index(&[
        ("a.md", "[[b]] [[x/deep/b]] [[無い]] [[img.png]]"),
        ("x/deep/b.md", ""),
        ("y/b.md", ""),
    ]);
    assert_eq!(ix.links("a.md"), vec!["y/b", "x/deep/b", "無い", "img.png"]);
}

#[test]
fn test_bv_22_resolve_markdown_link_relative_to_note() {
    // [BV-22] [文字](相対のパス.md) はノートのフォルダからの相対、無ければ根から
    let ix = index(&[
        ("p/a.md", "[x](c.md) [y](q/d.md) [z](../e.md)"),
        ("p/c.md", ""),
        ("q/d.md", ""),
        ("e.md", ""),
    ]);
    assert_eq!(ix.links("p/a.md"), vec!["p/c", "q/d", "e"]);
}

#[test]
fn test_bv_22_backlinks_and_has_link() {
    // [BV-22] 被リンクは行き先 → 元(パスの順、重ならない)。解けないリンクは被リンクに出ない
    let ix = index(&[
        ("a.md", "[[b]] [[b|again]] [[無い]]"),
        ("b.md", "[[c]]"),
        ("c.md", "[[b]]"),
    ]);
    assert_eq!(ix.backlinks("b.md"), vec!["a", "c"]);
    assert_eq!(ix.backlinks("a.md"), Vec::<String>::new());
    assert!(ix.has_link("a.md", "b"));
    assert!(ix.has_link("a.md", "b.md"));
    assert!(ix.has_link("a.md", "無い"));
    assert!(!ix.has_link("a.md", "c"));
    assert!(!ix.has_link("zzz.md", "b"));
}

#[test]
fn test_bv_22_has_link_to_non_note_file_by_path() {
    // [BV-22] .base のような .md でないファイル: 根からのパスか、ファイルの名前で書いたリンクに当たる
    let ix = index(&[
        ("a.md", "[[Projects.base]]"),
        ("b.md", "[[P/Projects.base]]"),
        ("c.md", "[[Projects]]"),
    ]);
    assert!(ix.has_link("a.md", "P/Projects.base"));
    assert!(ix.has_link("b.md", "P/Projects.base"));
    assert!(!ix.has_link("c.md", "P/Projects.base"));
}

#[test]
fn test_bv_22_links_toward_non_note_file() {
    // [BV-22] .base のようなノートでないファイルを名前かパスで指すリンクは、そのファイルのパスにそろえる
    let ix = index(&[
        ("a.md", "[[Projects.base]] [[Projects]] [[b]]"),
        ("b.md", ""),
    ]);
    assert_eq!(
        ix.links_toward("a.md", "P/Projects.base"),
        vec!["P/Projects.base", "Projects", "b"]
    );
}
