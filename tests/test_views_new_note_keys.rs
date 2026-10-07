//! [CE-27][CE-33] views.toml の new_note の mode・required・hidden・body は知っている項目(警告しない)で、
//! 保存し直すと消した値は戻らない。specs/_changes/2026-10-07-views-new-note-keys.md。

use mdgrid::newnote::NewNote;
use mdgrid::views::{load_views, save_views, NativeView};

#[test]
fn test_ce_27_views_new_note_new_keys() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("mdgrid-vnk-{}-{nanos}", std::process::id()));
    let conf = root.join("config");
    let notes = root.join("notes");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&notes).unwrap();
    let full = NewNote {
        mode: "editor".into(),
        required: vec!["due".into()],
        hidden: vec!["created".into()],
        body: "templates/t.md".into(),
        ..NewNote::default()
    };
    let view = |n: NewNote| NativeView {
        name: "v".into(),
        new_note: Some(n),
        ..NativeView::default()
    };
    save_views(&conf, &notes, &[view(full.clone())]).unwrap();
    let (views, warns) = load_views(&conf, &notes);
    assert!(
        warns.is_empty(),
        "知っている項目なので警告しない: {warns:?}"
    );
    assert_eq!(views[0].new_note.as_ref(), Some(&full));
    // 値を消して保存し直す → 戻らない(知らない項目として持ち越さない)。
    save_views(&conf, &notes, &[view(NewNote::default())]).unwrap();
    let (views, _) = load_views(&conf, &notes);
    let _ = std::fs::remove_dir_all(&root);
    let n = views[0].new_note.clone().unwrap_or_default();
    assert!(
        n.mode.is_empty() && n.required.is_empty() && n.body.is_empty(),
        "{n:?}"
    );
}
