//! 表の行を伸ばす設定 `display.row_extra_lines`(CV-8)のビューごとの上書き(views.toml の settings.display)。
//! 0〜9 の整数でなければ警告して既定(上書きを持たない)。ビューそのものは捨てない。
//! 記録: specs/_changes/2026-10-12-row-lines.md(タスク #6。照合の指摘)。環境変数は書き換えない。

fn views_toml(name: &str, display: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "mdgrid-row-lines-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let conf = root.join("config");
    let notes = root.join("notes");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&notes).unwrap();
    let p = notes
        .to_str()
        .unwrap()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let text = format!(
        "[[target]]\npath = \"{p}\"\n\n  [[target.view]]\n  name = \"v\"\n  order = []\n  hidden = []\n  filters_expr = []\n  [target.view.settings.display]\n  {display}\n"
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    (conf, notes)
}

#[test]
fn test_cv_8_views_override_reads_value() {
    // [CV-8] ビューごとの上書き(views.toml の settings.display)の `row_extra_lines = 2` → 2、警告なし。
    let (conf, notes) = views_toml("ok", "row_extra_lines = 2");
    let (views, warns) = mdgrid::views::load_views(&conf, &notes);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].settings.display.row_extra_lines, Some(2));
}

#[test]
fn test_cv_8_views_override_out_of_range_warns() {
    // [CV-8] ビューごとの上書きの `row_extra_lines` が範囲の外・整数でない → 警告して既定(上書きを持たない)。
    // ビューそのものは捨てない。
    for v in ["12", "-1", "300", "1.5", "\"two\""] {
        let (conf, notes) = views_toml("bad", &format!("row_extra_lines = {v}"));
        let (views, warns) = mdgrid::views::load_views(&conf, &notes);
        assert_eq!(warns.len(), 1, "{v}: {warns:?}");
        assert!(warns[0].contains("row_extra_lines"), "{v}: {warns:?}");
        assert_eq!(views.len(), 1, "{v}: ビューは読む");
        assert_eq!(views[0].settings.display.row_extra_lines, None, "{v}");
    }
}
