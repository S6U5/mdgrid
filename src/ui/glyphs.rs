//! 画面の記号(SR-45)。設定 `terminal.nerd_icons` が true ならアイコン用のフォント(Nerd Font)の記号、既定は今の字の
//! 記号。記号を使う所(設定のボタン・画面の型のボタンと窓・列の見出しの型の印・カレンダーの題)は、ここを通す。
//! 絵文字は使わない。Nerd Font の記号は私用の領域の字で、幅は字の幅の決まり(width.rs。CV-6)で数える。

use super::app::App;

/// 記号の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Glyph {
    /// 設定のボタンの歯車。
    Gear,
    /// 画面の型: 表。
    Table,
    /// 画面の型: 関係マップ。
    RelMap,
    /// 型の印: 数。
    Number,
    /// 型の印: 日付(カレンダーの題も)。
    Date,
    /// 型の印: 日時。
    DateTime,
    /// 型の印: リスト。
    List,
    /// 型の印: 札の列。
    Select,
    /// 型の印: チェック(今の字の記号は部品の形 `check` で決まる)。
    Check,
}

/// 今の字の記号(既定)。チェックは呼ぶ側が部品の形から決めるので、ここでは既定の ✓。カレンダーの題は記号なし。
fn plain(g: Glyph) -> &'static str {
    match g {
        Glyph::Gear => "⚙",
        Glyph::Table => "▦",
        Glyph::RelMap => "◇",
        Glyph::Number => "#",
        Glyph::Date | Glyph::DateTime => "◷",
        Glyph::List => "⋮",
        Glyph::Select => "◉",
        Glyph::Check => "✓",
    }
}

/// アイコン用のフォントの記号(Nerd Font の Font Awesome の字)。
fn nerd(g: Glyph) -> &'static str {
    match g {
        Glyph::Gear => "\u{f013}",     // nf-fa-gear
        Glyph::Table => "\u{f0ce}",    // nf-fa-table
        Glyph::RelMap => "\u{f0e8}",   // nf-fa-sitemap
        Glyph::Number => "\u{f292}",   // nf-fa-hashtag
        Glyph::Date => "\u{f073}",     // nf-fa-calendar
        Glyph::DateTime => "\u{f017}", // nf-fa-clock_o
        Glyph::List => "\u{f0ca}",     // nf-fa-list_ul
        Glyph::Select => "\u{f192}",   // nf-fa-dot_circle_o
        Glyph::Check => "\u{f046}",    // nf-fa-check_square_o
    }
}

/// 記号を使うか(設定 `terminal.nerd_icons`)。
pub(crate) fn nerd_on(app: &App) -> bool {
    app.nerd_icons
}

/// 記号 `g` の字(設定に従う)。
pub(crate) fn glyph(app: &App, g: Glyph) -> &'static str {
    if nerd_on(app) {
        nerd(g)
    } else {
        plain(g)
    }
}

/// カレンダーの題の頭の記号(既定は無し。アイコンを使うときだけ記号と空白)。
pub(crate) fn calendar_head(app: &App) -> String {
    if nerd_on(app) {
        format!("{} ", nerd(Glyph::Date))
    } else {
        String::new()
    }
}
