//! 色の上書き(SR-40・SR-41): `[look.colors]` でテーマの色の役割ごとに色を変え、`[look.colors.values]` で値ごとの色を
//! 決める。色は `"#rrggbb"`・`"#rgb"` か名前。役割と名前の並びはカタログ(docs/catalog/index.html。SR-38)と
//! 同じにし、試験で突き合わせる。

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

/// 値の色の照合の鍵(前後の空白を除いて小文字)。
pub fn value_key(s: &str) -> String {
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
        let k = value_key(item);
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

/// `"#rrggbb"` の書き方(画面が書くファイルに色を書くとき)。
pub fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

#[cfg(test)]
#[path = "test_colors_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_bounds_unit.rs"]
mod bounds;
