//! 列の名前を見せる所(ビューの帯の条件の札・カンバンとガントの窓の題・新しいノートの窓の欄・木の画面の列の見出し)に、
//! 列の見出しと同じく `.base` の `properties` の displayName を使う(base-view.display-name-everywhere)試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った(新しいノートの窓と木の画面は 2026-11-25 に見つけた所で、受け入れの例は人が
//! 書き足す)。まとまりの見出しは test_group_values.rs が確かめる。表は `.base` で開き、カンバン・ガント・木の設定は
//! 印の `[[table]]`(その `.base` を指す)に書く。画面の文字で確かめる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use mdgrid::config::Config;
use mdgrid::settings::{Cond, Dir, Group, Op};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// status に displayName「状態」、owner は displayName なし。
const YAML: &str = "properties:\n  note.status:\n    displayName: 状態\nviews:\n  - type: table\n    name: V\n    order:\n      - status\n      - owner\n";

const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\nowner: carol\n---\n"),
    ("beta.md", "---\nstatus: doing\nowner: alice\n---\n"),
    ("gamma.md", "---\nstatus: done\nowner: bob\n---\n"),
];

/// `notes` と `.base`(中身 `yaml`)を置き、印の `[[table]]` にその `.base` と `line` を書いて、`.base` で開く。
fn boot(tag: &str, notes: &[(&str, &str)], yaml: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    tmp.write("T.base", yaml);
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes/T.base\"\n{line}\n"),
    )
    .unwrap();
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
    (tmp, a)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の中で `what` を含む最初の行。
fn line_with(s: &str, what: &str) -> String {
    s.lines()
        .find(|l| l.contains(what))
        .unwrap_or_else(|| panic!("「{what}」の行が無い:\n{s}"))
        .to_string()
}

/// 列 `col` の値 `value` を除く絞り込みを掛ける。
fn drop_value(a: &mut App, col: &str, value: &str) {
    a.settings.filters = vec![Cond {
        col: col.into(),
        op: Op::Drop(vec![Some(value.into())]),
    }];
    a.refresh();
}

#[test]
fn test_display_name_filter_chip_uses_display_name() {
    // [utest -> req~base-view.display-name-everywhere~1] 絞り込み「status が done を除く」のビューの帯の札は「状態: done を除く」。
    let (_t, mut a) = boot("dn_chip", NOTES, YAML, "");
    drop_value(&mut a, "status", "done");
    let s = screen(&a);
    let band = line_with(&s, "done を除く");
    assert!(band.contains("状態: done を除く"), "{s}");
    assert!(!band.contains("status:"), "{s}");
    // 前提: ノートに書く値と設定の列は id のまま。
    assert_eq!(a.settings.filters[0].col, "status");
}

#[test]
fn test_display_name_filter_chip_without_display_name_keeps_raw_name() {
    // [utest -> req~base-view.display-name-everywhere~1] displayName の無い列 owner の絞り込みの帯の札は「owner: …」のまま。
    let (_t, mut a) = boot("dn_chip_raw", NOTES, YAML, "");
    drop_value(&mut a, "owner", "carol");
    let s = screen(&a);
    assert!(
        line_with(&s, "carol を除く").contains("owner: carol を除く"),
        "{s}"
    );
}

#[test]
fn test_display_name_sort_and_group_chips_use_display_name() {
    // [utest -> req~base-view.display-name-everywhere~1] 帯の並べ替えとグループの札も、列の見出しと同じ displayName「状態」。
    let (_t, mut a) = boot("dn_chip_sort", NOTES, YAML, "");
    a.settings.sorts = vec![("status".into(), Dir::Desc)];
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    let chips = a.bar_chips();
    assert!(
        chips.iter().any(|c| c.starts_with("並べ替え: 状態 ")),
        "{chips:?}"
    );
    assert!(
        chips.iter().any(|c| c.starts_with("グループ: 状態 ")),
        "{chips:?}"
    );
    assert!(!chips.iter().any(|c| c.contains("status")), "{chips:?}");
}

#[test]
fn test_display_name_kanban_title() {
    // [utest -> req~base-view.display-name-everywhere~1] 同じ表で kanban = { column = "status" } のカンバンの窓の題は「カンバン: 状態」。
    let (_t, mut a) = boot("dn_kanban", NOTES, YAML, "kanban = { column = \"status\" }");
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "カンバンが開かない: {:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("カンバン: 状態"), "{s}");
    assert!(!s.contains("カンバン: status"), "{s}");
}

#[test]
fn test_display_name_gantt_title() {
    // [utest -> req~base-view.display-name-everywhere~1] note.start に displayName「開始」、note.due に「期限」で
    // gantt = { start = "start", end = "due" } のガントの窓の題は「ガント: 開始 → 期限(…)」。
    let yaml = "properties:\n  note.start:\n    displayName: 開始\n  note.due:\n    displayName: 期限\nviews:\n  - type: table\n    name: V\n    order:\n      - start\n      - due\n";
    let notes = [
        ("alpha.md", "---\nstart: 2026-10-01\ndue: 2026-10-07\n---\n"),
        ("beta.md", "---\nstart: 2026-10-03\ndue: 2026-10-05\n---\n"),
    ];
    let (_t, mut a) = boot(
        "dn_gantt",
        &notes,
        yaml,
        "gantt = { start = \"start\", end = \"due\" }",
    );
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "ガントが開かない: {:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("ガント: 開始 → 期限("), "{s}");
    assert!(!s.contains("start → due"), "{s}");
}

/// 窓の行: 上の縁に `title` のある行から、下の縁(`╰`・`└`)の前の行まで。
fn window_lines(s: &str, title: &str) -> Vec<String> {
    let lines: Vec<&str> = s.lines().collect();
    let top = lines
        .iter()
        .position(|l| l.contains(title))
        .unwrap_or_else(|| panic!("「{title}」の窓が無い:\n{s}"));
    lines[top..]
        .iter()
        .take_while(|l| !l.contains('╰') && !l.contains('└'))
        .map(|l| (*l).to_string())
        .collect()
}

#[test]
fn test_display_name_new_note_window_fields() {
    // [utest -> req~base-view.display-name-everywhere~1] 新しいノートの窓(a)の欄の名前も、列の見出しと同じ displayName
    // (status の欄は「状態」。displayName の無い owner は owner のまま)。
    let (_t, mut a) = boot("dn_new_note", NOTES, YAML, "");
    ch(&mut a, 'a');
    let s = screen(&a);
    let win = window_lines(&s, mdgrid::i18n::Msg::NoteFormTitle.text().trim());
    assert!(win.iter().any(|l| l.contains("状態")), "{s}");
    assert!(win.iter().any(|l| l.contains("owner")), "{s}");
    assert!(!win.iter().any(|l| l.contains("status")), "{s}");
}

#[test]
fn test_display_name_tree_column_headings() {
    // [utest -> req~base-view.display-name-everywhere~1] 木の画面(T)の右の欄の列の見出しも、列の見出しと同じ displayName。
    let yaml = "properties:\n  note.status:\n    displayName: 状態\nviews:\n  - type: table\n    name: V\n    order:\n      - status\n      - owner\n      - folder\n";
    let notes = [
        (
            "alpha.md",
            "---\nstatus: todo\nowner: carol\nfolder: 仕事\n---\n",
        ),
        (
            "beta.md",
            "---\nstatus: doing\nowner: alice\nfolder: 仕事\n---\n",
        ),
    ];
    let (_t, mut a) = boot("dn_tree", &notes, yaml, "tree = { path = \"folder\" }");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "木が開かない: {:?}", a.message);
    let s = screen(&a);
    let head = line_with(&s, super::view::LABEL_HEADER.text());
    assert!(head.contains("状態"), "{s}");
    assert!(head.contains("owner"), "{s}");
    assert!(!head.contains("status"), "{s}");
}

#[test]
fn test_display_name_chart_title() {
    // [utest -> req~base-view.display-name-everywhere~1] 同じ表で chart = { x = "status" } のグラフの窓の題も、列の名前は「状態」(status でない)。
    let (_t, mut a) = boot("dn_chart", NOTES, YAML, "chart = { x = \"status\" }");
    ch(&mut a, 'P');
    assert_eq!(a.mode, Mode::Chart, "グラフが開かない: {:?}", a.message);
    let s = screen(&a);
    let title = line_with(&s, "グラフ: ");
    assert!(title.contains("状態"), "{title}");
    assert!(!title.contains("status"), "{title}");
}
