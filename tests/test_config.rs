//! タスク 8 の受け入れ: 設定(CLI-3・SR-13)と見た目の状態(SR-11・SR-12)。
//! 実装を見ずに、docs/design.md の `src/config.rs` のインターフェースだけを使う。
//!
//! 設定の TOML の形(この試験が決める形。Config の項目に素直に写す):
//!
//! ```toml
//! candidates = 30          # CE-3 の候補の上限(既定 20)
//! poll_ms = 500            # BV-9 の読み直しの間隔(既定 1000)
//! ambiguous_wide = true    # CV-6 の East Asian Ambiguous を幅2にする(既定 false)
//! color = false            # 色なし(既定 true)
//!
//! [keys.table]             # [keys.<モード>] の下に キーの表記 = 動作の名前
//! j = "none"               # "none" でそのキーを外す
//! "ctrl+n" = "down"
//!
//! [keys.edit]
//! "ctrl+r" = "revert"
//! ```
//!
//! keys は (モード, キー, 動作) の並びに写る。並びの順は問わない。

use mdgrid::config::{apply_state, config_path, load_state, parse, save_state, ViewState};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-config-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// フォルダの下の全ファイルの相対パス(並べたもの)。
fn list_files(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    out.push(p.strip_prefix(root).unwrap().to_string_lossy().into_owned());
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn state(
    order: &[&str],
    hidden: &[&str],
    widths: &[(&str, u16)],
    folded: &[&str],
    view: Option<&str>,
) -> ViewState {
    ViewState {
        order: s(order),
        hidden: s(hidden),
        widths: widths.iter().map(|(k, w)| (k.to_string(), *w)).collect(),
        folded: s(folded),
        view: view.map(|v| v.to_string()),
        settings: Default::default(),
    }
}

fn assert_state_eq(a: &ViewState, b: &ViewState) {
    assert_eq!(a.order, b.order, "order");
    assert_eq!(a.hidden, b.hidden, "hidden");
    assert_eq!(a.widths, b.widths, "widths");
    assert_eq!(a.folded, b.folded, "folded");
    assert_eq!(a.view, b.view, "view");
}

fn assert_state_empty(a: &ViewState) {
    assert!(a.order.is_empty(), "order が空でない: {:?}", a.order);
    assert!(a.hidden.is_empty(), "hidden が空でない: {:?}", a.hidden);
    assert!(a.widths.is_empty(), "widths が空でない: {:?}", a.widths);
    assert!(a.folded.is_empty(), "folded が空でない: {:?}", a.folded);
    assert!(a.view.is_none(), "view が None でない: {:?}", a.view);
}

fn has_key(keys: &[(String, String, String)], mode: &str, key: &str, action: &str) -> bool {
    keys.iter()
        .any(|(m, k, a)| m == mode && k == key && a == action)
}

// ---- 設定(CLI-3・SR-13) ----

#[test]
fn test_cli_3_empty_config_gives_defaults() {
    // [CLI-3] 設定が無くても(空の文字列)既定で動く。
    let (c, warnings) = parse("").expect("空の設定は読める");
    assert!(warnings.is_empty(), "空の設定で警告: {:?}", warnings);
    assert_eq!(c.resolved().candidates, 20);
    assert_eq!(c.poll_ms, 1000);
    assert!(!c.terminal.ambiguous_wide);
    assert!(c.terminal.color);
    assert!(c.keys.is_empty());
}

#[test]
fn test_cli_3_reads_each_item() {
    // [CLI-3] 候補の数・読み直しの間隔・Ambiguous の幅・色を読む。
    let text =
        "poll_ms = 500\n[edit]\ncandidates = 30\n[terminal]\nambiguous_wide = true\ncolor = false\n";
    let (c, warnings) = parse(text).expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(c.resolved().candidates, 30);
    assert_eq!(c.poll_ms, 500);
    assert!(c.terminal.ambiguous_wide);
    assert!(!c.terminal.color);
    assert!(c.keys.is_empty());
}

#[test]
fn test_cli_3_partial_config_keeps_other_defaults() {
    // [CLI-3] 書いていない項目は既定のまま。
    let (c, _) = parse("poll_ms = 250\n").expect("読める");
    assert_eq!(c.poll_ms, 250);
    assert_eq!(c.resolved().candidates, 20);
    assert!(!c.terminal.ambiguous_wide);
    assert!(c.terminal.color);
}

#[test]
fn test_sr_13_keys_rebind_and_none() {
    // [SR-13] キーの割り当て直しと "none" での取り外しが (モード, キー, 動作) に写る。
    let text = r#"
[keys.table]
j = "none"
"ctrl+n" = "down"

[keys.edit]
"ctrl+r" = "revert"
"#;
    let (c, warnings) = parse(text).expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(c.keys.len(), 3, "keys: {:?}", c.keys);
    assert!(has_key(&c.keys, "table", "j", "none"), "keys: {:?}", c.keys);
    assert!(
        has_key(&c.keys, "table", "ctrl+n", "down"),
        "keys: {:?}",
        c.keys
    );
    assert!(
        has_key(&c.keys, "edit", "ctrl+r", "revert"),
        "keys: {:?}",
        c.keys
    );
    // ほかの項目は既定のまま。
    assert_eq!(c.resolved().candidates, 20);
    assert!(c.terminal.color);
}

#[test]
fn test_cli_3_unknown_item_warns_and_continues() {
    // [CLI-3] 知らない項目は警告にとどめ、止めない。警告の文に項目の名前が出る。
    let text = "frobnicate = true\n[edit]\ncandidates = 5\n";
    let (c, warnings) = parse(text).expect("知らない項目があっても Ok");
    assert_eq!(c.resolved().candidates, 5, "知っている項目は読む");
    assert!(
        warnings.iter().any(|w| w.contains("frobnicate")),
        "警告に項目の名前が無い: {:?}",
        warnings
    );
}

#[test]
fn test_cli_3_broken_toml_is_err() {
    // [CLI-3] 壊れた TOML は Err(理由1行)。
    for text in [
        "candidates = = 3\n",
        "[keys.table\nj = \"down\"\n",
        "color = \"\n",
    ] {
        match parse(text) {
            Err(reason) => {
                assert!(!reason.is_empty(), "理由が空: {:?}", text);
                assert!(
                    !reason.trim_end().contains('\n'),
                    "理由が1行でない: {:?}",
                    reason
                );
            }
            Ok(_) => panic!("壊れた TOML が Ok: {:?}", text),
        }
    }
}

#[test]
fn test_cli_3_config_path_follows_xdg_config_home() {
    // [CLI-3] 置き場は $XDG_CONFIG_HOME/mdgrid/config.toml、無ければ ~/.config/mdgrid/config.toml。
    // 環境変数を触るのはこのテストだけ。終わったら元に戻す。
    let saved_xdg = std::env::var_os("XDG_CONFIG_HOME");
    let saved_home = std::env::var_os("HOME");

    let result = std::panic::catch_unwind(|| {
        // 絶対パスでない値は無視する決まりなので、どの OS でも絶対パスになる一時フォルダの下を使う。
        let xdg = std::env::temp_dir().join("mdgrid-xdg-test");
        std::env::set_var("XDG_CONFIG_HOME", &xdg);
        assert_eq!(config_path(), Some(xdg.join("mdgrid").join("config.toml")));

        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("HOME", "/tmp/mdgrid-home-test");
        assert_eq!(
            config_path(),
            Some(PathBuf::from(
                "/tmp/mdgrid-home-test/.config/mdgrid/config.toml"
            ))
        );
    });

    match saved_xdg {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
    match saved_home {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

// ---- 見た目の状態(SR-11・SR-12) ----

#[test]
fn test_sr_11_save_then_load_round_trip() {
    // [SR-11] 保存した見た目の状態を読み戻せる。
    let t = TempDir::new("roundtrip");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let st = state(
        &["title", "status", "due"],
        &["due"],
        &[("title", 24), ("status", 8)],
        &["進行中"],
        Some("表"),
    );
    save_state(&dir, &notes, "表", &st).expect("保存できる");
    let back = load_state(&dir, &notes, "表");
    assert_state_eq(&back, &st);
}

#[test]
fn test_sr_11_load_missing_is_empty() {
    // [SR-11] 保存が無いときは既定(空)。
    let t = TempDir::new("missing");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    assert_state_empty(&load_state(&dir, &notes, "表"));
}

#[test]
fn test_sr_11_save_writes_nothing_outside_dir() {
    // [SR-11] 見た目の状態はノートのフォルダ(target のフォルダ)に書かない。dir の中にだけ作る。
    let t = TempDir::new("outside");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    std::fs::write(notes.join("a.md"), "---\ntitle: a\n---\n").unwrap();
    let base = notes.join("tasks.base");
    std::fs::write(&base, "views:\n  - type: table\n    name: 表\n").unwrap();

    let before_notes = list_files(&notes);
    let st = state(&["title"], &[], &[("title", 10)], &[], Some("表"));
    save_state(&dir, &notes, "表", &st).expect("フォルダの状態を保存");
    save_state(&dir, &base, "表", &st).expect(".base の状態を保存");

    assert_eq!(
        list_files(&notes),
        before_notes,
        "ノートのフォルダに新しいファイルができた"
    );
    // t の直下には state と notes のほか何も無い。
    let mut top: Vec<String> = std::fs::read_dir(t.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    top.sort();
    assert_eq!(top, s(&["notes", "state"]));
    assert!(!list_files(&dir).is_empty(), "dir に何も書かれていない");
}

#[test]
fn test_sr_12_state_is_per_target_and_view() {
    // [SR-12] 対象のパスとビューの名前ごとに別の状態。
    let t = TempDir::new("per");
    let dir = t.mkdir("state");
    let a = t.mkdir("a");
    let b = t.mkdir("b");
    let s_a1 = state(&["x", "y"], &[], &[("x", 5)], &[], Some("v1"));
    let s_a2 = state(&["y", "x"], &["x"], &[], &["g"], Some("v2"));
    let s_b1 = state(&["z"], &[], &[("z", 9)], &[], Some("v1"));
    save_state(&dir, &a, "v1", &s_a1).unwrap();
    save_state(&dir, &a, "v2", &s_a2).unwrap();
    save_state(&dir, &b, "v1", &s_b1).unwrap();

    assert_state_eq(&load_state(&dir, &a, "v1"), &s_a1);
    assert_state_eq(&load_state(&dir, &a, "v2"), &s_a2);
    assert_state_eq(&load_state(&dir, &b, "v1"), &s_b1);
    // 保存していない組み合わせは空。
    assert_state_empty(&load_state(&dir, &b, "v2"));
}

#[test]
fn test_sr_12_broken_state_file_gives_default() {
    // [SR-12] 状態のファイルが壊れていたら捨てて既定(空)。パニックしない。
    let t = TempDir::new("broken");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let st = state(&["title"], &["x"], &[("title", 7)], &["g"], Some("表"));
    save_state(&dir, &notes, "表", &st).unwrap();

    let files = list_files(&dir);
    assert!(!files.is_empty(), "状態のファイルが無い");
    for garbage in [
        &b"order = = [\n\xff\xfe not toml"[..],
        &b""[..],
        &b"order = 3\nwidths = \"x\"\n"[..],
    ] {
        for f in &files {
            std::fs::write(dir.join(f), garbage).unwrap();
        }
        let back = std::panic::catch_unwind(|| load_state(&dir, &notes, "表"))
            .expect("壊れた状態のファイルでパニックした");
        assert_state_empty(&back);
    }
}

#[test]
fn test_sr_12_apply_state_reconciles_columns() {
    // [SR-12] 状態の並びと隠す列は今ある列にだけ当て、状態に無い列は右に足し、無くなった列は落ちる。
    let columns = s(&["a", "b", "c", "d"]);
    // x は今の列に無い(無くなった列)。d は状態に無い(増えた列)。
    let st = state(&["c", "x", "b", "a"], &["b", "x"], &[], &[], None);
    let (order, hidden) = apply_state(&columns, &st);
    assert_eq!(order, s(&["c", "b", "a", "d"]));
    assert_eq!(hidden, s(&["b"]));
}

#[test]
fn test_sr_12_apply_state_empty_keeps_columns() {
    // [SR-12] 状態が空なら今の列の並びのまま、隠す列なし。
    let columns = s(&["title", "status", "due"]);
    let (order, hidden) = apply_state(&columns, &state(&[], &[], &[], &[], None));
    assert_eq!(order, columns);
    assert!(hidden.is_empty());
}

#[test]
fn test_sr_12_apply_state_new_columns_appended_in_current_order() {
    // [SR-12] 状態に無い列が複数あれば、今の列の順で右に足す。
    let columns = s(&["a", "b", "c", "d", "e"]);
    let (order, hidden) = apply_state(&columns, &state(&["d", "b"], &[], &[], &[], None));
    assert_eq!(order, s(&["d", "b", "a", "c", "e"]));
    assert!(hidden.is_empty());
}
