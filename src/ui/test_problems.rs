//! 型と決まりに合わない値(問題)の画面の受け入れ試験(CV-9・NV-30・CE-36 の候補)。
//! 記録: specs/_changes/2026-10-14-type-check.md(タスク #2・#3)。

use super::startup::Startup;
use super::test_screen::{ch, press, text, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 3つの問題のある表の設定: due は日付、status は必須で todo・doing・done だけ。
const RULES: &str = "[types]\ndue = \"date\"\n\n[rules.status]\nrequired = true\nvalues = [\"todo\", \"doing\", \"done\"]\n";

/// 問題: b の due(日付ではない)・c の status(キーが無い)・d の status(選べる値ではない)。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ndue: 2026-10-01\nstatus: todo\n---\n"),
    ("b.md", "---\ndue: someday\nstatus: todo\n---\n"),
    ("c.md", "---\ndue: 2026-10-03\n---\n"),
    ("d.md", "---\ndue: 2026-10-04\nstatus: wip\n---\n"),
    ("e.md", "---\ndue: 2026-10-05\nstatus: done\n---\n"),
];

/// 問題のセル(ファイル名・列)と、跳んだときのメッセージ。
const PROBLEMS: &[(&str, &str, &str)] = &[
    ("b.md", "due", "due: 型が合わない(日付ではない)"),
    ("c.md", "status", "status: 必須のキーが無い"),
    ("d.md", "status", "status: 選べる値ではない"),
];

fn boot(name: &str, notes: &[(&str, &str)], toml: &str, readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn file_of(id: &str) -> String {
    id.rsplit(['/', '\\']).next().unwrap().to_string()
}

/// 今のセル(ファイル名・列の名前)。
fn here(a: &App) -> (String, String) {
    (file_of(&a.rows[a.row].0), a.cols[a.col].clone())
}

/// 表の行の順(同じ行では列の順)に並べた問題(ファイル名・列・メッセージ)。
fn ordered(a: &App) -> Vec<(String, String, &'static str)> {
    let mut out = Vec::new();
    for r in &a.rows {
        let file = file_of(&r.0);
        for c in &a.cols {
            if let Some(p) = PROBLEMS.iter().find(|p| p.0 == file && p.1 == c) {
                out.push((file.clone(), c.clone(), p.2));
            }
        }
    }
    out
}

fn palette(a: &mut App, name: &str) {
    ch(a, ':');
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// ヘッダー(行の数「N行」のある行)。メッセージ行の文と分けて見るため。
fn header(s: &str) -> &str {
    s.lines()
        .find(|l| l.contains("行  ") || l.trim_end().ends_with('行'))
        .unwrap_or_else(|| panic!("ヘッダーが無い:\n{s}"))
}

/// 画面の中で `word` を含む最初の行。
fn line_with<'s>(s: &'s str, word: &str) -> &'s str {
    s.lines()
        .find(|l| l.contains(word))
        .unwrap_or_else(|| panic!("「{word}」の行が無い:\n{s}"))
}

#[test]
fn test_cv_9_rule_violation_marked() {
    // [CV-9] values に無い値(wip)と、型と合わない値(someday)のセルに `!`。合う値(todo・done・日付)には付かない。
    let (_t, a) = boot("cv9_mark", NOTES, RULES, false);
    let s = screen(&a);
    assert!(s.contains("!wip"), "{s}");
    assert!(s.contains("!someday"), "{s}");
    for ok in ["todo", "done", "2026-10-01"] {
        assert!(
            !s.contains(&format!("!{ok}")),
            "「{ok}」に印が付いている:\n{s}"
        );
    }
    assert!(
        !line_with(&s, "2026-10-05").contains('!'),
        "問題の無い行(e)に印:\n{s}"
    );
}

#[test]
fn test_cv_9_required_missing_key_marked() {
    // [CV-9] required = true で status のキーが無いノート(c)の空のセルに `!`。
    let (_t, a) = boot("cv9_req", NOTES, RULES, false);
    let s = screen(&a);
    assert!(line_with(&s, "2026-10-03").contains('!'), "{s}");
}

#[test]
fn test_nv_30_next_problem_wraps_with_reason() {
    // [NV-30] `!` で表の順に次の問題へ跳び、メッセージ行に「列: 理由」。3つめのあとの `!` で1つめに戻る。
    let (_t, mut a) = boot("nv30_next", NOTES, RULES, false);
    let want = ordered(&a);
    assert_eq!(want.len(), 3);
    let start = here(&a);
    assert!(
        !want.iter().any(|w| (w.0.clone(), w.1.clone()) == start),
        "前提: 始めのセルは問題でない"
    );
    for w in want.iter().chain(want.iter().take(1)) {
        ch(&mut a, '!');
        assert_eq!(here(&a), (w.0.clone(), w.1.clone()), "{}", screen(&a));
        let s = screen(&a);
        assert!(s.contains(w.2), "メッセージ「{}」が無い:\n{s}", w.2);
    }
}

#[test]
fn test_nv_30_prev_problem() {
    // [NV-30] `@` で前の問題へ。1つめから `@` → 端を回って3つめ、もう一度 → 2つめ。
    let (_t, mut a) = boot("nv30_prev", NOTES, RULES, false);
    let want = ordered(&a);
    ch(&mut a, '!');
    assert_eq!(here(&a), (want[0].0.clone(), want[0].1.clone()));
    for w in [&want[2], &want[1]] {
        ch(&mut a, '@');
        assert_eq!(here(&a), (w.0.clone(), w.1.clone()), "{}", screen(&a));
        assert!(screen(&a).contains(w.2), "{}", screen(&a));
    }
}

#[test]
fn test_nv_30_problem_count_in_bottom_band() {
    // [NV-30] 下の帯に問題の数「問題 3」。
    let (_t, a) = boot("nv30_count", NOTES, RULES, false);
    let s = screen(&a);
    assert!(s.contains("問題 3"), "{s}");
}

#[test]
fn test_nv_30_no_problems() {
    // [NV-30] 問題の無い表 → 帯に問題の数が出ず、`!` で「問題は無い」を出して動かない。
    let notes = &[NOTES[0], NOTES[4]];
    let (_t, mut a) = boot("nv30_none", notes, RULES, false);
    assert!(!screen(&a).contains("問題 "), "{}", screen(&a));
    let before = here(&a);
    ch(&mut a, '!');
    assert_eq!(here(&a), before);
    assert!(screen(&a).contains("問題は無い"), "{}", screen(&a));
}

#[test]
fn test_nv_30_problems_only_toggle() {
    // [NV-30] パレットの problems_only → 問題のある行(b・c・d)だけ、ヘッダーに「問題のある行だけ」。
    // もう一度 → 元の5行に戻る。
    let (_t, mut a) = boot("nv30_only", NOTES, RULES, false);
    assert_eq!(a.rows.len(), 5);
    palette(&mut a, "problems_only");
    let s = screen(&a);
    assert_eq!(a.rows.len(), 3, "{s}");
    assert!(header(&s).contains("問題のある行だけ"), "{s}");
    assert!(!s.contains("2026-10-01"), "問題の無い行(a)が見える:\n{s}");
    palette(&mut a, "problems_only");
    assert_eq!(a.rows.len(), 5);
    let s = screen(&a);
    assert!(!header(&s).contains("問題のある行だけ"), "{s}");
}

#[test]
fn test_nv_30_problems_only_esc_clears() {
    // [NV-30] 絞っている間に Esc → 外れて元の行に戻る。
    let (_t, mut a) = boot("nv30_esc", NOTES, RULES, false);
    palette(&mut a, "problems_only");
    assert_eq!(a.rows.len(), 3);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.rows.len(), 5);
    let s = screen(&a);
    assert!(!header(&s).contains("問題のある行だけ"), "{s}");
}

#[test]
fn test_nv_30_click_count_toggles() {
    // [NV-30] 下の帯の「問題 3」のクリック → 問題のある行だけに絞る。
    let (_t, mut a) = boot("nv30_click", NOTES, RULES, false);
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("問題 3"))
        .unwrap_or_else(|| panic!("「問題 3」が無い:\n{s}"));
    let x = width::width(&line[..line.find("問題 3").unwrap()]);
    a.click(x as u16 + 1, y as u16);
    assert_eq!(a.rows.len(), 3, "{}", screen(&a));
    let s = screen(&a);
    assert!(header(&s).contains("問題のある行だけ"), "{s}");
}

#[test]
fn test_nv_30_readonly() {
    // [NV-30][WB-15] 読むだけの起動でも `!` で跳び、problems_only で絞れる。
    let (_t, mut a) = boot("nv30_ro", NOTES, RULES, true);
    let want = ordered(&a);
    ch(&mut a, '!');
    assert_eq!(here(&a), (want[0].0.clone(), want[0].1.clone()));
    palette(&mut a, "problems_only");
    assert_eq!(a.rows.len(), 3, "{}", screen(&a));
}

#[test]
fn test_ce_36_values_listed_as_candidates() {
    // [CE-36] values のある文字の列(status)で Enter → 候補のリストに values の todo・doing・done が並ぶ
    // (ノートに無い doing も)。
    let (_t, mut a) = boot("ce36_list", NOTES, RULES, false);
    a.row = a.rows.iter().position(|r| r.0.ends_with("a.md")).unwrap();
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    assert_eq!(format!("{:?}", a.mode), "Edit");
    let items: Vec<String> = a
        .active_list()
        .unwrap_or_else(|| panic!("候補のリストが無い:\n{}", screen(&a)))
        .items
        .iter()
        .map(|i| i.text.clone())
        .collect();
    for v in ["todo", "doing", "done"] {
        assert!(
            items.iter().any(|i| i == v),
            "候補に「{v}」が無い: {items:?}"
        );
    }
    assert!(screen(&a).contains("doing"), "{}", screen(&a));
}

#[test]
fn test_nv_30_folded_group_not_counted_or_jumped() {
    // [NV-30] まとまりを畳むと、その中の問題は下の帯の数にも跳ぶ先にも入れない(数と跳べる先をそろえる)。
    // status でまとめ、b(due が日付でない)のある todo のまとまりを畳む → 数は 2、`!` は b に止まらない。
    let (_t, mut a) = boot("nv30_fold", NOTES, RULES, false);
    a.settings.group = mdgrid::settings::Group::By {
        col: "status".into(),
        dir: mdgrid::settings::Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    assert_eq!(a.built.problems, 3, "{}", screen(&a));
    a.row = a
        .slots
        .iter()
        .position(|s| matches!(s, grid::Slot::Head(g) if a.groups[*g].0.contains("todo")))
        .expect("todo の見出し");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.built.problems, 2, "{}", screen(&a));
    assert!(screen(&a).contains("問題 2"), "{}", screen(&a));
    for _ in 0..4 {
        ch(&mut a, '!');
        let grid::Slot::Row(r) = a.slots[a.row] else {
            panic!("ノートの行でない: {}", screen(&a));
        };
        assert_ne!(file_of(&a.rows[r].0), "b.md", "{}", screen(&a));
    }
}

#[test]
fn test_ce_36_number_values_listed_as_numbers() {
    // [CE-36] 数の列(`[types] priority = "number"`)の values も候補に並び、選ぶと数として書く(文字にしない)。
    let notes: &[(&str, &str)] = &[("a.md", "---\npriority: 1\n---\n")];
    let toml =
        "[types]\npriority = \"number\"\n\n[rules.priority]\nvalues = [\"1\", \"2\", \"3\"]\n";
    let (_t, mut a) = boot("ce36_num", notes, toml, false);
    a.row = 0;
    a.col = a.cols.iter().position(|c| c == "priority").unwrap();
    press(&mut a, KeyCode::Enter);
    let items: Vec<String> = a
        .active_list()
        .unwrap_or_else(|| panic!("候補のリストが無い:\n{}", screen(&a)))
        .items
        .iter()
        .map(|i| i.text.clone())
        .collect();
    for v in ["1", "2", "3"] {
        assert!(
            items.iter().any(|i| i == v),
            "候補に「{v}」が無い: {items:?}"
        );
    }
    // 3 を選んで決める → ためた値は数の 3。
    let three = items.iter().position(|i| i == "3").unwrap();
    let now = items.iter().position(|i| i == "1").unwrap();
    for _ in now..three {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    let row = a.rows[0].clone();
    assert_eq!(
        a.changes.pending(&row, "priority"),
        Some(&mdgrid::source::NewValue::Int(3)),
        "{}",
        screen(&a)
    );
}
