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
    // [NV-27] 設計の下に画面(2桁)・その下に部品(4桁、リストの先頭のリンク)・API(2桁)。メモは一番上の段。
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
    // [NV-27] 並べ替えは兄弟の中で効く: n の昇順と降順で API と画面の順が入れ替わり、子はいつも親の下。
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
    // [NV-27] 設計で Z → 子孫が隠れて ▸ 設計。もう一度 Z → 戻る。子の無い行は理由。
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
    // [NV-27] 設計を絞り込みで外す → 画面と API が一番上の段(行は落ちない)。親子をオフ → 字下げなし。
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
    // [NV-27] 設定の画面の「親子」の区画: 左に「親子 —」。Enter でオン → 上に「未反映 1」、反映で字下げ。
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
    // [NV-27] 名前の欄の ▾ をクリック → 畳んで ▸。もう一度 → 開く。名前のほかの所のクリックは畳まない。
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
