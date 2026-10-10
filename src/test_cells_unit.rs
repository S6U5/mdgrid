//! セルの部品の設定(SR-35)。

use super::*;

fn parse(text: &str) -> (Cells, crate::style::Style, Vec<String>) {
    let (c, w) = crate::config::parse(text).unwrap();
    let r = c.resolved();
    (r.cells, r.style, w)
}

#[test]
fn test_sr_35_cells_string_and_table() {
    // [SR-35] look.cells は全体の切り替え。部品の種類ごとは [look.style] の文字のままの形、列ごとは [look.columns]。
    // 列の設定が部品の形より優先。
    let (c, s, w) = parse("");
    assert!(w.is_empty());
    assert!(c.on("x", Part::Checkbox, &s) && c.on("x", Part::Chips, &s));
    let (c, s, _) = parse("[look]\ncells = \"plain\"\n");
    assert!(!c.on("x", Part::Checkbox, &s) && !c.on("x", Part::Icons, &s));
    let (c, s, w) = parse(
        "[look.style]\ncheck = \"text\"\n[look.columns]\nstatus = \"plain\"\nowner = \"chip\"\ndone = \"rich\"\n",
    );
    assert!(w.is_empty(), "{w:?}");
    assert!(!c.on("x", Part::Checkbox, &s), "部品の形で切る");
    assert!(
        c.on("done", Part::Checkbox, &s),
        "列の rich が部品の形より優先"
    );
    assert!(c.on("x", Part::Chips, &s));
    assert!(!c.on("status", Part::Chips, &s) && !c.on("status", Part::Select, &s));
    assert!(c.forced_chip("owner") && c.on("owner", Part::Select, &s));
    let (c, s, _) = parse("[look]\ncells = \"plain\"\n[look.columns]\ntags = \"rich\"\n");
    assert!(!c.on("x", Part::Chips, &s) && c.on("tags", Part::Chips, &s));
    let (c, s, _) = parse("[look.style]\nlinks = \"plain\"\n");
    assert!(!c.on("x", Part::Links, &s) && c.on("x", Part::Chips, &s));
}

#[test]
fn test_sr_35_cells_bad_values_warn() {
    // [SR-35] 知らない値は警告して既定(rich)のまま。
    for text in [
        "[look]\ncells = \"fancy\"\n",
        "[look.style]\ncheck = \"yes\"\n",
        "[look]\nsparkles = true\n",
        "[look.columns]\nstatus = \"bold\"\n",
    ] {
        let (c, _, w) = parse(text);
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
