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
    pub const ALL: [Theme; 7] = [
        Theme::Default,
        Theme::Nord,
        Theme::SolarizedLight,
        Theme::Dracula,
        Theme::Gruvbox,
        Theme::PinkMonster,
        Theme::DozyPink,
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
