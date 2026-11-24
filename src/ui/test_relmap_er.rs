//! 関係マップを表どうしの図として読む(relations.map-self-links・relations.map-column-types)。
//! 見本: tasks の routine が routines のノートを、parent が tasks のノートを指し、due が日付の列。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use super::*;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// tasks のフォルダを、色 `color` と設定 `toml` で開く(tasks・routines は登録した表)。
fn boot(name: &str, color: ColorMode, no_color: bool, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    let w = |rel: &str, body: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    };
    w("routines/weekly.md", "---\nevery: week\n---\n");
    w(
        "tasks/a.md",
        "---\ndue: 2026-11-01\nroutine: \"[[weekly]]\"\n---\n",
    );
    w(
        "tasks/b.md",
        "---\ndue: 2026-11-02\nroutine: \"[[weekly]]\"\nparent: \"[[a]]\"\n---\n",
    );
    w("tasks/c.md", "---\nparent: \"[[a]]\"\n---\n");
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    for (name, dir) in [("Tasks", "tasks"), ("Routines", "routines")] {
        places::save(
            &cfg,
            Place {
                name: name.into(),
                group: String::new(),
                path: n.join(dir).canonicalize().unwrap(),
                view: None,
            },
        )
        .unwrap();
    }
    let (config, warns) = mdgrid::config::parse(toml).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    let dir = n.join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), color);
    a.resize(160, 40);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn open_map(a: &mut App) {
    ch(a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
}

/// 選んでいるつながりの添字と、tasks → tasks のつながりの添字。
fn self_link(a: &App) -> usize {
    let rm = a.relmap.as_ref().expect("関係マップ");
    rm.links
        .iter()
        .position(|l| l.from == "Tasks" && l.to == "Tasks" && l.column == "parent")
        .expect("tasks.parent → tasks のつながり")
}

#[test]
fn test_self_link_arrow_selects_parent_many_to_one() {
    // [utest -> req~relations.map-self-links~1, req~relations.map-mouse-select-link~1] tasks の箱から tasks の箱へ戻る矢印をクリックすると、詳細に元の列 parent と多対一が出る。
    let (_t, mut a) = boot("er_self_click", ColorMode::None, false, "");
    open_map(&mut a);
    let k = self_link(&a);
    let rm = a.relmap.as_ref().unwrap();
    let (mx, my) = rm
        .map
        .cells
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.iter()
                .position(|c| c.role == mdgrid::relmap::Role::Arrow && c.link == Some(k))
                .map(|x| (x, y))
        })
        .unwrap_or_else(|| panic!("tasks → tasks の矢印が無い:\n{}", screen(&a)));
    // 一覧の窓は 0 から 26 桁、盤の窓の中は 27 桁目・2 行目から(盤は窓に収まるのでずらしは無い)。
    let top = super::bands::top_rows(&a) + super::bands::bar_padding(&a);
    a.click((26 + 1 + mx) as u16, (2 + top + my) as u16);
    assert_eq!(a.relmap.as_ref().unwrap().link, Some(k), "{}", screen(&a));
    // 詳細の窓(右の端): 題・種類・列。
    let s = screen(&a);
    assert!(s.contains("Tasks.parent → Tasks"), "{s}");
    assert!(
        s.lines()
            .any(|l| l.contains("種類") && l.contains("多対一")),
        "{s}"
    );
    assert!(
        s.lines().any(|l| l.contains("列") && l.contains("parent")),
        "{s}"
    );
}

#[test]
fn test_self_link_is_one_of_two_links() {
    // [utest -> req~relations.map-self-links~1] routine が routines を、parent が tasks を指すと、つながりは tasks → routines と tasks → tasks の2つ(狭い画面のつながりの一覧も2行)。
    let (_t, mut a) = boot("er_self_two", ColorMode::None, false, "");
    a.resize(70, 30);
    open_map(&mut a);
    let rm = a.relmap.as_ref().unwrap();
    let mut got: Vec<String> = rm.links.iter().map(mdgrid::relmap::link_text).collect();
    got.sort();
    assert_eq!(
        got,
        [
            "Tasks.parent → Tasks (N:1)",
            "Tasks.routine → Routines (N:1)"
        ]
    );
    let s = screen(&a);
    for want in &got {
        assert!(s.contains(want.as_str()), "{want}:\n{s}");
    }
}

#[test]
fn test_self_loop_readable_at_every_width() {
    // [utest -> req~relations.layout-readable~1, req~relations.map-self-links~1] 自分へ戻る矢印があっても、幅 160・100・60 で行は端末の幅に収まり、箱の題と列の文字は線で消えない。広い2つでは輪の矢印が見える。
    let (_t, mut a) = boot("er_self_widths", ColorMode::None, false, "");
    for w in [160u16, 100, 60] {
        a.resize(w, 30);
        open_map(&mut a);
        let s = screen(&a);
        for l in s.lines() {
            assert!(
                super::width::width(l) <= w as usize,
                "幅 {w} を超える行: {l}"
            );
        }
        if w >= 100 {
            assert!(s.contains("Tasks (3)"), "幅 {w}:\n{s}");
            assert!(s.contains("parent → Tasks"), "幅 {w}:\n{s}");
            assert!(s.contains("routine → Routines"), "幅 {w}:\n{s}");
            assert!(
                s.lines()
                    .any(|l| l.contains("Tasks (3)") && l.contains("│◀")),
                "題の行の右の縁に戻る矢印(幅 {w}):\n{s}"
            );
        } else {
            assert!(s.contains("Tasks.parent → Tasks (N:1)"), "幅 {w}:\n{s}");
        }
        ch(&mut a, 'R');
        assert_eq!(a.mode, Mode::Table);
    }
}

#[test]
fn test_column_marks_in_box_with_and_without_color() {
    // [utest -> req~relations.map-column-types~1] tasks の箱の due に日付の印(列の見出しと同じ ◷)、parent と routine にリンクの印。色を使う表示でも、色を使わない表示(--no-color)でも同じ印が字で出る。
    for (name, color, no_color) in [
        ("er_marks_rgb", ColorMode::Rgb, false),
        ("er_marks_none", ColorMode::Rgb, true),
    ] {
        let (_t, mut a) = boot(name, color, no_color, "");
        assert_eq!(a.color == ColorMode::None, no_color);
        open_map(&mut a);
        let s = screen(&a);
        for want in ["◷ due", "↪ parent → Tasks", "↪ routine → Routines"] {
            assert!(s.contains(want), "{name}: {want}\n{s}");
        }
        press_esc(&mut a);
    }
}

#[test]
fn test_column_marks_follow_nerd_icons() {
    // [utest -> req~relations.map-column-types~1, req~screen.nerd-icons-symbols~1] 設定 terminal.nerd_icons では、箱の印も見出しと同じアイコン用のフォントの記号(日付は nf-fa-calendar、リンクは nf-fa-link)。
    let (_t, mut a) = boot(
        "er_marks_nerd",
        ColorMode::Rgb,
        false,
        "[terminal]\nnerd_icons = true\n",
    );
    open_map(&mut a);
    let s = screen(&a);
    assert!(s.contains("\u{f073} due"), "{s}");
    assert!(s.contains("\u{f0c1} parent → Tasks"), "{s}");
    assert!(!s.contains("◷ due") && !s.contains("↪"), "{s}");
}

#[test]
fn test_column_marks_follow_types_setting_of_current_table() {
    // [utest -> req~relations.map-column-types~1] 今の表の列の印は、見出しと同じに設定の [types] で決めた型に従う(due を文字にすると日付の印は消える)。リンクの列は型を決めてもリンクの印のまま。
    let (_t, mut a) = boot("er_marks_types", ColorMode::None, false, "");
    a.prof
        .resolved
        .types
        .insert("due".into(), mdgrid::types::Kind::Text);
    a.prof
        .resolved
        .types
        .insert("parent".into(), mdgrid::types::Kind::Date);
    open_map(&mut a);
    let s = screen(&a);
    assert!(!s.contains("◷"), "{s}");
    assert!(s.contains("↪ parent → Tasks"), "{s}");
}

fn press_esc(a: &mut App) {
    super::test_screen::press(a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}
