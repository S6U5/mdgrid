//! リレーションの画面(REL-2・REL-3・REL-10。行き先を開く・つながった行は test_relations_open.rs)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

/// notes/ の下に tasks・projects・members。tasks の行が表の行。
pub(super) fn workspace(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("projects/mdgrid.md", "---\nstatus: active\n---\n");
    w("projects/Other Project.md", "---\nstatus: idea\n---\n");
    w("members/Alice.md", "---\nrole: dev\n---\n");
    w(
        "tasks/a build.md",
        "---\nproject: \"[[mdgrid]]\"\nassignee: ../members/Alice\nrelated:\n  - \"[[mdgrid]]\"\nstatus: done\n---\n",
    );
    w(
        "tasks/b docs.md",
        "---\nproject: \"[[nothing]]\"\nassignee: ../members/Alice\nrelated:\n  - \"[[mdgrid]]\"\n  - \"[[ghost]]\"\nstatus: todo\n---\n",
    );
    w(
        "tasks/c test.md",
        "---\nproject: \"[mdgrid](../projects/mdgrid.md)\"\nstatus: todo\n---\n",
    );
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

/// tasks のフォルダを開く(projects・members は登録した表)。
pub(super) fn boot_tasks(tmp: &Tmp) -> App {
    let cfg = tmp.0.join("config");
    for (name, dir) in [
        ("Projects", "projects"),
        ("Members", "members"),
        ("Tasks", "tasks"),
    ] {
        places::save(
            &cfg,
            Place {
                name: name.into(),
                group: String::new(),
                path: tmp.notes().join(dir).canonicalize().unwrap(),
                view: None,
            },
        )
        .unwrap();
    }
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

pub(super) fn row_of(a: &App, name: &str) -> RowId {
    a.src
        .rows()
        .into_iter()
        .find(|r| r.0.ends_with(&format!("{name}.md")))
        .unwrap_or_else(|| panic!("{name} の行が無い"))
}

/// その行・列に移る。
pub(super) fn goto(a: &mut App, name: &str, col: &str) {
    let r = row_of(a, name);
    assert!(a.select_note(std::path::Path::new(&r.0)), "{name} の位置");
    a.col = a
        .cols
        .iter()
        .position(|c| c == col)
        .unwrap_or_else(|| panic!("{col} の列"));
}

fn line_of<'a>(s: &'a str, name: &str) -> &'a str {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が画面に無い:\n{s}"))
}

#[test]
fn test_rel_2_names_in_cells() {
    // [utest -> req~relations.link-display~1] リンクは行き先のノートの名前で見せる(3つの書き方のどれでも。リストは並べる)。普通の値はそのまま。
    let tmp = workspace("rel_names");
    let a = boot_tasks(&tmp);
    let s = screen(&a);
    let build = line_of(&s, "a build");
    assert!(
        build.contains("mdgrid") && !build.contains("[[mdgrid]]"),
        "{build}"
    );
    assert!(
        build.contains("Alice") && !build.contains("../members"),
        "{build}"
    );
    assert!(
        build.contains("[mdgrid]") && build.contains("done"),
        "{build}"
    );
    let test = line_of(&s, "c test");
    assert!(
        test.contains("mdgrid") && !test.contains("../projects"),
        "{test}"
    );
}

#[test]
fn test_rel_10_missing_target_marked() {
    // [utest -> req~relations.broken-link~1] 行き先の無い [[…]] は、書いた名前に印 `?` を付けて見せる。リストの要素も。
    let tmp = workspace("rel_missing");
    let a = boot_tasks(&tmp);
    let s = screen(&a);
    let docs = line_of(&s, "b docs");
    assert!(docs.contains("?nothing"), "{docs}");
    assert!(docs.contains("[mdgrid, ?ghost]"), "{docs}");
}

#[test]
fn test_rel_3_pick_from_target_folder() {
    // [utest -> req~relations.link-input-candidates~1] リンクの列の候補は行き先のフォルダのノートの名前。選ぶと列の形(ここでは [[…]])で書く。
    let tmp = workspace("rel_pick");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "project");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let s = screen(&a);
    assert!(s.contains("Other Project") && s.contains("mdgrid"), "{s}");
    assert!(!s.contains("[[Other Project]]"), "候補は名前で見せる:\n{s}");
    typing(&mut a, "other");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    let row = row_of(&a, "a build");
    assert_eq!(
        a.changes.pending(&row, "project"),
        Some(&NewValue::Str("[[Other Project]]".into()))
    );
    assert!(
        line_of(&screen(&a), "a build").contains("*Other Project"),
        "{}",
        screen(&a)
    );
}

#[test]
fn test_rel_3_list_column_many_to_many() {
    // [utest -> req~relations.link-input-candidates~1, req~relations.link-input-list~1] リンクのリストの列(多対多)は、行き先のフォルダのノートを名前で並べ、複数を付けられる。
    let tmp = workspace("rel_many");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "related");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{}", screen(&a));
    let s = screen(&a);
    assert!(
        s.contains("[x] mdgrid") && s.contains("[ ] Other Project"),
        "{s}"
    );
    typing(&mut a, "Other Project");
    press(&mut a, KeyCode::Enter); // 検索中の Enter は付けるだけ
    press(&mut a, KeyCode::Tab); // 確定(screen.checklist-keys: 検索が空の Enter は付け外し)
    let row = row_of(&a, "a build");
    assert_eq!(
        a.changes.pending(&row, "related"),
        Some(&NewValue::List(vec![
            "[[mdgrid]]".into(),
            "[[Other Project]]".into()
        ]))
    );
}

#[test]
fn test_rel_1_plain_text_column_unchanged() {
    // [utest -> req~relations.link-forms-plain-text~1] リンクでない列(status)は今までどおりの候補と見せ方。
    let tmp = workspace("rel_plain");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "status");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("todo") && s.contains("done"), "{s}");
    assert!(!s.contains("Other Project"), "{s}");
    let _ = PathBuf::new();
}

/// 今の端末の大きさで描いた画面の行。
fn sized_lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    super::test_screen::text(term.backend().buffer())
        .lines()
        .map(str::to_string)
        .collect()
}

/// 行 `line` の、字 `c` の桁(表示の幅で)の全部。
fn cols_of(line: &str, c: char) -> Vec<usize> {
    let mut out = Vec::new();
    let mut x = 0;
    for ch in line.chars() {
        if ch == c {
            out.push(x);
        }
        x += super::width::width(&ch.to_string());
    }
    out
}

/// 行 `line` の桁 [from, to) の字。
fn span_of(line: &str, from: usize, to: usize) -> String {
    let mut out = String::new();
    let mut x = 0;
    for ch in line.chars() {
        if x >= from && x < to {
            out.push(ch);
        }
        x += super::width::width(&ch.to_string());
    }
    out
}

#[test]
fn test_rel_3_link_candidates_window_framed() {
    // [utest -> req~screen.frames~1, req~relations.link-input-candidates~1] リンクの列の候補の窓は、入力の欄の側の縁も `╭─…─╮`(上に出すときは
    // `╰─…─╯`)で閉じ、入力の欄の行と候補の行のあいだを縁で分ける(縁が無いと、同じ行の右の列の札や文字が窓の中に続いて見えた。
    // 2026-11-25 の実地の操作で見つかった)。下に出す窓(1行目の a build)と、低い端末で上に出す窓(c test)の両方。
    let tmp = workspace("rel_pick_frame");
    for (name, h, below) in [("a build", 24u16, true), ("c test", 12, false)] {
        let mut a = boot_tasks(&tmp);
        a.resize(120, h);
        goto(&mut a, name, "project");
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Edit);
        let l = sized_lines(&a);
        let s = l.join("\n");
        let (_, y) = super::input::cursor(&a).expect("入力の欄");
        let y = y as usize;
        let item = l
            .iter()
            .position(|t| t.contains("Other Project"))
            .unwrap_or_else(|| panic!("{name}: 候補が無い:\n{s}"));
        let (edge_y, corners, far) = if below {
            (y + 1, ('╭', '╮'), ('╰', '╯'))
        } else {
            (y - 1, ('╰', '╯'), ('╭', '╮'))
        };
        assert_eq!(item > y, below, "{name}: 窓の向き:\n{s}");
        let edge = &l[edge_y];
        let (Some(&x0), Some(&x1)) = (
            cols_of(edge, corners.0).first(),
            cols_of(edge, corners.1).first(),
        ) else {
            panic!("{name}: 入力の欄のすぐ隣の行に窓の縁 {corners:?} が無い:\n{s}");
        };
        let inner = span_of(edge, x0 + 1, x1);
        // 上に出す窓の入力の欄の側の縁は下の縁で、その窓のキー(screen.window-key-hints)を線のあいだに並べる。
        // キーの字を除けば、縁の中は横の線(とキーの前後の空白)だけ。
        let lines_only = if below {
            inner.clone()
        } else {
            let mut t = inner.clone();
            for k in ["Enter 確定", "Esc 取り消し"] {
                assert!(t.contains(k), "{name}: 下の縁のキー「{k}」:\n{s}");
                t = t.replace(k, "");
            }
            assert!(
                t.starts_with("─ ") && t.ends_with('─') && t.trim_end_matches('─').ends_with(' '),
                "{name}: キーの前後は線と空白「{inner}」:\n{s}"
            );
            t.replace(' ', "")
        };
        assert!(
            !lines_only.is_empty() && lines_only.chars().all(|c| c == '─'),
            "{name}: 縁の中は横の線だけ「{inner}」:\n{s}"
        );
        // 候補の行は縁の内側(左右の縦の線のあいだ)。
        assert_eq!(cols_of(&l[item], '│').first(), Some(&x0), "{name}:\n{s}");
        // 反対の側の縁も同じ桁で閉じる。
        let other = l
            .iter()
            .find(|t| cols_of(t, far.0).contains(&x0) && cols_of(t, far.1).contains(&x1))
            .is_some();
        assert!(other, "{name}: 反対の側の縁 {far:?}:\n{s}");
    }
}
