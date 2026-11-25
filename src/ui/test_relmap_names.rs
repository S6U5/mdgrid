//! 関係マップとつながった行の一覧の、列の名前(base-view.display-name-everywhere)とノートの名前
//! (screen.note-name-everywhere)と、詳細の「逆向き」(relations.map-reverse)の試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った: 表「会議」の attendees に displayName「出席者」があり、会議の attendees が
//! 表「人」の「安藤」を指す。人の meetings(displayName「出た会議」)は「安藤」からその会議を指し返す。
//! 同じ表の中のつながりは、問いの related(互いに指す)と parent・children。画面の文字で確かめる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use mdgrid::config::{parse, Config};
use mdgrid::i18n::Msg;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 日付の頭を隠す設定。
const HEAD: &str = "[display]\nnote_name = '^\\d{8}_'\n";

const MEETINGS_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"Meetings\")\nproperties:\n  note.attendees:\n    displayName: 出席者\nviews:\n  - type: table\n    name: V\n    order:\n      - attendees\n";

const PEOPLE_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"People\")\nproperties:\n  note.meetings:\n    displayName: 出た会議\n  note.kana:\n    displayName: 読み\nviews:\n  - type: table\n    name: V\n    order:\n      - kana\n";

const PEOPLE_NOTES: &[(&str, &str)] = &[
    (
        "Meetings/20261102_定例.md",
        "---\nattendees:\n  - \"[[安藤]]\"\n---\n",
    ),
    (
        "People/安藤.md",
        "---\nkana: あんどう\nmeetings:\n  - \"[[20261102_定例]]\"\n---\n",
    ),
    ("People/井上.md", "---\nkana: いのうえ\n---\n"),
];

/// 設定の TOML を読む(警告が無いことも確かめる)。
fn cfg(text: &str) -> Config {
    let (c, w) = parse(text).expect("設定の TOML は読める");
    assert!(w.is_empty(), "設定の警告: {w:?}");
    c
}

/// 会議と人の2つの `.base` を置き、ワークスペースを2つの表にして、人の `.base` で開く。
fn boot_people(tag: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for d in ["Meetings", "People"] {
        std::fs::create_dir_all(tmp.notes().join(d)).unwrap();
    }
    for (n, t) in PEOPLE_NOTES {
        tmp.write(n, t);
    }
    tmp.write("会議.base", MEETINGS_BASE);
    tmp.write("人.base", PEOPLE_BASE);
    let path = tmp.notes().join("人.base");
    let base = mdgrid::base::parse(PEOPLE_BASE).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(160, 40);
    a.start(Startup {
        config: cfg(HEAD),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: path,
        base: Some((base, "人.base".into(), None)),
    });
    while !a.loaded() {
        a.load_step(100);
    }
    let table = |name: &str, file: &str| WsTable {
        name: name.into(),
        path: tmp.notes().canonicalize().unwrap().join(file),
        ..WsTable::default()
    };
    a.scope = Some(Scope {
        name: "人と会議".into(),
        source: Source::Chosen,
        tables: vec![table("人", "人.base"), table("会議", "会議.base")],
        profile: Default::default(),
        warnings: Vec::new(),
    });
    (tmp, a)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 関係マップを開き、`from`.`column` のつながりを選ぶ。
fn open_link(a: &mut App, from: &str, column: &str) {
    ch(a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    let rm = a.relmap.as_mut().expect("関係マップ");
    let k = rm
        .links
        .iter()
        .position(|l| l.from == from && l.column == column)
        .unwrap_or_else(|| panic!("{from}.{column} のつながりが無い"));
    let t = rm
        .infos
        .iter()
        .position(|i| i.name == from)
        .expect("元の表");
    rm.sel = t;
    rm.link = Some(k);
}

/// 詳細の窓の、項目の名前 `key` の行の値(項目の名前の後ろの語を空白でつないだもの)。行が無ければ None。
fn detail_value(s: &str, key: &str) -> Option<String> {
    s.lines().find_map(|l| {
        let at = l.find(&format!("│{key} "))?;
        let rest = &l[at + "│".len() + key.len()..];
        let end = rest.find('│').unwrap_or(rest.len());
        Some(rest[..end].split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

#[test]
fn test_relmap_box_and_detail_use_display_name_of_the_column_table() {
    // [utest -> req~base-view.display-name-everywhere~3] 会議の attendees に displayName「出席者」: 会議.attendees の
    // つながりを選ぶと、会議の箱の行は「出席者 → 人」、詳細の題は「会議.出席者 → 人」、詳細の「列」は「出席者」で、
    // attendees はどこにも出ない。
    let (_t, mut a) = boot_people("rmn_box");
    open_link(&mut a, "会議", "attendees");
    let s = screen(&a);
    assert!(s.contains("出席者 → 人"), "{s}");
    assert!(s.contains("会議.出席者 → 人"), "{s}");
    assert_eq!(
        detail_value(&s, Msg::RelMapColumn.text()).as_deref(),
        Some("出席者"),
        "{s}"
    );
    assert!(!s.contains("attendees"), "{s}");
}

#[test]
fn test_relmap_reverse_across_tables_uses_display_name() {
    // [utest -> req~relations.map-reverse~1, req~base-view.display-name-everywhere~3] 人の meetings が「安藤」から
    // その会議を指し返す: 会議.attendees の詳細の「逆向き」は meetings(人の表の displayName で「出た会議」)。
    let (_t, mut a) = boot_people("rmn_rev_cross");
    open_link(&mut a, "会議", "attendees");
    let s = screen(&a);
    assert_eq!(
        detail_value(&s, Msg::RelMapReverse.text()).as_deref(),
        Some("出た会議"),
        "{s}"
    );
}

#[test]
fn test_linked_rows_list_uses_display_name_and_note_name() {
    // [utest -> req~base-view.display-name-everywhere~3, req~screen.note-name-everywhere~2] 人の表の「安藤」の行で
    // 「つながった行」を開くと、一覧の行は「会議 › 出席者 ← 定例」(列は会議の displayName、名前は頭を隠す)。
    let (_t, mut a) = boot_people("rmn_linked");
    let at = a
        .rows
        .iter()
        .position(|r| a.src.label(r).contains("安藤"))
        .expect("安藤の行");
    a.row = a
        .slots
        .iter()
        .position(|s| *s == super::grid::Slot::Row(at))
        .unwrap();
    a.start_linked_rows();
    let s = screen(&a);
    assert!(s.contains("会議 › 出席者 ← 定例"), "{s}");
    assert!(!s.contains("attendees") && !s.contains("20261102_"), "{s}");
}

/// 同じ表の中のつながり: A と B は related で互いに指し、C は parent で A を指す。`extra` は A に足すキーの行。
fn boot_questions(tag: &str, extra: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write(
        "A.md",
        &format!("---\nrelated:\n  - \"[[B]]\"\n{extra}---\n"),
    );
    tmp.write("B.md", "---\nrelated:\n  - \"[[A]]\"\n---\n");
    tmp.write("C.md", "---\nparent: \"[[A]]\"\n---\n");
    let conf = tmp.0.join("config");
    std::fs::create_dir_all(&conf).unwrap();
    places::save(
        &conf,
        Place {
            name: "問い".into(),
            group: String::new(),
            path: tmp.notes().canonicalize().unwrap(),
            view: None,
        },
    )
    .unwrap();
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(160, 40);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(conf),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

#[test]
fn test_relmap_reverse_same_table_without_back_pairs_is_none() {
    // [utest -> req~relations.map-reverse~1] 問いの related(A と B が互いに指す)を選ぶと、詳細に「逆向き」の行が無い
    // (同じ表の parent は、related の組を指し返さないので出ない)。
    let (_t, mut a) = boot_questions("rmn_rev_self", "");
    open_link(&mut a, "問い", "related");
    let s = screen(&a);
    assert_eq!(detail_value(&s, Msg::RelMapReverse.text()), None, "{s}");
}

#[test]
fn test_relmap_reverse_same_table_matches_back_pairs() {
    // [utest -> req~relations.map-reverse~1] A の children が C を指し、C の parent が A を指す: 問い.parent の
    // 詳細の「逆向き」は「children」だけ(related は出ない)。
    let (_t, mut a) = boot_questions("rmn_rev_children", "children:\n  - \"[[C]]\"\n");
    open_link(&mut a, "問い", "parent");
    let s = screen(&a);
    assert_eq!(
        detail_value(&s, Msg::RelMapReverse.text()).as_deref(),
        Some("children"),
        "{s}"
    );
}
