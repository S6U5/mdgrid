//! -w で選んだワークスペース(WS-6): 開いた表がそこに無ければ、そう言う。

use super::*;

#[test]
fn test_ws_6_chosen_without_the_table_warns() {
    // [WS-6] -w で選んだワークスペースに無い表を開いたら、範囲は選んだものにして、そう言う。入っていれば言わない。
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-wsc-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("tasks")).unwrap();
    std::fs::create_dir_all(d.join("other")).unwrap();
    let d = std::fs::canonicalize(&d).unwrap();
    let apps = vec![Workspace {
        name: "W".into(),
        tables: vec![WsTable {
            name: "T".into(),
            path: d.join("tasks"),
            view: None,
            profile: Default::default(),
        }],
        profile: Default::default(),
    }];
    let s = resolve(&d.join("other"), Some("W"), &apps, &[])
        .unwrap()
        .unwrap();
    assert_eq!(s.source, Source::Chosen);
    assert_eq!(s.warnings.len(), 1, "{:?}", s.warnings);
    assert!(s.warnings[0].contains("other") && s.warnings[0].contains("W"));
    let s = resolve(&d.join("tasks"), Some("W"), &apps, &[])
        .unwrap()
        .unwrap();
    assert!(s.warnings.is_empty(), "{:?}", s.warnings);
    let _ = std::fs::remove_dir_all(&d);
}
