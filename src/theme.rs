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
        self.palette().is_some_and(|p| {
            (u32::from(p.bg[0]) + u32::from(p.bg[1]) + u32::from(p.bg[2])) > 3 * 128
        })
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
                0x002e_3440,
                0x00d8_dee9,
                0x0088_c0d0,
                0x005e_81ac,
                0x00ec_eff4,
                0x003b_4252,
                0x00a3_be8c,
                0x0081_a1c1,
                0x00ec_eff4,
                0x0035_3c4a,
                0x00d8_dee9,
                0x00eb_cb8b,
                0x00a3_be8c,
                0x00bf_616a,
                0x00eb_cb8b,
                0x004c_566a,
            ],
            Theme::SolarizedLight => [
                0x00fd_f6e3,
                0x0058_6e75,
                0x0026_8bd2,
                0x0026_8bd2,
                0x00fd_f6e3,
                0x00ee_e8d5,
                0x0065_7b83,
                0x00cb_4b16,
                0x0007_3642,
                0x00f5_efdc,
                0x0058_6e75,
                0x00d3_3682,
                0x0085_9900,
                0x00dc_322f,
                0x00b5_8900,
                0x00d5_e5f0,
            ],
            Theme::Dracula => [
                0x0028_2a36,
                0x00f8_f8f2,
                0x00bd_93f9,
                0x0044_475a,
                0x0050_fa7b,
                0x0062_72a4,
                0x00f8_f8f2,
                0x00ff_79c6,
                0x00f1_fa8c,
                0x002f_3240,
                0x00f8_f8f2,
                0x00ff_b86c,
                0x0050_fa7b,
                0x00ff_5555,
                0x00ff_b86c,
                0x0062_72a4,
            ],
            Theme::Gruvbox => [
                0x0028_2828,
                0x00eb_dbb2,
                0x00fa_bd2f,
                0x00d7_9921,
                0x0028_2828,
                0x003c_3836,
                0x00b8_bb26,
                0x00fe_8019,
                0x00fb_f1c7,
                0x0032_302f,
                0x00eb_dbb2,
                0x00fb_4934,
                0x00b8_bb26,
                0x00fb_4934,
                0x00fa_bd2f,
                0x0066_5c54,
            ],
            Theme::PinkMonster => [
                0x001f_0f1c,
                0x00ff_d6ec,
                0x00ff_4fb8,
                0x00ff_2e97,
                0x001f_0f1c,
                0x00ff_2e97,
                0x001f_0f1c,
                0x00ff_79c6,
                0x00ff_f07a,
                0x002c_1528,
                0x00ff_d6ec,
                0x00b6_ff4a,
                0x00b6_ff4a,
                0x00ff_5c5c,
                0x00ff_f07a,
                0x005a_2650,
            ],
            Theme::DozyPink => [
                0x00fb_f1e1,
                0x006b_4a3a,
                0x00d9_668f,
                0x00f2_a7bf,
                0x004a_2f25,
                0x00f2_a7bf,
                0x004a_2f25,
                0x00c8_875a,
                0x00b8_456f,
                0x00f6_e6cf,
                0x006b_4a3a,
                0x00d9_668f,
                0x004f_7a2e,
                0x00b0_3a2e,
                0x009a_5a10,
                0x00f3_d9a8,
            ],
            // SR-37: 墨(暗い地・薄い灰・青緑のアクセント)。
            Theme::Sumi => [
                0x0016_171b,
                0x00d8_d6d0,
                0x00d8_d6d0,
                0x0023_262c,
                0x00ec_e9e2,
                0x0016_171b,
                0x008a_8780,
                0x0086_b8ad,
                0x00ec_e9e2,
                0x001a_1b20,
                0x00d8_d6d0,
                0x0086_b8ad,
                0x0086_b8ad,
                0x00d9_7b6c,
                0x00d6_a85c,
                0x002a_3a37,
            ],
            // 石板(紫寄りのアクセント)。
            Theme::Slate => [
                0x001b_1c25,
                0x00e1_e2ea,
                0x00e1_e2ea,
                0x002a_2c3d,
                0x00f0_f0f8,
                0x001b_1c25,
                0x008e_90aa,
                0x009b_9cf7,
                0x00f4_f4fb,
                0x001f_2029,
                0x00e1_e2ea,
                0x009b_9cf7,
                0x007e_c699,
                0x00ef_7d7d,
                0x00e6_b450,
                0x0034_355a,
            ],
            // SaaS(明るい灰の地・藍のアクセント)と、その暗い地の版。
            Theme::Saas => [
                0x00f6_f7f9,
                0x001e_2430,
                0x001e_2430,
                0x00e6_e9fb,
                0x001e_2430,
                0x00f6_f7f9,
                0x006b_7383,
                0x005b_5bd6,
                0x000f_1420,
                0x00ee_f0f4,
                0x001e_2430,
                0x005b_5bd6,
                0x002f_9e66,
                0x00d4_483f,
                0x00c4_7f17,
                0x00e4_e6fb,
            ],
            Theme::SaasDark => [
                0x0014_161c,
                0x00e6_e8ee,
                0x00e6_e8ee,
                0x0025_2a3a,
                0x00f2_f3f8,
                0x0014_161c,
                0x008b_93a5,
                0x008b_8cf8,
                0x00f5_f6fa,
                0x0018_1b22,
                0x00e6_e8ee,
                0x008b_8cf8,
                0x004c_c38a,
                0x00f0_736a,
                0x00e7_a93b,
                0x002b_2f4a,
            ],
            // 紙(明るい地・青のアクセント)。
            Theme::Paper => [
                0x00f7_f8fa,
                0x001f_2328,
                0x001f_2328,
                0x00df_e7f8,
                0x001f_2328,
                0x00f7_f8fa,
                0x006b_7280,
                0x002f_63d8,
                0x000d_1117,
                0x00ef_f1f4,
                0x001f_2328,
                0x002f_63d8,
                0x002f_8a4a,
                0x00c2_3b3b,
                0x00a8_6b00,
                0x00db_e5fb,
            ],
        }))
    }
}

/// xterm の 256 色の色の立方体の各段の値。
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// xterm の 256 色の番号 `n` の RGB(`to_indexed` の逆)。16〜231 の色の立方体と 232〜255 の灰色だけで、0〜15 は
/// 端末ごとに色が違うので None。GA-2 のガントの帯で、番号が違っても色がほぼ同じ組を避けるのに使う。
pub fn indexed_rgb(n: u8) -> Option<[u8; 3]> {
    match n {
        16..=231 => {
            let k = usize::from(n - 16);
            Some([CUBE[k / 36], CUBE[(k / 6) % 6], CUBE[k % 6]])
        }
        232..=255 => {
            let v = 8 + 10 * (n - 232);
            Some([v, v, v])
        }
        _ => None,
    }
}

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
        let hex: String = part.chars().take_while(char::is_ascii_hexdigit).collect();
        if hex.is_empty() || hex.len() > 4 {
            return None;
        }
        let max = f64::from(16u32.pow(hex.len() as u32) - 1);
        ch[k] = f64::from(u32::from_str_radix(&hex, 16).ok()?) / max;
    }
    if rest.split('/').count() < 3 {
        return None;
    }
    Some(0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2] > 0.5)
}

#[cfg(test)]
#[path = "test_theme_auto_unit.rs"]
mod test_theme_auto;
