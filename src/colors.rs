//! 色の上書き(SR-40・SR-41): `[colors]` でテーマの色の役割ごとに色を変え、`[colors.values]` で値ごとの色を
//! 決める。色は `"#rrggbb"`・`"#rgb"` か名前。役割と名前の並びはカタログ(docs/catalog/index.html。SR-38)と
//! 同じにし、試験で突き合わせる。

use crate::i18n::Msg;
use crate::theme::Palette;

/// 役割の名前(設定の名前。並びは Palette の欄の順)。
pub const ROLES: &[&str] = &[
    "background",
    "text",
    "header",
    "selection",
    "selection_text",
    "band",
    "band_text",
    "accent",
    "strong",
    "zebra",
    "zebra_text",
    "mark",
    "added",
    "removed",
    "pending",
    "highlight",
];

/// 色の名前と色(どの地でも見分けられる中ほどの明るさ)。
pub const NAMED: &[(&str, [u8; 3])] = &[
    ("black", [17, 17, 17]),
    ("white", [245, 245, 245]),
    ("gray", [128, 132, 140]),
    ("red", [220, 76, 70]),
    ("orange", [234, 140, 52]),
    ("yellow", [214, 176, 52]),
    ("green", [70, 168, 98]),
    ("teal", [40, 160, 150]),
    ("cyan", [60, 170, 210]),
    ("blue", [70, 120, 220]),
    ("purple", [140, 100, 220]),
    ("magenta", [200, 80, 190]),
    ("pink", [232, 110, 160]),
    ("brown", [150, 100, 60]),
];

/// 色の上書き。役割は Palette の欄の順。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Colors {
    pub roles: [Option<[u8; 3]>; 16],
    /// 値の文字(前後の空白を除いて小文字にしたもの)と色。
    pub values: Vec<(String, [u8; 3])>,
}

/// 色の書き方を読む。読めなければ None。
pub fn parse_color(s: &str) -> Option<[u8; 3]> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('#') {
        if !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let v = |i: usize, n: usize| u8::from_str_radix(&h[i..i + n], 16).ok();
        return match h.len() {
            6 => Some([v(0, 2)?, v(2, 2)?, v(4, 2)?]),
            3 => Some([v(0, 1)? * 17, v(1, 1)? * 17, v(2, 1)? * 17]),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    NAMED.iter().find(|(n, _)| *n == lower).map(|(_, c)| *c)
}

fn key(s: &str) -> String {
    s.trim().to_lowercase()
}

impl Colors {
    /// 役割 `name` の上書き。
    pub fn role(&self, name: &str) -> Option<[u8; 3]> {
        let i = ROLES.iter().position(|r| *r == name)?;
        self.roles[i]
    }

    /// 値 `item` の色(SR-41)。決めていなければ None。
    pub fn value(&self, item: &str) -> Option<[u8; 3]> {
        let k = key(item);
        self.values.iter().find(|(v, _)| *v == k).map(|(_, c)| *c)
    }

    /// テーマの色の表に上書きを当てる。
    pub fn apply(&self, p: Palette) -> Palette {
        let mut v = [
            p.bg, p.fg, p.header, p.sel_bg, p.sel_fg, p.band_bg, p.band_fg, p.colhead, p.strong,
            p.zebra_bg, p.zebra_fg, p.mark, p.add, p.del, p.pending, p.hl_bg,
        ];
        for (slot, o) in v.iter_mut().zip(self.roles.iter()) {
            if let Some(c) = o {
                *slot = *c;
            }
        }
        Palette {
            bg: v[0],
            fg: v[1],
            header: v[2],
            sel_bg: v[3],
            sel_fg: v[4],
            band_bg: v[5],
            band_fg: v[6],
            colhead: v[7],
            strong: v[8],
            zebra_bg: v[9],
            zebra_fg: v[10],
            mark: v[11],
            add: v[12],
            del: v[13],
            pending: v[14],
            hl_bg: v[15],
        }
    }
}

/// `[colors]` を読む。`values` の下は値の色。知らない役割・読めない色は警告して無視。
pub fn read(value: &toml::Value, out: &mut Colors, warnings: &mut Vec<String>) {
    let Some(t) = value.as_table() else {
        warnings.push(Msg::ConfigWrongType.fill(&[&"colors", &Msg::WantColorsTable.text()]));
        return;
    };
    for (k, v) in t {
        if k == "values" {
            let Some(vt) = v.as_table() else {
                warnings.push(
                    Msg::ConfigWrongType.fill(&[&"colors.values", &Msg::WantColorsTable.text()]),
                );
                continue;
            };
            for (item, c) in vt {
                let name = format!("colors.values.{item}");
                match c.as_str().and_then(parse_color) {
                    Some(c) => {
                        let k = key(item);
                        if out.values.iter().any(|(v, _)| *v == k) {
                            warnings.push(Msg::ColorValueTwice.fill(&[&name]));
                        }
                        out.values.retain(|(v, _)| *v != k);
                        out.values.push((k, c));
                    }
                    None => {
                        warnings.push(Msg::ConfigWrongType.fill(&[&name, &Msg::WantColor.text()]))
                    }
                }
            }
            continue;
        }
        let name = format!("colors.{k}");
        let Some(i) = ROLES.iter().position(|r| r == k) else {
            warnings.push(Msg::ConfigUnknownItem.fill(&[&name]));
            continue;
        };
        match v.as_str().and_then(parse_color) {
            Some(c) => out.roles[i] = Some(c),
            None => warnings.push(Msg::ConfigWrongType.fill(&[&name, &Msg::WantColor.text()])),
        }
    }
}

#[cfg(test)]
#[path = "test_colors_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_bounds_unit.rs"]
mod bounds;
