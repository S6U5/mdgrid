//! 画面の表の書き出し(OUT-2・OUT-5)。specs/_changes/2026-10-07-export-table.md。

use super::keymap::Mode;
use super::test_screen::{ch, col_named, press, typing, Tmp};
use super::*;
use mdgrid::print;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\nmemo: \"x\\ty\"\nn: 3\n---\n"),
    ("b.md", "---\nstatus: done\nmemo: m\nn: 1\n---\n"),
    ("c.md", "---\nstatus: todo\nmemo: \"l1\\nl2\"\nn: 2\n---\n"),
];

fn app(name: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    a.nv.target = tmp.notes().to_path_buf();
    (tmp, a)
}

fn out(tmp: &Tmp, name: &str) -> std::path::PathBuf {
    tmp.notes().parent().unwrap().join(name)
}

#[test]
fn test_out_2_export_shown_table() {
    let (tmp, mut a) = app("out2shown");
    // 画面で絞り、列を1つ隠す。
    a.filter = Some("todo".into());
    a.refresh();
    col_named(&mut a, "memo");
    ch(&mut a, '-');
    assert_eq!(a.rows.len(), 2);
    let csv = out(&tmp, "out.csv");
    a.export_table_to(csv.to_str().unwrap());
    let text = std::fs::read_to_string(&csv).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(lines[0].starts_with("path,"), "{text}");
    assert!(!lines[0].contains("memo"), "隠した列は出ない: {text}");
    assert!(
        lines[1].ends_with("a.md,todo,3") || lines[1].contains("a.md"),
        "{text}"
    );
    assert!(!text.contains("b.md"), "絞った行は出ない: {text}");
    assert!(
        a.message.as_deref().unwrap().contains("2"),
        "{:?}",
        a.message
    );
    // tsv: タブ区切りで、セルの中のタブと改行は空白。
    col_named(&mut a, "status");
    let tsv = out(&tmp, "out.TSV");
    a.export_table_to(tsv.to_str().unwrap());
    let text = std::fs::read_to_string(&tsv).unwrap();
    assert!(text.lines().next().unwrap().starts_with("path\t"), "{text}");
    // 拡張子の分からない名前は書かずに理由、入力は残る。
    a.start_export_table();
    typing(&mut a, out(&tmp, "out.txt").to_str().unwrap());
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette);
    assert!(!out(&tmp, "out.txt").exists());
    // ノートは書き換えない。
    assert_eq!(super::test_screen::read(&tmp, "a.md"), NOTES[0].1);
}

#[test]
fn test_out_5_export_matches_print_with_path() {
    // 絞りも並べ替えもしない既定の表では、`--print --with-path` と同じ中身(だから --apply で戻せる)。
    let (tmp, mut a) = app("out5same");
    let json = out(&tmp, "out.json");
    a.export_table_to(json.to_str().unwrap());
    let got = std::fs::read_to_string(&json).unwrap();
    let mut want = print::table(a.src.as_ref(), print::ViewDef::Default, a.today, 0).unwrap();
    want.columns.insert(
        0,
        mdgrid::base::Column {
            id: "path".into(),
            title: "path".into(),
        },
    );
    let args = [tmp.notes().to_path_buf()];
    for (cells, id) in want.rows.iter_mut().zip(&want.row_ids) {
        let p = crate::pick_path(&args, std::path::Path::new(&id.0));
        cells.insert(
            0,
            print::PrintCell::Prop(Some(mdgrid::frontmatter::Value::Str(p))),
        );
    }
    assert_eq!(got, print::render(&want, print::Format::Json));
}

#[test]
fn test_out_5_overwrite_asks_and_home() {
    let (tmp, mut a) = app("out5over");
    let csv = out(&tmp, "exists.csv");
    std::fs::write(&csv, "old").unwrap();
    a.export_table_to(csv.to_str().unwrap());
    assert_eq!(a.mode, Mode::Palette, "上書きの確かめ");
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(std::fs::read_to_string(&csv).unwrap(), "old");
    a.export_table_to(csv.to_str().unwrap());
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert!(std::fs::read_to_string(&csv).unwrap().starts_with("path,"));
    // `~/` はホームのフォルダ。
    let home = tmp.notes().parent().unwrap().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let saved = std::env::var_os("HOME");
    std::env::set_var("HOME", &home);
    a.export_table_to("~/h.md");
    match saved {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    assert!(home.join("h.md").exists(), "{:?}", a.message);
}
