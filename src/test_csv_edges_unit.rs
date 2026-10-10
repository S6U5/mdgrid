//! [SC-15][SC-16] CSV の読み書きの端(照合で見つけた点)。specs/_changes/2026-10-10-csv-source.md。

use crate::csvfile::{encode, parse, set_many};
use crate::source::csv::Csv;
use crate::source::{Edit, NewValue, RowId, Source, Value};

fn tmp(name: &str, text: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("mdgrid-csv-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("t.csv");
    std::fs::write(&p, text).unwrap();
    p
}

fn edit(key: &str, v: NewValue) -> Edit {
    Edit {
        key: key.into(),
        value: v,
    }
}

fn save(src: &mut Csv, edits: &[(usize, Edit)]) {
    let rows = src.rows();
    let base = src.stamp(&rows[0]).unwrap();
    let edits: Vec<(RowId, Vec<Edit>)> = edits
        .iter()
        .map(|(i, e)| (rows[*i].clone(), vec![e.clone()]))
        .collect();
    src.save_unit(&base, &edits).unwrap();
}

#[test]
fn test_sc_16_single_column_clear_keeps_rows() {
    // [SC-16] 1列の表で値を空にしても行は消えず(`""`)、同じ保存の後ろの行への直しは、その行に書く。
    let p = tmp("one", "name\nx\ny\nz\nw\n");
    let mut src = Csv::open(&p).unwrap();
    save(
        &mut src,
        &[
            (1, edit("name", NewValue::Null)),
            (2, edit("name", NewValue::Str("Q".into()))),
        ],
    );
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "name\nx\n\"\"\nQ\nw\n"
    );
    assert_eq!(src.rows().len(), 4);
}

#[test]
fn test_sc_16_many_edits_use_original_positions() {
    // [SC-16] 長さの変わる直しを前の行に当てても、後ろの行の直しは正しい値の位置に入る。
    let f = parse(b"a,b\n1,2\n3,4\n", b',').unwrap();
    let out = set_many(
        b"a,b\n1,2\n3,4\n",
        &f,
        &[
            (0, 0, "長い値, 引用".into()),
            (1, 1, "Z".into()),
            (0, 1, "".into()),
        ],
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "a,b\n\"長い値, 引用\",\n3,Z\n"
    );
}

#[test]
fn test_sc_16_quote_only_when_needed() {
    // [SC-16] 引用符で囲むのは区切り・引用符・改行が入るときだけ(先頭・末尾の空白は囲まない)。
    assert_eq!(encode(" a ", b','), " a ");
    assert_eq!(encode("a\r\nb", b','), "\"a\r\nb\"");
}

#[test]
fn test_sc_15_lone_cr_refused() {
    // [SC-15] CR だけの改行のファイルは、読まずに理由(LF・CRLF でないと書き戻しで改行の形を守れない)。
    let e = parse(b"a,b\r1,2\r", b',').unwrap_err();
    assert!(e.contains("CR"), "{e}");
    // 引用符の中の CR は値。
    let f = parse(b"a\n\"x\ry\"\n", b',').unwrap();
    assert_eq!(f.records[0].fields[0].value, "x\ry");
}

#[test]
fn test_sc_15_duplicate_and_empty_headers() {
    // [SC-15] 同じ名前の列と名前の無い列も、別の列として読み書きする(取り違えない)。
    let p = tmp("dup", "a,a,\n1,2,3\n");
    let mut src = Csv::open(&p).unwrap();
    assert_eq!(src.columns(), ["a", "a#2", "#3"]);
    let r = src.rows()[0].clone();
    assert_eq!(src.get(&r, "a#2").value, Some(Value::Int(2)));
    assert_eq!(src.get(&r, "#3").value, Some(Value::Int(3)));
    save(&mut src, &[(0, edit("a#2", NewValue::Int(9)))]);
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "a,a,\n1,9,3\n");
}

#[test]
fn test_sc_16_float_stays_decimal() {
    // [SC-16] 小数の値は小数の形で書く(2.0 を 2 にしない)。
    let p = tmp("float", "x\n1.5\n");
    let mut src = Csv::open(&p).unwrap();
    save(&mut src, &[(0, edit("x", NewValue::Float(2.0)))]);
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "x\n2.0\n");
}

#[test]
fn test_sc_16_preview_against_changed_file_matches_write() {
    // [SC-16][WB-16] 外で行が足されたファイルにも、見せた差分のとおりに当てる(行を1度だけ読んだ位置で)。
    let p = tmp("prev", "id,v\nA,1\nB,2\n");
    let src = Csv::open(&p).unwrap();
    std::fs::write(&p, "id,v\nA,1\nB,2\nC,3\n").unwrap();
    let rows = src.rows();
    let (before, after) = src
        .preview_unit(&[(rows[1].clone(), vec![edit("v", NewValue::Int(5))])])
        .unwrap();
    assert_eq!(String::from_utf8(before).unwrap(), "id,v\nA,1\nB,2\nC,3\n");
    assert_eq!(String::from_utf8(after).unwrap(), "id,v\nA,1\nB,5\nC,3\n");
}
