//! タブの好み(NV-26)の読み書き: views.toml の対象の表の tab_order・hidden_tabs・tab_hint。

use crate::views::{self, NativeView, TabPrefs};
use std::path::PathBuf;

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-tabs-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn test_nv_26_tab_prefs_round_trip_keeps_views() {
    // [NV-26] 書いて読むと同じ。同じ対象のビューと既定のビュー、ほかの対象はそのまま。
    let d = dir("rt");
    let t = d.join("notes");
    let other = d.join("other");
    let v = NativeView {
        name: "進行中".into(),
        ..Default::default()
    };
    views::save_views(&d, &t, std::slice::from_ref(&v)).unwrap();
    views::save_views(&d, &other, std::slice::from_ref(&v)).unwrap();
    views::save_default_view(&d, &t, Some("進行中")).unwrap();
    let p = TabPrefs {
        order: vec!["進行中".into(), "既定の表".into()],
        hidden: vec!["既定の表".into()],
        hint: false,
    };
    views::save_tab_prefs(&d, &t, &p).unwrap();
    assert_eq!(views::load_tab_prefs(&d, &t), p);
    assert_eq!(views::load_default_view(&d, &t).as_deref(), Some("進行中"));
    assert_eq!(views::load_views(&d, &t).0, vec![v.clone()]);
    assert_eq!(views::load_tab_prefs(&d, &other), TabPrefs::default());
    // ビューを書き直しても好みは残る。
    views::save_views(&d, &t, &[v]).unwrap();
    assert_eq!(views::load_tab_prefs(&d, &t), p);
    // 知らない項目の警告にならない。
    assert!(views::load_views(&d, &t).1.is_empty());
}

#[test]
fn test_nv_26_tab_prefs_default_removes_keys() {
    // [NV-26] 既定に戻すと項目を消す。対象の表が無ければ既定を書いても表を作らない。
    let d = dir("def");
    let t = d.join("notes");
    views::save_tab_prefs(&d, &t, &TabPrefs::default()).unwrap();
    assert!(!d.join(views::FILE_NAME).exists());
    let p = TabPrefs {
        hint: false,
        ..Default::default()
    };
    views::save_tab_prefs(&d, &t, &p).unwrap();
    views::save_tab_prefs(&d, &t, &TabPrefs::default()).unwrap();
    let text = std::fs::read_to_string(d.join(views::FILE_NAME)).unwrap();
    assert!(!text.contains("tab_hint"), "{text}");
}

#[test]
fn test_nv_26_tab_prefs_broken_file_not_written() {
    // [NV-26] 壊れた views.toml は書き換えずに Err。読むと既定。
    let d = dir("broken");
    let t = d.join("notes");
    std::fs::write(d.join(views::FILE_NAME), "[[target]\n").unwrap();
    let p = TabPrefs {
        hint: false,
        ..Default::default()
    };
    assert!(views::save_tab_prefs(&d, &t, &p).is_err());
    assert_eq!(
        std::fs::read_to_string(d.join(views::FILE_NAME)).unwrap(),
        "[[target]\n"
    );
    assert_eq!(views::load_tab_prefs(&d, &t), TabPrefs::default());
}
