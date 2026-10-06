//! 4回目の点検の直し(specs/_changes/2026-10-06-audit4-first.md)。
//! [WB-20] `+++` の TOML のフロントマターのノートは読むだけ。[SR-3] 読み込みの途中で動かしていなければ、
//! 読み終えたとき先頭にいる。[NV-9] 1% 未満の割合は小数1桁。

use super::keymap::Mode;
use super::test_screen::{col_named, ctrl, press, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

const TOML: &str = "+++\ntitle = \"toml\"\n+++\nbody\n";

#[test]
fn test_wb_20_toml_frontmatter_is_read_only() {
    let tmp = Tmp::new("wb20");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", TOML);
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    let row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "b.md")
        .unwrap();
    let id = a.rows[row].clone();
    let lock = a.src.get(&id, "status").lock;
    assert!(
        lock.as_deref().is_some_and(|l| l.contains("YAML")),
        "{lock:?}"
    );
    // 選んで Enter しても入力は開かず、保存してもファイルは変わらない。
    a.row = row;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    ctrl(&mut a, 's');
    assert_eq!(read_file(&tmp, "b.md"), TOML);
}

fn read_file(tmp: &Tmp, name: &str) -> String {
    std::fs::read_to_string(tmp.notes().join(name)).unwrap()
}

#[test]
fn test_sr_3_load_keeps_top() {
    let tmp = Tmp::new("sr3top");
    // 下のフォルダから読まれても、名前の順で先頭が上に来る形。
    for d in ["z", "m", "a"] {
        std::fs::create_dir_all(tmp.notes().join(d)).unwrap();
        for i in 0..60 {
            tmp.write(&format!("{d}/n{i:03}.md"), &format!("---\nk: {i}\n---\n"));
        }
    }
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(7);
    }
    assert_eq!(a.rows.len(), 180);
    assert_eq!(a.row, 0, "先頭の行");
    assert_eq!(a.top, 0);
}

#[test]
fn test_nv_9_small_percent() {
    use super::freq::pct_text;
    assert_eq!(pct_text(81, 20000), "(0.4%)");
    assert_eq!(pct_text(1, 20000), "(<0.1%)");
    assert_eq!(pct_text(1, 100), "(1%)");
    assert_eq!(pct_text(3, 5), "(60%)");
    assert_eq!(pct_text(0, 5), "(0%)");
}

#[test]
fn test_wb_20_writeback_refuses_toml() {
    use mdgrid::source::NewValue;
    use mdgrid::writeback::{apply, Edit};
    let e = Edit {
        key: "status".into(),
        value: NewValue::Str("x".into()),
    };
    assert!(apply(TOML.as_bytes(), &[e]).is_err());
}
