//! 表の行を伸ばす設定 `display.row_extra_lines`(CV-8)の読み込みと書き出し(SR-21)の受け入れ試験。
//! 0〜9 の整数。既定 0。範囲の外・整数でない値は警告して既定。`--print-config` の `[display]` は13項目(2026-10-15-note-path で note_path、2026-10-15-note-name で note_name が増えた)。
//! 記録: specs/_changes/2026-10-12-row-lines.md(タスク #1)。環境変数は書き換えない(子の環境だけ)。

use mdgrid::config::parse;
use std::process::Command;

/// 決まった表示の設定の字(項目を名前で読む。型の形に頼らない)。
fn display_debug(text: &str) -> (String, Vec<String>) {
    let (c, warnings) = parse(text).expect("読める");
    (format!("{:?}", c.resolved().display), warnings)
}

#[test]
fn test_cv_8_config_default_is_zero() {
    // [CV-8] 書かなければ 0。
    let (d, w) = display_debug("");
    assert!(w.is_empty(), "{w:?}");
    assert!(d.contains("row_extra_lines: 0"), "{d}");
}

#[test]
fn test_cv_8_config_reads_value() {
    // [CV-8][SR-21] `[display] row_extra_lines = 2` → 2(警告なし)。上限の 9 も読める。
    for n in [2, 9] {
        let (d, w) = display_debug(&format!("[display]\nrow_extra_lines = {n}\n"));
        assert!(w.is_empty(), "{n}: {w:?}");
        assert!(d.contains(&format!("row_extra_lines: {n}")), "{d}");
    }
}

#[test]
fn test_cv_8_config_out_of_range_warns() {
    // [CV-8] `row_extra_lines = 12`・`-1`・`1.5`・`"two"` → 警告して既定の 0。
    for v in ["12", "-1", "1.5", "\"two\""] {
        let (d, w) = display_debug(&format!("[display]\nrow_extra_lines = {v}\n"));
        assert_eq!(w.len(), 1, "{v}: {w:?}");
        assert!(w[0].contains("display.row_extra_lines"), "{v}: {w:?}");
        assert!(d.contains("row_extra_lines: 0"), "{v}: {d}");
    }
}

#[test]
fn test_cv_8_config_view_override_is_known() {
    // [CV-8][SR-21] ビューごとの上書き(SR-20)の項目として知られている。
    assert!(
        mdgrid::display::OVERRIDE_KEYS.contains(&"row_extra_lines"),
        "{:?}",
        mdgrid::display::OVERRIDE_KEYS
    );
}

#[test]
fn test_sr_21_row_extra_lines_in_print_config() {
    // [SR-21][CV-8] --print-config の [display] に row_extra_lines = 0 が出て、項目は13。
    let nowhere =
        std::env::temp_dir().join(format!("mdgrid-row-lines-none-{}", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg("--print-config")
        .env("XDG_CONFIG_HOME", &nowhere)
        .env("XDG_STATE_HOME", &nowhere)
        .output()
        .expect("mdgrid を起動できる");
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let table: toml::Table = text.parse().expect("TOML");
    let display = table["display"].as_table().expect("[display]");
    assert_eq!(
        display.get("row_extra_lines").and_then(|v| v.as_integer()),
        Some(0),
        "{text}"
    );
    assert_eq!(display.len(), 13, "{display:?}");
}
