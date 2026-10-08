//! [SR-31] セルの入力欄と新しいノートの窓の見た目。specs/_changes/2026-10-08-edit-screen-design.md。

use super::startup::Startup;
use super::test_screen::{buffer, ch, col_named, make, press, screen, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\ndue: 2026-10-15\nn: 1\n---\n"),
    ("b.md", "---\nstatus: done\nn: 2\n---\n"),
];

#[test]
fn test_sr_31_input_box_is_wide_and_filled() {
    let (_t, mut a) = make("sr31input", NOTES);
    a.color = ColorMode::Indexed;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    let b = super::input::input_box(&a, &view::layout(&a), &view::visible_layout(&a).1).unwrap();
    assert!(b.w >= 16, "16桁の幅: {}", b.w);
    let buf = buffer(&a);
    let cell = &buf[(b.x as u16, b.y as u16)];
    assert!(cell
        .modifier
        .contains(Modifier::BOLD | Modifier::UNDERLINED));
    assert_ne!(
        cell.bg,
        ratatui::style::Color::Reset,
        "色を使うときは地の色"
    );
}

/// main と同じ道筋(Startup)で開く(新しいノートの作る場所が決まる)。
fn boot(name: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: mdgrid::config::Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

#[test]
fn test_sr_31_note_form_box_and_hints() {
    let (_t, mut a) = boot("sr31form");
    ch(&mut a, 'a');
    let s = screen(&a);
    assert!(s.contains("╭─ 新しいノート"), "{s}");
    assert!(s.contains("に作る"), "{s}");
    assert!(s.contains("^S 作る"), "窓の中に押せるキー: {s}");
    // 値の無い数の欄に型の手がかり。
    assert!(
        s.lines().any(|l| l.contains("n ") && l.contains("数")),
        "{s}"
    );
    // あいまいな幅を2と数える設定では ASCII の枠。
    press(&mut a, KeyCode::Esc);
    a.ambiguous_wide = true;
    ch(&mut a, 'a');
    let s = screen(&a);
    assert!(s.contains("+- 新しいノート") && !s.contains('╭'), "{s}");
}
