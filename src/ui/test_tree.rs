//! 親子のノートを字下げして並べ、z で畳む(NV-27)。

use super::keymap::Mode;
use super::settings::Sec;
use super::test_screen::{ch, make, press, screen, Tmp};
use super::*;
use mdgrid::settings::{Cond, Dir, Op};
use ratatui::crossterm::event::KeyCode;

/// 設計の下に画面(その下に部品)と API。メモは親なし。
fn notes(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("設計.md", "---\nstatus: doing\n---\n"),
            (
                "画面.md",
                "---\nparent: \"[[設計]]\"\nstatus: todo\nn: 1\n---\n",
            ),
            (
                "API.md",
                "---\nparent: \"[[設計]]\"\nstatus: todo\nn: 2\n---\n",
            ),
            (
                "部品.md",
                "---\nparent:\n  - \"[[画面]]\"\nstatus: done\n---\n",
            ),
            ("メモ.md", "---\nstatus: done\n---\n"),
        ],
    )
}

fn tree_on(a: &mut App) {
    a.settings.tree = Some("parent".into());
    a.regrid = true;
    a.refresh_if_needed();
}

/// 表の行の名前の欄(字下げと印を含む。右の空白は除く)。
fn names(a: &App) -> Vec<String> {
    let lay = super::view::layout(a);
    let (x0, w) = (lay.label_x(), lay.label_w);
    screen(a)
        .lines()
        .filter(|l| l.contains("todo") || l.contains("done") || l.contains("doing"))
        .filter(|l| !l.contains("status"))
        .map(|l| {
            let mut col = 0;
            let mut out = String::new();
            for c in l.chars() {
                let cw = super::width::width(&c.to_string());
                if col >= x0 && col + cw <= x0 + w {
                    out.push(c);
                }
                col += cw;
            }
            out.trim_end().to_string()
        })
        .collect()
}

#[test]
fn test_nv_27_indent_under_parent() {
    // [utest -> req~navigation.parent-child-rows~1] 設計の下に画面(2桁)・その下に部品(4桁、リストの先頭のリンク)・API(2桁)。メモは一番上の段。
    let (_t, mut a) = notes("nv27a");
    tree_on(&mut a);
    let n = names(&a);
    let at = |s: &str| {
        n.iter()
            .position(|x| x.trim_start().trim_start_matches(['▾', '▸', ' ']) == s)
            .unwrap()
    };
    assert!(n.contains(&"▾ 設計".to_string()), "{n:?}");
    assert!(n.contains(&"  ▾ 画面".to_string()), "{n:?}");
    assert!(n.contains(&"    部品".to_string()), "{n:?}");
    assert!(n.contains(&"  API".to_string()), "{n:?}");
    assert!(at("設計") < at("画面") && at("画面") < at("部品"), "{n:?}");
    assert!(at("設計") < at("API"), "{n:?}");
    assert!(n.contains(&"メモ".to_string()), "{n:?}");
}

#[test]
fn test_nv_27_sort_within_siblings() {
    // [utest -> req~navigation.parent-child-sibling-order~1] 並べ替えは兄弟の中で効く: n の昇順と降順で API と画面の順が入れ替わり、子はいつも親の下。
    let (_t, mut a) = notes("nv27b");
    tree_on(&mut a);
    a.set_sorts(vec![("n".into(), Dir::Asc)]);
    let asc = names(&a);
    a.set_sorts(vec![("n".into(), Dir::Desc)]);
    let desc = names(&a);
    let pos = |n: &Vec<String>, s: &str| n.iter().position(|x| x.ends_with(s)).unwrap();
    assert_ne!(
        pos(&asc, "API") < pos(&asc, "画面"),
        pos(&desc, "API") < pos(&desc, "画面"),
        "{asc:?} {desc:?}"
    );
    for n in [&asc, &desc] {
        assert!(
            pos(n, "設計") < pos(n, "API") && pos(n, "設計") < pos(n, "画面"),
            "{n:?}"
        );
        assert_eq!(pos(n, "部品"), pos(n, "画面") + 1, "{n:?}");
    }
}

#[test]
fn test_nv_27_shift_z_folds_and_unfolds() {
    // [utest -> req~navigation.parent-child-fold~2] 設計で Z → 子孫が隠れて ▸ 設計。もう一度 Z → 戻る。子の無い行は理由。
    let (_t, mut a) = notes("nv27c");
    tree_on(&mut a);
    let total = a.slots.len();
    let i = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains("設計")))
        .unwrap();
    a.row = i;
    ch(&mut a, 'Z');
    assert_eq!(a.slots.len(), total - 3);
    assert!(names(&a).contains(&"▸ 設計".to_string()), "{:?}", names(&a));
    assert!(!names(&a).iter().any(|n| n.ends_with("部品")));
    ch(&mut a, 'Z');
    assert_eq!(a.slots.len(), total);
    // 葉(部品)で Z。
    let j = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains("部品")))
        .unwrap();
    a.row = j;
    ch(&mut a, 'Z');
    assert_eq!(a.message.as_deref(), Some("この行に子の行は無い"));
}

#[test]
fn test_nv_27_missing_parent_goes_top_and_off_restores() {
    // [utest -> req~navigation.parent-child-orphans~3] 設計を絞り込みで外す → 画面と API が一番上の段(行は落ちない)。親子をオフ → 字下げなし。
    let (_t, mut a) = notes("nv27d");
    tree_on(&mut a);
    a.settings.filters = vec![Cond {
        col: "status".into(),
        op: Op::Drop(vec![Some("doing".into())]),
    }];
    a.regrid = true;
    a.refresh_if_needed();
    let n = names(&a);
    assert!(n.contains(&"▾ 画面".to_string()), "{n:?}");
    assert!(n.contains(&"API".to_string()), "{n:?}");
    assert!(n.contains(&"  部品".to_string()), "{n:?}");
    a.settings.tree = None;
    a.regrid = true;
    a.refresh_if_needed();
    assert!(
        names(&a)
            .iter()
            .all(|x| !x.starts_with(' ') && !x.contains('▾')),
        "{:?}",
        names(&a)
    );
}

#[test]
fn test_nv_27_settings_section_turns_on() {
    // [utest -> req~navigation.parent-child-rows~1] 設定の画面の「親子」の区画: 左に「親子 —」。Enter でオン → 上に「未反映 1」、反映で字下げ。
    let (_t, mut a) = notes("nv27e");
    ch(&mut a, 'o');
    assert_eq!(a.mode, Mode::Settings);
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Tree {
            break;
        }
        press(&mut a, KeyCode::Tab);
    }
    let s = screen(&a);
    assert!(s.contains("親子"), "{s}");
    assert!(s.contains("○ オフ") && s.contains("▾ parent"), "{s}");
    a.draft.as_mut().unwrap().sel[Sec::Tree as usize] = 0;
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("未反映 1"), "{}", screen(&a));
    let d = a.draft.as_mut().unwrap();
    d.sec = Sec::Buttons;
    d.sel[Sec::Buttons as usize] = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.settings.tree.as_deref(), Some("parent"));
    assert!(
        names(&a).contains(&"  ▾ 画面".to_string()),
        "{:?}",
        names(&a)
    );
}

#[test]
fn test_nv_27_click_mark_folds() {
    // [utest -> req~navigation.parent-child-fold~2] 名前の欄の ▾ をクリック → 畳んで ▸。もう一度 → 開く。名前のほかの所のクリックは畳まない。
    let (_t, mut a) = notes("nv27f");
    tree_on(&mut a);
    let total = a.slots.len();
    let i = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains("設計")))
        .unwrap();
    let y = (super::view::data_y(&a) + i - a.top) as u16;
    let x = super::view::layout(&a).label_x() as u16;
    a.click(x, y);
    assert_eq!(a.slots.len(), total - 3);
    assert!(names(&a).contains(&"▸ 設計".to_string()), "{:?}", names(&a));
    a.click(x + 1, y);
    assert_eq!(a.slots.len(), total);
    // 名前の文字のクリックは、畳まない(行を選ぶだけ)。
    a.row = 0;
    a.click(x + 3, y);
    assert_eq!(a.slots.len(), total);
}

/// status でグループ分けする。
fn group_by_status(a: &mut App) {
    a.settings.group = mdgrid::settings::Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.regrid = true;
    a.refresh_if_needed();
}

/// 設計(doing)の下に画面と API(どちらも todo)。
fn across_groups(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("設計.md", "---\nstatus: doing\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
            ("API.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
        ],
    )
}

/// 表の行(見出し・行・薄い行)ごとの、左の端の1桁と名前の欄(右の空白は除く)。見出しの上の空きは除く。
fn slot_lines(a: &App) -> Vec<(String, String)> {
    let lay = super::view::layout(a);
    let (x0, w) = (lay.label_x(), lay.label_w);
    let s = screen(a);
    let lines: Vec<&str> = s.lines().collect();
    let dy = super::view::data_y(a);
    (0..a.slots.len())
        // 見出しの上の空き(何も描かない選べない行)は除く。
        .filter(|&k| a.slots[k] != super::grid::Slot::Gap || !lines[dy + k].trim().is_empty())
        .map(|k| {
            let l = lines[dy + k];
            // まとまりの見出しは、印 `#` と行の全部の字(名前の欄の幅で切らない)。
            if matches!(a.slots[k], super::grid::Slot::Head(_)) {
                return ("#".to_string(), l.trim().to_string());
            }
            let mut col = 0;
            let (mut lead, mut name) = (String::new(), String::new());
            for c in l.chars() {
                let cw = super::width::width(&c.to_string());
                if col == 0 {
                    lead.push(c);
                } else if col >= x0 && col + cw <= x0 + w {
                    name.push(c);
                }
                col += cw;
            }
            // 今の行の `>` は、行の種類の印ではないので数えない。
            if lead == ">" {
                lead = " ".into();
            }
            (lead, name.trim_end().to_string())
        })
        .collect()
}

/// 名前が `name` の行を選んでいるか。
fn on(a: &App, name: &str) -> bool {
    a.cur_row()
        .is_some_and(|r| std::path::Path::new(&r.0).file_stem() == Some(name.as_ref()))
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 親が別のまとまりにある子は、子のまとまりの中で、`^` の付いた薄い親の行の下に2桁字下げして並ぶ。
/// 親のまとまりの「設計」はそのまま。
#[test]
fn test_away_parent_shown_as_context_row() {
    let (_t, mut a) = across_groups("nv27away1");
    tree_on(&mut a);
    group_by_status(&mut a);
    let got = slot_lines(&a);
    let names: Vec<(&str, &str)> = got.iter().map(|(l, n)| (l.trim(), n.as_str())).collect();
    // doing のまとまり(見出し・設計)、todo のまとまり(見出し・薄い設計・画面・API)。画面と API の順は名前の順。
    let todo_at = names
        .iter()
        .position(|(l, n)| *l == "#" && n.contains("todo"))
        .unwrap_or_else(|| panic!("{got:?}"));
    let doing: Vec<&(&str, &str)> = names[..todo_at].iter().filter(|(l, _)| *l != "#").collect();
    assert_eq!(doing, [&("", "設計")], "{got:?}");
    let mut todo: Vec<(&str, &str)> = names[todo_at + 1..].to_vec();
    assert_eq!(todo[0], ("^", "▾ 設計"), "{got:?}");
    todo[1..].sort_unstable();
    assert_eq!(todo[1..], [("", "  API"), ("", "  画面")], "{got:?}");
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 薄い親の行は、まとまりの見出しの数に入らず、status などのセルは空。名前は薄い字。
#[test]
fn test_away_context_row_not_counted_and_blank() {
    let (_t, mut a) = across_groups("nv27away5");
    tree_on(&mut a);
    group_by_status(&mut a);
    let s = screen(&a);
    let head = s
        .lines()
        .find(|l| l.contains("todo") && l.contains('('))
        .unwrap_or_else(|| panic!("todo の見出しが無い:\n{s}"));
    // 見出しの数は2件(薄い「設計」を数えない)。
    assert!(head.contains("(2") && !head.contains("(3"), "{head}");
    let ghost = s
        .lines()
        .find(|l| l.starts_with('^'))
        .unwrap_or_else(|| panic!("{s}"));
    assert!(ghost.contains("設計"), "{ghost}");
    assert!(
        !ghost.contains("doing") && !ghost.contains("todo"),
        "{ghost}"
    );
    // 薄い字: 名前の桁の字に DIM が付く。
    let buf = super::test_screen::buffer(&a);
    let y = (0..buf.area.height)
        .find(|&y| buf[(0, y)].symbol() == "^")
        .unwrap();
    let lay = super::view::layout(&a);
    let x = (lay.label_x()..lay.label_x() + lay.label_w)
        .find(|&x| buf[(x as u16, y)].symbol() == "設")
        .unwrap_or_else(|| panic!("{ghost}"));
    assert!(
        buf[(x as u16, y)]
            .modifier
            .contains(ratatui::style::Modifier::DIM),
        "{ghost}"
    );
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 見出しから j で薄い行を飛ばして画面を選び、k で薄い行を飛ばして見出しへ戻る。薄い行のクリックでは選びは変わらない。
#[test]
fn test_away_context_row_is_skipped_and_not_clickable() {
    let (_t, mut a) = across_groups("nv27away6");
    tree_on(&mut a);
    group_by_status(&mut a);
    let head = (0..a.slots.len())
        .rev()
        .find(|&k| matches!(a.slots[k], super::grid::Slot::Head(_)))
        .unwrap();
    a.row = head;
    ch(&mut a, 'j');
    let first = a.cur_row().expect("行を選んでいない");
    let first_name = std::path::Path::new(&first.0)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(
        ["API", "画面"].contains(&first_name.as_str()),
        "{first_name}"
    );
    ch(&mut a, 'k');
    assert_eq!(a.row, head);
    ch(&mut a, 'j');
    let before = a.row;
    let k = (0..a.slots.len()).find(|&k| line_of_slot_is_ghost(&a, k));
    let k = k.expect("薄い行が無い");
    let y = (super::view::data_y(&a) + k) as u16;
    let lay = super::view::layout(&a);
    a.click(lay.label_x() as u16 + 3, y);
    assert_eq!(a.row, before);
    assert!(on(&a, &first_name));
}

fn line_of_slot_is_ghost(a: &App, k: usize) -> bool {
    let s = screen(a);
    let lines: Vec<&str> = s.lines().collect();
    lines[super::view::data_y(a) + k].starts_with('^')
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 親と祖父が別のまとまりなら、薄い祖父・その下に薄い親・その下に4桁字下げした子。
#[test]
fn test_away_context_rows_for_grandparents() {
    let (_t, mut a) = make(
        "nv27away7",
        &[
            ("計画.md", "---\nstatus: doing\n---\n"),
            ("設計.md", "---\nparent: \"[[計画]]\"\nstatus: doing\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
        ],
    );
    tree_on(&mut a);
    group_by_status(&mut a);
    let got = slot_lines(&a);
    let todo_at = got
        .iter()
        .position(|(l, n)| l == "#" && n.contains("todo"))
        .unwrap_or_else(|| panic!("{got:?}"));
    let todo: Vec<(&str, &str)> = got[todo_at + 1..]
        .iter()
        .map(|(l, n)| (l.trim(), n.as_str()))
        .collect();
    assert_eq!(
        todo,
        [("^", "▾ 計画"), ("^", "  ▾ 設計"), ("", "    画面")],
        "{got:?}"
    );
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 薄い親の下の子の孫は、同じまとまりなら子の下にもう1段字下げ(薄い行は親だけ)。
#[test]
fn test_away_child_keeps_own_kids_indented() {
    let (_t, mut a) = make(
        "nv27away2",
        &[
            ("設計.md", "---\nstatus: doing\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
            ("部品.md", "---\nparent: \"[[画面]]\"\nstatus: todo\n---\n"),
        ],
    );
    tree_on(&mut a);
    group_by_status(&mut a);
    let got = slot_lines(&a);
    let todo_at = got
        .iter()
        .position(|(l, n)| l == "#" && n.contains("todo"))
        .unwrap_or_else(|| panic!("{got:?}"));
    let todo: Vec<(&str, &str)> = got[todo_at + 1..]
        .iter()
        .map(|(l, n)| (l.trim(), n.as_str()))
        .collect();
    assert_eq!(
        todo,
        [("^", "▾ 設計"), ("", "  ▾ 画面"), ("", "    部品")],
        "{got:?}"
    );
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// グループ分けをしなければ薄い行は無く、画面は設計の下に2桁字下げ。
#[test]
fn test_no_group_no_context_row() {
    let (_t, mut a) = across_groups("nv27away3");
    tree_on(&mut a);
    let got = slot_lines(&a);
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(
        got.iter().all(|(l, _)| l.trim() != "^" && l != "#"),
        "{got:?}"
    );
    assert!(got.iter().any(|(_, n)| n == "  画面"), "{got:?}");
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// 親を絞り込みで外すと、薄い行は無く、子は一番上の段。
#[test]
fn test_filtered_out_parent_no_context_row() {
    let (_t, mut a) = across_groups("nv27away4");
    tree_on(&mut a);
    a.settings.filters = vec![Cond {
        col: "status".into(),
        op: Op::Drop(vec![Some("doing".into())]),
    }];
    group_by_status(&mut a);
    let got = slot_lines(&a);
    let rows: Vec<(&str, &str)> = got
        .iter()
        .filter(|(l, _)| l != "#")
        .map(|(l, n)| (l.trim(), n.as_str()))
        .collect();
    let mut rows = rows;
    rows.sort_unstable();
    assert_eq!(rows, [("", "API"), ("", "画面")], "{got:?}");
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// ガントの画面には薄い行を出さず、画面と API は字下げなしで並ぶ。
#[test]
fn test_away_context_not_in_gantt() {
    let (_t, mut a) = make(
        "nv27away8",
        &[
            ("設計.md", "---\nstatus: doing\nstart: 2026-10-01\ndue: 2026-10-03\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\nstart: 2026-10-02\ndue: 2026-10-04\n---\n"),
            ("API.md", "---\nparent: \"[[設計]]\"\nstatus: todo\nstart: 2026-10-02\ndue: 2026-10-05\n---\n"),
        ],
    );
    tree_on(&mut a);
    group_by_status(&mut a);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    for n in ["画面", "API"] {
        let l = s
            .lines()
            .find(|l| l.contains(n))
            .unwrap_or_else(|| panic!("{n}:\n{s}"));
        // 窓の枠 `│` のあと、選びの印か空白の1桁のすぐ後ろに名前(字下げの空白が無い)。
        let at = l.find(n).unwrap();
        let inner = l[..at].trim_start_matches('│');
        assert!(inner.chars().count() <= 1, "{n} は字下げなし: {l:?}");
    }
    let g = a.gantt.as_ref().unwrap();
    assert_eq!(g.rows.len(), 3);
}

// [utest -> req~navigation.parent-child-away-mark~2]
/// ヘルプの印の一覧の「行の左」に `^` と、別のまとまりにある親を添えた行である旨。
#[test]
fn test_help_lists_context_row_mark() {
    let (_t, mut a) = across_groups("nv27away9");
    ch(&mut a, '?');
    let mut found = false;
    for _ in 0..40 {
        let s = screen(&a);
        if s.lines().any(|l| {
            l.trim_start().trim_start_matches("> ").starts_with("^ ") && l.contains("別のまとまり")
        }) {
            found = true;
            break;
        }
        press(&mut a, KeyCode::PageDown);
    }
    assert!(found, "ヘルプに ^ の印が無い:\n{}", screen(&a));
}

/// 設計(p2)の下に画面(p1)と API(p3)。3つとも todo で、同じまとまりに入る。
fn same_group(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("設計.md", "---\nstatus: todo\npriority: p2\n---\n"),
            (
                "画面.md",
                "---\nparent: \"[[設計]]\"\nstatus: todo\npriority: p1\n---\n",
            ),
            (
                "API.md",
                "---\nparent: \"[[設計]]\"\nstatus: todo\npriority: p3\n---\n",
            ),
            ("メモ.md", "---\nstatus: doing\npriority: p1\n---\n"),
        ],
    )
}

// [utest -> req~navigation.parent-child-groups~1]
/// グループ分けをしても、同じまとまりの子は、そのまとまりの中で親の下に字下げして並ぶ。
#[test]
fn test_same_group_children_under_parent() {
    let (_t, mut a) = same_group("nv27grp1");
    tree_on(&mut a);
    group_by_status(&mut a);
    let n = names(&a);
    let at = n
        .iter()
        .position(|x| x == "▾ 設計")
        .unwrap_or_else(|| panic!("{n:?}"));
    let mut kids = n[at + 1..at + 3].to_vec();
    kids.sort();
    assert_eq!(kids, vec!["  API", "  画面"], "{n:?}");
}

// [utest -> req~navigation.parent-child-groups~1]
/// まとまりの中で並べ替えても、子は親より上に出ず、並べ替えは兄弟の中で効く。
#[test]
fn test_same_group_sort_keeps_children_under_parent() {
    let (_t, mut a) = same_group("nv27grp2");
    a.settings.sorts = vec![("priority".into(), Dir::Asc)];
    tree_on(&mut a);
    group_by_status(&mut a);
    let n = names(&a);
    let at = n
        .iter()
        .position(|x| x == "▾ 設計")
        .unwrap_or_else(|| panic!("{n:?}"));
    assert_eq!(n[at + 1..at + 3], ["  画面", "  API"], "{n:?}");
}
