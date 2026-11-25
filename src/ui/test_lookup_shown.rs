//! 式の列の、つながったノートから引いた値の見せ方(cell-view.lookup-shown)の試験。ワークスペースの印の `[table.base]` の式で
//! 台帳を開き、描いた画面の文字で確かめる。期待値は要求の受け入れの例から取った。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 本・抜き書き・人・会議の台帳(1つの表に全部の行が出る)。
const NOTES: &[(&str, &str)] = &[
    ("Library/霧の灯台守.md", "---\nkind: book\n---\n"),
    (
        "Quotes/抜き書き甲.md",
        "---\nsource: \"[[霧の灯台守]]\"\n---\n",
    ),
    ("People/安藤 真.md", "---\nrole: dev\n---\n"),
    (
        "Meetings/20261124_週次定例.md",
        "---\nwhen: 2026-11-24T10:00\nattendees:\n  - \"[[安藤 真]]\"\n---\n",
    ),
];

/// 印: ノートの名前の頭の日付を隠し、日付を YYYY/MM/DD で見せ、kind の book を「本」と呼ぶ。式の列 book・kind・ms・met。
const MARKER: &str = r#"[display]
note_name = '^\d{8}_'

[dates]
format = "YYYY/MM/DD"

[rules.kind]
values = ["book", "article"]
labels = { book = "本" }

[[table]]
name = "台帳"

[table.base.formulas]
book = 'source.asFile()'
kind = 'source.asFile().properties.kind'
ms = 'file.linkedFrom("attendees")'
met = 'file.linkedFrom("attendees").map(value.properties.when).max().date()'

[[table.base.views]]
type = "table"
name = "全部"
order = ["formula.book", "formula.kind", "formula.ms", "formula.met"]
"#;

fn boot(name: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (rel, t) in NOTES {
        let p = tmp.notes().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, t).unwrap();
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), MARKER).unwrap();
    let p = tmp.notes().join("台帳.base");
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(160, 24);
    a.start(Startup {
        config: mdgrid::config::Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: p.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn line_with(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("「{name}」の行が無い:\n{s}"))
        .to_string()
}

#[test]
fn test_lookup_shown_note_value_is_name() {
    // [utest -> req~cell-view.lookup-shown~1] 式 book = source.asFile() の列は、出典の本の名前「霧の灯台守」(Library/霧の灯台守 でない)
    let (_t, a) = boot("lookup_name");
    let s = screen(&a);
    let row = line_with(&s, "抜き書き甲");
    assert!(row.contains("霧の灯台守"), "{s}");
    assert!(!row.contains("Library/"), "{s}");
}

#[test]
fn test_lookup_shown_note_name_pattern() {
    // [utest -> req~cell-view.lookup-shown~1] note_name = '^\d{8}_' で、20261124_週次定例 だけが指す人の行の ms のセルは [週次定例]
    let (_t, a) = boot("lookup_pattern");
    let s = screen(&a);
    let row = line_with(&s, "安藤 真");
    assert!(row.contains("[週次定例]"), "{s}");
    assert!(!row.contains("20261124_"), "{s}");
}

#[test]
fn test_lookup_shown_value_label() {
    // [utest -> req~cell-view.lookup-shown~1] [rules.kind] values = ["book", "article"]・labels = { book = "本" } で、式 kind = source.asFile().properties.kind のセルは「本」
    let (_t, a) = boot("lookup_label");
    let s = screen(&a);
    let row = line_with(&s, "抜き書き甲");
    assert!(row.contains("本"), "{s}");
    assert!(!row.contains("book"), "{s}");
}

#[test]
fn test_lookup_shown_date_format() {
    // [utest -> req~cell-view.lookup-shown~1] [dates] format = "YYYY/MM/DD" で、一番遅い会議が 2026-11-24T10:00 の人の met のセルは 2026/11/24
    let (_t, a) = boot("lookup_date");
    let s = screen(&a);
    let row = line_with(&s, "安藤 真");
    assert!(row.contains("2026/11/24"), "{s}");
}
