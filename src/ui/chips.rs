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
    let (bg, fg) = PAIRS[(hash(item) % PAIRS.len() as u64) as usize];
    Some(Style::default().bg(c(bg)?).fg(c(fg)?))
}
