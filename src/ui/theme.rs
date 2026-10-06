//! 画面のテーマの塗り替え(SR-26・SR-27)。色の表は mdgrid::theme。
//!
//! 描き終えたバッファを役割で塗り替える後段の1か所(docs/design.md の「テーマ」)。各部品の描き方には
//! 手を入れないので、テーマ `default`(パレットなし)の画面は塗り替えの無いときと同じ。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::view;
use mdgrid::theme::{to_indexed, Palette};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

/// テーマの意味の色(SR-28): パレットがあり色を使う表示なら、`role` の色を端末の色の扱いで。
/// 無ければ None(呼ぶ側はテーマなしの色を使う)。
pub(crate) fn meaning(app: &App, role: fn(&Palette) -> [u8; 3]) -> Option<Color> {
    let p = app.theme.palette()?;
    let rgb = role(&p);
    match app.color {
        ColorMode::None => None,
        ColorMode::Indexed => Some(Color::Indexed(to_indexed(rgb))),
        ColorMode::Rgb => Some(Color::Rgb(rgb[0], rgb[1], rgb[2])),
    }
}

/// `full`(端末の全体。空けておく最下行と右端も含む)のセルを、テーマの役割の色で塗り替える。
/// 色を使わない表示とパレットの無いテーマでは何もしない。
pub(crate) fn paint(buf: &mut Buffer, full: Rect, app: &App) {
    let Some(p) = app.theme.palette() else {
        return;
    };
    let indexed = match app.color {
        ColorMode::None => return,
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
    // 一行おきの色(ui/display.rs の zebra)。
    let (zebra_bg, zebra_fg) = if indexed {
        (Color::Indexed(237), Color::Indexed(252))
    } else {
        (Color::Rgb(58, 58, 58), Color::Rgb(208, 208, 208))
    };
    let Palette {
        bg,
        fg,
        header,
        sel_bg,
        sel_fg,
        band_bg,
        band_fg,
        colhead,
        strong,
        zebra_bg: z_bg,
        zebra_fg: z_fg,
        mark,
        ..
    } = p;
    // 下の帯は描く範囲(最下行を空けた高さ)の下から2行目。
    let band_y = full.y + full.height.saturating_sub(1).saturating_sub(2);
    // 列の見出しの行(表の画面のときだけ)。ほかの太字+下線(入力ボックス・一致した名前)は強調。
    let colhead_y = table_screen(app)
        .then(|| (view::data_y(app) as u16).checked_sub(1))
        .flatten()
        .map(|d| full.y + d);
    // 右下の隅の1セルは書かない(端末の自動折り返しを避ける。ui::draw と同じ理由)。
    let corner = (
        full.right().saturating_sub(1),
        full.bottom().saturating_sub(1),
    );
    let area = full.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if (x, y) == corner {
                continue;
            }
            let cell = &mut buf[(x, y)];
            // 一行おきの色は「色なし」とみなして役割で塗る(行が偶数番目かどうかだけを持ち越す)。
            let zebra = cell.bg == zebra_bg;
            let own_fg = cell.fg != Color::Reset && !(zebra && cell.fg == zebra_fg);
            let own_bg = cell.bg != Color::Reset && !zebra;
            if cell.modifier.contains(Modifier::REVERSED) {
                // 反転で自分の色を持つセルはそのまま。
                if own_fg || own_bg {
                    continue;
                }
                let (b, f) = if y == band_y {
                    (band_bg, band_fg)
                } else {
                    (sel_bg, sel_fg)
                };
                cell.bg = c(b);
                cell.fg = c(f);
                cell.modifier.remove(Modifier::REVERSED);
                continue;
            }
            if !own_fg {
                let m = cell.modifier;
                let role = if y == full.y {
                    header
                } else if m.contains(Modifier::BOLD | Modifier::UNDERLINED) {
                    if Some(y) == colhead_y {
                        colhead
                    } else {
                        strong
                    }
                } else if m.contains(Modifier::BOLD) {
                    strong
                } else if x == full.x && cell.symbol() == ">" {
                    mark
                } else if zebra {
                    z_fg
                } else {
                    fg
                };
                cell.fg = c(role);
            }
            if !own_bg {
                cell.bg = c(if zebra { z_bg } else { bg });
            }
        }
    }
}

/// 表の画面か(ヘルプ・保存の確認の差分・詳細・ビューの設定の画面でない。view::render の分かれ方と同じ)。
fn table_screen(app: &App) -> bool {
    !(app.mode == Mode::Help
        || (app.mode == Mode::Confirm && app.review.is_some())
        || (app.detail.is_some() && matches!(app.mode, Mode::Detail | Mode::Edit | Mode::ListPick))
        || (app.draft.is_some() && matches!(app.mode, Mode::Settings | Mode::SettingsText)))
}
