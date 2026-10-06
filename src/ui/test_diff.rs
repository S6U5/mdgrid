use super::*;
use DiffLine::*;

fn s(x: &str) -> String {
    x.to_string()
}

#[test]
fn test_wb_9_line_diff_with_context() {
    // [WB-9] 値だけが変わった行を `-` / `+`、前後1行の文脈。離れた2か所は省略の印で区切る。
    let before = "---\ntitle: a\nstatus: todo\ndue: x\nk1: 1\nk2: 2\nk3: 3\nlast: z\n---\nbody\n";
    let after = "---\ntitle: a\nstatus: done\ndue: x\nk1: 1\nk2: 2\nk3: 3\nlast: y\n---\nbody\n";
    assert_eq!(
        diff(before.as_bytes(), after.as_bytes()),
        vec![
            Same(s("title: a")),
            Del(s("status: todo")),
            Add(s("status: done")),
            Same(s("due: x")),
            Gap,
            Same(s("k3: 3")),
            Del(s("last: z")),
            Add(s("last: y")),
            Same(s("---")),
        ]
    );
    // [WB-3] キーを足す1行だけ。CRLF は見せない。
    let before = "---\r\na: 1\r\n---\r\n";
    let after = "---\r\na: 1\r\nb: x\r\n---\r\n";
    assert_eq!(
        diff(before.as_bytes(), after.as_bytes()),
        vec![Same(s("a: 1")), Add(s("b: x")), Same(s("---"))]
    );
    assert!(diff(b"a\n", b"a\n").is_empty());
}
