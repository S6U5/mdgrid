//! 画面のテーマ(SR-26・SR-27)。設定の名前と、役割ごとの色の表(パレット)。
//! 画面への塗り替えは実行ファイルの側(src/ui/theme.rs)。設計は docs/design.md の「テーマ」。

/// 画面のテーマ。`Default` は今の見た目(色の表を持たない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Default,
    Nord,
    SolarizedLight,
    Dracula,
    Gruvbox,
    PinkMonster,
    DozyPink,
    /// SR-37: 落ち着いた組。
    Sumi,
    Slate,
    Saas,
    SaasDark,
    Paper,
}

/// 役割ごとの色(RGB)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// 地。
    pub bg: [u8; 3],
    /// 文字。
    pub fg: [u8; 3],
    /// 1行目(ヘッダー)の文字。
    pub header: [u8; 3],
    /// 選んでいるセル。
    pub sel_bg: [u8; 3],
    pub sel_fg: [u8; 3],
    /// 下の帯。
    pub band_bg: [u8; 3],
    pub band_fg: [u8; 3],
    /// 列の見出し(太字と下線)。
    pub colhead: [u8; 3],
    /// 強調(太字)。
    pub strong: [u8; 3],
    /// 一行おきの色(SR-20)。
    pub zebra_bg: [u8; 3],
    pub zebra_fg: [u8; 3],
    /// 左端の印(`>`)。
    pub mark: [u8; 3],
    /// 保存の確認の差分の足した行・消した行の文字(SR-28)。
    pub add: [u8; 3],
    pub del: [u8; 3],
    /// ためる変更のセルの文字(SR-28)。
    pub pending: [u8; 3],
    /// 検索の一致と同じ値の強調の地(SR-28)。
    pub hl_bg: [u8; 3],
}

const fn hex(v: u32) -> [u8; 3] {
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// 色の表の1行(順: bg fg header sel_bg sel_fg band_bg band_fg colhead strong zebra_bg zebra_fg mark
/// add del pending hl_bg)。
const fn palette(v: [u32; 16]) -> Palette {
    Palette {
        bg: hex(v[0]),
        fg: hex(v[1]),
        header: hex(v[2]),
        sel_bg: hex(v[3]),
        sel_fg: hex(v[4]),
        band_bg: hex(v[5]),
        band_fg: hex(v[6]),
        colhead: hex(v[7]),
        strong: hex(v[8]),
        zebra_bg: hex(v[9]),
        zebra_fg: hex(v[10]),
        mark: hex(v[11]),
        add: hex(v[12]),
        del: hex(v[13]),
        pending: hex(v[14]),
        hl_bg: hex(v[15]),
    }
}

impl Theme {
    /// 全部のテーマ(設定の文書と同じ順)。
    pub const ALL: [Theme; 12] = [
        Theme::Default,
        Theme::Nord,
        Theme::SolarizedLight,
        Theme::Dracula,
        Theme::Gruvbox,
        Theme::PinkMonster,
        Theme::DozyPink,
        Theme::Sumi,
        Theme::Slate,
        Theme::Saas,
        Theme::SaasDark,
        Theme::Paper,
    ];

    /// 設定の名前。
    pub fn name(self) -> &'static str {
        match self {
            Theme::Default => "default",
            Theme::Nord => "nord",
            Theme::SolarizedLight => "solarized-light",
            Theme::Dracula => "dracula",
            Theme::Gruvbox => "gruvbox",
            Theme::PinkMonster => "pink-monster",
            Theme::DozyPink => "dozy-pink",
            Theme::Sumi => "sumi",
            Theme::Slate => "slate",
            Theme::Saas => "saas",
            Theme::SaasDark => "saas-dark",
            Theme::Paper => "paper",
        }
    }

    /// 地が明るい組か(SR-39 の auto で、明るい地の端末に合わせる見本の判断にも使う)。
    pub fn is_light(self) -> bool {
        self.palette()
            .is_some_and(|p| (p.bg[0] as u32 + p.bg[1] as u32 + p.bg[2] as u32) > 3 * 128)
    }

    /// SR-39: `theme = "auto"` の解決。地が明るいと分かれば `light`、それ以外(暗い・分からない)は `dark`。
    pub fn auto(bg_light: Option<bool>, light: Theme, dark: Theme) -> Theme {
        if bg_light == Some(true) {
            light
        } else {
            dark
        }
    }

    /// 設定の名前から(大文字・小文字は区別する)。知らない名前は None。
    pub fn parse(s: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.name() == s)
    }

    /// 役割ごとの色。`Default` は None(塗り替えない)。
    pub fn palette(self) -> Option<Palette> {
        Some(palette(match self {
            Theme::Default => return None,
            Theme::Nord => [
                0x2e3440, 0xd8dee9, 0x88c0d0, 0x5e81ac, 0xeceff4, 0x3b4252, 0xa3be8c, 0x81a1c1,
                0xeceff4, 0x353c4a, 0xd8dee9, 0xebcb8b, 0xa3be8c, 0xbf616a, 0xebcb8b, 0x4c566a,
            ],
            Theme::SolarizedLight => [
                0xfdf6e3, 0x586e75, 0x268bd2, 0x268bd2, 0xfdf6e3, 0xeee8d5, 0x657b83, 0xcb4b16,
                0x073642, 0xf5efdc, 0x586e75, 0xd33682, 0x859900, 0xdc322f, 0xb58900, 0xd5e5f0,
            ],
            Theme::Dracula => [
                0x282a36, 0xf8f8f2, 0xbd93f9, 0x44475a, 0x50fa7b, 0x6272a4, 0xf8f8f2, 0xff79c6,
                0xf1fa8c, 0x2f3240, 0xf8f8f2, 0xffb86c, 0x50fa7b, 0xff5555, 0xffb86c, 0x6272a4,
            ],
            Theme::Gruvbox => [
                0x282828, 0xebdbb2, 0xfabd2f, 0xd79921, 0x282828, 0x3c3836, 0xb8bb26, 0xfe8019,
                0xfbf1c7, 0x32302f, 0xebdbb2, 0xfb4934, 0xb8bb26, 0xfb4934, 0xfabd2f, 0x665c54,
            ],
            Theme::PinkMonster => [
                0x1f0f1c, 0xffd6ec, 0xff4fb8, 0xff2e97, 0x1f0f1c, 0xff2e97, 0x1f0f1c, 0xff79c6,
                0xfff07a, 0x2c1528, 0xffd6ec, 0xb6ff4a, 0xb6ff4a, 0xff5c5c, 0xfff07a, 0x5a2650,
            ],
            Theme::DozyPink => [
                0xfbf1e1, 0x6b4a3a, 0xd9668f, 0xf2a7bf, 0x4a2f25, 0xf2a7bf, 0x4a2f25, 0xc8875a,
                0xb8456f, 0xf6e6cf, 0x6b4a3a, 0xd9668f, 0x4f7a2e, 0xb03a2e, 0x9a5a10, 0xf3d9a8,
            ],
            // SR-37: 墨(暗い地・薄い灰・青緑のアクセント)。
            Theme::Sumi => [
                0x16171b, 0xd8d6d0, 0xd8d6d0, 0x23262c, 0xece9e2, 0x16171b, 0x8a8780, 0x86b8ad,
                0xece9e2, 0x1a1b20, 0xd8d6d0, 0x86b8ad, 0x86b8ad, 0xd97b6c, 0xd6a85c, 0x2a3a37,
            ],
            // 石板(紫寄りのアクセント)。
            Theme::Slate => [
                0x1b1c25, 0xe1e2ea, 0xe1e2ea, 0x2a2c3d, 0xf0f0f8, 0x1b1c25, 0x8e90aa, 0x9b9cf7,
                0xf4f4fb, 0x1f2029, 0xe1e2ea, 0x9b9cf7, 0x7ec699, 0xef7d7d, 0xe6b450, 0x34355a,
            ],
            // SaaS(明るい灰の地・藍のアクセント)と、その暗い地の版。
            Theme::Saas => [
                0xf6f7f9, 0x1e2430, 0x1e2430, 0xe6e9fb, 0x1e2430, 0xf6f7f9, 0x6b7383, 0x5b5bd6,
                0x0f1420, 0xeef0f4, 0x1e2430, 0x5b5bd6, 0x2f9e66, 0xd4483f, 0xc47f17, 0xe4e6fb,
            ],
            Theme::SaasDark => [
                0x14161c, 0xe6e8ee, 0xe6e8ee, 0x252a3a, 0xf2f3f8, 0x14161c, 0x8b93a5, 0x8b8cf8,
                0xf5f6fa, 0x181b22, 0xe6e8ee, 0x8b8cf8, 0x4cc38a, 0xf0736a, 0xe7a93b, 0x2b2f4a,
            ],
            // 紙(明るい地・青のアクセント)。
            Theme::Paper => [
                0xf7f8fa, 0x1f2328, 0x1f2328, 0xdfe7f8, 0x1f2328, 0xf7f8fa, 0x6b7280, 0x2f63d8,
                0x0d1117, 0xeff1f4, 0x1f2328, 0x2f63d8, 0x2f8a4a, 0xc23b3b, 0xa86b00, 0xdbe5fb,
            ],
        }))
    }
}

/// xterm の 256 色の色の立方体の各段の値。
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// RGB に最も近い xterm の 256 色の番号(SR-15・SR-27)。16〜231 の色の立方体と 232〜255 の灰色から選ぶ
/// (0〜15 は端末ごとに色が違うので使わない)。距離は RGB の二乗の和。同じ距離なら立方体を先に取る。
pub fn to_indexed(c: [u8; 3]) -> u8 {
    let dist = |a: [u8; 3]| -> u32 {
        (0..3)
            .map(|i| {
                let d = i32::from(a[i]) - i32::from(c[i]);
                (d * d) as u32
            })
            .sum()
    };
    let near = |v: u8| -> usize {
        (0..6)
            .min_by_key(|&k| (i32::from(CUBE[k]) - i32::from(v)).abs())
            .unwrap_or(0)
    };
    let (r, g, b) = (near(c[0]), near(c[1]), near(c[2]));
    let mut best = (16 + 36 * r + 6 * g + b) as u8;
    let mut best_d = dist([CUBE[r], CUBE[g], CUBE[b]]);
    for i in 0..24u8 {
        let v = 8 + 10 * i;
        let d = dist([v, v, v]);
        if d < best_d {
            best = 232 + i;
            best_d = d;
        }
    }
    best
}

/// SR-39: 環境変数 `COLORFGBG`(`文字;地` か `文字;他;地`)の地の色の番号から、地が明るいか。読めなければ None。
/// 地の番号 7(明るい灰)と 9〜15(明るい色)を明るいとみなす。
pub fn light_from_colorfgbg(v: &str) -> Option<bool> {
    let bg: u8 = v.rsplit(';').next()?.trim().parse().ok()?;
    Some(matches!(bg, 7 | 9..=15))
}

/// SR-39: 端末への問い合わせ(OSC 11)の答え `ESC ] 11 ; rgb:RRRR/GGGG/BBBB` から、地が明るいか。
/// 読めなければ None。明るさは輝度(sRGB の重み)で半分より上を明るいとみなす。
pub fn light_from_osc11(reply: &str) -> Option<bool> {
    let rest = &reply[reply.find("rgb:")? + 4..];
    let mut ch = [0f64; 3];
    for (k, part) in rest.split('/').take(3).enumerate() {
        let hex: String = part.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if hex.is_empty() || hex.len() > 4 {
            return None;
        }
        let max = (16u32.pow(hex.len() as u32) - 1) as f64;
        ch[k] = u32::from_str_radix(&hex, 16).ok()? as f64 / max;
    }
    if rest.split('/').count() < 3 {
        return None;
    }
    Some(0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2] > 0.5)
}

#[cfg(test)]
#[path = "test_theme_auto_unit.rs"]
mod test_theme_auto;
