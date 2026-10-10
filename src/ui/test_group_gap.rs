//! まとまりの見出しの上の空き(SR-30)の受け入れの試験。記録: specs/_changes/2026-10-07-group-gap.md のタスク 1。
//! 仕様: specs/screen/spec.md の SR-30。
//!
//! 材料は5つのノート(status が todo 3・done 2)。status でまとめると、まとまりは done(2行)・todo(3行)の順。

use super::grid::Slot;
use super::test_screen::{app_of, ch, screen, Tmp};
use super::*;
use mdgrid::config::parse;
use mdgrid::display::Item;
use mdgrid::settings::{Dir, Group};

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let notes = [
        ("a.md", "alpha", "todo"),
        ("b.md", "beta", "done"),
        ("c.md", "gamma", "todo"),
        ("d.md", "delta", "done"),
        ("e.md", "eps", "todo"),
    ];
    for (n, t, s) in notes {
        tmp.write(n, &format!("---\ntitle: {t}\nstatus: {s}\n---\n"));
    }
    tmp
}

/// 設定 `config` で開き、`group` なら status でまとめる。
fn open(name: &str, config: &str, group: bool) -> (Tmp, App) {
    let tmp = vault(name);
    let mut a = app_of(&tmp, ColorMode::None);
    a.configure(&parse(config).unwrap().0);
    if group {
        a.settings.group = Group::By {
            col: "status".into(),
            dir: Dir::Asc,
            hide_empty: false,
        };
    }
    a.refresh();
    a.row = 0;
    a.top = 0;
    (tmp, a)
}

fn gap_at(a: &App) -> Vec<usize> {
    (0..a.slots.len())
        .filter(|&i| a.slots[i] == Slot::Gap)
        .collect()
}

fn head_at(a: &App, g: usize) -> usize {
    a.slots.iter().position(|s| *s == Slot::Head(g)).unwrap()
}

/// 画面の行(表の i 番目の slot の行)の文字。
fn line_of(a: &App, i: usize) -> String {
    let s = screen(a);
    let y = view::data_y(a) + i - a.top;
    s.lines().nth(y).unwrap_or("").to_string()
}

#[test]
fn test_sr_30_gap_above_second_heading_only() {
    // [SR-30] 2つ目の見出しの上に空きの行が1つ。1つ目の見出しの上には無い。
    let (_t, a) = open("ggon", "[display]\ngroup_gap = true\n", true);
    assert_eq!(a.groups.len(), 2);
    assert_eq!(a.slots[0], Slot::Head(0), "1つ目の見出しが先頭");
    let h1 = head_at(&a, 1);
    assert_eq!(gap_at(&a), vec![h1 - 1], "空きは2つ目の見出しのすぐ上だけ");
    assert_eq!(line_of(&a, h1 - 1).trim(), "", "空きの行は何も描かない");
    assert!(line_of(&a, h1).contains("todo"), "{}", screen(&a));
    // 行として数えない: ノートの行は5、見出しは2のまま。
    assert_eq!(a.rows.len(), 5);
    assert_eq!(a.slots.len(), 5 + 2 + 1);
}

#[test]
fn test_sr_30_cursor_skips_gap() {
    // [SR-30] 1つ目のまとまりの最後の行で j → 空きを飛ばして2つ目の見出し。k で戻る。
    let (_t, mut a) = open("ggcur", "[display]\ngroup_gap = true\n", true);
    let h1 = head_at(&a, 1);
    a.row = h1 - 2; // 1つ目のまとまりの最後の行
    assert!(matches!(a.slots[a.row], Slot::Row(_)));
    ch(&mut a, 'j');
    assert_eq!(a.row, h1, "j で2つ目の見出しへ");
    assert_eq!(a.cur_head(), Some(1));
    ch(&mut a, 'k');
    assert_eq!(a.row, h1 - 2, "k で1つ目のまとまりの最後の行へ");
    // 下の帯の数は見出し・行の数え方のまま。
    ch(&mut a, 'j');
    assert!(screen(&a).contains("2/2"), "{}", screen(&a));
}

#[test]
fn test_sr_30_click_on_gap_keeps_selection() {
    // [SR-30] 空きの行をクリックしても選びは変わらない。
    let (_t, mut a) = open("ggclick", "[display]\ngroup_gap = true\n", true);
    let gap = gap_at(&a)[0];
    a.row = 1;
    let before = (a.row, a.col);
    let x = 3u16;
    let y = (view::data_y(&a) + gap - a.top) as u16;
    assert!(view::hit(&a, x, y).is_none());
    a.click(x, y);
    assert_eq!((a.row, a.col), before);
}

#[test]
fn test_sr_30_default_has_no_gap() {
    // [SR-30] group_gap の無い設定 → 今と同じ(空きは無い)。
    let (_t, a) = open("ggoff", "", true);
    assert!(gap_at(&a).is_empty());
    assert_eq!(a.slots.len(), 5 + 2);
    assert!(!a.shows(Item::GroupGap));
}

#[test]
fn test_sr_30_no_groups_no_gap() {
    // [SR-30] まとまりの無いビューでは、group_gap が true でも空きは出ない。
    let (_t, a) = open("ggnogroup", "[display]\ngroup_gap = true\n", false);
    assert!(a.groups.is_empty());
    assert!(gap_at(&a).is_empty());
}

#[test]
fn test_sr_30_view_override_and_row_numbers() {
    // [SR-30][SR-20] 設定に無くても、ビューの上書きで空きを出せる。行番号は空きを数えず 1〜5 が続く。
    let (_t, mut a) = open("ggview", "[display]\nrow_numbers = true\n", true);
    assert!(gap_at(&a).is_empty());
    let base = a.display;
    a.settings.display.set(Item::GroupGap, true, &base);
    a.refresh();
    let h1 = head_at(&a, 1);
    assert_eq!(gap_at(&a), vec![h1 - 1]);
    let s = screen(&a);
    for n in 1..=5 {
        assert!(s.contains(&format!("{n} ")), "行番号 {n}: {s}");
    }
    assert!(!line_of(&a, h1 - 1).trim().starts_with('6'), "{s}");
}

#[test]
fn test_sr_30_print_config_and_views_key() {
    // [SR-30] `--print-config` の [display] に group_gap = false。views.toml の上書きの項目にもある。
    let c = mdgrid::config::Config::default();
    assert!(!c.resolved().display.group_gap);
    let (c, w) = mdgrid::config::parse("[display]\ngroup_gap = true\n").unwrap();
    assert!(w.is_empty() && c.resolved().display.group_gap);
    assert!(mdgrid::display::OVERRIDE_KEYS.contains(&"group_gap"));
    let text = mdgrid::config::default_toml();
    assert!(text.contains("group_gap = false"), "{text}");
}
