//! まとまりの見出しの値を、その列のセルと同じ部品(札の形と値の色)で描く(navigation.group-heading-parts)。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;
use mdgrid::settings::{Dir, Group};
use ratatui::style::Color;

/// tags が [仕事] の行と [家] の行。設定 `cfg` を当て、tags でグループ分けする。
fn boot(name: &str, cfg: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ntags: [仕事]\nx: 1\n---\n");
    tmp.write("b.md", "---\ntags: [家]\nx: 2\n---\n");
    let mut a = app_of(&tmp, color);
    let (c, w) = mdgrid::config::parse(cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.settings.group = Group::By {
        col: "tags".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.regrid = true;
    a.refresh_if_needed();
    // 見出しを選ばない(選びの見た目を重ねない)。
    a.row = (0..a.slots.len())
        .rev()
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(_)))
        .unwrap();
    a.col = a.cols.iter().position(|c| c == "x").unwrap();
    (tmp, a)
}

/// 「仕事」のまとまりの見出しの行で、字 `仕` のある (桁, 行)。
fn heading_char(a: &App) -> (u16, u16) {
    let buf = buffer(a);
    for y in 0..buf.area.height {
        let line: String = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        // 全角の字の後ろの桁は空白なので、字を1つずつ見る。
        if line.contains("tags:") && line.contains('仕') {
            let x = (0..buf.area.width)
                .find(|&x| buf[(x, y)].symbol() == "仕")
                .unwrap();
            return (x, y);
        }
    }
    panic!("仕事 の見出しが無い:\n{}", screen(a));
}

const CHIP: &str =
    "[look.style]\ntags = \"chip\"\n\n[look.colors.values]\n\"仕事\" = \"#3366cc\"\n";

#[test]
fn test_group_heading_value_is_drawn_as_the_cell_part() {
    // [utest -> req~navigation.group-heading-parts~1] 札の形(chip)と値の色(仕事 = #3366cc)の表を tags でまとめる → 見出しの「仕事」の
    // 字の地は #3366cc で、件数 (1件) も出る。
    let (_t, a) = boot("ghparts1", CHIP, ColorMode::Rgb);
    let (x, y) = heading_char(&a);
    assert_eq!(
        buffer(&a)[(x, y)].bg,
        Color::Rgb(0x33, 0x66, 0xcc),
        "{}",
        screen(&a)
    );
    let s = screen(&a);
    assert!(
        s.lines().any(|l| l.contains("仕事") && l.contains("(1件)")),
        "{s}"
    );
}

#[test]
fn test_group_heading_plain_column_stays_text() {
    // [utest -> req~navigation.group-heading-parts~1] 同じ表で `[look.columns] tags = "plain"` → 見出しの「仕事」の地は #3366cc でない(文字のまま)。
    let cfg = format!("{CHIP}\n[look.columns]\ntags = \"plain\"\n");
    let (_t, a) = boot("ghparts2", &cfg, ColorMode::Rgb);
    let (x, y) = heading_char(&a);
    assert_ne!(
        buffer(&a)[(x, y)].bg,
        Color::Rgb(0x33, 0x66, 0xcc),
        "{}",
        screen(&a)
    );
}

#[test]
fn test_group_heading_no_color_is_unchanged() {
    // [utest -> req~navigation.group-heading-parts~1] 色を使わない表示では、見出しは今までどおり `▾ tags: 仕事(1件)`。
    let (_t, a) = boot("ghparts3", CHIP, ColorMode::None);
    let s = screen(&a);
    assert!(s.lines().any(|l| l.contains("▾ tags: 仕事(1件)")), "{s}");
}
