//! セルの部品の設定(SR-35)。

use super::*;

fn parse(text: &str) -> (Cells, Vec<String>) {
    let (c, w) = crate::config::parse(text).unwrap();
    (c.cells, w)
}

#[test]
fn test_sr_35_cells_string_and_table() {
    // [SR-35] cells は文字列か表。種類ごとの真偽と列ごとの見せ方。列の設定が種類の設定より優先。
    let (c, w) = parse("");
    assert!(w.is_empty());
    assert!(c.on("x", Part::Checkbox) && c.on("x", Part::Chips));
    let (c, _) = parse("cells = \"plain\"\n");
    assert!(!c.on("x", Part::Checkbox) && !c.on("x", Part::Icons));
    let (c, w) = parse(
        "[cells]\ncheckbox = false\n[cells.columns]\nstatus = \"plain\"\nowner = \"chip\"\ndone = \"rich\"\n",
    );
    assert!(w.is_empty(), "{w:?}");
    assert!(!c.on("x", Part::Checkbox), "種類ごとに切る");
    assert!(
        c.on("done", Part::Checkbox),
        "列の rich が種類の設定より優先"
    );
    assert!(c.on("x", Part::Chips));
    assert!(!c.on("status", Part::Chips) && !c.on("status", Part::Select));
    assert!(c.forced_chip("owner") && c.on("owner", Part::Select));
    let (c, _) = parse("[cells]\nstyle = \"plain\"\n[cells.columns]\ntags = \"rich\"\n");
    assert!(!c.on("x", Part::Chips) && c.on("tags", Part::Chips));
}

#[test]
fn test_sr_35_cells_bad_values_warn() {
    // [SR-35] 知らない値は警告して既定(rich)のまま。
    for text in [
        "cells = \"fancy\"\n",
        "[cells]\ncheckbox = \"yes\"\n",
        "[cells]\nsparkles = true\n",
        "[cells.columns]\nstatus = \"bold\"\n",
    ] {
        let (c, w) = parse(text);
        assert_eq!(w.len(), 1, "{text}: {w:?}");
        assert!(c.rich, "{text}");
    }
}

#[test]
fn test_sr_35_select_detection() {
    // [SR-35] 種類の少ない短い文字の列: くり返しがあり、違う値が少なく、短い。
    assert!(looks_like_select(
        ["todo", "doing", "todo", "done", "todo", "doing"].into_iter()
    ));
    assert!(
        !looks_like_select(["a", "b", "c"].into_iter()),
        "くり返しが無い"
    );
    assert!(!looks_like_select(["x"].into_iter()), "1行だけ");
    let long = "a very long sentence that is not a status";
    assert!(!looks_like_select([long, long, long].into_iter()));
    assert!(!looks_like_select(["a\nb", "a\nb"].into_iter()));
    let many: Vec<String> = (0..13)
        .flat_map(|i| [i.to_string(), i.to_string()])
        .collect();
    assert!(
        !looks_like_select(many.iter().map(String::as_str)),
        "違う値が多すぎる"
    );
}
