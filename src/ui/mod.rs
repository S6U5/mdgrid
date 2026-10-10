//! 画面(実行ファイルの側)。形は docs/design.md の「画面の形」。
//! ライブラリ(核)は ratatui に依存しない。核には Source と Changes だけで触れる(SC-14)。

pub mod app;
pub mod bands;
pub mod calendar;
pub mod cell;
pub(crate) mod chips;
pub mod columns;
pub mod detail;
// 差分は --apply と共有するので src/diff.rs にある(今までの道 `ui::diff` はそのまま使える)。
pub(crate) use crate::diff;
pub mod display;
pub mod entry;
pub mod export;
pub mod external;
pub mod freq;
pub mod grid;
pub mod help;
pub mod input;
pub mod keymap;
pub mod list;
pub mod listpick;
pub mod look;
pub mod look_section;
pub mod menu;
pub mod native_io;
pub mod native_views;
pub mod nav;
pub mod new_note;
pub mod note_rename;
pub mod places;
pub mod popup;
pub(crate) mod profile;
pub mod relations;
pub mod relmap;
pub mod review;
pub mod settings;
pub mod settings_pick;
pub mod settings_view;
pub mod sorts;
pub mod startup;
pub mod termbg;
pub mod theme;
pub mod view;
pub mod view_tabs;
pub mod width;
pub mod workspace;

pub use app::{dumb_terminal, App, ColorMode};

use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::num::NonZeroU16;
use unicode_segmentation::UnicodeSegmentation;

/// 画面を描く。最下行と右端の1桁は空ける(端末の自動折り返しを避ける)。
pub fn draw(frame: &mut Frame, app: &App) {
    let full = frame.area();
    let area = Rect {
        x: full.x,
        y: full.y,
        width: full.width.saturating_sub(1),
        height: full.height.saturating_sub(1),
    };
    let mut lines = view::render(app, area.width as usize, area.height as usize);
    // SR-10: `TERM=dumb` では絵文字を出さない(同じ幅の `?` と空白に置き換えるので、桁はずれない)。
    if app.no_emoji {
        for line in &mut lines {
            for span in &mut line.spans {
                if width::has_emoji(&span.content) {
                    span.content = width::replace_emoji(&span.content).into();
                }
            }
        }
    }
    // SR-33: モダンな見た目では、画面ごとに残っている反転(ビューの設定・ヘルプのパレットなど)も、
    // 選びの背景の色にそろえる。classic と色なしでは反転のまま。
    if let Some(l) = look::look(app) {
        for line in &mut lines {
            line.style = l.swap_reverse(line.style);
            for span in &mut line.spans {
                span.style = l.swap_reverse(span.style);
            }
        }
    }
    if app.ambiguous_wide {
        write_lines(frame.buffer_mut(), area, &lines);
    } else {
        frame.render_widget(Paragraph::new(lines), area);
    }
    // SR-26・SR-27: テーマは描き終えたバッファを役割で塗り替える(色なし・default では何もしない)。
    theme::paint(frame.buffer_mut(), full, app);
    // SR-17: 入力ボックスでは端末のカーソルを入力の位置に置く(IME の変換の窓がそこに出る)。
    // 置かなければ ratatui がカーソルを隠す。
    if let Some((x, y)) = view::cursor(app) {
        if x < area.width && y < area.height {
            frame.set_cursor_position((area.x + x, area.y + y));
        }
    }
}

/// East Asian Ambiguous を幅2で数えるとき(CV-6)の描画。ratatui はそれを幅1と数えるので、
/// 書記素ごとに width.rs の幅でセルに置き、幅を ratatui より広く数えた書記素には `ForcedWidth` を付けて、
/// 後ろのセルを書かない(端末が2桁に描く分を空ける)。行は view が幅に合わせて作ってある。
fn write_lines(buf: &mut Buffer, area: Rect, lines: &[Line<'static>]) {
    for (dy, line) in lines.iter().enumerate().take(area.height as usize) {
        let y = area.y + dy as u16;
        let mut x = 0usize;
        'line: for span in &line.spans {
            let style = line.style.patch(span.style);
            for g in span.content.graphemes(true) {
                let gw = width::grapheme_width(g);
                if gw == 0 {
                    continue;
                }
                if x + gw > area.width as usize {
                    break 'line;
                }
                let cx = area.x + x as u16;
                let cell = &mut buf[(cx, y)];
                cell.set_symbol(g).set_style(style);
                if gw > width::narrow_width(g) {
                    if let Some(n) = NonZeroU16::new(gw as u16) {
                        cell.set_diff_option(CellDiffOption::ForcedWidth(n));
                    }
                }
                for k in 1..gw {
                    buf[(cx + k as u16, y)].reset();
                }
                x += gw;
            }
        }
    }
}

#[cfg(test)]
mod test_action_menu;
#[cfg(test)]
mod test_action_menu_gaps;
#[cfg(test)]
mod test_action_menu_more;
#[cfg(test)]
mod test_add_column;
#[cfg(test)]
mod test_audit4;
#[cfg(test)]
mod test_bom_values;
#[cfg(test)]
mod test_borders;
#[cfg(test)]
mod test_bulk_outside;
#[cfg(test)]
mod test_calendar;
#[cfg(test)]
mod test_calendar_seconds;
#[cfg(test)]
mod test_checkbox;
#[cfg(test)]
mod test_checkbox_two_state;
#[cfg(test)]
mod test_clear_input;
#[cfg(test)]
mod test_click_name;
#[cfg(test)]
mod test_colors_more;
#[cfg(test)]
mod test_colors_screen;
#[cfg(test)]
mod test_datetime_offsets;
#[cfg(test)]
mod test_default_view;
#[cfg(test)]
mod test_detail;
#[cfg(test)]
mod test_detail_body;
#[cfg(test)]
mod test_detail_look;
#[cfg(test)]
mod test_display_more;
#[cfg(test)]
mod test_display_options;
#[cfg(test)]
mod test_edit_design;
#[cfg(test)]
mod test_export;
#[cfg(test)]
mod test_external;
#[cfg(test)]
mod test_freq_chips;
#[cfg(test)]
mod test_group_gap;
#[cfg(test)]
mod test_header_buttons;
#[cfg(test)]
mod test_key_ops;
#[cfg(test)]
mod test_list_chips;
#[cfg(test)]
mod test_list_narrow;
#[cfg(test)]
mod test_load_throttle;
#[cfg(test)]
mod test_look;
#[cfg(test)]
mod test_modern_rest;
#[cfg(test)]
mod test_note_editor;
#[cfg(test)]
mod test_note_form;
#[cfg(test)]
mod test_note_time;
#[cfg(test)]
mod test_places;
#[cfg(test)]
mod test_places_align;
#[cfg(test)]
mod test_places_more;
#[cfg(test)]
mod test_relations;
#[cfg(test)]
mod test_relations_open;
#[cfg(test)]
mod test_relmap;
#[cfg(test)]
mod test_relmap_click;
#[cfg(test)]
mod test_relmap_new_note;
#[cfg(test)]
mod test_relmap_new_note_load;
#[cfg(test)]
mod test_relmap_scroll;
#[cfg(test)]
mod test_review6;
#[cfg(test)]
mod test_rich_cells;
#[cfg(test)]
mod test_rich_cells_first;
#[cfg(test)]
mod test_rich_cells_fit;
#[cfg(test)]
mod test_rich_cells_more;
#[cfg(test)]
mod test_screen_tabs_fixed;
#[cfg(test)]
mod test_selects_cache;
#[cfg(test)]
mod test_sort_more;
#[cfg(test)]
mod test_sort_remove;
#[cfg(test)]
mod test_sort_window;
#[cfg(test)]
mod test_style_frames;
#[cfg(test)]
mod test_style_list_pill;
#[cfg(test)]
mod test_style_overlay;
#[cfg(test)]
mod test_style_props;
#[cfg(test)]
mod test_style_screen;
#[cfg(test)]
mod test_style_selected;
#[cfg(test)]
mod test_theme_meaning;
#[cfg(test)]
mod test_time_picker;
#[cfg(test)]
mod test_trust_notices;
#[cfg(test)]
mod test_view_tabs_auto;
#[cfg(test)]
mod test_workspace;
// 受け入れの試験(書き換えない)の書き方に当たる clippy の指摘は、ここで許す。
#[cfg(test)]
#[allow(clippy::manual_contains, clippy::cloned_ref_to_slice_refs)]
mod test_freq;
#[cfg(test)]
mod test_freq_more;
#[cfg(test)]
mod test_gaps;
#[cfg(test)]
mod test_grid;
#[cfg(test)]
mod test_help;
#[cfg(test)]
mod test_help_markers;
#[cfg(test)]
mod test_input;
#[cfg(test)]
mod test_keys_doc;
#[cfg(test)]
mod test_language;
#[cfg(test)]
mod test_language_more;
#[cfg(test)]
mod test_list_type;
#[cfg(test)]
mod test_look_keep;
#[cfg(test)]
mod test_look_section;
#[cfg(test)]
mod test_native_state;
#[cfg(test)]
mod test_native_views;
#[cfg(test)]
mod test_nav;
#[cfg(test)]
mod test_new_note;
#[cfg(test)]
mod test_new_note_folders;
#[cfg(test)]
mod test_new_note_gaps;
#[cfg(test)]
mod test_new_note_infolder;
#[cfg(test)]
mod test_new_note_more;
#[cfg(test)]
mod test_new_note_order;
#[cfg(test)]
mod test_note_column;
#[cfg(test)]
mod test_note_rename;
#[cfg(test)]
mod test_pending_shape;
#[cfg(test)]
mod test_plural_en;
#[cfg(test)]
mod test_screen;
#[cfg(test)]
mod test_search_bar;
#[cfg(test)]
mod test_search_not_found;
#[cfg(test)]
mod test_settings_close;
#[cfg(test)]
mod test_settings_head;
#[cfg(test)]
mod test_settings_screen;
#[cfg(test)]
mod test_shift_arrows;
#[cfg(test)]
mod test_startup;
#[cfg(test)]
mod test_summaries;
#[cfg(test)]
mod test_themes;
#[cfg(test)]
mod test_themes_more;
#[cfg(test)]
mod test_tree;
#[cfg(test)]
mod test_tree_held;
#[cfg(test)]
mod test_type_replaces_more;
#[cfg(test)]
mod test_untyped_date;
#[cfg(test)]
mod test_unwritable;
#[cfg(test)]
mod test_view_delete_undo;
#[cfg(test)]
mod test_view_tabs;
#[cfg(test)]
mod test_wbs;
