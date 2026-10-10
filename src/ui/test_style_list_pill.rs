//! 候補の窓でも札の形のまま(SR-36): 丸い札(pill と nerd_font)は窓の中でも丸い端、選んでいる候補も札。

use super::test_screen::{app_of, buffer, press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

#[test]
fn test_sr_36_value_list_keeps_pill_shape() {
    // [SR-36] status = "pill" と nerd_font → 候補の doing・todo が丸い端つき。選んでいる候補(今の値)も札の地のまま、太字と下線。
    let tmp = Tmp::new("sr36_list_pill");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let (c, w) =
        mdgrid::config::parse("[terminal]\nnerd_font = true\n\n[look.style]\nstatus = \"pill\"\n")
            .unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("\u{e0b6}doing\u{e0b4}"), "{s}");
    assert!(
        s.contains("\u{e0b6}todo\u{e0b4}"),
        "選んでいる候補も丸い札: {s}"
    );
    let buf = buffer(&a);
    let chip = super::chips::style(&a, "todo").and_then(|st| st.bg);
    let hit = (0..buf.area.height).find_map(|y| {
        (1..buf.area.width.saturating_sub(4)).find_map(|x| {
            let word: String = (x..x + 4)
                .map(|i| buf[(i, y)].symbol().to_string())
                .collect();
            (word == "todo" && buf[(x - 1, y)].symbol() == "\u{e0b6}").then_some((x, y))
        })
    });
    let (x, y) = hit.unwrap_or_else(|| panic!("窓の todo が無い:\n{s}"));
    assert_eq!(Some(buf[(x, y)].bg), chip, "札の地");
    assert!(
        buf[(x, y)].modifier.contains(Modifier::UNDERLINED),
        "選びの下線"
    );
}

#[test]
fn test_sr_36_picking_cell_keeps_parts() {
    // [SR-36] saas で status のセルで Enter(候補が開く)→ セルは淡い札のまま。文字を打つと入力の欄になる。
    let tmp = Tmp::new("sr36_picking");
    for (n, s) in [("a", "doing"), ("b", "todo"), ("c", "doing")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let (c, _) = mdgrid::config::parse("[look]\npreset = \"saas\"\n").unwrap();
    a.configure(&c);
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let y = view::data_y(&a) as u16;
    let row = |a: &App| -> String {
        let b = buffer(a);
        (0..b.area.width)
            .map(|x| b[(x, y)].symbol().to_string())
            .collect()
    };
    assert!(row(&a).contains("● doing"), "札のまま: {}", row(&a));
    super::test_screen::ch(&mut a, 'x');
    assert!(
        !row(&a).contains("● doing"),
        "打ったら入力の欄: {}",
        row(&a)
    );
}

#[test]
fn test_sr_36_tint_rounds_with_nerd_font() {
    // [SR-36] 淡い札(tint)は nerd_font のとき両端が丸い端の字(幅は同じ)。無ければ空白。
    let tmp = Tmp::new("sr36_tint_caps");
    for (n, s) in [("a", "doing"), ("b", "todo"), ("c", "doing")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\ntags: [ui]\n---\n"),
        );
    }
    for (nerd, want) in [(true, "\u{e0b6}● todo\u{e0b4}"), (false, " ● todo ")] {
        let mut a = app_of(&tmp, ColorMode::Rgb);
        let cfg = format!("[terminal]\nnerd_font = {nerd}\n\n[look]\npreset = \"saas\"\n");
        let (c, _) = mdgrid::config::parse(&cfg).unwrap();
        a.configure(&c);
        a.col = a.cols.iter().position(|c| c == "tags").unwrap();
        let s = screen(&a);
        assert!(s.contains(want), "nerd_font = {nerd}: {s}");
    }
}
