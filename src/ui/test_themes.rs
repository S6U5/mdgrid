//! 画面のテーマ(SR-26・SR-27)の受け入れの試験。
//! 記録: specs/_changes/2026-10-06-themes.md(設計の節の公開のインターフェースと色の表)。
//!
//! 実装より先に、実装を見ずに書いた。テーマは `mdgrid::theme::{Theme, Palette, to_indexed}`、
//! 設定は `mdgrid::config::parse` の最上位の `theme`(`Config.theme`)。
//! 描く範囲は 80×24 のうち最下行と右端の1桁を空けた 79×23。1行目(y = 0)はヘッダー、
//! 下の帯は y = 21、メッセージ行は y = 22。
//! 「選んでいるセル」と「下の帯」は、テーマなしの同じ画面で反転(REVERSED)になっているセルの位置で引く。

use super::test_screen::{app_of, buffer, text, Tmp};
use super::*;
use mdgrid::config::{self, Config};
use mdgrid::theme::{to_indexed, Palette, Theme};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

/// 80×24 の下の帯の行。
const BAND: u16 = 21;

const NORD_BG: Color = Color::Rgb(0x2e, 0x34, 0x40);
const NORD_SEL_BG: Color = Color::Rgb(0x5e, 0x81, 0xac);
const NORD_SEL_FG: Color = Color::Rgb(0xec, 0xef, 0xf4);
const NORD_BAND_BG: Color = Color::Rgb(0x3b, 0x42, 0x52);
const NORD_HEADER: Color = Color::Rgb(0x88, 0xc0, 0xd0);
const NORD_ZEBRA_BG: Color = Color::Rgb(0x35, 0x3c, 0x4a);
const NORD_ZEBRA_FG: Color = Color::Rgb(0xd8, 0xde, 0xe9);

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: alpha\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: beta\nstatus: done\n---\n"),
    ("c.md", "---\ntitle: gamma\nstatus: todo\n---\n"),
    ("d.md", "---\ntitle: delta\nstatus: done\n---\n"),
    (
        "e.md",
        "---\ntitle: eps\nstatus: todo\n---\n**太字** の本文\n",
    ),
];

// ---- 道具 ----

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    tmp
}

/// 設定の文(TOML)を読む。読めなければ落ちる。
fn cfg(text: &str) -> Config {
    config::parse(text).unwrap().0
}

/// 色の扱い `color` で開き、設定 `config` を当てた App。
/// テーマの塗り替え(SR-26)は今までの見た目(`look = "classic"`)の上で確かめる(モダンな見た目 SR-33 は
/// test_look.rs)。
fn themed(tmp: &Tmp, color: ColorMode, config: &str) -> App {
    let mut a = app_of(tmp, color);
    a.configure(&cfg(&format!("look = \"classic\"\n{config}")));
    a
}

/// 既定の設定に `look = "classic"` だけを当てた App(テーマの機能が無いときの画面の比べる元)。
fn plain(tmp: &Tmp, color: ColorMode) -> App {
    let mut a = app_of(tmp, color);
    a.configure(&cfg("look = \"classic\"\n"));
    let _ = Config::default();
    a
}

fn rgb(c: [u8; 3]) -> Color {
    Color::Rgb(c[0], c[1], c[2])
}

/// 全部のセルの (x, y)。
fn cells(buf: &Buffer) -> impl Iterator<Item = (u16, u16)> + '_ {
    let a = buf.area;
    (0..a.height).flat_map(move |y| (0..a.width).map(move |x| (x, y)))
}

/// 2つのバッファが各セルの記号・前景・背景・修飾まで同じこと。
fn assert_same_cells(got: &Buffer, want: &Buffer, what: &str) {
    assert_eq!(got.area, want.area, "{what}: 大きさ");
    for (x, y) in cells(want) {
        let (g, w) = (&got[(x, y)], &want[(x, y)]);
        assert_eq!(g.symbol(), w.symbol(), "{what}: ({x},{y}) の記号");
        assert_eq!(g.fg, w.fg, "{what}: ({x},{y}) の前景");
        assert_eq!(g.bg, w.bg, "{what}: ({x},{y}) の背景");
        assert_eq!(g.modifier, w.modifier, "{what}: ({x},{y}) の修飾");
    }
}

/// テーマなしの画面で反転になっているセルのうち、表の行 `y` にあるもの(選んでいるセル)。
fn reversed_on(buf: &Buffer, y: u16) -> Vec<u16> {
    (0..buf.area.width)
        .filter(|&x| buf[(x, y)].modifier.contains(Modifier::REVERSED))
        .collect()
}

// ---- SR-26: 塗り分け ----

#[test]
fn test_sr_26_nord_paints_background_selection_and_band() {
    // [SR-26] theme = "nord" → 左上の背景が #2e3440、選んでいるセルの背景が #5e81ac(反転は外れる)、
    // 下の帯の背景が #3b4252。
    let tmp = vault("thnord");
    let base = buffer(&plain(&tmp, ColorMode::Rgb));
    let a = themed(&tmp, ColorMode::Rgb, "theme = \"nord\"\n");
    let buf = buffer(&a);
    let s = text(&buf);

    // 地。
    assert_eq!(buf[(0u16, 0u16)].bg, NORD_BG, "左上の背景\n{s}");
    // 空けておく最下行と右端も地の色。
    assert_eq!(buf[(78u16, 23u16)].bg, NORD_BG, "最下行の背景\n{s}");
    // 右下の隅の1セルは書かない(自動折り返しを避ける)。
    assert_eq!(
        buf[(79u16, 23u16)].bg,
        Color::Reset,
        "右下の隅は塗らない\n{s}"
    );
    assert_eq!(buf[(79u16, 5u16)].bg, NORD_BG, "右端の背景\n{s}");
    // 1行目はヘッダーの色(テーマなしで色を持たないとき)。
    assert_eq!(base[(0u16, 0u16)].fg, Color::Reset);
    assert_eq!(buf[(0u16, 0u16)].fg, NORD_HEADER, "ヘッダーの前景\n{s}");

    // 選んでいるセル: テーマなしで反転の、最初のノートの行のセル。
    let dy = view::data_y(&a) as u16;
    let sel = reversed_on(&base, dy);
    assert!(
        !sel.is_empty(),
        "テーマなしで選んでいるセルが反転でない\n{}",
        text(&base)
    );
    for x in sel {
        let c = &buf[(x, dy)];
        assert_eq!(c.bg, NORD_SEL_BG, "選んでいるセル ({x},{dy}) の背景\n{s}");
        assert_eq!(c.fg, NORD_SEL_FG, "選んでいるセル ({x},{dy}) の前景\n{s}");
        assert!(
            !c.modifier.contains(Modifier::REVERSED),
            "選んでいるセル ({x},{dy}) の反転が外れる"
        );
    }

    // 下の帯: テーマなしで反転の、帯の行のセル。
    let band = reversed_on(&base, BAND);
    assert!(
        !band.is_empty(),
        "テーマなしで下の帯が反転でない\n{}",
        text(&base)
    );
    for x in band {
        let c = &buf[(x, BAND)];
        assert_eq!(c.bg, NORD_BAND_BG, "下の帯 ({x},{BAND}) の背景\n{s}");
        assert!(
            !c.modifier.contains(Modifier::REVERSED),
            "下の帯 ({x},{BAND}) の反転が外れる"
        );
    }
}

#[test]
fn test_sr_26_every_theme_keeps_the_text() {
    // [SR-26] 7つのどのテーマでも、画面の文字はテーマなしと1字も違わない(色だけを変える)。
    let tmp = vault("thtext");
    for color in [ColorMode::Rgb, ColorMode::Indexed] {
        let want = text(&buffer(&plain(&tmp, color)));
        for t in Theme::ALL {
            let a = themed(&tmp, color, &format!("theme = \"{}\"\n", t.name()));
            let got = text(&buffer(&a));
            assert_eq!(
                got,
                want,
                "テーマ {} ({color:?}) で文字が変わった",
                t.name()
            );
        }
    }
}

#[test]
fn test_sr_26_every_theme_but_default_paints() {
    // [SR-26] default 以外の6つは、左上の地をテーマの bg で塗る。
    let tmp = vault("thall");
    for t in Theme::ALL {
        let a = themed(&tmp, ColorMode::Rgb, &format!("theme = \"{}\"\n", t.name()));
        let buf = buffer(&a);
        match t.palette() {
            Some(p) => assert_eq!(buf[(0u16, 0u16)].bg, rgb(p.bg), "テーマ {}", t.name()),
            None => assert_eq!(t, Theme::Default, "パレットが無いのは default だけ"),
        }
    }
}

#[test]
fn test_sr_26_palettes_follow_the_mockups() {
    // [SR-26] 色の表(見本の試作の値)。順: bg fg header sel_bg sel_fg band_bg band_fg colhead strong
    // zebra_bg zebra_fg mark。
    let table: [(&str, [u32; 12]); 6] = [
        (
            "nord",
            [
                0x2e3440, 0xd8dee9, 0x88c0d0, 0x5e81ac, 0xeceff4, 0x3b4252, 0xa3be8c, 0x81a1c1,
                0xeceff4, 0x353c4a, 0xd8dee9, 0xebcb8b,
            ],
        ),
        (
            "solarized-light",
            [
                0xfdf6e3, 0x586e75, 0x268bd2, 0x268bd2, 0xfdf6e3, 0xeee8d5, 0x657b83, 0xcb4b16,
                0x073642, 0xf5efdc, 0x586e75, 0xd33682,
            ],
        ),
        (
            "dracula",
            [
                0x282a36, 0xf8f8f2, 0xbd93f9, 0x44475a, 0x50fa7b, 0x6272a4, 0xf8f8f2, 0xff79c6,
                0xf1fa8c, 0x2f3240, 0xf8f8f2, 0xffb86c,
            ],
        ),
        (
            "gruvbox",
            [
                0x282828, 0xebdbb2, 0xfabd2f, 0xd79921, 0x282828, 0x3c3836, 0xb8bb26, 0xfe8019,
                0xfbf1c7, 0x32302f, 0xebdbb2, 0xfb4934,
            ],
        ),
        (
            "pink-monster",
            [
                0x1f0f1c, 0xffd6ec, 0xff4fb8, 0xff2e97, 0x1f0f1c, 0xff2e97, 0x1f0f1c, 0xff79c6,
                0xfff07a, 0x2c1528, 0xffd6ec, 0xb6ff4a,
            ],
        ),
        (
            "dozy-pink",
            [
                0xfbf1e1, 0x6b4a3a, 0xd9668f, 0xf2a7bf, 0x4a2f25, 0xf2a7bf, 0x4a2f25, 0xc8875a,
                0xb8456f, 0xf6e6cf, 0x6b4a3a, 0xd9668f,
            ],
        ),
    ];
    let hex = |v: u32| [(v >> 16) as u8, (v >> 8) as u8, v as u8];
    for (name, v) in table {
        let t = Theme::parse(name).unwrap_or_else(|| panic!("{name} が読めない"));
        let p: Palette = t
            .palette()
            .unwrap_or_else(|| panic!("{name} のパレットが無い"));
        let got = [
            p.bg, p.fg, p.header, p.sel_bg, p.sel_fg, p.band_bg, p.band_fg, p.colhead, p.strong,
            p.zebra_bg, p.zebra_fg, p.mark,
        ];
        let want = v.map(hex);
        assert_eq!(got, want, "{name} の色");
    }
    assert!(Theme::Default.palette().is_none());
}

#[test]
fn test_sr_26_zebra_uses_theme_colors() {
    // [SR-26][SR-20] 一行おきの色(zebra = true)は、テーマの zebra_bg・zebra_fg で塗る。
    let tmp = vault("thzebra");
    let a = themed(
        &tmp,
        ColorMode::Rgb,
        "theme = \"nord\"\n\n[display]\nzebra = true\n",
    );
    let buf = buffer(&a);
    let s = text(&buf);
    let dy = view::data_y(&a);
    let mut even = 0;
    for (k, slot) in a.slots.iter().enumerate() {
        if let grid::Slot::Row(r) = slot {
            if r % 2 == 1 {
                even += 1;
                let c = &buf[(3u16, (dy + k) as u16)];
                assert_eq!(c.bg, NORD_ZEBRA_BG, "行番号 {} の背景\n{s}", r + 1);
                assert_eq!(c.fg, NORD_ZEBRA_FG, "行番号 {} の前景\n{s}", r + 1);
            } else if *r != a.row {
                let c = &buf[(3u16, (dy + k) as u16)];
                assert_eq!(c.bg, NORD_BG, "行番号 {} は地の色\n{s}", r + 1);
            }
        }
    }
    assert!(even >= 2, "偶数番目の行が2つ以上\n{s}");
}

// ---- SR-27: 設定・既定・色なし・256 色 ----

#[test]
fn test_sr_27_default_is_unchanged() {
    // [SR-27] 設定なしと theme = "default" の画面は、テーマの機能が無いときと各セルまで同じ。
    let tmp = vault("thdefault");
    for color in [ColorMode::Rgb, ColorMode::Indexed, ColorMode::None] {
        let want = buffer(&plain(&tmp, color));
        let none = buffer(&{
            let mut a = app_of(&tmp, color);
            a.look_classic = true;
            a
        });
        assert_same_cells(&none, &want, &format!("設定を当てない ({color:?})"));
        let empty = buffer(&themed(&tmp, color, ""));
        assert_same_cells(&empty, &want, &format!("設定なし ({color:?})"));
        let named = buffer(&themed(&tmp, color, "theme = \"default\"\n"));
        assert_same_cells(&named, &want, &format!("theme = \"default\" ({color:?})"));
    }
    assert_eq!(Config::default().theme, Theme::Default);
    assert_eq!(cfg("").theme, Theme::Default);
}

#[test]
fn test_sr_27_default_zebra_is_unchanged() {
    // [SR-27][SR-20] 既定のテーマの一行おきの色は今と同じ。
    let tmp = vault("thdefzebra");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let mut c = cfg("look = \"classic\"\n\n[display]\nzebra = true\n");
    c.theme = Theme::Default;
    a.configure(&c);
    let want = buffer(&a);
    let b = themed(
        &tmp,
        ColorMode::Rgb,
        "theme = \"default\"\n\n[display]\nzebra = true\n",
    );
    assert_same_cells(&buffer(&b), &want, "default と zebra");
}

#[test]
fn test_sr_27_theme_names() {
    // [SR-27][SR-37] 設定の名前は12(見本の7つと落ち着いた組の5つ)。知らない名前は None。
    let names = [
        ("default", Theme::Default),
        ("nord", Theme::Nord),
        ("solarized-light", Theme::SolarizedLight),
        ("dracula", Theme::Dracula),
        ("gruvbox", Theme::Gruvbox),
        ("pink-monster", Theme::PinkMonster),
        ("dozy-pink", Theme::DozyPink),
        ("sumi", Theme::Sumi),
        ("slate", Theme::Slate),
        ("saas", Theme::Saas),
        ("saas-dark", Theme::SaasDark),
        ("paper", Theme::Paper),
    ];
    for (name, t) in names {
        assert_eq!(Theme::parse(name), Some(t), "{name}");
        assert_eq!(t.name(), name);
        assert_eq!(
            cfg(&format!("theme = \"{name}\"\n")).theme,
            t,
            "設定の {name}"
        );
    }
    assert_eq!(Theme::ALL.len(), 12);
    assert_eq!(Theme::ALL, names.map(|(_, t)| t), "Theme::ALL の順");
    for bad in ["neon", "", "Nord", "solarized_light", "solarized"] {
        assert_eq!(Theme::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn test_sr_27_unknown_theme_warns_and_falls_back() {
    // [SR-27][CLI-3] theme = "neon" → 警告に `theme` と出て、既定の見た目で起動。
    let (c, warnings) = config::parse("theme = \"neon\"\n").expect("知らない名前でも読める");
    assert_eq!(c.theme, Theme::Default);
    assert!(
        warnings.iter().any(|w| w.contains("theme")),
        "警告に theme が無い: {warnings:?}"
    );
    // 型の違う値も同じ。
    let (c, warnings) = config::parse("theme = 3\n").expect("型の違う値でも読める");
    assert_eq!(c.theme, Theme::Default);
    assert!(
        warnings.iter().any(|w| w.contains("theme")),
        "警告に theme が無い: {warnings:?}"
    );
    // 正しい名前なら警告は出ない。
    let (_, warnings) = config::parse("theme = \"nord\"\n").unwrap();
    assert!(
        !warnings.iter().any(|w| w.contains("theme")),
        "正しい名前で警告: {warnings:?}"
    );

    // 画面も既定と同じ。
    let tmp = vault("thneon");
    let want = buffer(&plain(&tmp, ColorMode::Rgb));
    let got = buffer(&themed(&tmp, ColorMode::Rgb, "theme = \"neon\"\n"));
    assert_same_cells(&got, &want, "theme = \"neon\"");
}

#[test]
fn test_sr_27_no_color_ignores_theme() {
    // [SR-27][SR-10] 色を使わない表示(NO_COLOR と同じ ColorMode::None)では、theme = "dracula" でも色の指定が無い。
    let tmp = vault("thnocolor");
    let a = themed(&tmp, ColorMode::None, "theme = \"dracula\"\n");
    let buf = buffer(&a);
    let s = text(&buf);
    for (x, y) in cells(&buf) {
        let c = &buf[(x, y)];
        assert_eq!(c.fg, Color::Reset, "({x},{y}) の前景\n{s}");
        assert_eq!(c.bg, Color::Reset, "({x},{y}) の背景\n{s}");
    }
    assert_same_cells(&buf, &buffer(&plain(&tmp, ColorMode::None)), "色なし");
}

#[test]
fn test_sr_27_color_false_ignores_theme() {
    // [SR-27] 設定の color = false でも、テーマを書いても色の指定が無い。
    let tmp = vault("thcolorfalse");
    for color in [ColorMode::Rgb, ColorMode::Indexed] {
        let a = themed(&tmp, color, "color = false\ntheme = \"dracula\"\n");
        let buf = buffer(&a);
        let s = text(&buf);
        for (x, y) in cells(&buf) {
            let c = &buf[(x, y)];
            assert_eq!(c.fg, Color::Reset, "({x},{y}) の前景 ({color:?})\n{s}");
            assert_eq!(c.bg, Color::Reset, "({x},{y}) の背景 ({color:?})\n{s}");
        }
    }
}

#[test]
fn test_sr_27_indexed_uses_256_colors() {
    // [SR-27][SR-15] トゥルーカラーに対応しない端末(ColorMode::Indexed)では、近い 256 色の番号で塗る。
    let tmp = vault("thindexed");
    let base = buffer(&plain(&tmp, ColorMode::Indexed));
    let a = themed(&tmp, ColorMode::Indexed, "theme = \"nord\"\n");
    let buf = buffer(&a);
    let s = text(&buf);
    assert_eq!(
        buf[(0u16, 0u16)].bg,
        Color::Indexed(to_indexed([0x2e, 0x34, 0x40])),
        "左上の背景\n{s}"
    );
    let dy = view::data_y(&a) as u16;
    let sel = reversed_on(&base, dy);
    assert!(!sel.is_empty());
    for x in sel {
        assert_eq!(
            buf[(x, dy)].bg,
            Color::Indexed(to_indexed([0x5e, 0x81, 0xac])),
            "選んでいるセル ({x},{dy})\n{s}"
        );
    }
    for (x, y) in cells(&buf) {
        let c = &buf[(x, y)];
        assert!(
            !matches!(c.fg, Color::Rgb(..)) && !matches!(c.bg, Color::Rgb(..)),
            "({x},{y}) にトゥルーカラー: {:?} / {:?}\n{s}",
            c.fg,
            c.bg
        );
    }
}

#[test]
fn test_sr_27_to_indexed_picks_nearest() {
    // [SR-27][SR-15] to_indexed は xterm の 256 色の表(16〜255 の色の立方体と灰色)で最も近い番号。
    assert_eq!(to_indexed([0, 0, 0]), 16);
    assert_eq!(to_indexed([255, 255, 255]), 231);
    assert_eq!(to_indexed([255, 0, 0]), 196);
    assert_eq!(to_indexed([0, 0, 255]), 21);
    assert_eq!(to_indexed([0x5f, 0x87, 0xaf]), 67);
    // 灰色の段(232 + i、値 8 + 10i)。
    assert_eq!(to_indexed([0x80, 0x80, 0x80]), 244);
    assert_eq!(to_indexed([8, 8, 8]), 232);
    for t in Theme::ALL {
        if let Some(p) = t.palette() {
            for c in [p.bg, p.fg, p.sel_bg, p.band_bg, p.zebra_bg] {
                assert!(to_indexed(c) >= 16, "{} の {c:?}", t.name());
            }
        }
    }
}
