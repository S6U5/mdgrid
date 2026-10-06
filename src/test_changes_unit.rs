use super::*;
use crate::source::markdown::Markdown;
use std::path::PathBuf;
use std::time::Duration;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const R: &str = "---\nstatus: todo\ntitle: R\n---\nbody\n";

fn setup(name: &str) -> (Dir, Markdown, RowId) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-unit-changes-{}-{}-{}",
        name,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("r.md"), R).unwrap();
    let mut src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    while !src.load(100).done {}
    let row = src.rows().into_iter().next().unwrap();
    (Dir(dir), src, row)
}

fn edit_outside(dir: &Dir, bytes: &str) {
    let p = dir.0.join("r.md");
    let old = std::fs::metadata(&p).unwrap();
    std::fs::write(&p, bytes).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old.modified().unwrap() + Duration::from_secs(10))
        .unwrap();
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

#[test]
fn undo_keeps_external_mark() {
    // [WB-16] [WB-10] 外で変更 → undo → まだためた変更があれば印は残る。
    let (dir, mut src, r) = setup("ext");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    edit_outside(&dir, "---\nstatus: todo\ntitle: R outside\n---\nbody\n");
    let changed = src.changed();
    ch.note_external(&changed);
    assert!(ch.external(&r));
    assert!(ch.undo());
    assert!(ch.pending(&r, "status").is_some());
    assert!(ch.external(&r));
    assert!(ch.previews(&src)[0].as_ref().unwrap().external);
    assert!(ch.redo());
    assert!(ch.external(&r));
    // 全部取り消してやり直しても、印は付いている側に倒す
    assert!(ch.undo());
    assert!(ch.undo());
    assert!(!ch.external(&r));
    assert!(ch.redo());
    assert!(ch.external(&r));
}

#[test]
fn undo_keeps_overwrite_base() {
    // [WB-4] [WB-16] overwrite → undo → 基準は overwrite で取ったもの。保存は止まらない。
    let (dir, mut src, r) = setup("ow");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    let outside = "---\nstatus: todo\ntitle: R outside\n---\nbody\n";
    edit_outside(&dir, outside);
    let changed = src.changed();
    ch.note_external(&changed);
    ch.overwrite(&mut src, &r).unwrap();
    let base = src.stamp(&r);
    assert!(ch.undo());
    assert_eq!(ch.states.get(&r).and_then(|s| s.base), base);
    assert!(!ch.external(&r));
    // 全部取り消してやり直しても overwrite の基準
    assert!(ch.undo());
    assert!(ch.redo());
    assert_eq!(ch.states.get(&r).and_then(|s| s.base), base);
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Saved));
    let text = std::fs::read_to_string(dir.0.join("r.md")).unwrap();
    assert_eq!(text, outside.replace("status: todo", "status: done"));
}

#[test]
fn overwrite_seen_only_when_hash_matches() {
    // [WB-16] [WB-9] 見せた内容のハッシュと同じときだけ基準を進める。違えば基準も印も変えず、保存は止まる。
    let (dir, mut src, r) = setup("seen");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    let base = ch.states.get(&r).and_then(|s| s.base);
    let shown = "---\nstatus: todo\ntitle: R outside\n---\nbody\n";
    edit_outside(&dir, shown);
    let changed = src.changed();
    ch.note_external(&changed);
    let seen = crate::source::content_hash(shown.as_bytes());
    // 見せたあとに、また外で変わった。
    let again = "---\nstatus: todo\ntitle: R again\n---\nbody\n";
    edit_outside(&dir, again);
    assert!(!ch.overwrite_seen(&mut src, &r, seen).unwrap());
    assert!(ch.external(&r));
    assert_eq!(ch.states.get(&r).and_then(|s| s.base), base);
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Changed));
    assert_eq!(std::fs::read_to_string(dir.0.join("r.md")).unwrap(), again);
    // 見直した内容のハッシュなら進めて、書ける。
    let seen = crate::source::content_hash(again.as_bytes());
    assert!(ch.overwrite_seen(&mut src, &r, seen).unwrap());
    assert!(!ch.external(&r));
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Saved));
    assert_eq!(
        std::fs::read_to_string(dir.0.join("r.md")).unwrap(),
        again.replace("status: todo", "status: done")
    );
}

#[test]
fn save_rows_writes_only_given_rows() {
    // [WB-9] [WB-14] 指定の行だけを書き、ほかの行のためた変更は残す。
    let (dir, _first, r) = setup("rows");
    std::fs::write(dir.0.join("q.md"), R).unwrap();
    let mut src = Markdown::open(std::slice::from_ref(&dir.0)).unwrap();
    while !src.load(100).done {}
    let q = src
        .rows()
        .into_iter()
        .find(|x| x.0.ends_with("q.md"))
        .unwrap();
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    ch.set(&src, &q, "status", s("done")).unwrap();
    let res = ch.save_rows(&mut src, &[q.clone(), q.clone()]);
    assert_eq!(res.len(), 1);
    assert!(matches!(res[0].1, Outcome::Saved));
    assert_eq!(ch.count(), 1);
    assert!(ch.pending(&r, "status").is_some());
    assert_eq!(std::fs::read_to_string(dir.0.join("r.md")).unwrap(), R);
    assert_eq!(
        std::fs::read_to_string(dir.0.join("q.md")).unwrap(),
        R.replace("todo", "done")
    );
    // ためた変更の無い行は何もしない。
    assert!(ch.save_rows(&mut src, &[q]).is_empty());
}

#[test]
fn same_value_compares_types() {
    // [WB-17] 型つきで比べる。Int・Float は数として同じ。数と文字、Null と空の文字列は違う。
    assert!(same_value(&NewValue::Null, &Value::Null));
    assert!(same_value(&s("a"), &Value::Str("a".into())));
    assert!(!same_value(&s("a "), &Value::Str("a".into())));
    assert!(same_value(&NewValue::Bool(true), &Value::Bool(true)));
    assert!(!same_value(&NewValue::Bool(true), &Value::Bool(false)));
    assert!(same_value(&NewValue::Int(3), &Value::Int(3)));
    assert!(same_value(&NewValue::Int(3), &Value::Float(3.0)));
    assert!(same_value(&NewValue::Float(3.0), &Value::Int(3)));
    assert!(same_value(&NewValue::Float(0.5), &Value::Float(0.5)));
    assert!(!same_value(
        &NewValue::Float(f64::NAN),
        &Value::Float(f64::NAN)
    ));
    assert!(!same_value(&NewValue::Int(3), &Value::Str("3".into())));
    assert!(!same_value(&s("3"), &Value::Int(3)));
    assert!(!same_value(&NewValue::Null, &Value::Str(String::new())));
    assert!(!same_value(&s(""), &Value::Null));
    assert!(!same_value(&s("true"), &Value::Bool(true)));
    assert!(!same_value(&NewValue::Null, &Value::Other));
    assert!(!same_value(&NewValue::Null, &Value::List(vec![])));
}

#[test]
fn same_value_date_compares_with_read_string() {
    // [WB-17][WB-18] 日付は読んだ文字列と比べる(素の日付も Str で読まれる)。
    let d = |v: &str| NewValue::Date(v.into());
    assert!(same_value(
        &d("2026-10-05"),
        &Value::Str("2026-10-05".into())
    ));
    assert!(!same_value(
        &d("2026-10-05"),
        &Value::Str("2026-10-06".into())
    ));
    assert!(!same_value(&d("2026-10-05"), &Value::Null));
    assert!(!same_as_read(&d("2026-10-05"), &None));
}

#[test]
fn same_value_compares_lists_by_strings() {
    // [WB-17] [CE-19] List は要素の文字列の並びで比べる。空の並びは Null・空の文字列と同じ(書いても `key:`
    // にしかならないか、書くと `key: ""` が `key:` に変わるので書かない)。
    let list = |items: &[&str]| NewValue::List(items.iter().map(|s| s.to_string()).collect());
    let strs =
        |items: &[&str]| Value::List(items.iter().map(|s| Value::Str(s.to_string())).collect());
    assert!(same_value(&list(&["a", "b"]), &strs(&["a", "b"])));
    assert!(!same_value(&list(&["b", "a"]), &strs(&["a", "b"])));
    assert!(!same_value(&list(&["a"]), &strs(&["a", "b"])));
    assert!(!same_value(
        &list(&["1"]),
        &Value::List(vec![Value::Int(1)])
    ));
    assert!(same_value(&list(&[]), &Value::List(vec![])));
    assert!(same_value(&list(&[]), &Value::Null));
    assert!(same_value(&list(&[]), &Value::Str(String::new())));
    assert!(!same_value(&list(&["a"]), &Value::Null));
    assert!(!same_value(&list(&["a"]), &Value::Str(String::new())));
    assert!(!same_value(&list(&[]), &Value::Str("a".into())));
    assert!(same_as_read(&list(&[]), &None));
    assert!(!same_as_read(&list(&["a"]), &None));
}

#[test]
fn int_float_compare_without_rounding() {
    // [WB-17] 2^53 を超える整数は f64 に丸めて比べない。小数が整数値で i64 の範囲の中のときだけ同じ。
    let big = 1i64 << 53;
    assert!(same_value(&NewValue::Int(big), &Value::Float(big as f64)));
    let (over, near) = (big + 1, big as f64);
    assert!(!same_value(&NewValue::Int(over), &Value::Float(near)));
    assert!(!same_value(&NewValue::Float(near), &Value::Int(over)));
    assert!(!same_value(
        &NewValue::Int(i64::MAX),
        &Value::Float(i64::MAX as f64)
    ));
    assert!(same_value(
        &NewValue::Int(i64::MIN),
        &Value::Float(i64::MIN as f64)
    ));
    assert!(!same_value(&NewValue::Float(1e300), &Value::Int(i64::MAX)));
    assert!(!same_value(&NewValue::Float(-1e300), &Value::Int(i64::MIN)));
    assert!(!same_value(&NewValue::Float(3.5), &Value::Int(3)));
    let inf = f64::INFINITY;
    assert!(!same_value(&NewValue::Float(inf), &Value::Int(i64::MAX)));
    assert!(!same_value(&NewValue::Float(f64::NAN), &Value::Int(0)));
    assert!(same_value(&NewValue::Float(-0.0), &Value::Int(0)));
}

#[test]
fn external_change_to_same_value_drops_pending() {
    // [WB-17] [WB-16] ためた値と同じ値に外で直された → 読み直したあとは外れ(印も外れ)、保存で何も書かない。undo で戻る。
    let (dir, mut src, r) = setup("extsame");
    let file = || std::fs::read_to_string(dir.0.join("r.md")).unwrap();
    let first = "---\ndue: 2026-09-30\ntitle: D\n---\nbody\n";
    edit_outside(&dir, first);
    src.changed();
    let mut ch = Changes::new();
    ch.set(&src, &r, "due", s("2026-10-05")).unwrap();
    assert_eq!(ch.count(), 1);

    let outside = "---\ndue: 2026-10-05\ntitle: DX\n---\nbody\n";
    edit_outside(&dir, outside);
    let changed = src.changed();
    assert!(changed.contains(&r));
    ch.note_external(&changed);
    ch.drop_same(&src, &changed);
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&r, "due").is_none());
    assert!(!ch.external(&r));
    assert!(ch.save(&mut src).is_empty());
    assert_eq!(file(), outside);

    // 外す操作も1手: undo で戻り、印と基準も戻る(保存は止まる)。
    assert!(ch.undo());
    assert_eq!(ch.count(), 1);
    assert!(ch.external(&r));
    assert!(matches!(ch.pending(&r, "due"), Some(NewValue::Str(v)) if v == "2026-10-05"));

    // 「外の変更の上に書く」で読み直したときも外す。書くものが無いので true で、保存しても何も書かない。
    assert!(ch.overwrite(&mut src, &r).is_ok());
    assert_eq!(ch.count(), 0);
    assert!(!ch.external(&r));
    assert!(ch.save(&mut src).is_empty());
    assert_eq!(file(), outside);
    assert!(ch.undo());
    assert_eq!(ch.count(), 1);
}

#[test]
fn overwrite_seen_drops_same_and_keeps_other_cells() {
    // [WB-17] [WB-16] 見せた内容の上に書くとき、読み直した値と同じセルは外し、違うセルだけを書く(クオートを付けない)。
    let (dir, mut src, r) = setup("owsame");
    let first = "---\ndue: 2026-09-30\ntitle: D\n---\nbody\n";
    edit_outside(&dir, first);
    src.changed();
    let mut ch = Changes::new();
    ch.set(&src, &r, "due", s("2026-10-05")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    let outside = "---\ndue: 2026-10-05\ntitle: DX\n---\nbody\n";
    edit_outside(&dir, outside);
    let changed = src.changed();
    ch.note_external(&changed);
    let seen = crate::source::content_hash(outside.as_bytes());
    assert!(ch.overwrite_seen(&mut src, &r, seen).unwrap());
    assert_eq!(ch.count(), 1);
    assert!(ch.pending(&r, "due").is_none());
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Saved));
    assert_eq!(
        std::fs::read_to_string(dir.0.join("r.md")).unwrap(),
        "---\ndue: 2026-10-05\ntitle: T\n---\nbody\n"
    );
}

#[test]
fn set_each_applies_per_row_values_as_one_step() {
    // [WB-17] [CE-4] [CE-10] 行ごとの値(None = 外す)を1手で当てる。重ねた行は最初だけ。外すものもためるものも無ければ手を積まない。
    let (dir, _first, r) = setup("each");
    std::fs::write(dir.0.join("q.md"), R).unwrap();
    let mut src = Markdown::open(std::slice::from_ref(&dir.0)).unwrap();
    while !src.load(100).done {}
    let q = src
        .rows()
        .into_iter()
        .find(|x| x.0.ends_with("q.md"))
        .unwrap();
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    let items = [
        (r.clone(), None),
        (r.clone(), Some(s("x"))),
        (q.clone(), Some(s("doing"))),
    ];
    assert!(ch.set_each(&src, "status", &items).is_empty());
    assert!(ch.pending(&r, "status").is_none());
    assert!(ch.pending(&r, "title").is_some());
    assert!(matches!(ch.pending(&q, "status"), Some(NewValue::Str(v)) if v == "doing"));
    assert!(ch.undo());
    assert!(matches!(ch.pending(&r, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert!(ch.pending(&q, "status").is_none());
    assert!(ch.redo());
    // 何も変えない(外すものが無い・元と同じ値)なら手を積まない。
    let items = [(r.clone(), None), (q.clone(), Some(s("doing")))];
    assert!(ch.set_each(&src, "status", &items).is_empty());
    // 次の undo は前の set_each の手を戻す(何も変えない set_each は手を積んでいない)。
    assert!(ch.undo());
    assert!(matches!(ch.pending(&r, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert!(ch.pending(&q, "status").is_none());
}

#[test]
fn drop_same_keeps_redo() {
    // [WB-17] [WB-10] 外の変化で積む手(drop_same)は、やり直しの積みを消さない。
    let (dir, mut src, r) = setup("keepredo");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    assert!(ch.undo());
    // title の T は取り消してやり直しに積んだまま、外で status を done に直す。
    edit_outside(&dir, "---\nstatus: done\ntitle: R\n---\nbody\n");
    let changed = src.changed();
    ch.note_external(&changed);
    ch.drop_same(&src, &changed);
    assert_eq!(ch.count(), 0);
    assert!(ch.redo(), "[WB-17] drop_same がやり直しを消した");
    assert!(matches!(ch.pending(&r, "title"), Some(NewValue::Str(v)) if v == "T"));
    // 利用者の操作(set)はやり直しを消す。
    assert!(ch.undo());
    ch.set(&src, &r, "status", s("x")).unwrap();
    assert!(!ch.redo());
}

#[test]
fn undo_after_overwrite_does_not_requote_same_value() {
    // [WB-17] overwrite で外れた同じ値のセルが undo で戻っても、保存では書かない(クオートだけが変わらない)。
    let (dir, mut src, r) = setup("owundo");
    edit_outside(&dir, "---\ndue: 2026-09-30\ntitle: D\n---\nbody\n");
    src.changed();
    let mut ch = Changes::new();
    ch.set(&src, &r, "due", s("2026-10-05")).unwrap();
    ch.set(&src, &r, "title", s("T")).unwrap();
    edit_outside(&dir, "---\ndue: 2026-10-05\ntitle: DX\n---\nbody\n");
    let changed = src.changed();
    ch.note_external(&changed);
    ch.overwrite(&mut src, &r).unwrap();
    assert!(ch.pending(&r, "due").is_none());
    assert!(ch.undo());
    assert!(ch.pending(&r, "due").is_some());
    // 差分にも due は出ない。
    let p = ch.previews(&src);
    let after = String::from_utf8(p[0].as_ref().unwrap().after.clone()).unwrap();
    assert_eq!(after, "---\ndue: 2026-10-05\ntitle: T\n---\nbody\n");
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Saved));
    assert_eq!(
        std::fs::read_to_string(dir.0.join("r.md")).unwrap(),
        "---\ndue: 2026-10-05\ntitle: T\n---\nbody\n"
    );
    assert_eq!(ch.count(), 0);
}

#[test]
fn save_drops_row_whose_cells_all_match_read() {
    // [WB-17] ためたセルが全部読んだ値と同じ行は、書かずにためる変更から外す。差分にも出さない。
    let (dir, mut src, r) = setup("allsame");
    edit_outside(&dir, "---\ndue: 2026-09-30\ntitle: D\n---\nbody\n");
    src.changed();
    let mut ch = Changes::new();
    ch.set(&src, &r, "due", s("2026-10-05")).unwrap();
    let outside = "---\ndue: 2026-10-05\ntitle: DX\n---\nbody\n";
    edit_outside(&dir, outside);
    let changed = src.changed();
    ch.note_external(&changed);
    ch.overwrite(&mut src, &r).unwrap();
    assert!(ch.undo());
    assert_eq!(ch.count(), 1);
    assert!(ch.previews(&src).is_empty());
    assert!(ch.save(&mut src).is_empty());
    assert_eq!(ch.count(), 0);
    assert_eq!(
        std::fs::read_to_string(dir.0.join("r.md")).unwrap(),
        outside
    );
}

#[test]
fn revert_empties_row_state_and_undo_restores_mark() {
    // [WB-17] [WB-16] 元の値に戻すと行の基準と印も消え、undo で元の基準と印に戻る(取り消しの決まりと同じ)。
    let (dir, mut src, r) = setup("revert");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    let base = ch.states.get(&r).and_then(|s| s.base);
    edit_outside(&dir, "---\nstatus: todo\ntitle: R outside\n---\nbody\n");
    let changed = src.changed();
    ch.note_external(&changed);
    assert!(ch.external(&r));
    ch.set(&src, &r, "status", s("todo")).unwrap();
    assert_eq!(ch.count(), 0);
    assert!(!ch.states.contains_key(&r));
    assert!(!ch.external(&r));
    assert!(ch.undo());
    assert!(ch.external(&r));
    assert_eq!(ch.states.get(&r).and_then(|s| s.base), base);
    // 同じ値をもう一度入れても手は積まない(redo が残る)。
    ch.set(&src, &r, "title", s("R outside")).unwrap();
    assert!(ch.redo());
    assert_eq!(ch.count(), 0);
}

#[test]
fn undo_discard_restores_mark_and_base() {
    // [WB-16] 捨てるを取り消す → 元の基準と印に戻る(保存は止まる)。
    let (dir, mut src, r) = setup("discard");
    let mut ch = Changes::new();
    ch.set(&src, &r, "status", s("done")).unwrap();
    let base = ch.states.get(&r).and_then(|s| s.base);
    edit_outside(&dir, "---\nstatus: todo\ntitle: R outside\n---\nbody\n");
    let changed = src.changed();
    ch.note_external(&changed);
    ch.discard(&r);
    assert!(!ch.external(&r));
    assert!(ch.undo());
    assert!(ch.external(&r));
    assert_eq!(ch.states.get(&r).and_then(|s| s.base), base);
    let res = ch.save(&mut src);
    assert!(matches!(res[0].1, Outcome::Changed));
}

#[test]
fn scalar_onto_list_cell_is_skip() {
    // [CE-16] [CE-18] 書けるリストのセル(値がリスト・列の型がリストで値が空)にリストでない値 → Skip(理由つき)。
    // リストと空にする(Null)は当てられる。
    let (dir, _src, _r) = setup("listcell");
    std::fs::write(dir.0.join("l.md"), "---\ntags: [a]\n---\n").unwrap();
    std::fs::write(dir.0.join("m.md"), "---\ntitle: m\n---\n").unwrap();
    let mut src = Markdown::open(std::slice::from_ref(&dir.0)).unwrap();
    while !src.load(100).done {}
    let row = |name: &str| {
        src.rows()
            .into_iter()
            .find(|r| src.label(r) == name)
            .unwrap()
    };
    let (l, m) = (row("l.md"), row("m.md"));
    let mut ch = Changes::new();
    for (r, v) in [
        (&l, s("b")),
        (&m, s("b")),
        (&m, NewValue::Int(1)),
        (&l, NewValue::Bool(true)),
    ] {
        let e = ch.set(&src, r, "tags", v).unwrap_err();
        assert!(e.reason.contains("リスト"), "{}", e.reason);
    }
    assert_eq!(ch.count(), 0);
    ch.set(&src, &m, "tags", NewValue::List(vec!["b".into()]))
        .unwrap();
    ch.set(&src, &l, "tags", NewValue::Null).unwrap();
    assert_eq!(ch.count(), 2);
    // 一括(set_many)でも飛ばす。
    let skips = ch.set_many(&src, &[l.clone(), m.clone()], "tags", s("c"));
    assert_eq!(skips.len(), 2);
}
