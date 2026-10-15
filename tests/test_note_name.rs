//! ノートの欄の名前の正規表現 `display.note_name`(SR-47・SR-21)の設定の読み込みと、`--print-config`・文書の
//! 受け入れ試験。読めない正規表現と文字列でない値は `config.toml: display.note_name:` で始まる警告1つで、使わない。
//! 空の文字列は書かないのと同じ。記録: specs/_decisions/2026-10-15-note-name.md。環境変数は書き換えない(子の環境だけ)。
//! 画面の見え方は src/ui/test_note_name.rs。

use mdgrid::config::parse;
use std::path::PathBuf;
use std::process::Command;

/// 設定の TOML を読み、決まった `note_name` の正規表現の文字(無ければ None)と警告を返す。
fn note_name(text: &str) -> (Option<String>, Vec<String>) {
    let (c, w) = parse(text).expect("読める");
    (c.resolved().display.note_name.map(|n| n.pattern()), w)
}

// ---- SR-47: [display] note_name を読む ----

#[test]
fn test_sr_47_config_reads_valid_regex() {
    // [SR-47] 読める正規表現は警告なしで読み、そのまま使う。書かなければ無し。
    let (n, w) = note_name("[display]\nnote_name = '^[0-9]{6}-'\n");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(n.as_deref(), Some("^[0-9]{6}-"));
    let (n, w) = note_name("");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(n, None, "既定は無し");
}

#[test]
fn test_sr_47_config_bad_regex_warns_and_unused() {
    // [SR-47][CLI-3] 読めない正規表現・文字列でない値 → `config.toml: display.note_name: ` で始まる警告1つで、
    // 正規表現は使わない。ほかの項目は読む。
    for v in ["'('", "'[あ-'", "true", "8", "['^x']"] {
        let text = format!("[display]\nnote_name = {v}\nzebra = true\n");
        let (c, w) = parse(&text).expect("読める");
        assert_eq!(w.len(), 1, "{v}: {w:?}");
        assert!(
            w[0].starts_with("config.toml: display.note_name: "),
            "{v}: {w:?}"
        );
        let d = c.resolved().display;
        assert_eq!(d.note_name, None, "{v}: 使わない");
        assert!(d.zebra, "{v}: ほかの項目は読む");
    }
}

#[test]
fn test_sr_47_config_empty_string_is_none() {
    // [SR-47] `note_name = ""` は書かないのと同じ。警告しない。
    let (n, w) = note_name("[display]\nnote_name = \"\"\n");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(n, None);
}

// ---- SR-47: views.toml のビューの設定の display ----

/// views.toml にビュー「w」を1つ書き、`settings.display` に `display` の行を入れる。
fn views_toml(name: &str, display: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "mdgrid-note-name-{name}-{}-{}",
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
        "[[table]]\npath = \"{p}\"\n\n  [[table.view]]\n  name = \"w\"\n  order = []\n  hidden = []\n  filters_expr = []\n  [table.view.settings.display]\n  {display}\n"
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    (conf, notes)
}

#[test]
fn test_sr_47_views_override_reads_regex() {
    // [SR-47][BV-17] ビューの設定の `display` の読める `note_name` を警告なしで読み、ビューの上書きとして持つ。
    let (conf, notes) = views_toml("ok", "note_name = '_draft$'");
    let (views, warns) = mdgrid::views::load_views(&conf, &notes);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(views.len(), 1);
    let n = views[0].settings.display.note_name.map(|n| n.pattern());
    assert_eq!(n.as_deref(), Some("_draft$"));
    let _ = std::fs::remove_dir_all(conf.parent().unwrap());
}

#[test]
fn test_sr_47_views_override_bad_regex_warns_view_kept() {
    // [SR-47][CLI-3] ビューの上書きの読めない正規表現・文字列でない値 → 警告1つで上書きを持たない。ビューは捨てない。
    for (i, v) in ["note_name = '(x'", "note_name = 3"].iter().enumerate() {
        let (conf, notes) = views_toml(&format!("bad{i}"), v);
        let (views, warns) = mdgrid::views::load_views(&conf, &notes);
        assert_eq!(warns.len(), 1, "{v}: {warns:?}");
        assert!(warns[0].contains("note_name"), "{v}: {warns:?}");
        assert_eq!(views.len(), 1, "{v}: ビューは読む");
        assert_eq!(views[0].name, "w");
        assert_eq!(views[0].settings.display.note_name, None, "{v}");
        let _ = std::fs::remove_dir_all(conf.parent().unwrap());
    }
}

// ---- SR-47・SR-21: --print-config と文書 ----

#[test]
fn test_sr_47_note_name_in_print_config() {
    // [SR-47][SR-21][CLI-11] --print-config の [display] に note_name = ""(既定なし)が出る。読み直しても警告なし。
    let nowhere =
        std::env::temp_dir().join(format!("mdgrid-note-name-none-{}", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg("--print-config")
        .env("XDG_CONFIG_HOME", &nowhere)
        .env("XDG_STATE_HOME", &nowhere)
        .output()
        .expect("mdgrid を起動できる");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).expect("出力は UTF-8");
    let table: toml::Table = text.parse().expect("TOML");
    let display = table["display"].as_table().expect("[display]");
    assert_eq!(
        display.get("note_name").and_then(|v| v.as_str()),
        Some(""),
        "{text}"
    );
    let (_, w) = parse(&text).expect("読める");
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn test_sr_47_note_name_in_config_docs() {
    // [SR-47][CLI-12] docs/config.md と docs/config.ja.md に項目の見出し `### \`display.note_name\`` がある。
    for rel in ["docs/config.md", "docs/config.ja.md"] {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
        let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel} を読めない: {e}"));
        assert!(
            text.lines()
                .any(|l| l.trim_end() == "### `display.note_name`"),
            "{rel} に `display.note_name` の見出しが無い"
        );
    }
}
