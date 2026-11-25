//! セルの印と部品の見せ方の試験: 式の列の `#` は色を使わない表示だけ(cell-view.formula-mark-no-color)、部品の形で
//! 描くセルにも `!` を出す(cell-view.marks-on-parts)、札にするかは表の全部の行で決める(screen.rich-cells-table-wide)、
//! 重ねた窓の縁の外に札の丸い端だけを残さない(screen.window-chip-remnants)。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った。画面の文字とバッファのセルの色で確かめる。

use super::app::{App, ColorMode};
use super::test_screen::{app_of, ch, text, Tmp};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

/// 丸い札の左と右の端の字(Nerd Font)。
const LCAP: &str = "\u{e0b6}";
const RCAP: &str = "\u{e0b4}";

/// App の大きさで描いたバッファ。
fn buffer(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    t.backend().buffer().clone()
}

fn screen(a: &App) -> String {
    text(&buffer(a))
}

/// 画面の中で `name` を含む最初の行。
fn line_with(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("「{name}」の行が無い:\n{s}"))
        .to_string()
}

/// 設定の文字を読んで当てる(警告なし)。
fn configure(a: &mut App, conf: &str) {
    let (c, w) = mdgrid::config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
}

/// `.base`(中身 `yaml`)を `<一時>/notes/t.base` に置き、main と同じ道筋で色 `color` の表として開く。
fn open_base(tmp: &Tmp, yaml: &str, color: ColorMode) -> App {
    let p = tmp.notes().join("t.base");
    std::fs::write(&p, yaml).unwrap();
    let t = crate::open_target(&[p], None).unwrap();
    let mut a = App::new(Box::new(t.src), color);
    let (base, name, idx) = t.base.unwrap();
    a.set_base(base, name, idx.unwrap_or(0));
    a.resize(100, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

// ---- 式の列の `#`(cell-view.formula-mark-no-color) ----

const FORMULA: &str = "formulas:\n  double: 'rating * 2'\nviews:\n  - type: table\n    name: V\n    order: [rating, formula.double]\n";

/// rating が 3 の行(alpha)と 1 の行(beta)。
fn formula_notes(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    tmp.write("alpha.md", "---\nrating: 3\n---\n");
    tmp.write("beta.md", "---\nrating: 1\n---\n");
    tmp
}

#[test]
fn test_formula_cells_have_no_hash_in_color() {
    // [utest -> req~cell-view.formula-mark-no-color~1] 色のある表示では、式の列の見出しは「ƒ double」で、rating が 3 の行のセルは 6(`#` が無い)の薄い表示。
    let tmp = formula_notes("fm_color");
    let a = open_base(&tmp, FORMULA, ColorMode::Rgb);
    let b = buffer(&a);
    let s = text(&b);
    assert!(s.contains("ƒ double"), "{s}");
    let row = line_with(&s, "alpha");
    assert!(row.contains('6'), "{s}");
    assert!(!row.contains('#'), "色のある表示で # が付いた: {row}");
    let y = s.lines().position(|l| l.contains("alpha")).unwrap() as u16;
    let x = (0..b.area.width)
        .rev()
        .find(|&x| b[(x, y)].symbol() == "6")
        .expect("6 の字");
    assert!(
        b[(x, y)].modifier.contains(ratatui::style::Modifier::DIM),
        "薄い表示"
    );
}

#[test]
fn test_formula_cells_keep_hash_without_color() {
    // [utest -> req~cell-view.formula-mark-no-color~1] 色を使わない表示では、同じセルは #6。
    let tmp = formula_notes("fm_nocolor");
    let a = open_base(&tmp, FORMULA, ColorMode::None);
    let s = screen(&a);
    assert!(line_with(&s, "alpha").contains("#6"), "{s}");
}

#[cfg(unix)]
#[test]
fn test_read_only_note_cell_keeps_hash_in_color() {
    // [utest -> req~cell-view.formula-mark-no-color~1] 色のある表示でも、書けないノートの status のセルは今までどおり `#` が付く
    // (式の列でない読むだけのセルは変えない)。
    use std::os::unix::fs::PermissionsExt;
    let tmp = Tmp::new("fm_lock");
    tmp.write("alpha.md", "---\nstatus: todo\n---\n");
    tmp.write("beta.md", "---\nstatus: doing\n---\n");
    let p = tmp.notes().join("alpha.md");
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o444)).unwrap();
    let a = app_of(&tmp, ColorMode::Rgb);
    let s = screen(&a);
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(line_with(&s, "alpha").contains("#todo"), "{s}");
    assert!(!line_with(&s, "beta").contains('#'), "{s}");
}

// ---- 部品の形のセルの印(cell-view.marks-on-parts) ----

/// copy が kindle(候補に無い)の行と bought の行。
fn copy_notes(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    tmp.write("alpha.md", "---\ncopy: kindle\n---\n");
    tmp.write("beta.md", "---\ncopy: bought\n---\n");
    tmp.write("gamma.md", "---\ncopy: bought\n---\n");
    tmp
}

#[test]
fn test_marks_on_parts_status_forms_keep_misfit_mark() {
    // [utest -> req~cell-view.marks-on-parts~1] copy の候補に無い kindle のセルは、status の部品の形がどれでも `!` のあとに kindle の部品。
    // 候補にある bought のセルには `!` が無い。
    let tmp = copy_notes("mp_status");
    let mut fails = Vec::new();
    for form in [
        "dot", "shape", "text", "pill", "tint", "solid", "soft", "chip",
    ] {
        let mut a = app_of(&tmp, ColorMode::Rgb);
        configure(
            &mut a,
            &format!(
                "[terminal]\nnerd_font = true\n\n[look.columns]\ncopy = \"chip\"\n\n[look.style]\nstatus = \"{form}\"\n\n[rules.copy]\nvalues = [\"bought\", \"library\"]\n"
            ),
        );
        let s = screen(&a);
        let bad = line_with(&s, "alpha");
        match (bad.find('!'), bad.find("kindle")) {
            (Some(m), Some(k)) if m < k => {}
            _ => fails.push(format!("{form}: kindle の前に ! が無い: {bad:?}")),
        }
        let good = line_with(&s, "beta");
        if good.contains('!') {
            fails.push(format!("{form}: bought に ! が付いた: {good:?}"));
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

#[test]
fn test_marks_on_parts_tags_forms_keep_misfit_mark() {
    // [utest -> req~cell-view.marks-on-parts~1] tags の候補が a だけで tags が [a, z] の行は、tags の部品の形がどれでも `!` で始まる。
    let tmp = Tmp::new("mp_tags");
    tmp.write("alpha.md", "---\ntags: [a, z]\n---\n");
    tmp.write("beta.md", "---\ntags: [a]\n---\n");
    let mut fails = Vec::new();
    for form in [
        "dots", "hash", "brackets", "pill", "tint", "solid", "soft", "chip",
    ] {
        let mut a = app_of(&tmp, ColorMode::Rgb);
        configure(
            &mut a,
            &format!(
                "[terminal]\nnerd_font = true\n\n[look.style]\ntags = \"{form}\"\n\n[rules.tags]\nvalues = [\"a\"]\n"
            ),
        );
        let s = screen(&a);
        let bad = line_with(&s, "alpha");
        let cell = bad.split_once("alpha").map_or("", |(_, r)| r).trim_start();
        if !cell.starts_with('!') {
            fails.push(format!("{form}: tags のセルが ! で始まらない: {bad:?}"));
        }
        if line_with(&s, "beta").contains('!') {
            fails.push(format!("{form}: [a] に ! が付いた"));
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

// ---- 札にするかを表の全部の行で決める(screen.rich-cells-table-wide) ----

const TWO_VIEWS: &str = "views:\n  - type: table\n    name: 全部\n    order: [status]\n  - type: table\n    name: 残り\n    filters: 'status != \"todo\"'\n    order: [status]\n";

/// status が todo・todo・doing・done の4行。
fn status_notes(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: todo\n---\n");
    tmp.write("c.md", "---\nstatus: doing\n---\n");
    tmp.write("d.md", "---\nstatus: done\n---\n");
    tmp
}

#[test]
fn test_rich_cells_table_wide_same_in_every_view() {
    // [utest -> req~screen.rich-cells-table-wide~1] ビュー「残り」(status != todo。doing と done だけ)を開いても、status のセルは
    // ビュー「全部」と同じく札。
    let tmp = status_notes("tw_views");
    let mut a = open_base(&tmp, TWO_VIEWS, ColorMode::Rgb);
    assert!(a.is_select("status"), "全部のビューで札");
    a.select_view(1);
    a.refresh();
    assert_eq!(a.rows.len(), 2, "前提: 残りのビューは2行");
    assert!(a.is_select("status"), "残りのビューでも札");
    let s = screen(&a);
    let y = s.lines().position(|l| l.contains("doing")).unwrap() as u16;
    let b = buffer(&a);
    let x = (0..b.area.width)
        .find(|&x| b[(x, y)].symbol() == "d")
        .unwrap();
    let want = super::chips::parts(&a, &["doing".to_string()], false)
        .map(|p| p.items[0].iter().any(|(_, st)| st.is_some()));
    assert_eq!(want, Some(true), "前提: 札の形に色がある");
    assert_ne!(
        (b[(x, y)].fg, b[(x, y)].bg),
        (ratatui::style::Color::Reset, ratatui::style::Color::Reset),
        "doing のセルに札の色が無い:\n{s}"
    );
}

#[test]
fn test_rich_cells_table_wide_ignores_rows_outside_global_filter() {
    // [utest -> req~screen.rich-cells-table-wide~1] `.base` の全体の絞り込み file.inFolder("A") の中の status がくり返さず(doing・done)、
    // 外の行に todo がくり返す表では、status は文字のまま(全体の絞り込みの外の行は数えない)。
    let tmp = Tmp::new("tw_global");
    std::fs::create_dir_all(tmp.notes().join("A")).unwrap();
    std::fs::create_dir_all(tmp.notes().join("B")).unwrap();
    tmp.write("A/x.md", "---\nstatus: doing\n---\n");
    tmp.write("A/y.md", "---\nstatus: done\n---\n");
    tmp.write("B/p.md", "---\nstatus: todo\n---\n");
    tmp.write("B/q.md", "---\nstatus: todo\n---\n");
    let yaml = "filters:\n  and:\n    - file.inFolder(\"A\")\nviews:\n  - type: table\n    name: V\n    order: [status]\n";
    let a = open_base(&tmp, yaml, ColorMode::Rgb);
    assert_eq!(a.rows.len(), 2, "前提: A の2行");
    assert!(!a.is_select("status"));
}

#[test]
fn test_rich_cells_table_wide_look_columns_plain() {
    // [utest -> req~screen.rich-cells-table-wide~1] 同じ表に `[look.columns] status = "plain"` なら、どちらのビューでも文字のまま。
    let tmp = status_notes("tw_plain");
    let mut a = open_base(&tmp, TWO_VIEWS, ColorMode::Rgb);
    configure(&mut a, "[look.columns]\nstatus = \"plain\"\n");
    assert!(!a.is_select("status"));
    a.select_view(1);
    a.refresh();
    assert!(!a.is_select("status"));
}

// ---- 窓の縁の外の札の切れ端(screen.window-chip-remnants) ----

/// バッファの丸い端の字のうち、隣(左の端は右、右の端は左)の字の地が端の字の色でないもの(体の無い切れ端)の (x, y)。
/// 2つ目は、体の隣にある丸い端の字の数。
fn orphan_caps(b: &Buffer) -> (Vec<(u16, u16)>, usize) {
    let mut orphans = Vec::new();
    let mut kept = 0;
    for y in 0..b.area.height {
        for x in 0..b.area.width {
            let c = &b[(x, y)];
            let side = match c.symbol() {
                LCAP => x.checked_add(1).filter(|&n| n < b.area.width),
                RCAP => x.checked_sub(1),
                _ => continue,
            };
            let body = side.map(|n| b[(n, y)].bg);
            if body == Some(c.fg) {
                kept += 1;
            } else {
                orphans.push((x, y));
            }
        }
    }
    (orphans, kept)
}

#[test]
fn test_window_leaves_no_lone_chip_caps() {
    // [utest -> req~screen.window-chip-remnants~1] 丸い札の status の列の位置を1桁ずつずらした表で新しいノートの窓を開くと、
    // どの位置でも窓の縁の外に体の無い丸い端の字(右の端 U+E0B4・左の端 U+E0B6)が残らない。窓の外の札の丸い端は残る。
    let mut fails = Vec::new();
    for w in 1..=36 {
        let tmp = Tmp::new(&format!("wcr{w}"));
        let pad = "x".repeat(w);
        for i in 0..14 {
            tmp.write(
                &format!("n{i:02}.md"),
                &format!("---\nm1: {pad}\nm2: {pad}\nstatus: todo\n---\n"),
            );
        }
        let mut a = app_of(&tmp, ColorMode::Rgb);
        a.resize(120, 30);
        configure(
            &mut a,
            "[terminal]\nnerd_font = true\n\n[look.columns]\nstatus = \"chip\"\n\n[look.style]\nstatus = \"pill\"\n",
        );
        let (before, kept) = orphan_caps(&buffer(&a));
        assert!(before.is_empty(), "窓の前から切れ端: {before:?}");
        assert!(kept > 0, "前提: 丸い札がある");
        ch(&mut a, 'a');
        assert!(
            a.note.flow.is_some(),
            "新しいノートの窓が開かない: {:?}",
            a.message
        );
        let (after, kept) = orphan_caps(&buffer(&a));
        if !after.is_empty() {
            fails.push(format!("m の幅 {w}: 切れ端 {after:?}"));
        }
        if kept == 0 {
            fails.push(format!("m の幅 {w}: 窓の外の札の端まで消えた"));
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}
