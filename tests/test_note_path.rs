//! ノートの欄のフォルダの出し方 `display.note_path`(SR-46・SR-29)の設定の読み込みと、`--print-config`・
//! 文書(SR-21・CLI-12)の受け入れ試験。値は `"common"`(既定)・`"name"`・`"full"`・`"off"`(提案中の4つ目。
//! 欄に名前も見出しも出さず、行の印だけ)。知らない値は警告して既定。
//! 記録: specs/_changes/2026-10-15-note-path.md(タスク #1)。環境変数は書き換えない(子の環境だけ)。
//!
//! 仮定(実装役に渡す): 決まった値は `Display` の `note_path`(`TabsMode` と同じ形の列挙 Common・Name・Full・Off)で、
//! Debug の字は `note_path: Common` などになる。試験は型に頼らず、Debug の字と TOML で確かめる。

use mdgrid::config::{parse, KEYS};
use mdgrid::profile::{resolve, Layer, Origin, Place, Profile};
use std::path::PathBuf;
use std::process::Command;

/// 決まった表示の設定の字(項目を名前で読む。型の形に頼らない)と警告。
fn display_debug(text: &str) -> (String, Vec<String>) {
    let (c, warnings) = parse(text).expect("読める");
    (format!("{:?}", c.resolved().display), warnings)
}

// ---- SR-46: [display] note_path を読む ----

#[test]
fn test_sr_46_config_default_is_common() {
    // [SR-46][SR-21] 書かなければ "common"。
    let (d, w) = display_debug("");
    assert!(w.is_empty(), "{w:?}");
    assert!(d.contains("note_path: Common"), "{d}");
}

#[test]
fn test_sr_46_config_reads_each_value() {
    // [SR-46] `[display] note_path` の4つの値を警告なしで読む。ほかの項目は既定のまま。
    for (v, want) in [
        ("common", "Common"),
        ("name", "Name"),
        ("full", "Full"),
        ("off", "Off"),
    ] {
        let (d, w) = display_debug(&format!("[display]\nnote_path = \"{v}\"\n"));
        assert!(w.is_empty(), "{v}: {w:?}");
        assert!(d.contains(&format!("note_path: {want}")), "{v}: {d}");
        assert!(d.contains("row_extra_lines: 0"), "ほかは既定: {d}");
    }
}

#[test]
fn test_sr_46_unknown_value_warns_and_keeps_default() {
    // [SR-46][CLI-3] `note_path = "short"`・型の違う値 → `config.toml: display.note_path:` で始まる警告1つと既定。
    // 知らない項目の警告ではなく、値の警告(項目は知っている)。ほかの項目は読む。
    for v in ["\"short\"", "\"Name\"", "true", "1"] {
        let (d, w) = display_debug(&format!("[display]\nnote_path = {v}\nzebra = true\n"));
        assert_eq!(w.len(), 1, "{v}: {w:?}");
        assert!(
            w[0].starts_with("config.toml: display.note_path: "),
            "{v}: {w:?}"
        );
        assert!(!w[0].contains("知らない項目"), "値の警告: {v}: {w:?}");
        assert!(d.contains("note_path: Common"), "{v}: {d}");
        assert!(d.contains("zebra: true"), "ほかの項目は読む: {d}");
    }
}

#[test]
fn test_sr_46_known_item_and_view_override_key() {
    // [SR-46][SR-21][CLI-12] `display.note_path` は設定の項目の表にあり、ビューの上書きの項目でもある。
    assert!(KEYS.contains(&"display.note_path"), "{KEYS:?}");
    assert!(
        mdgrid::display::OVERRIDE_KEYS.contains(&"note_path"),
        "{:?}",
        mdgrid::display::OVERRIDE_KEYS
    );
}

// ---- SR-46・SR-44: 範囲ごとに書ける ----

fn layer(place: Place, label: &str, toml: &str) -> Layer {
    let t: toml::Table = toml.parse().unwrap();
    let mut w = Vec::new();
    let profile = Profile::read_scoped(&t, label, "", &[], &mut w);
    assert!(w.is_empty(), "{label}: {w:?}");
    Layer {
        origin: Origin::new(place, label),
        profile,
    }
}

fn note_path_of(layers: &[Layer]) -> String {
    let mut w = Vec::new();
    let r = resolve(layers, &Vec::new(), &mut w);
    assert!(w.is_empty(), "{w:?}");
    format!("{:?}", r.display)
}

#[test]
fn test_sr_46_scopes_narrow_wins() {
    // [SR-46][SR-44] 全体 "name"、ワークスペース "full"、表 "common" → 表の "common"。表が無ければ "full"。
    // ワークスペースだけ "name" → "name"。
    let global = layer(
        Place::Config,
        "config.toml",
        "[display]\nnote_path = \"name\"\n",
    );
    let ws = layer(Place::Workspace, "ws", "[display]\nnote_path = \"full\"\n");
    let table = layer(
        Place::TableApp,
        "views.toml",
        "[display]\nnote_path = \"common\"\n",
    );
    let d = note_path_of(&[global.clone(), ws.clone(), table]);
    assert!(d.contains("note_path: Common"), "{d}");
    let d = note_path_of(&[global.clone(), ws]);
    assert!(d.contains("note_path: Full"), "{d}");
    let d = note_path_of(&[global]);
    assert!(d.contains("note_path: Name"), "{d}");
    let only_ws = layer(Place::Workspace, "ws", "[display]\nnote_path = \"name\"\n");
    let d = note_path_of(&[only_ws]);
    assert!(d.contains("note_path: Name"), "{d}");
}

/// views.toml にビュー「v」を1つ書き、`settings.display` に `display` の行を入れる。
fn views_toml(name: &str, display: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "mdgrid-note-path-{name}-{}-{}",
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
        "[[table]]\npath = \"{p}\"\n\n  [[table.view]]\n  name = \"v\"\n  order = []\n  hidden = []\n  filters_expr = []\n  [table.view.settings.display]\n  {display}\n"
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    (conf, notes)
}

#[test]
fn test_sr_46_views_override_reads_value() {
    // [SR-46][BV-17] mdgrid のビューの設定の `display` の `note_path = "full"` を警告なしで読む。
    let (conf, notes) = views_toml("ok", "note_path = \"full\"");
    let (views, warns) = mdgrid::views::load_views(&conf, &notes);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(views.len(), 1);
    let d = format!("{:?}", views[0].settings.display);
    assert!(d.contains("note_path: Some(Full)"), "{d}");
    let _ = std::fs::remove_dir_all(conf.parent().unwrap());
}

#[test]
fn test_sr_46_views_override_unknown_value_warns() {
    // [SR-46][CLI-3] ビューの上書きの知らない値 → 警告して上書きを持たない。ビューそのものは捨てない。
    let (conf, notes) = views_toml("bad", "note_path = \"short\"");
    let (views, warns) = mdgrid::views::load_views(&conf, &notes);
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(warns[0].contains("note_path"), "{warns:?}");
    assert_eq!(views.len(), 1, "ビューは読む");
    let d = format!("{:?}", views[0].settings.display);
    assert!(d.contains("note_path: None"), "{d}");
    let _ = std::fs::remove_dir_all(conf.parent().unwrap());
}

// ---- SR-21・SR-46: --print-config と文書 ----

fn print_config() -> String {
    let nowhere =
        std::env::temp_dir().join(format!("mdgrid-note-path-none-{}", std::process::id()));
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
    String::from_utf8(out.stdout).expect("出力は UTF-8")
}

#[test]
fn test_sr_21_note_path_in_print_config() {
    // [SR-21][SR-46][CLI-11] --print-config の [display] に note_path = "common" が出て、項目は13(2026-10-15-note-name で note_name が増えた)。
    // 出力を読み直しても警告なし。
    let text = print_config();
    let table: toml::Table = text.parse().expect("TOML");
    let display = table["display"].as_table().expect("[display]");
    assert_eq!(
        display.get("note_path").and_then(|v| v.as_str()),
        Some("common"),
        "{text}"
    );
    assert_eq!(display.len(), 13, "[display] は13項目: {display:?}");
    let (_, w) = parse(&text).expect("読める");
    assert!(w.is_empty(), "{w:?}");
}

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

#[test]
fn test_sr_21_note_path_in_config_docs() {
    // [SR-21][SR-46][CLI-12] docs/config.md と docs/config.ja.md に項目の見出し `### \`display.note_path\`` があり、
    // その下に4つの値が書いてある。
    for rel in ["docs/config.md", "docs/config.ja.md"] {
        let text = repo_file(rel);
        let lines: Vec<&str> = text.lines().collect();
        let i = lines
            .iter()
            .position(|l| l.trim_end() == "### `display.note_path`")
            .unwrap_or_else(|| panic!("{rel} に `display.note_path` の見出しが無い"));
        let body: String = lines[i + 1..]
            .iter()
            .take_while(|l| !l.starts_with('#'))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        for v in ["common", "name", "full", "off"] {
            assert!(body.contains(v), "{rel}: 値 {v} の説明が無い:\n{body}");
        }
    }
}
