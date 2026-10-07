//! [CE-32] 雛形の変数と本文の雛形、[CE-27] required・hidden の読み書き。
//! specs/_changes/2026-10-07-new-note-form.md。

use super::{build_with, expand, NewNote, Vars};
use crate::types;
use crate::writeback::{Edit, NewValue};

fn vars() -> Vars {
    Vars {
        today: types::parse_date("2026-10-07").unwrap(),
        minutes: 20 * 60 + 15,
        name: "打ち合わせ".into(),
        folder: "Tasks".into(),
    }
}

#[test]
fn test_ce_32_template_variables() {
    let v = vars();
    assert_eq!(expand("{date}", &v), "2026-10-07");
    assert_eq!(expand("{date+7}", &v), "2026-10-14");
    assert_eq!(expand("{date-1}", &v), "2026-10-06");
    assert_eq!(expand("{date:YYYY/MM/DD}", &v), "2026/10/07");
    assert_eq!(expand("{time}", &v), "20:15");
    assert_eq!(expand("{now}", &v), "2026-10-07T20:15");
    assert_eq!(expand("{name} in {folder}", &v), "打ち合わせ in Tasks");
    // 2026-10-07 は水曜。
    assert!(["We", "水"].contains(&expand("{weekday}", &v).as_str()));
    // 知らない変数・読めない形・閉じない括弧は文字のまま。
    assert_eq!(expand("{unknown}", &v), "{unknown}");
    assert_eq!(expand("{date:QQ}", &v), "{date:QQ}");
    assert_eq!(expand("a {b", &v), "a {b");
}

#[test]
fn test_ce_32_hidden_created_and_body() {
    let rule = NewNote {
        hidden: vec!["created".into()],
        set: vec![
            ("created".into(), NewValue::Str("{now}".into())),
            ("tags".into(), NewValue::List(vec!["{date}".into()])),
        ],
        ..NewNote::default()
    };
    let answers = vec![Edit {
        key: "status".into(),
        value: NewValue::Str("todo".into()),
    }];
    let out = build_with(
        &rule,
        &[],
        &answers,
        &vars(),
        Some("# {name}\n作成: {date}\n"),
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    // 型の分からない列の日時の形の文字は、画面(new_note.rs)が WB-18 で囲まずに書く。ここは値だけを見る。
    assert!(
        text.contains("created: ") && text.contains("2026-10-07T20:15"),
        "{text}"
    );
    assert!(
        text.contains("  - \"2026-10-07\""),
        "リストの要素も埋める: {text}"
    );
    assert!(text.contains("status: todo"), "{text}");
    assert!(
        text.ends_with("---\n# 打ち合わせ\n作成: 2026-10-07\n"),
        "{text}"
    );
}

#[test]
fn test_ce_27_required_hidden_body_round_trip() {
    let t: toml::Value = toml::from_str(
        "required = [\"due\"]\nhidden = [\"created\", \"file.name\"]\nbody = \"templates/t.md\"\n",
    )
    .unwrap();
    let n = NewNote::from_toml(&t).unwrap();
    assert_eq!(n.required, ["due"]);
    assert_eq!(n.hidden, ["created"], "file.* は書けないので捨てる");
    assert_eq!(n.body, "templates/t.md");
    assert_eq!(NewNote::from_toml(&n.to_toml()).unwrap(), n);
}
