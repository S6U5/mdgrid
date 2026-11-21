//! ワークスペースを横断して探す(specs/requirements/navigation.md の navigation.across-*)の画面の試験。
//! 期待値は要求の受け入れの例から取った: tasks に「見積もり」を含む行が2つ、projects に1つ → 「tasks」の下に2行、
//! 「projects」の下に1行。どこにも当たらない → 「当たる行がありません」。表は裏の糸で読むので、`across_poll` で
//! 拾い終えるまで待つ(`settle`)。読んでいる途中の画面は、裏の糸の代わりに試験が結果を送って確かめる。

use super::across::Hit;
use super::keymap::{Action, Mode};
use super::test_screen::{make, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::Place;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// tasks の表: 「見積もり」を含む行が2つ(題とメモ)、含まない行が1つ。
const TASKS: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 見積もりを出す\nstatus: todo\n---\n"),
    (
        "b.md",
        "---\ntitle: 請求書\nmemo: 見積もりの数字を写す\n---\n",
    ),
    ("c.md", "---\ntitle: 歯医者\nstatus: done\n---\n"),
];

/// 横断の検索を入れた設定を当てる。
fn across_on(a: &mut App) {
    a.configure(&Config {
        search_across: true,
        ..Config::default()
    });
}

/// projects の表(ノート「見積もりの返事」と、当たらない1つ)を tmp の中に足し、tasks(tmp のノートのフォルダ)と
/// projects を範囲にする。`missing` なら、無いフォルダの表 gone も足す。
fn workspace(a: &mut App, tmp: &Tmp, missing: bool) -> PathBuf {
    // tasks のフォルダの外に置く(中に置くと、tasks の表にも入る)。
    let proj = tmp.notes().parent().unwrap().join("projects");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("見積もりの返事.md"), "---\nclient: 森\n---\n").unwrap();
    std::fs::write(proj.join("看板.md"), "---\nclient: 林\n---\n").unwrap();
    let mut tables = vec![
        WsTable {
            name: "tasks".into(),
            path: tmp.notes().canonicalize().unwrap(),
            ..WsTable::default()
        },
        WsTable {
            name: "projects".into(),
            path: proj.canonicalize().unwrap(),
            ..WsTable::default()
        },
    ];
    if missing {
        tables.push(WsTable {
            name: "gone".into(),
            path: tmp.notes().join("gone"),
            ..WsTable::default()
        });
    }
    a.scope = Some(Scope {
        name: "Work".into(),
        source: Source::Chosen,
        tables,
        profile: Default::default(),
        warnings: Vec::new(),
    });
    proj.canonicalize().unwrap()
}

/// 裏の糸が読み終えるまで拾う(上限 10 秒)。
fn settle(a: &mut App) {
    let end = Instant::now() + Duration::from_secs(10);
    while a.across_busy() && Instant::now() < end {
        a.across_poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    a.across_poll();
}

/// パレットの「ワークスペースを横断して探す」で `word` を探し、読み終えるまで待つ。
fn search(a: &mut App, word: &str) {
    press(a, KeyCode::Char(':'));
    typing(a, "across_search");
    press(a, KeyCode::Enter);
    assert!(matches!(a.nv.ask, Some(super::native_views::Ask::Across)));
    typing(a, word);
    press(a, KeyCode::Enter);
    settle(a);
}

/// 画面の行(前後の空白を除く)。
fn lines(a: &App) -> Vec<String> {
    screen(a).lines().map(|l| l.trim().to_string()).collect()
}

fn label() -> &'static str {
    mdgrid::i18n::Msg::CmdAcrossSearch.text()
}

#[test]
fn test_across_option_off_by_default() {
    // [utest -> req~navigation.across-option~1] 設定に [search] が無い → パレットにもヘルプにも出ない。across = true → 出る。
    let (_t, mut a) = make("across-option", TASKS);
    let (c, w) = mdgrid::config::parse("").unwrap();
    assert!(w.is_empty() && !c.search_across);
    a.configure(&c);
    let cands = help::candidates(&a, "across");
    assert!(cands.iter().all(|c| c.label != label()), "既定で出ている");
    let help: Vec<String> = help::help_lines(&a, Mode::Table, 79)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(!help.iter().any(|l| l.contains("across_search")));
    let (c, w) = mdgrid::config::parse("[search]\nacross = true\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    assert!(c.search_across);
    a.configure(&c);
    let cands = help::candidates(&a, "across");
    assert_eq!(cands.first().map(|c| c.label.as_str()), Some(label()));
    let help: Vec<String> = help::help_lines(&a, Mode::Table, 79)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        help.iter().any(|l| l.contains(":across_search")),
        "{help:#?}"
    );
}

#[test]
fn test_across_option_wrong_type_warns() {
    // [utest -> req~navigation.across-option~1] 真偽でない値・知らない項目は警告して既定(出さない)のまま。
    let (c, w) = mdgrid::config::parse("[search]\nacross = \"yes\"\nfast = true\n").unwrap();
    assert!(!c.search_across);
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().any(|x| x.contains("search.across")), "{w:?}");
    assert!(w.iter().any(|x| x.contains("search.fast")), "{w:?}");
}

#[test]
fn test_across_results_grouped_per_table() {
    // [utest -> req~navigation.across-results~1] tasks に2行・projects に1行 → 表ごとに、ノートの名前と当たった列と値を添えて並ぶ。
    let (t, mut a) = make("across-results", TASKS);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    search(&mut a, "見積もり");
    let ls = lines(&a);
    let at = |s: &str| ls.iter().position(|l| l.starts_with(s));
    let tasks = at("tasks(2)").unwrap_or_else(|| panic!("{ls:#?}"));
    let projects = at("projects(1)").unwrap_or_else(|| panic!("{ls:#?}"));
    assert_eq!(projects, tasks + 3, "tasks の下に2行: {ls:#?}");
    // 行は「(選びの印) ノートの名前 列: 値」。
    let name = |l: &str| {
        l.trim_start_matches('>')
            .split_whitespace()
            .next()
            .map(str::to_string)
    };
    let under_tasks = &ls[tasks + 1..projects];
    assert!(
        under_tasks
            .iter()
            .any(|l| name(l).as_deref() == Some("a") && l.contains("title: 見積もりを出す")),
        "{ls:#?}"
    );
    assert!(
        under_tasks
            .iter()
            .any(|l| name(l).as_deref() == Some("b") && l.contains("memo: 見積もりの数字を写す")),
        "{ls:#?}"
    );
    assert!(
        ls[projects + 1].contains("見積もりの返事")
            && ls[projects + 1].contains("ノート: 見積もりの返事"),
        "{ls:#?}"
    );
    assert!(!screen(&a).contains("看板") && !screen(&a).contains("歯医者"));
}

#[test]
fn test_across_results_same_matching_as_search() {
    // [utest -> req~navigation.across-results~1] 表の中の検索と同じ当て方: 小文字の語は大文字小文字を区別せず、大文字を含む語は区別する。
    let notes = [
        ("a.md", "---\ntitle: Todo list\n---\n"),
        ("b.md", "---\ntitle: todo later\n---\n"),
    ];
    let (t, mut a) = make("across-smartcase", &notes);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    search(&mut a, "todo");
    assert!(lines(&a).iter().any(|l| l.starts_with("tasks(2)")));
    press(&mut a, KeyCode::Esc);
    search(&mut a, "Todo");
    let ls = lines(&a);
    assert!(ls.iter().any(|l| l.starts_with("tasks(1)")), "{ls:#?}");
    assert_eq!(a.across_hit_count(), 1);
    assert!(ls.iter().any(|l| l.contains("title: Todo list")), "{ls:#?}");
}

#[test]
fn test_across_results_none() {
    // [utest -> req~navigation.across-results~1] どこにも当たらない → 「当たる行がありません」。
    let (t, mut a) = make("across-none", TASKS);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    search(&mut a, "請求書の束");
    assert!(
        screen(&a).contains("当たる行がありません"),
        "{}",
        screen(&a)
    );
    assert_eq!(a.across_hit_count(), 0);
}

#[test]
fn test_across_open_other_table() {
    // [utest -> req~navigation.across-open~1] projects の「見積もりの返事」で Enter → projects へ移り、検索の語とその行を渡す。
    let (t, mut a) = make("across-open", TASKS);
    let proj = workspace(&mut a, &t, false);
    across_on(&mut a);
    search(&mut a, "見積もり");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    assert!(screen(&a).contains("> 見積もりの返事"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert!(a.switch_to.as_ref().is_some_and(|p| p.name == "projects"));
    let row = proj.join("見積もりの返事.md");
    assert_eq!(a.switch_select.as_deref(), Some(row.as_path()));
    assert_eq!(
        a.switch_search,
        Some((proj.clone(), "見積もり".to_string()))
    );
    assert!(a.quit, "ためた変更が無ければすぐ移る");
    assert!(a.palette.is_none());
}

#[test]
fn test_across_open_asks_when_unsaved() {
    // [utest -> req~navigation.across-open~1, req~workspace.drawer-unsaved~1] ためた変更があれば、引き出しと同じく保存か取りやめを確かめる。取りやめで移らない。
    let (t, mut a) = make("across-unsaved", TASKS);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    super::test_screen::edit_cell(&mut a, 2, "title", "歯医者に行く");
    assert_eq!(a.changes.count(), 1);
    search(&mut a, "見積もり");
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    assert!(!a.quit);
    a.apply(Action::Back);
    assert!(a.switch_to.is_none());
    assert!(a.switch_search.is_none());
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_across_open_this_table() {
    // [utest -> req~navigation.across-open~1] 今の表の行なら、その場で検索の欄に語を入れ、その行を選ぶ。
    let (t, mut a) = make("across-here", TASKS);
    workspace(&mut a, &t, false);
    a.nv.target = t.notes().canonicalize().unwrap();
    across_on(&mut a);
    search(&mut a, "見積もり");
    let hits: Vec<Hit> = a
        .across
        .found
        .iter()
        .flat_map(|(_, r)| r.clone().unwrap_or_default())
        .collect();
    let k = hits.iter().position(|h| h.note == "b").unwrap();
    for _ in 0..k {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    assert!(a.switch_to.is_none() && !a.quit);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.filter.as_deref(), Some("見積もり"));
    assert_eq!(a.rows.len(), 2, "検索で2行に絞る");
    let (row, _) = a.selected().unwrap();
    assert_eq!(a.src.label(&row), "b.md");
}

#[test]
fn test_across_progress_shows_tables_read_and_results_so_far() {
    // [utest -> req~navigation.across-progress~1] 5 つの表のうち 3 つを読んだ → 読んだ表の結果が並び、「3 / 5 を読みました」。
    let (t, mut a) = make("across-progress", TASKS);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    a.apply(Action::AcrossSearch);
    typing(&mut a, "見積もり");
    press(&mut a, KeyCode::Enter);
    // 裏の糸の代わりに試験が結果を送る(届くのは今の世代の分だけ)。
    a.across_stop();
    let (tx, rx) = std::sync::mpsc::channel();
    a.across.rx = Some(rx);
    a.across.total = 5;
    let generation = a.across.generation;
    let place = |name: &str| Place {
        name: name.into(),
        group: String::new(),
        path: PathBuf::from(name),
        view: None,
    };
    let hit = |note: &str| Hit {
        row: format!("/x/{note}.md"),
        note: note.into(),
        col: "title".into(),
        value: format!("{note}の見積もり"),
    };
    tx.send((generation, (place("tasks"), Ok(vec![hit("a")]))))
        .unwrap();
    tx.send((generation, (place("people"), Ok(Vec::new()))))
        .unwrap();
    tx.send((generation, (place("gone"), Err("無い".into()))))
        .unwrap();
    tx.send((generation - 1, (place("old"), Ok(vec![hit("old")]))))
        .unwrap();
    a.across_poll();
    assert!(a.across_busy());
    let s = screen(&a);
    assert!(s.contains("3 / 5 を読みました"), "{s}");
    assert!(s.contains("tasks(1)") && s.contains("aの見積もり"), "{s}");
    assert!(!s.contains("oldの見積もり"), "古い世代の結果は捨てる: {s}");
    assert!(s.contains("読めなかった表: gone"), "{s}");
    // 残りが届くと進みの行が消える。
    tx.send((generation, (place("projects"), Ok(vec![hit("p")]))))
        .unwrap();
    tx.send((generation, (place("clients"), Ok(Vec::new()))))
        .unwrap();
    a.across_poll();
    assert!(!a.across_busy());
    let s = screen(&a);
    assert!(!s.contains("を読みました"), "{s}");
    assert!(s.contains("projects(1)"), "{s}");
}

#[test]
fn test_across_progress_esc_stops_and_returns() {
    // [utest -> req~navigation.across-progress~1] 読んでいる途中で Esc → 止めて元の表の画面のまま(行も選びも変えない)。
    let (t, mut a) = make("across-esc", TASKS);
    workspace(&mut a, &t, false);
    across_on(&mut a);
    a.row = 1;
    let before = (a.rows.len(), a.row, a.filter.clone());
    a.apply(Action::AcrossSearch);
    typing(&mut a, "見積もり");
    press(&mut a, KeyCode::Enter);
    a.across_stop();
    let (tx, rx) = std::sync::mpsc::channel();
    a.across.rx = Some(rx);
    a.across.total = 2;
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.palette.is_none());
    assert!(!a.across_busy(), "止めた");
    assert!(
        tx.send((
            a.across.generation,
            (
                Place {
                    name: "tasks".into(),
                    group: String::new(),
                    path: PathBuf::new(),
                    view: None,
                },
                Ok(Vec::new())
            )
        ))
        .is_err(),
        "結果を受ける口を閉じた"
    );
    assert_eq!((a.rows.len(), a.row, a.filter.clone()), before);
    assert!(!screen(&a).contains("を読みました"));
}

#[test]
fn test_across_progress_unreadable_table_is_named_at_the_end() {
    // [utest -> req~navigation.across-progress~1] 読めない表は飛ばし、結果の終わりに読めなかった表の名前。
    let (t, mut a) = make("across-gone", TASKS);
    workspace(&mut a, &t, true);
    across_on(&mut a);
    search(&mut a, "見積もり");
    let ls = lines(&a);
    let projects = ls
        .iter()
        .position(|l| l.starts_with("projects(1)"))
        .unwrap();
    let gone = ls
        .iter()
        .position(|l| l.starts_with("読めなかった表: gone"))
        .unwrap_or_else(|| panic!("{ls:#?}"));
    assert!(gone > projects, "{ls:#?}");
    assert!(ls.iter().any(|l| l.starts_with("tasks(2)")));
}
