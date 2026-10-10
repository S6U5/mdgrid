//! 札の色(SR-35): 値の文字から決まる8色。どの端末の地でも読めるよう、地と文字の色を両方決める。

use super::app::{App, ColorMode};
use mdgrid::theme::to_indexed;
use ratatui::style::{Color, Style};

/// (地, 文字)。落ち着いた中ほどの明るさの地に、明るい文字。
const PAIRS: [([u8; 3], [u8; 3]); 8] = [
    ([44, 82, 130], [235, 240, 248]),
    ([40, 110, 70], [232, 246, 236]),
    ([100, 70, 140], [242, 236, 250]),
    ([150, 90, 30], [252, 242, 230]),
    ([140, 50, 60], [250, 234, 236]),
    ([30, 110, 120], [230, 246, 248]),
    ([140, 60, 110], [250, 234, 244]),
    ([85, 85, 95], [238, 238, 242]),
];

/// 値の文字の指紋(FNV-1a)。同じ値はいつも同じ色。
fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

/// 値 `item` の札の見た目。色を使わない表示では None。
pub(crate) fn style(app: &App, item: &str) -> Option<Style> {
    let c = |rgb: [u8; 3]| match app.color {
        ColorMode::Indexed => Some(Color::Indexed(to_indexed(rgb))),
        ColorMode::Rgb => Some(Color::Rgb(rgb[0], rgb[1], rgb[2])),
        ColorMode::None => None,
    };
    // SR-41: 設定で決めた値の色があれば、それを地にする(文字は地の明るさで黒か白)。
    if let Some(bg) = app.colors.value(item) {
        return Some(Style::default().bg(c(bg)?).fg(c(contrast(bg))?));
    }
    let (bg, fg) = PAIRS[(hash(item) % PAIRS.len() as u64) as usize];
    Some(Style::default().bg(c(bg)?).fg(c(fg)?))
}

// ---- 値の部品の形(SR-36) ----

/// 部品の1切れ(文字と、見た目。None は行の見た目のまま)。
pub(crate) type Seg = (String, Option<Style>);

/// 値の部品: 前置き、要素ごとの切れ、要素の間の切れ。文字は前置き + 要素を区切りでつないだもの。
pub(crate) struct Parts {
    pub lead: Vec<Seg>,
    pub items: Vec<Vec<Seg>>,
    pub sep: Vec<Seg>,
}

impl Parts {
    /// 見せる文字の全部(幅を数えるのにも使う)。
    pub(crate) fn text(&self) -> String {
        let mut s: String = self.lead.iter().map(|(t, _)| t.as_str()).collect();
        for (k, item) in self.items.iter().enumerate() {
            if k > 0 {
                s.extend(self.sep.iter().map(|(t, _)| t.as_str()));
            }
            s.extend(item.iter().map(|(t, _)| t.as_str()));
        }
        s
    }
}

fn rgb(app: &App, c: [u8; 3]) -> Option<Color> {
    match app.color {
        ColorMode::Indexed => Some(Color::Indexed(to_indexed(c))),
        ColorMode::Rgb => Some(Color::Rgb(c[0], c[1], c[2])),
        ColorMode::None => None,
    }
}

/// 値の意味の分類(よく使う英語の状態の語)。分からなければ None(値の文字から決まる色)。
fn meaning(item: &str) -> Option<usize> {
    let w = item.trim().to_lowercase();
    const DONE: &[&str] = &["done", "closed", "complete", "completed", "finished", "yes"];
    const DOING: &[&str] = &[
        "doing",
        "in progress",
        "in-progress",
        "wip",
        "active",
        "started",
        "review",
    ];
    const BAD: &[&str] = &["blocked", "failed", "error", "stuck"];
    const TODO: &[&str] = &["todo", "to do", "open", "new", "backlog", "next"];
    [DONE, DOING, BAD, TODO]
        .iter()
        .position(|ws| ws.contains(&w.as_str()))
}

/// 値の色(点・形・文字の色・淡い札に使う)。状態の語は意味の色、ほかは値の文字から決まる色。
fn hue(app: &App, item: &str) -> [u8; 3] {
    // SR-41: 設定で決めた値の色が先。
    if let Some(c) = app.colors.value(item) {
        return c;
    }
    let pal = app.palette();
    match meaning(item) {
        Some(0) => pal.map_or([126, 198, 138], |p| p.add),
        Some(1) => pal.map_or([217, 178, 95], |p| p.pending),
        Some(2) => pal.map_or([224, 108, 108], |p| p.del),
        // todo などは薄い文字と同じ濃さ(地と文字の色から)。
        Some(_) => pal.map_or([122, 127, 136], |p| mixc(p.fg, p.bg, 0.3)),
        None => {
            let (bg, fg) = PAIRS[(hash(item) % PAIRS.len() as u64) as usize];
            // 暗い地では札の地の色を明るくし、明るい地では少し暗くして、文字の色として読めるように。
            if app.theme.is_light() {
                mixc(bg, [0, 0, 0], 0.1)
            } else {
                mixc(bg, fg, 0.35)
            }
        }
    }
}

fn mixc(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let m = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
}

/// 地の色(テーマ。既定のテーマは端末の地が分からないので暗い地とみなす)。
fn ground(app: &App) -> [u8; 3] {
    app.palette().map_or([28, 29, 32], |p| p.bg)
}

fn fg_of(app: &App, c: [u8; 3]) -> Option<Style> {
    rgb(app, c).map(|c| Style::default().fg(c))
}

/// 淡い札(SaaS): 地に値の色を薄く混ぜた地と、値の色の文字(明るい地では濃く)。
fn tint(app: &App, item: &str) -> Option<Style> {
    let h = hue(app, item);
    let g = ground(app);
    let light = app.theme.is_light();
    let bg = mixc(g, h, if light { 0.16 } else { 0.22 });
    let fg = if light {
        mixc(h, [0, 0, 0], 0.5)
    } else {
        mixc(h, [255, 255, 255], 0.5)
    };
    Some(Style::default().bg(rgb(app, bg)?).fg(rgb(app, fg)?))
}

/// 透けない札(SR-36 の solid): 値の色をそのまま地にし、文字は地の明るさで黒か白。
fn solid(app: &App, item: &str) -> Option<Style> {
    let h = hue(app, item);
    let lum = 0.2126 * h[0] as f32 + 0.7152 * h[1] as f32 + 0.0722 * h[2] as f32;
    let fg = if lum > 150.0 {
        [20, 20, 24]
    } else {
        [250, 250, 252]
    };
    Some(Style::default().bg(rgb(app, h)?).fg(rgb(app, fg)?))
}

/// 値の札の地の色(丸い札・角を落とした札の端の色)。
fn chip_bg(app: &App, item: &str) -> Option<Color> {
    style(app, item).and_then(|s| s.bg)
}

/// 状態の形の記号(意味が分からなければ ◆)。
fn shape(item: &str) -> &'static str {
    match meaning(item) {
        Some(0) => "●",
        Some(1) => "◐",
        Some(2) => "⊘",
        Some(_) => "○",
        None => "◆",
    }
}

/// 値 `items` を部品の形で(SR-36)。`list` はリストの列、そうでなければ札の列(くり返す短い値)。
/// 部品にしない形(plain)なら None(呼ぶ側が値の文字を使う)。
pub(crate) fn parts(app: &App, items: &[String], list: bool) -> Option<Parts> {
    use mdgrid::style::{Status, Tags};
    // 丸い札は Nerd Font のときだけ(無ければ角を落とした札)。
    #[derive(PartialEq)]
    enum K {
        Dot,
        Shape,
        Text,
        Pill,
        Tint,
        Solid,
        Soft,
        Chip,
        Dots,
        Hash,
        Brackets,
    }
    let k = if list {
        match app.style.tags {
            Tags::Dots => K::Dots,
            Tags::Hash => K::Hash,
            Tags::Brackets => K::Brackets,
            Tags::Pill if app.nerd_font => K::Pill,
            Tags::Pill | Tags::Soft => K::Soft,
            Tags::Tint => K::Tint,
            Tags::Solid => K::Solid,
            Tags::Chip => K::Chip,
            Tags::Plain => return None,
        }
    } else {
        match app.style.status {
            Status::Dot => K::Dot,
            Status::Shape => K::Shape,
            Status::Text => K::Text,
            Status::Pill if app.nerd_font => K::Pill,
            Status::Pill | Status::Soft => K::Soft,
            Status::Tint => K::Tint,
            Status::Solid => K::Solid,
            Status::Chip => K::Chip,
            Status::Plain => return None,
        }
    };
    let none = |t: &str| -> Seg { (t.to_string(), None) };
    let dim = fg_of(
        app,
        app.palette()
            .map_or([110, 112, 118], |p| mixc(p.fg, p.bg, 0.3)),
    );
    let item_segs = |v: &String| -> Vec<Seg> {
        // 値の文字は無害にしてから(端末の制御の文字を画面に出さない)。
        let v = &super::width::sanitize(v);
        let h = hue(app, v);
        match k {
            K::Dot => vec![("● ".into(), fg_of(app, h)), none(v)],
            K::Shape => vec![(format!("{} ", shape(v)), fg_of(app, h)), none(v)],
            K::Text => vec![(v.clone(), fg_of(app, h))],
            K::Pill | K::Soft => {
                let (l, r) = if k == K::Pill {
                    ("\u{e0b6}", "\u{e0b4}")
                } else {
                    ("▐", "▌")
                };
                let cap = chip_bg(app, v).map(|c| Style::default().fg(c));
                vec![(l.into(), cap), (v.clone(), style(app, v)), (r.into(), cap)]
            }
            // 淡い札。Nerd Font があれば、両端の空白を丸い端の字にする(幅は同じ)。
            K::Tint if !list => {
                let t = tint(app, v);
                let dot = t.map(|s| s.fg(rgb(app, h).unwrap_or(Color::Reset)));
                match tint_caps(app, t) {
                    Some((l, r)) => vec![l, ("●".into(), dot), (format!(" {v}"), t), r],
                    None => vec![(" ".into(), t), ("●".into(), dot), (format!(" {v} "), t)],
                }
            }
            K::Tint => {
                let t = tint(app, v);
                match tint_caps(app, t) {
                    Some((l, r)) => vec![l, (v.clone(), t), r],
                    None => vec![(format!(" {v} "), t)],
                }
            }
            // 透けない札(値の色をそのまま地に)。Nerd Font があれば両端を丸く。
            K::Solid => {
                let t = solid(app, v);
                match tint_caps(app, t) {
                    Some((l, r)) => vec![l, (v.clone(), t), r],
                    None => vec![(format!(" {v} "), t)],
                }
            }
            K::Chip => vec![(format!(" {v} "), style(app, v))],
            K::Dots => vec![none(v)],
            K::Hash => vec![(format!("#{v}"), fg_of(app, h))],
            K::Brackets => vec![("[".into(), dim), (v.clone(), None), ("]".into(), dim)],
        }
    };
    let (lead, sep) = match k {
        // 四角い札は今までの形(頭に地の色の無い空白1つ)。
        K::Chip => (vec![none(" ")], vec![none(" ")]),
        K::Dots => (vec![], vec![(" · ".into(), dim)]),
        _ => (vec![], vec![none(" ")]),
    };
    Some(Parts {
        lead,
        items: items.iter().map(item_segs).collect(),
        sep,
    })
}

/// 窓(候補の一覧・値の件数)の値に重ねる見た目(SR-36)。文字と幅は変えないので、形のうち色だけを写す:
/// 札の形は札の色、淡い札は淡い地、点・形・文字の色・タグの色の形は値の色の文字。部品にしない形は None。
pub(crate) fn overlay(app: &App, item: &str, list: bool) -> Option<Style> {
    use mdgrid::style::{Status, Tags};
    enum K {
        Chip,
        Tint,
        Solid,
        Hue,
    }
    let k = if list {
        match app.style.tags {
            Tags::Chip | Tags::Pill | Tags::Soft => K::Chip,
            Tags::Tint => K::Tint,
            Tags::Solid => K::Solid,
            Tags::Hash => K::Hue,
            Tags::Dots | Tags::Brackets | Tags::Plain => return None,
        }
    } else {
        match app.style.status {
            Status::Chip | Status::Pill | Status::Soft => K::Chip,
            Status::Tint => K::Tint,
            Status::Solid => K::Solid,
            Status::Dot | Status::Shape | Status::Text => K::Hue,
            Status::Plain => return None,
        }
    };
    match k {
        K::Chip => style(app, item),
        K::Tint => tint(app, item),
        K::Solid => solid(app, item),
        K::Hue => fg_of(app, hue(app, item)),
    }
}

/// 窓の候補も丸い札(SR-36 の pill・tint と nerd_font)で描くか。描くなら窓を2桁広げ、両端に丸い端の字を置く。
pub(crate) fn pill_caps(app: &App, list: bool) -> bool {
    use mdgrid::style::{Status, Tags};
    app.nerd_font
        && app.color != ColorMode::None
        && if list {
            matches!(app.style.tags, Tags::Pill | Tags::Tint | Tags::Solid)
        } else {
            matches!(
                app.style.status,
                Status::Pill | Status::Tint | Status::Solid
            )
        }
}

/// 淡い札の丸い端(Nerd Font のときだけ)。端の字は札の地の色。
fn tint_caps(app: &App, t: Option<Style>) -> Option<(Seg, Seg)> {
    if !app.nerd_font {
        return None;
    }
    let cap = Style::default().fg(t?.bg?);
    Some((
        ("\u{e0b6}".into(), Some(cap)),
        ("\u{e0b4}".into(), Some(cap)),
    ))
}

/// 地 `bg` の上で読める文字の色(明るい地には黒、暗い地には白)。
fn contrast(bg: [u8; 3]) -> [u8; 3] {
    let lum = 0.2126 * bg[0] as f32 + 0.7152 * bg[1] as f32 + 0.0722 * bg[2] as f32;
    if lum > 150.0 {
        [20, 20, 24]
    } else {
        [250, 250, 252]
    }
}
