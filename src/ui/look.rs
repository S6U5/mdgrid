//! モダンな見た目(SR-33): lazygit のように、選びは反転でなく背景の色、窓の枠とキーはアクセントの色、
//! 説明は薄い色。色と飾りだけを変え、画面の文字・印・構成は変えない(SR-26)。
//!
//! 色を使わない表示(SR-10)と `look = "classic"` では None を返し、各部品は今までどおり(反転)に描く。
//! 色はテーマ(SR-26)の色の表から。既定のテーマ(色の表なし)は、端末の地の色を変えない控えめな組。

use super::app::{App, ColorMode};
use mdgrid::theme::{to_indexed, Palette};
use ratatui::style::{Color, Modifier, Style};

/// モダンな見た目の色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Look {
    /// 枠・キー・見出し・選んだタブの札。
    pub accent: Color,
    /// 説明・選んでいないタブ・表の列の区切り。
    pub dim: Color,
    /// 選んでいる行・候補・項目の背景。
    pub sel_bg: Color,
    /// その文字(None なら変えない)。
    pub sel_fg: Option<Color>,
    /// アクセントの背景の上の文字(選んだタブの札)。
    pub on_accent: Color,
    /// テーマの色の表があるとき、下の帯の (地, 文字)。ヘッダーと帯の色はテーマのまま(SR-26)。
    pub band: Option<(Color, Color)>,
}

/// 2つの色の中間(t は b の割合)。
fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let m = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
}

/// 今の設定のモダンな見た目の色。classic か色なしなら None。
pub(crate) fn look(app: &App) -> Option<Look> {
    if app.look_classic {
        return None;
    }
    let indexed = match app.color {
        ColorMode::None => return None,
        ColorMode::Indexed => true,
        ColorMode::Rgb => false,
    };
    let c = |rgb: [u8; 3]| {
        if indexed {
            Color::Indexed(to_indexed(rgb))
        } else {
            Color::Rgb(rgb[0], rgb[1], rgb[2])
        }
    };
    Some(match app.theme.palette() {
        Some(Palette {
            bg,
            fg: _,
            sel_bg,
            sel_fg,
            colhead,
            band_bg,
            band_fg,
            ..
        }) => Look {
            accent: c(colhead),
            dim: c(mix(band_fg, band_bg, 0.4)),
            // `fg`・`bg` は帯の外の薄い文字に使わない(帯の色から作る)。
            sel_bg: c(sel_bg),
            sel_fg: Some(c(sel_fg)),
            on_accent: c(bg),
            band: Some((c(band_bg), c(band_fg))),
        },
        None => Look {
            accent: Color::Cyan,
            dim: Color::DarkGray,
            sel_bg: if indexed {
                Color::Indexed(237)
            } else {
                Color::Rgb(58, 58, 70)
            },
            sel_fg: None,
            on_accent: Color::Black,
            band: None,
        },
    })
}

impl Look {
    /// 選んでいるものの見た目(反転の代わり)。
    pub(crate) fn selected(&self) -> Style {
        let st = Style::default()
            .bg(self.sel_bg)
            .add_modifier(Modifier::BOLD);
        match self.sel_fg {
            Some(f) => st.fg(f),
            None => st,
        }
    }

    /// 反転を含む見た目を、モダンな見た目に置き換える(反転でなければそのまま)。
    pub(crate) fn swap_reverse(&self, st: Style) -> Style {
        if st.add_modifier.contains(Modifier::REVERSED) {
            let mut s = st;
            s.add_modifier.remove(Modifier::REVERSED);
            s.patch(self.selected())
        } else {
            st
        }
    }

    /// 枠の線の見た目。
    pub(crate) fn border(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// 薄い文字。
    pub(crate) fn faint(&self) -> Style {
        Style::default().fg(self.dim)
    }

    /// キー(下の帯)。
    pub(crate) fn key(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// 選んだタブの札。
    pub(crate) fn pill(&self) -> Style {
        Style::default()
            .bg(self.accent)
            .fg(self.on_accent)
            .add_modifier(Modifier::BOLD)
    }
}
