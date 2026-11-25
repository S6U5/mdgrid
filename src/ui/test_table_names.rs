//! 表の名前の見せ方: 印の `[[table]]` に `name` と `path` を両方書いた表は、引き出しと同じく、ヘッダーのパンくずと
//! 最近開いた表の一覧にもその `name` を出す試験。値は作ったもの(name = "予定"、path は notes のフォルダ)。
//!
//! 要求の文はまだ無い(人が書き足す)。印は要求ができてから付ける。

use super::app::{App, ColorMode};
use super::startup::Startup;
use super::test_screen::{screen, Tmp};
use mdgrid::config::Config;
use mdgrid::recent::Recent;
use mdgrid::source::markdown::Markdown;

/// 根(一時フォルダ)に印を置き、[[table]] に `table` を書いて、notes のフォルダを開く。
fn boot(tag: &str, table: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\ntitle: 会議\n---\n");
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\n{table}\n"),
    )
    .unwrap();
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(160, 24);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// ヘッダーの行。
fn header(a: &App) -> String {
    screen(a)
        .lines()
        .nth(super::bands::head_y(a) as usize)
        .unwrap_or_default()
        .to_string()
}

#[test]
fn test_breadcrumb_uses_marker_table_name() {
    // [utest -> req~screen.table-name-marker~1] 印の [[table]] に name = "予定" と path = "notes" の表を開くと、パンくずの表の名前は「予定」(フォルダの名前 notes でない)。
    let (_t, a) = boot("tn_crumb", "name = \"予定\"\npath = \"notes\"");
    let head = header(&a);
    assert!(head.contains("予定 ▾"), "{head}");
    assert!(!head.contains("notes ▾"), "{head}");
}

#[test]
fn test_breadcrumb_without_marker_name_keeps_folder_name() {
    // [utest -> req~screen.table-name-marker~1] 印の [[table]] に path だけを書いた表のパンくずは、今までどおりフォルダの名前(notes)。
    let (_t, a) = boot("tn_crumb_path", "path = \"notes\"");
    let head = header(&a);
    assert!(head.contains("notes ▾"), "{head}");
}

#[test]
fn test_recent_list_uses_marker_table_name() {
    // [utest -> req~screen.table-name-marker~1] 最近開いた表の一覧の行の名前も、印の name(「予定」)。フォルダの名前 notes の行にしない。
    let (t, _a) = boot("tn_recent", "name = \"予定\"\npath = \"notes\"");
    let r = Recent {
        path: t.notes().canonicalize().unwrap(),
        view: None,
        at: "2026-11-25 09:00".into(),
    };
    let lines = super::places::recent_labels(std::slice::from_ref(&r));
    assert!(lines[0].starts_with("予定 "), "{lines:?}");
}
