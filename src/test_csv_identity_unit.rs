//! [SC-16][WB-4][WB-16] CSV の行の鍵は行の位置でなく、変わらない行の番号。外で行が足されたり消えたりしても、
//! 直しを別の行に書かない。specs/_changes/2026-10-10-csv-source.md。

use crate::source::csv::Csv;
use crate::source::{Edit, EditError, NewValue, RowId, SaveError, Source};

fn tmp(name: &str, text: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-csvid-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("t.csv");
    std::fs::write(&p, text).unwrap();
    p
}

fn v5(row: &RowId) -> Vec<(RowId, Vec<Edit>)> {
    vec![(
        row.clone(),
        vec![Edit {
            key: "v".into(),
            value: NewValue::Int(5),
        }],
    )]
}

/// 外で書き換え、更新時刻を進める(変化の検出に確実に見つけさせる)。
fn outside(p: &std::path::Path, text: &str) {
    let old = std::fs::metadata(p).unwrap().modified().unwrap();
    std::fs::write(p, text).unwrap();
    std::fs::File::options()
        .write(true)
        .open(p)
        .unwrap()
        .set_modified(old + std::time::Duration::from_secs(10))
        .unwrap();
}

#[test]
fn test_sc_16_row_inserted_outside_keeps_the_edited_row() {
    // 開いたあとに外で先頭に行が足される → B の直しは B に当たる(差分も、読み直して書くときも)。
    let p = tmp("ins", "id,v\nA,1\nB,2\n");
    let mut src = Csv::open(&p).unwrap();
    let b = src.rows()[1].clone();
    outside(&p, "id,v\nZ,0\nA,1\nB,2\n");
    let (_, after) = src.preview_unit(&v5(&b)).unwrap();
    assert_eq!(String::from_utf8(after).unwrap(), "id,v\nZ,0\nA,1\nB,5\n");
    // 外の変化を受けて読み直す(画面の poll と「外の変更の上に書く」と同じ)。B の鍵は変わらない。
    let changed = src.changed();
    assert!(!changed.is_empty());
    assert_eq!(src.label(&b), "B");
    let base = src.stamp(&b).unwrap();
    src.save_unit(&base, &v5(&b)).unwrap();
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "id,v\nZ,0\nA,1\nB,5\n"
    );
}

#[test]
fn test_sc_16_row_changed_outside_is_not_written() {
    // 直した行そのものが外で変わった(または消えた)→ 別の行には書かず、理由を出して止める。
    let p = tmp("chg", "id,v\nA,1\nB,2\nC,3\n");
    let mut src = Csv::open(&p).unwrap();
    let b = src.rows()[1].clone();
    outside(&p, "id,v\nA,1\nB,9\nC,3\n");
    assert!(matches!(
        src.preview_unit(&v5(&b)),
        Err(EditError::NotEditable(_))
    ));
    src.changed();
    let base = src.stamp(&src.rows()[0]).unwrap();
    assert!(matches!(
        src.save_unit(&base, &v5(&b)),
        Err(SaveError::Edit(EditError::NotEditable(_)))
    ));
    outside(&p, "id,v\nA,1\nC,3\n");
    src.changed();
    let base = src.stamp(&src.rows()[0]).unwrap();
    assert!(src.save_unit(&base, &v5(&b)).is_err());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "id,v\nA,1\nC,3\n");
}

#[test]
fn test_sc_16_row_keys_survive_own_writes() {
    // 自分で書いた(値の直し・末尾に足す)あとも、行の鍵は同じ行を指す。
    let p = tmp("own", "id,v\nA,1\nB,2\n");
    let mut src = Csv::open(&p).unwrap();
    let rows = src.rows();
    let base = src.stamp(&rows[0]).unwrap();
    src.save_unit(&base, &v5(&rows[0])).unwrap();
    let (added, _, _) = src.append_row().unwrap();
    assert_eq!(src.rows()[..2], rows[..]);
    assert_eq!(src.rows()[2], added);
    assert_eq!(src.label(&rows[1]), "B");
}
