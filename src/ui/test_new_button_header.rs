//! ビューの帯を出さないとき(タブも検索の欄も隠したとき)は、「+ 新規」をヘッダーの右の端のボタンの左に出す(CE-25)。
//! 記録: specs/_changes/2026-10-13-new-button-header.md(タスク #1)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const HIDDEN: &str = "[display]\ntabs = \"never\"\nsearch_bar = false\n";

fn boot(name: &str, toml: &str, readonly: bool, w: u16) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(w, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn test_ce_25_new_in_header_when_bar_hidden() {
    // [CE-25] 帯を出さない → ヘッダーの行の、画面の型のメニューのボタンの左に「a + 新規」。押すと名前の欄。
    let (_t, mut a) = boot("ce25h_hidden", HIDDEN, false, 100);
    let l = lines(&a);
    let s = l.join("\n");
    let head = super::bands::head_y(&a) as usize;
    let row = &l[head];
    let n = row
        .find("a + 新規")
        .unwrap_or_else(|| panic!("ヘッダーに「+ 新規」:\n{s}"));
    let m = row.find("▦ 表 ▾").unwrap_or_else(|| panic!("{s}"));
    assert!(n < m, "メニューのボタンの左:\n{s}");
    assert_eq!(s.matches("+ 新規").count(), 1, "1つだけ:\n{s}");
    let x = width::width(&row[..n]) + 2;
    a.click(x as u16, head as u16);
    assert!(
        a.note.flow.is_some(),
        "名前の欄が出る:\n{}",
        lines(&a).join("\n")
    );
}

#[test]
fn test_ce_25_new_in_bar_not_header_when_bar_shown() {
    // [CE-25] 帯を出すとき(既定)は今までどおり帯の右の端だけで、ヘッダーには出さない。
    let (_t, a) = boot("ce25h_shown", "", false, 100);
    let l = lines(&a);
    let head = super::bands::head_y(&a) as usize;
    assert!(!l[head].contains("+ 新規"), "{}", l.join("\n"));
    assert_eq!(l.join("\n").matches("+ 新規").count(), 1);
}

#[test]
fn test_ce_25_new_in_header_not_readonly() {
    // [CE-25] 読むだけの起動では出さない。
    let (_t, a) = boot("ce25h_ro", HIDDEN, true, 100);
    assert!(!lines(&a).join("\n").contains("+ 新規"));
}

#[test]
fn test_ce_25_new_in_header_dropped_when_narrow() {
    // [CE-25][SR-42] 狭くて入らなければ出さず、パンくずとメニューのボタンを残す。
    let (_t, a) = boot("ce25h_narrow", HIDDEN, false, 30);
    let l = lines(&a);
    let head = super::bands::head_y(&a) as usize;
    assert!(l[head].contains("▦ 表 ▾"), "{}", l.join("\n"));
    assert!(l[head].contains("notes"), "{}", l.join("\n"));
}

#[test]
fn test_ce_25_new_in_header_narrow_keeps_new_over_settings() {
    // [CE-25][SR-42] 設定のボタンまで入らない幅(80 未満)では設定を先に省き、「+ 新規」とメニューを残す。
    let (_t, a) = boot("ce25h_mid", HIDDEN, false, 60);
    let l = lines(&a);
    let head = super::bands::head_y(&a) as usize;
    assert!(l[head].contains("a + 新規"), "{}", l.join("\n"));
    assert!(l[head].contains("▦ 表 ▾"), "{}", l.join("\n"));
    assert!(!l[head].contains("設定"), "{}", l.join("\n"));
}

#[test]
fn test_ce_25_relmap_new_in_header_when_bar_hidden() {
    // [CE-25][REL-8] 帯を出さないとき、関係マップでも表の画面と同じくヘッダーの行の、画面の型のメニューのすぐ左に
    // 「+ 新規」(窓の縁には出さない。メニューの文字の幅が画面で違うので、桁はメニューに合わせて動く)。
    use super::test_relations::{boot_tasks, workspace};
    use super::test_screen::ch;
    let tmp = workspace("ce25h_rel");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    a.display.tabs = mdgrid::display::TabsMode::Never;
    a.display.search_bar = false;
    let head = super::bands::head_y(&a) as usize;
    ch(&mut a, 'R');
    let r = lines(&a);
    let s = r.join("\n");
    let n = r[head].find("a + 新規").unwrap_or_else(|| panic!("{s}"));
    let m = r[head]
        .find("◇ 関係マップ ▾")
        .unwrap_or_else(|| panic!("{s}"));
    assert!(
        n < m && r[head][n..m].trim_end().ends_with("新規"),
        "メニューのすぐ左:\n{s}"
    );
    assert_eq!(s.matches("+ 新規").count(), 1, "{s}");
}

#[test]
fn test_ce_25_relmap_header_new_click_opens_name() {
    // [CE-25] 帯を出さないとき、関係マップのヘッダーの「+ 新規」を押す → 選んでいる表で名前の欄が開く。
    use super::test_relations::{boot_tasks, workspace};
    use super::test_screen::ch;
    let tmp = workspace("ce25h_relclick");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    a.display.tabs = mdgrid::display::TabsMode::Never;
    a.display.search_bar = false;
    ch(&mut a, 'R');
    let head = super::bands::head_y(&a) as usize;
    let r = lines(&a);
    let n = r[head]
        .find("a + 新規")
        .unwrap_or_else(|| panic!("{}", r.join("\n")));
    let x = width::width(&r[head][..n]) + 2;
    a.click(x as u16, head as u16);
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert!(a.note.flow.is_some(), "名前の欄が出る");
}

#[test]
fn test_ce_25_new_in_header_tabs_auto_single_view() {
    // [CE-25][SR-34] tabs = "auto" でビューが1つ、検索の欄も隠す → 帯を出さないので、ヘッダーに「+ 新規」。
    let (_t, a) = boot(
        "ce25h_auto",
        "[display]\ntabs = \"auto\"\nsearch_bar = false\n",
        false,
        100,
    );
    let l = lines(&a);
    let head = super::bands::head_y(&a) as usize;
    assert!(l[head].contains("a + 新規"), "{}", l.join("\n"));
}
