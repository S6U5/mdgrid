//! 表の既定の並べ替え(印の `[[table]]` の `sort`。base-view.table-sort ほか)と、画面で決めた並べ替えのあとに既定を
//! 最後の決め手として付けること・並べ替えの窓とボタンとビューの設定での既定の見せ方(navigation.sort-default-tiebreak
//! ほか)の試験。期待値は要求の受け入れの例から取った(実装の前に書いた)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_grid::labels;
use super::test_screen::{ch, col_named, press, text, Tmp};
use super::test_settings_screen::{apply_btn, draft, open, section};
use super::*;
use mdgrid::config::Config;
use mdgrid::settings::Dir;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// base-view.table-sort の例: date が 2026-11-20 の a・2026-11-25 の b・2026-11-22 の c(名前の順は a・b・c)。
const ABC: &[(&str, &str)] = &[
    ("a.md", "---\ndate: 2026-11-20\nstatus: todo\n---\n"),
    ("b.md", "---\ndate: 2026-11-25\nstatus: todo\n---\n"),
    ("c.md", "---\ndate: 2026-11-22\nstatus: done\n---\n"),
];

/// navigation.sort-default-tiebreak の例: a(todo・2026-11-30)・b(done・2026-11-10)・c(todo・2026-11-05)・d(done・2026-11-20)。
const ABCD: &[(&str, &str)] = &[
    (
        "a.md",
        "---\nstatus: todo\ndue: 2026-11-30\ntitle: t\n---\n",
    ),
    (
        "b.md",
        "---\nstatus: done\ndue: 2026-11-10\ntitle: t\n---\n",
    ),
    (
        "c.md",
        "---\nstatus: todo\ndue: 2026-11-05\ntitle: t\n---\n",
    ),
    (
        "d.md",
        "---\nstatus: done\ndue: 2026-11-20\ntitle: t\n---\n",
    ),
];

const DATE_DESC: &str = "sort = [{ property = \"date\", direction = \"DESC\" }]";
const DUE_ASC: &str = "sort = [{ property = \"due\", direction = \"ASC\" }]";

/// 根(一時フォルダ)の印の `[[table]]`(表はフォルダ notes。ビューの無い表)に `line` を書いて開く。
fn boot(tag: &str, notes: &[(&str, &str)], line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let a = start_folder(&tmp);
    (tmp, a)
}

/// ノートのフォルダ(ビューの無い表)を開く。
fn start_folder(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    start(tmp, tmp.notes(), Box::new(src), None)
}

/// `target` を開く(`base` は `.base` を開くときの定義)。設定の置き場は tmp/config、状態は tmp/state。
fn start(
    tmp: &Tmp,
    target: PathBuf,
    src: Box<dyn mdgrid::source::Source>,
    base: Option<(mdgrid::base::Base, String, Option<usize>)>,
) -> App {
    let mut a = App::new(src, ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

/// ノートのフォルダの印に `marker` を書き、印の表「台帳」(`[table.base]` を書いた表)を開く。
fn boot_inline(tag: &str, notes: &[(&str, &str)], marker: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let p = tmp.notes().join("台帳.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let a = start(&tmp, p, t.src, t.base);
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の中で `what` を含む行(無ければ落とす)。
fn line_with(s: &str, what: &str) -> String {
    s.lines()
        .find(|l| l.contains(what))
        .unwrap_or_else(|| panic!("「{what}」の行が無い:\n{s}"))
        .to_string()
}

fn names(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|x| (*x).to_string()).collect()
}

/// 並べ替えの窓の決まりの行(`arrow_name` と向きの語(昇順・降順)を含む行。ボタンの行は向きの語を持たない)。
fn rule_line(s: &str, arrow_name: &str) -> String {
    s.lines()
        .find(|l| l.contains(arrow_name) && (l.contains("昇順") || l.contains("降順")))
        .unwrap_or_else(|| panic!("窓に「{arrow_name}」の決まりの行が無い:\n{s}"))
        .to_string()
}

/// 並べ替えの窓の決まりの行の数。
fn rule_lines(s: &str) -> usize {
    s.lines()
        .filter(|l| {
            (l.contains("↑ ") || l.contains("↓ ")) && (l.contains("昇順") || l.contains("降順"))
        })
        .count()
}

/// 並べ替えのボタンの (文字, 効いているか)。
fn sort_button(a: &App) -> (String, bool) {
    let bar = super::bands::view_bar(a, a.size.0 as usize);
    let (_, t, _, on) = bar
        .buttons
        .iter()
        .find(|(_, _, b, _)| *b == super::bands::ViewButton::Sort)
        .expect("並べ替えのボタンがある");
    (t.clone(), *on)
}

// ---- base-view.table-sort ----

#[test]
fn test_table_sort_no_view() {
    // [utest -> req~base-view.table-sort~1] sort = date の降順の、ビューの無い表 → 行は b・c・a(ファイルの名前の順でない)。
    let (_t, a) = boot("ts-noview", ABC, DATE_DESC);
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]));
}

#[test]
fn test_table_sort_two_rules() {
    // [utest -> req~base-view.table-sort~1] sort = status の昇順、date の降順 → c・b・a(status の順、同じ status の中は新しい順)。
    let (_t, a) = boot(
        "ts-two",
        ABC,
        "sort = [{ property = \"status\", direction = \"ASC\" }, { property = \"date\", direction = \"DESC\" }]",
    );
    assert_eq!(labels(&a), names(&["c.md", "b.md", "a.md"]));
}

#[test]
fn test_table_sort_inline_view_and_own_view() {
    // [utest -> req~base-view.table-sort~1] 印の [[table.base.views]] の sort の無いビュー「全部」と、並べ替えを持たない mdgrid の
    // ビュー「下書き」 → どちらのタブも b・c・a。
    let marker = format!(
        "[[table]]\nname = \"台帳\"\n{DATE_DESC}\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n"
    );
    let tmp = Tmp::new("ts-inline");
    for (n, t) in ABC {
        tmp.write(n, t);
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let p = tmp.notes().join("台帳.base");
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    mdgrid::views::save_views(
        &cfg,
        &p,
        &[mdgrid::views::NativeView {
            name: "下書き".into(),
            ..Default::default()
        }],
    )
    .unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = start(&tmp, p, t.src, t.base);
    assert_eq!(a.view_names(), names(&["全部", "下書き"]));
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]), "「全部」");
    a.select_view_named("下書き");
    assert_eq!(a.view_index(), 1);
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]), "「下書き」");
}

#[test]
fn test_table_sort_own_view_wins() {
    // [utest -> req~base-view.table-sort-own-view~1] 表の既定は date の降順、ビュー「名前の順」は file.name の昇順 → a・b・c。
    let marker = format!(
        "[[table]]\nname = \"台帳\"\n{DATE_DESC}\n\n[[table.base.views]]\ntype = \"table\"\nname = \"名前の順\"\nsort = [{{ property = \"file.name\", direction = \"ASC\" }}]\n"
    );
    let (_t, a) = boot_inline("ts-own", ABC, &marker);
    assert_eq!(a.view_names(), names(&["名前の順"]));
    assert_eq!(labels(&a), names(&["a.md", "b.md", "c.md"]));
}

#[test]
fn test_table_sort_kanban_lane() {
    // [utest -> req~base-view.table-sort-outputs~1] sort = date の降順と kanban = status、どれも todo の alpha(11-20)・bravo(11-25)・
    // charlie(11-22) → B のカンバンの todo のレーンの札は上から bravo・charlie・alpha。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\ndate: 2026-11-20\nstatus: todo\n---\n"),
        ("bravo.md", "---\ndate: 2026-11-25\nstatus: todo\n---\n"),
        ("charlie.md", "---\ndate: 2026-11-22\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot(
        "ts-kanban",
        notes,
        &format!("{DATE_DESC}\nkanban = {{ column = \"status\" }}"),
    );
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban);
    let s = screen(&a);
    let row = |n: &str| {
        s.lines()
            .position(|l| l.contains(n))
            .unwrap_or_else(|| panic!("札 {n} が無い:\n{s}"))
    };
    assert!(row("bravo") < row("charlie"), "{s}");
    assert!(row("charlie") < row("alpha"), "{s}");
}

#[test]
fn test_table_sort_bad_rules() {
    // [utest -> req~base-view.table-sort-bad~1] property の無い決まりと direction = "DOWN" の決まりは警告して使わず、
    // 残りの { property = "title" }(向きを省くと昇順)で並べる → title の aa・mm・zz の順(b・c・a)。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: zz\ndate: 2026-11-20\n---\n"),
        ("b.md", "---\ntitle: aa\ndate: 2026-11-25\n---\n"),
        ("c.md", "---\ntitle: mm\ndate: 2026-11-22\n---\n"),
    ];
    let (_t, a) = boot(
        "ts-bad",
        notes,
        "sort = [{ direction = \"DESC\" }, { property = \"date\", direction = \"DOWN\" }, { property = \"title\" }]",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].sort[1].property"), "{m}");
    assert!(m.contains("table[1].sort[2].direction"), "{m}");
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]));
}

#[test]
fn test_table_sort_not_array() {
    // [utest -> req~base-view.table-sort-bad~1] sort = "date"(配列でない) → 警告に table[1].sort、既定の並べ替えは無い(名前の順)。
    let (_t, a) = boot("ts-notarray", ABC, "sort = \"date\"");
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].sort"), "{m}");
    assert_eq!(labels(&a), names(&["a.md", "b.md", "c.md"]));
}

#[test]
fn test_table_sort_missing_column() {
    // [utest -> req~base-view.table-sort-bad~1] property = "dat"(どのノートにも無い列) → 読み終えたら警告に table[1].sort[1].property と dat。
    let (_t, a) = boot(
        "ts-nocol",
        ABC,
        "sort = [{ property = \"dat\", direction = \"DESC\" }]",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].sort[1].property"), "{m}");
    assert!(m.contains("dat"), "{m}");
}

// ---- navigation.header-sort-cycle・navigation.sort-default-tiebreak ----

#[test]
fn test_header_cycle_back_to_table_default() {
    // [utest -> req~navigation.header-sort-cycle~2] sort = due の昇順の表で title の見出しで s を3回 → due の昇順に戻り、
    // 知らせは「並べ替えを解いた(既定の並び)」。
    let (_t, mut a) = boot("ts-cycle", ABCD, DUE_ASC);
    let due = names(&["c.md", "b.md", "d.md", "a.md"]);
    assert_eq!(labels(&a), due);
    col_named(&mut a, "title");
    for _ in 0..3 {
        ch(&mut a, 's');
    }
    assert_eq!(labels(&a), due);
    assert_eq!(a.message.as_deref(), Some("並べ替えを解いた(既定の並び)"));
}

#[test]
fn test_tiebreak_header_sort() {
    // [utest -> req~navigation.sort-default-tiebreak~1] sort = due の昇順の表で status の見出しで s → b・d・c・a
    // (status の昇順、同じ status の中は due の昇順)。
    let (_t, mut a) = boot("ts-tie-head", ABCD, DUE_ASC);
    col_named(&mut a, "status");
    ch(&mut a, 's');
    assert_eq!(a.settings.sorts, [("status".to_string(), Dir::Asc)]);
    assert_eq!(labels(&a), names(&["b.md", "d.md", "c.md", "a.md"]));
}

#[test]
fn test_tiebreak_window_rules() {
    // [utest -> req~navigation.sort-default-tiebreak~1] 窓で status の降順と title の昇順を決めた(title はどれも同じ) → c・a・b・d。
    let (_t, mut a) = boot("ts-tie-win", ABCD, DUE_ASC);
    a.set_sorts(vec![
        ("status".to_string(), Dir::Desc),
        ("title".to_string(), Dir::Asc),
    ]);
    assert_eq!(labels(&a), names(&["c.md", "a.md", "b.md", "d.md"]));
}

#[test]
fn test_tiebreak_same_column_not_doubled() {
    // [utest -> req~navigation.sort-default-tiebreak~1] due の見出しで s を2回(due の降順) → due の降順で、窓に既定の due は重ならない。
    let (_t, mut a) = boot("ts-tie-same", ABCD, DUE_ASC);
    col_named(&mut a, "due");
    ch(&mut a, 's');
    ch(&mut a, 's');
    assert_eq!(labels(&a), names(&["a.md", "d.md", "b.md", "c.md"]));
    ch(&mut a, 'S');
    let s = screen(&a);
    assert_eq!(rule_lines(&s), 1, "{s}");
    assert!(!rule_line(&s, "↓ due").contains("既定"), "{s}");
}

// ---- navigation.sort-window-default・navigation.sort-window-reset ----

#[test]
fn test_window_lists_table_default() {
    // [utest -> req~navigation.sort-window-default~1] sort = date の降順、sort の無いビュー → 窓の決まりの行は「↓ date」に
    // 降順と「既定・印」を添えた1行で、右の端に「×」が無い。
    let (_t, mut a) = boot("ts-win-def", ABC, DATE_DESC);
    ch(&mut a, 'S');
    assert_eq!(a.mode, Mode::Sorts);
    let s = screen(&a);
    assert_eq!(rule_lines(&s), 1, "{s}");
    let l = rule_line(&s, "↓ date");
    assert!(l.contains("降順") && l.contains("既定・印"), "{s}");
    assert!(!l.contains('×'), "{s}");
}

#[test]
fn test_window_lists_view_sort() {
    // [utest -> req~navigation.sort-window-default~1] ビューの sort が due の昇順の .base のビュー → 窓の決まりの行は「↑ due」と「既定・ビュー」。
    let marker = "[[table]]\nname = \"台帳\"\n\n[[table.base.views]]\ntype = \"table\"\nname = \"期限\"\nsort = [{ property = \"due\", direction = \"ASC\" }]\n";
    let (_t, mut a) = boot_inline("ts-win-view", ABCD, marker);
    assert_eq!(labels(&a), names(&["c.md", "b.md", "d.md", "a.md"]));
    ch(&mut a, 'S');
    let s = screen(&a);
    assert!(rule_line(&s, "↑ due").contains("既定・ビュー"), "{s}");
}

#[test]
fn test_window_enter_on_default_makes_it_own() {
    // [utest -> req~navigation.sort-window-default~1] 窓の date(既定)の行で Enter → 「↑ date」で既定を添えず、表は date の昇順。
    let (_t, mut a) = boot("ts-win-enter", ABC, DATE_DESC);
    ch(&mut a, 'S');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.settings.sorts, [("date".to_string(), Dir::Asc)]);
    assert_eq!(labels(&a), names(&["a.md", "c.md", "b.md"]));
    let s = screen(&a);
    assert!(!rule_line(&s, "↑ date").contains("既定"), "{s}");
}

#[test]
fn test_window_default_not_removable() {
    // [utest -> req~navigation.sort-window-default~1] 窓の date(既定)の行で d(外す) → 行は「↓ date」(既定・印)のまま、外せない知らせ。
    let (_t, mut a) = boot("ts-win-rm", ABC, DATE_DESC);
    ch(&mut a, 'S');
    ch(&mut a, 'd');
    assert_eq!(a.mode, Mode::Sorts);
    let s = screen(&a);
    assert!(rule_line(&s, "↓ date").contains("既定・印"), "{s}");
    assert!(
        a.message.as_deref().is_some_and(|m| m.contains("外せない")),
        "{:?}",
        a.message
    );
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]));
}

#[test]
fn test_window_native_view_sort_is_own() {
    // [utest -> req~navigation.sort-window-default~1] 表の既定は date の降順、定義の並べ替えが due の昇順の mdgrid のビュー →
    // 窓の決まりの行は「↑ due」(×、既定を添えない)と「↓ date」(既定・印)の2行。
    let tmp = Tmp::new("ts-win-native");
    for (n, t) in ABCD {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes\"\n{DATE_DESC}\n"),
    )
    .unwrap();
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    mdgrid::views::save_views(
        &cfg,
        &tmp.notes(),
        &[mdgrid::views::NativeView {
            name: "期限".into(),
            settings: mdgrid::settings::Settings {
                sorts: vec![("due".to_string(), Dir::Asc)],
                ..Default::default()
            },
            ..Default::default()
        }],
    )
    .unwrap();
    let mut a = start_folder(&tmp);
    a.select_view_named("期限");
    assert_eq!(labels(&a), names(&["c.md", "b.md", "d.md", "a.md"]));
    ch(&mut a, 'S');
    let s = screen(&a);
    assert_eq!(rule_lines(&s), 2, "{s}");
    let due = rule_line(&s, "↑ due");
    assert!(due.contains('×') && !due.contains("既定"), "{s}");
    assert!(rule_line(&s, "↓ date").contains("既定・印"), "{s}");
}

#[test]
fn test_window_reset_to_default() {
    // [utest -> req~navigation.sort-window-reset~1] title の見出しで s のあと、窓の「既定に戻す」 → date の降順に戻り、窓は
    // 「↓ date」(既定・印)だけ、開き直しても date の降順。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: aa\ndate: 2026-11-20\n---\n"),
        ("b.md", "---\ntitle: zz\ndate: 2026-11-25\n---\n"),
        ("c.md", "---\ntitle: mm\ndate: 2026-11-22\n---\n"),
    ];
    let (tmp, mut a) = boot("ts-reset", notes, DATE_DESC);
    col_named(&mut a, "title");
    ch(&mut a, 's');
    assert_eq!(labels(&a), names(&["a.md", "c.md", "b.md"]));
    ch(&mut a, 'S');
    let s = screen(&a);
    assert!(s.contains("既定に戻す"), "{s}");
    // 決まりの行(title・既定の date)、足す行、既定に戻す の順。
    for _ in 0..3 {
        ch(&mut a, 'j');
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(labels(&a), names(&["b.md", "c.md", "a.md"]));
    let s = screen(&a);
    assert_eq!(rule_lines(&s), 1, "{s}");
    assert!(rule_line(&s, "↓ date").contains("既定・印"), "{s}");
    ch(&mut a, 'q');
    let b = start_folder(&tmp);
    assert_eq!(labels(&b), names(&["b.md", "c.md", "a.md"]));
}

#[test]
fn test_window_no_reset_when_unchanged() {
    // [utest -> req~navigation.sort-window-reset~1] 画面で並べ替えを変えていない → 窓に「既定に戻す」が無い。
    let (_t, mut a) = boot("ts-noreset", ABC, DATE_DESC);
    ch(&mut a, 'S');
    let s = screen(&a);
    assert!(s.contains("並べ替えを足す"), "{s}");
    assert!(!s.contains("既定に戻す"), "{s}");
}

// ---- navigation.sort-button-default ----

#[test]
fn test_button_shows_default_off() {
    // [utest -> req~navigation.sort-button-default~1] sort = date の降順と title の昇順 → ボタンは「↓ date +1」で効いていない見た目。
    let (_t, mut a) = boot(
        "ts-btn",
        ABCD,
        "sort = [{ property = \"date\", direction = \"DESC\" }, { property = \"title\", direction = \"ASC\" }]",
    );
    let (t, on) = sort_button(&a);
    assert!(t.contains("↓ date +1"), "{t}");
    assert!(!on, "既定のままは効いていない見た目");
    // [utest -> req~navigation.sort-button-default~1] title の見出しで s → ボタンは「↑ title」で効いている見た目。
    col_named(&mut a, "title");
    ch(&mut a, 's');
    let (t, on) = sort_button(&a);
    assert!(t.contains("↑ title") && !t.contains('+'), "{t}");
    assert!(on, "画面で変えたら効いている見た目");
}

// ---- navigation.settings-sort-default ----

#[test]
fn test_settings_section_lists_default() {
    // [utest -> req~navigation.settings-sort-default~1] sort = date の降順 → ビューの設定の並べ替えの区画に「1. date ↓」と
    // 「既定・印」の行と「+ 並べ替えを足す」。
    let (_t, mut a) = boot("ts-set", ABC, DATE_DESC);
    open(&mut a);
    section(&mut a, super::settings::Sec::Sorts);
    let s = screen(&a);
    assert!(line_with(&s, "1. date ↓").contains("既定・印"), "{s}");
    assert!(s.contains("+ 並べ替えを足す"), "{s}");
}

#[test]
fn test_settings_section_edit_default() {
    // [utest -> req~navigation.settings-sort-default~1] 区画の date(既定)の行で Enter のあと反映 → 表は date の昇順、
    // ビューの設定の並べ替えは date の昇順。
    let (_t, mut a) = boot("ts-set-edit", ABC, DATE_DESC);
    open(&mut a);
    section(&mut a, super::settings::Sec::Sorts);
    assert_eq!(draft(&a).at(super::settings::Sec::Sorts), 0);
    press(&mut a, KeyCode::Enter);
    apply_btn(&mut a);
    assert_eq!(a.settings.sorts, [("date".to_string(), Dir::Asc)]);
    assert_eq!(labels(&a), names(&["a.md", "c.md", "b.md"]));
}
