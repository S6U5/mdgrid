//! 登録の保存で、places.toml の手書きの所を消さない(CLI-18)。

use super::*;

fn place(name: &str, group: &str, path: &str) -> Place {
    Place {
        name: name.into(),
        group: group.into(),
        path: PathBuf::from(path),
        view: None,
    }
}

const HAND: &str = "# よく使う表\n\n[[place]]\nname = \"Tasks\"\ngroup = \"Work\"\npath = \"~/notes/tasks\"   # 仕事\n\n# 趣味の表\n[[place]]\nname = \"Books\"\npath = \"~/notes/books\"\n";

#[test]
fn test_cli_18_add_keeps_hand_written_text() {
    // [CLI-18] 足すときは末尾に付け、コメント・空行・~ のパスはそのまま。読み直すと3つ。
    let out = upsert(HAND, &place("New", "", "/x/new"));
    assert!(out.starts_with(HAND), "{out}");
    assert!(
        out.ends_with("[[place]]\nname = \"New\"\npath = \"/x/new\"\n"),
        "{out}"
    );
    let (ps, w) = parse(&out);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(ps.len(), 3);
}

#[test]
fn test_cli_18_replace_only_that_block() {
    // [CLI-18] 同じ名前は、その区画の中身だけを置き換える。前置きのコメント「# 趣味の表」とほかの区画は残る。
    let out = upsert(HAND, &place("Tasks", "Job", "/x/t"));
    assert!(
        out.contains("# よく使う表\n") && out.contains("# 趣味の表\n[[place]]\nname = \"Books\""),
        "{out}"
    );
    assert!(out.contains("path = \"~/notes/books\""), "{out}");
    assert!(
        out.contains("group = \"Job\"\nname = \"Tasks\"\npath = \"/x/t\"\n"),
        "{out}"
    );
    assert!(
        !out.contains("# 仕事"),
        "置き換えた区画の中の行は新しい中身に: {out}"
    );
    let (ps, _) = parse(&out);
    assert_eq!(
        ps.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["Tasks", "Books"]
    );
}

#[test]
fn test_cli_18_broken_block_is_kept() {
    // [CLI-18] 読めない区画(path の無い行)は文字のまま残り、ほかの登録は効く。空のファイルにも足せる。
    let text = "[[place]]\nname = \"NoPath\"\n";
    let out = upsert(text, &place("A", "", "/a"));
    assert!(out.starts_with(text), "{out}");
    assert_eq!(parse(&out).0.len(), 1);
    assert_eq!(
        upsert("", &place("A", "", "/a")),
        "[[place]]\nname = \"A\"\npath = \"/a\"\n"
    );
}
