//! まとまりを値の候補の順に並べ(navigation.group-values-order)、見出しとカンバンのレーンの題を、
//! `[rules.<列>]` の `icons`・`labels` で見せる(navigation.group-values-icon)試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った。まとまりの順は `App.groups` の見出しの鍵(値のまま)で、
//! 見せ方は描いた画面の文字で確かめる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use mdgrid::config::Config;
use mdgrid::i18n::Msg;
use mdgrid::settings::{Dir, Group};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の候補(受け入れの例の順)。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n";

/// status が doing・todo・waiting・blocked・(なし) の行。owner は候補の無い列。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: doing\nowner: carol\n---\n"),
    ("beta.md", "---\nstatus: todo\nowner: alice\n---\n"),
    ("gamma.md", "---\nstatus: waiting\nowner: bob\n---\n"),
    ("delta.md", "---\nstatus: blocked\nowner: alice\n---\n"),
    ("epsilon.md", "---\ntitle: e\n---\n"),
];

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて、`notes` を表として開く。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{top}\n[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// 列 `col` で向き `dir` にまとめ直す。
fn group_by(a: &mut App, col: &str, dir: Dir) {
    a.settings.group = Group::By {
        col: col.into(),
        dir,
        hide_empty: false,
    };
    a.refresh();
}

/// まとまりの見出しの鍵(上から)。
fn heads(a: &App) -> Vec<String> {
    a.groups.iter().map(|(h, _)| h.clone()).collect()
}

fn want(list: &[&str]) -> Vec<String> {
    list.iter().map(std::string::ToString::to_string).collect()
}

/// 値の無い行のまとまりの見出し。
fn empty() -> &'static str {
    Msg::EmptyHeading.text()
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の中で `name` を含む最初の行。
fn line_with(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("「{name}」の行が無い:\n{s}"))
        .to_string()
}

#[test]
fn test_group_values_order_ascending_follows_candidates() {
    // [utest -> req~navigation.group-values-order~1] 候補 todo・doing・waiting・done の status で昇順にまとめると、
    // まとまりは todo・doing・waiting・blocked・(なし) の順。
    let (_t, mut a) = boot("gvo_asc", NOTES, RULES, "");
    group_by(&mut a, "status", Dir::Asc);
    assert_eq!(
        heads(&a),
        want(&["todo", "doing", "waiting", "blocked", empty()])
    );
}

#[test]
fn test_group_values_order_descending_reverses_candidates() {
    // [utest -> req~navigation.group-values-order~1] 同じ表を降順にまとめると、まとまりは waiting・doing・todo・blocked・(なし) の順。
    let (_t, mut a) = boot("gvo_desc", NOTES, RULES, "");
    group_by(&mut a, "status", Dir::Desc);
    assert_eq!(
        heads(&a),
        want(&["waiting", "doing", "todo", "blocked", empty()])
    );
}

#[test]
fn test_group_values_order_without_candidates_keeps_text_order() {
    // [utest -> req~navigation.group-values-order~1] 値の候補の無い列 owner で昇順にまとめると、今までどおり値の文字の順。
    let (_t, mut a) = boot("gvo_owner", NOTES, RULES, "");
    group_by(&mut a, "owner", Dir::Asc);
    assert_eq!(heads(&a), want(&["alice", "bob", "carol", empty()]));
}

#[test]
fn test_group_values_order_base_group_by_follows_candidates() {
    // [utest -> req~navigation.group-values-order~1] `.base` の groupBy で status ごとに昇順にまとめても、まとまりは
    // todo・doing・waiting・blocked・(なし) の順(ビューの設定でまとめ直したときと同じ)。
    let tmp = Tmp::new("gvo_base");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let yaml = "views:\n  - type: table\n    name: V\n    groupBy:\n      property: status\n      direction: ASC\n";
    tmp.write("T.base", yaml);
    let path = tmp.notes().join("T.base");
    let base = mdgrid::base::parse(yaml).unwrap();
    let (config, w) = mdgrid::config::parse(RULES).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let src = Markdown::open_vault(&path).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: path,
        base: Some((base, "T.base".into(), None)),
    });
    while !a.loaded() {
        a.load_step(100);
    }
    assert_eq!(
        heads(&a),
        want(&["todo", "doing", "waiting", "blocked", empty()])
    );
}

/// 受け入れの例の設定: 候補 todo・doing、todo に字 ○、todo と doing に名前。
const ICONS: &str = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nicons = { todo = \"○\" }\nlabels = { todo = \"未着手\", doing = \"進行中\" }\n";

/// todo 1件(alpha)・doing 1件(beta)。
const NOTES2: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: doing\n---\n"),
];

#[test]
fn test_group_values_icon_headings_show_glyph_and_name_cells_keep_value() {
    // [utest -> req~navigation.group-values-icon~1] status でまとめると、見出しは「○ 未着手」と「進行中」で、
    // todo の行の status のセルは todo のまま。
    let (_t, mut a) = boot("gvi_head", NOTES2, ICONS, "");
    group_by(&mut a, "status", Dir::Asc);
    let s = screen(&a);
    assert!(s.contains("status: ○ 未着手("), "{s}");
    assert!(s.contains("status: 進行中("), "{s}");
    assert!(
        !s.contains("status: todo(") && !s.contains("status: doing("),
        "{s}"
    );
    // tbd~navigation.value-labels-cells の仮決め: セルは値のまま。
    let row = line_with(&s, "alpha");
    assert!(row.contains("todo") && !row.contains("未着手"), "{row}");
    // ノートに書く値も鍵も値のまま。
    assert_eq!(heads(&a), want(&["todo", "doing"]));
}

#[test]
fn test_group_values_icon_kanban_lane_titles() {
    // [utest -> req~navigation.group-values-icon~1] 同じ設定で kanban = { column = "status" } の表をカンバンで開くと、
    // レーンの題は「○ 未着手」と「進行中」。
    let (_t, mut a) = boot(
        "gvi_kanban",
        NOTES2,
        ICONS,
        "kanban = { column = \"status\" }",
    );
    ch(&mut a, 'B');
    assert_ne!(a.mode, Mode::Table, "カンバンが開かない: {:?}", a.message);
    let s = screen(&a);
    let head = line_with(&s, "(1)");
    assert!(head.contains("○ 未着手 (1)"), "{s}");
    assert!(head.contains("進行中 (1)"), "{s}");
    assert!(!head.contains("todo (") && !head.contains("doing ("), "{s}");
}

#[test]
fn test_group_values_icon_unknown_value_warns_once_at_start_and_is_not_used() {
    // [utest -> req~navigation.group-values-icon~1] values に無い blocked の字を書いて起動すると、
    // `icons.blocked` を言う cli.config-file-warning-form の形の警告が1行で、その字は使わない。
    let top = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nicons = { blocked = \"!\" }\n";
    let notes = [("alpha.md", "---\nstatus: blocked\n---\n")];
    let (_t, mut a) = boot("gvi_warn", &notes, top, "");
    let m = a.message.clone().unwrap_or_default();
    let hits: Vec<&str> = m
        .split(" / ")
        .filter(|w| w.contains("rules.status.icons.blocked: "))
        .collect();
    assert_eq!(hits.len(), 1, "警告は1行: {m}");
    assert!(hits[0].contains("workspace.toml"), "ファイルの名前: {m}");
    group_by(&mut a, "status", Dir::Asc);
    let s = screen(&a);
    assert!(s.contains("status: blocked("), "{s}");
    assert!(!s.contains("! blocked"), "{s}");
}

#[test]
fn test_group_heading_uses_display_name_of_hidden_column() {
    // [utest -> req~base-view.default-headings-display-name~1] `.base` の properties に displayName のある列で
    // まとめたら、その列をビューに出していなくても、まとまりの見出しの列の名前は displayName(「状態: 」)。
    let tmp = Tmp::new("gv_dn");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let yaml = "properties:\n  note.status:\n    displayName: 状態\nviews:\n  - type: table\n    name: V\n    order:\n      - owner\n    groupBy:\n      property: status\n      direction: ASC\n";
    tmp.write("T.base", yaml);
    let path = tmp.notes().join("T.base");
    let base = mdgrid::base::parse(yaml).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: path,
        base: Some((base, "T.base".into(), None)),
    });
    while !a.loaded() {
        a.load_step(100);
    }
    let s = screen(&a);
    assert!(s.contains("状態: "), "{s}");
    assert!(!s.contains("status: "), "{s}");
}
