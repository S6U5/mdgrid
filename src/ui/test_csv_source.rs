//! [SC-15][SC-16] CSV・TSV を表として開き、直した値だけを書き戻す。specs/_changes/2026-10-10-csv-source.md。

use super::app::App;
use super::keymap::Mode;
use super::test_screen::{ch, ctrl, edit_cell, press, screen, Tmp};
use super::ColorMode;
use mdgrid::source::csv::Csv;
use mdgrid::source::Value;
use ratatui::crossterm::event::KeyCode;

const LEDGER: &str =
    "\u{FEFF}name,qty,code,note\r\nりんご,3,007,\"赤い, 甘い\"\r\nみかん,12,010,\r\nbad,1\r\n";

fn open(name: &str, file: &str, text: &str) -> (Tmp, App) {
    let t = Tmp::new(name);
    t.write(file, text);
    let src = Csv::open(&t.notes().join(file)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    (t, a)
}

fn row_of(a: &App, label: &str) -> usize {
    a.rows.iter().position(|r| a.src.label(r) == label).unwrap()
}

fn disk(t: &Tmp, file: &str) -> String {
    std::fs::read_to_string(t.notes().join(file)).unwrap()
}

#[test]
fn test_sc_15_open_csv_as_table() {
    // [SC-15] 1行目が列、2行目からが行。値は文字のまま(先頭の 0 は消さない)、数は数。列の数が合わない行は理由つきで読むだけ。
    let (_t, a) = open("sc15", "台帳.csv", LEDGER);
    assert_eq!(a.src.columns(), ["name", "qty", "code", "note"]);
    assert_eq!(a.rows.len(), 3);
    let r = a.rows[row_of(&a, "りんご")].clone();
    assert_eq!(a.src.get(&r, "qty").value, Some(Value::Int(3)));
    assert_eq!(a.src.get(&r, "code").value, Some(Value::Str("007".into())));
    assert_eq!(
        a.src.get(&r, "note").value,
        Some(Value::Str("赤い, 甘い".into()))
    );
    let bad = a.rows[row_of(&a, "bad")].clone();
    let lock = a.src.get(&bad, "name").lock.unwrap_or_default();
    assert!(lock.contains('2') && lock.contains('4'), "{lock}");
    let s = screen(&a);
    assert!(s.contains("りんご") && s.contains("007"), "{s}");
}

#[test]
fn test_sc_15_unreadable_csv_reason() {
    // [SC-15] UTF-8 でない・空のファイルは開かず理由。
    let t = Tmp::new("sc15bad");
    std::fs::write(t.notes().join("sjis.csv"), b"name\n\x82\xa0\n").unwrap();
    t.write("empty.csv", "");
    let e = Csv::open(&t.notes().join("sjis.csv")).err().unwrap();
    assert!(e.contains("UTF-8"), "{e}");
    assert!(Csv::open(&t.notes().join("empty.csv")).is_err());
}

#[test]
fn test_sc_16_save_only_edited_cells() {
    // [SC-16] 2つの行の値を直して保存 → その値のバイトだけが変わり、BOM・CRLF・引用符・ほかの行はそのまま。
    let (t, mut a) = open("sc16", "台帳.csv", LEDGER);
    let apple = row_of(&a, "りんご");
    let orange = row_of(&a, "みかん");
    edit_cell(&mut a, apple, "qty", "5");
    edit_cell(&mut a, orange, "note", "箱, 2つ");
    assert_eq!(a.changes.count(), 2);
    assert_eq!(disk(&t, "台帳.csv"), LEDGER);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    // 1つのファイルなので、保存の確認は1つ(2つの行の直しを1つの差分で見せる)。
    let r = a.review.as_ref().unwrap();
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].label, "台帳.csv");
    let s = screen(&a);
    assert!(
        s.contains("+りんご,5,007") && s.contains("+みかん,12,010,\"箱, 2つ\""),
        "{s}"
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    assert_eq!(
        disk(&t, "台帳.csv"),
        "\u{FEFF}name,qty,code,note\r\nりんご,5,007,\"赤い, 甘い\"\r\nみかん,12,010,\"箱, 2つ\"\r\nbad,1\r\n"
    );
}

#[test]
fn test_sc_16_external_change_refused() {
    // [SC-16][WB-4] 開いたあとに外でファイルが変わったら、書かずに止める。
    let (t, mut a) = open("sc16ext", "a.tsv", "name\tqty\nx\t1\n");
    edit_cell(&mut a, 0, "qty", "2");
    let p = t.notes().join("a.tsv");
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    std::fs::write(&p, "name\tqty\nx\t9\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old + std::time::Duration::from_secs(10))
        .unwrap();
    let r = a.rows[0].clone();
    let base = a.src.stamp(&r).unwrap();
    let edits = [mdgrid::source::Edit {
        key: "qty".into(),
        value: mdgrid::source::NewValue::Str("2".into()),
    }];
    assert!(matches!(
        a.src.save(&r, &base, &edits),
        Err(mdgrid::source::SaveError::Changed)
    ));
    assert_eq!(disk(&t, "a.tsv"), "name\tqty\nx\t9\n");
}

#[test]
fn test_sc_16_external_change_stops_whole_file() {
    // [SC-16][WB-16] ためたまま外でファイルが変わる → 保存は止まり、ファイルの見出しに印。d で2行の直しをまとめて捨てる。
    let (t, mut a) = open("sc16stop", "a.csv", "name,qty\nx,1\ny,2\n");
    edit_cell(&mut a, 0, "qty", "5");
    edit_cell(&mut a, 1, "qty", "6");
    let p = t.notes().join("a.csv");
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    std::fs::write(&p, "name,qty\nx,1\ny,2\nz,3\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old + std::time::Duration::from_secs(10))
        .unwrap();
    a.poll();
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    assert!(a.review.as_ref().unwrap().items[0].external);
    assert_eq!(disk(&t, "a.csv"), "name,qty\nx,1\ny,2\nz,3\n");
    ch(&mut a, 'd');
    assert_eq!(a.changes.count(), 0);
}
