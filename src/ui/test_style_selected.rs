//! 選んだセルも部品の形のまま(SR-36): 丸い札・淡い札が、選んだときだけ四角い塗りに変わらない。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;
use ratatui::style::Modifier;

fn boot(name: &str, cfg: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, s) in [("a", "doing"), ("b", "todo"), ("c", "doing")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let (c, w) = mdgrid::config::parse(cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.refresh_if_needed();
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    (tmp, a)
}

/// 今の行で文字 `t` の始まる桁と行。
fn at(a: &App, t: &str) -> (u16, u16) {
    let y = view::data_y(a) as u16;
    let buf = buffer(a);
    let cells: Vec<String> = (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect();
    let x = (0..cells.len())
        .find(|&x| cells[x..].concat().starts_with(t))
        .unwrap_or_else(|| panic!("{t} が無い:\n{}", screen(a)));
    (x as u16, y)
}

#[test]
fn test_sr_36_selected_cell_keeps_parts() {
    // [SR-36] saas(淡い札): 選んだセルの札も、選んでいないセルと同じ淡い地。選びは太字と下線で重ねる。
    let (_t, a) = boot("sr36_sel_tint", "[look]\npreset = \"saas\"\n");
    let buf = buffer(&a);
    let (x, y) = at(&a, "doing");
    let want = super::chips::parts(&a, &["doing".to_string()], false)
        .and_then(|p| p.items[0][0].1)
        .and_then(|s| s.bg);
    assert_eq!(Some(buf[(x, y)].bg), want, "札の淡い地のまま");
    assert!(buf[(x, y)].modifier.contains(Modifier::UNDERLINED));
    // sumi(点): 点は値の色のまま(選びのアクセントの色に塗りつぶさない)。
    let (_u, b) = boot("sr36_sel_dot", "");
    let buf = buffer(&b);
    let (x, y) = at(&b, "● doing");
    let dot = super::chips::parts(&b, &["doing".to_string()], false)
        .and_then(|p| p.items[0][0].1)
        .and_then(|s| s.fg);
    assert_eq!(Some(buf[(x, y)].fg), dot, "点は値の色");
    assert!(buf[(x + 2, y)].modifier.contains(Modifier::UNDERLINED));
}
