//! 表の見せ方の切り替え(SR-20・SR-21・SR-30)の核。画面に依存しない。
//!
//! 設定の `[display]`(`Display`)が既定を決め、ビューの設定(`Settings::display`。`DisplayOverride`)が
//! 設定と違う項目だけを持って、ビューごとに上書きする。検索の欄は設定の最上位の `search_bar` を既定にする
//! (`[display]` には足さない)。

use crate::i18n::Msg;
use serde::{Deserialize, Serialize};

/// ビューのタブの行(SR-34): いつも・ビューが2つ以上のときだけ・出さない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsMode {
    #[default]
    Always,
    Auto,
    Never,
}

impl TabsMode {
    pub const NAMES: &'static [&'static str] = &["always", "auto", "never"];

    pub fn name(self) -> &'static str {
        match self {
            TabsMode::Always => "always",
            TabsMode::Auto => "auto",
            TabsMode::Never => "never",
        }
    }

    pub fn parse(s: &str) -> Option<TabsMode> {
        match s {
            "always" => Some(TabsMode::Always),
            "auto" => Some(TabsMode::Auto),
            "never" => Some(TabsMode::Never),
            _ => None,
        }
    }

    /// 表示の区画で次に切り替える値(always → auto → never → always)。
    pub fn next(self) -> TabsMode {
        match self {
            TabsMode::Always => TabsMode::Auto,
            TabsMode::Auto => TabsMode::Never,
            TabsMode::Never => TabsMode::Always,
        }
    }
}

impl Serialize for TabsMode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for TabsMode {
    /// 名前のほか、前の版の状態のファイル・views.toml の真偽(true = always、false = never)も受ける。
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            B(bool),
            S(String),
        }
        match Raw::deserialize(d)? {
            Raw::B(true) => Ok(TabsMode::Always),
            Raw::B(false) => Ok(TabsMode::Never),
            Raw::S(s) => TabsMode::parse(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("unknown tabs mode {s:?}"))),
        }
    }
}

/// ビューの帯の検索の欄と絞り込み・並べ替えのボタンの描き方(NV-23): 線なし(既定。地の色で示す)・
/// 線で四角く囲む(線は帯の上下の余白の行に描く)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonsMode {
    #[default]
    Plain,
    Box,
}

impl ButtonsMode {
    pub const NAMES: &'static [&'static str] = &["plain", "box"];

    pub fn name(self) -> &'static str {
        match self {
            ButtonsMode::Plain => "plain",
            ButtonsMode::Box => "box",
        }
    }

    pub fn parse(s: &str) -> Option<ButtonsMode> {
        match s {
            "plain" => Some(ButtonsMode::Plain),
            "box" => Some(ButtonsMode::Box),
            _ => None,
        }
    }

    /// 表示の区画で次に切り替える値(plain ↔ box)。
    pub fn next(self) -> ButtonsMode {
        match self {
            ButtonsMode::Plain => ButtonsMode::Box,
            ButtonsMode::Box => ButtonsMode::Plain,
        }
    }
}

/// 余白の行の数(SR-1 の `top_padding`・NV-23 の `bar_padding`)。0〜2 を 0.5 刻みで、半行の数で持つ(0〜4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Pad(pub u8);

impl Pad {
    /// 上限(2 行 = 半行 4 つ)。
    pub const MAX: u8 = 4;
    /// 既定(1 行。端数は環境で半行にも1行にもなるので既定にしない。2026-10-12-padding-default-one)。
    pub const DEFAULT: Pad = Pad(2);

    /// 設定の値(0〜2 の 0.5 刻み)から。範囲の外・刻みに合わない値は None。
    pub fn from_f64(v: f64) -> Option<Pad> {
        let h = v * 2.0;
        if !(0.0..=f64::from(Pad::MAX)).contains(&h) || h.fract() != 0.0 {
            return None;
        }
        Some(Pad(h as u8))
    }

    /// 設定に書く値(0・0.5・1・1.5・2)。
    pub fn value(self) -> f64 {
        f64::from(self.0) / 2.0
    }

    /// 画面に出す文字(「0.5」「1」など)。
    pub fn text(self) -> String {
        if self.0.is_multiple_of(2) {
            (self.0 / 2).to_string()
        } else {
            format!("{}.5", self.0 / 2)
        }
    }

    /// 表示の区画で次に切り替える値(0 → 0.5 → 1 → 1.5 → 2 → 0)。
    pub fn next(self) -> Pad {
        if self.0 >= Pad::MAX {
            Pad(0)
        } else {
            Pad(self.0 + 1)
        }
    }

    /// 端末の行に直した (空き行の数, 半分の塗りの行があるか)。半分の塗りの字を使えなければ端数は切り上げる。
    pub fn rows(self, half_ok: bool) -> (usize, bool) {
        let full = usize::from(self.0 / 2);
        let half = self.0 % 2 == 1;
        if half && !half_ok {
            (full + 1, false)
        } else {
            (full, half)
        }
    }
}

impl Serialize for Pad {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if self.0.is_multiple_of(2) {
            s.serialize_i64(i64::from(self.0 / 2))
        } else {
            s.serialize_f64(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for Pad {
    /// 0.5 のような小数と、1 のような整数の両方を受ける。
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            I(i64),
            F(f64),
        }
        let v = match Raw::deserialize(d)? {
            Raw::I(i) => i as f64,
            Raw::F(f) => f,
        };
        Pad::from_f64(v).ok_or_else(|| serde::de::Error::custom(format!("bad padding {v}")))
    }
}

impl Serialize for ButtonsMode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for ButtonsMode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        ButtonsMode::parse(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown buttons mode {s:?}")))
    }
}

/// 設定の `[display]`(SR-21)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Display {
    /// 表の左に1始まりの行番号(既定 false)。
    pub row_numbers: bool,
    /// 一行おきの色(既定 false。色を使わない表示では付けない)。
    pub zebra: bool,
    /// 列の間に `│` を引く(既定 false)。
    pub column_lines: bool,
    /// 2つ目以降のまとまりの見出しの上に空きの行を入れる(SR-30。既定 false)。
    pub group_gap: bool,
    /// ビューのタブの行(SR-34。既定 always)。
    pub tabs: TabsMode,
    /// 表の上の検索の欄(NV-23。既定 true)。
    pub search_bar: bool,
    /// 設定の帯を出す(既定 true)。
    pub chips: bool,
    /// ビューの帯のボタンの描き方(NV-23。既定 plain)。
    pub buttons: ButtonsMode,
    /// ビューの帯の上下に空ける行の数(NV-23。0〜2 の 0.5 刻み。既定 1)。
    pub bar_padding: Pad,
    /// ヘッダーの上に空ける行の数(SR-1。0〜2 の 0.5 刻み。既定 1)。
    pub top_padding: Pad,
    /// データの1行を最大で「1 + この数」の行の高さまで伸ばす(CV-8。0〜9。既定 0)。
    pub row_extra_lines: u8,
}

impl Default for Display {
    fn default() -> Self {
        Display {
            row_numbers: false,
            zebra: false,
            column_lines: false,
            group_gap: false,
            tabs: TabsMode::Always,
            search_bar: true,
            chips: true,
            buttons: ButtonsMode::Plain,
            bar_padding: Pad::DEFAULT,
            top_padding: Pad::DEFAULT,
            row_extra_lines: 0,
        }
    }
}

/// 行を伸ばす設定の上限(CV-8)。
pub const ROW_EXTRA_MAX: u8 = 9;
/// 設定の画面で切り替える上限(CV-8。0 → 1 → 2 → 3 → 0)。
const ROW_EXTRA_CYCLE: u8 = 3;

/// 表示の区画で次に切り替える行を伸ばす数(0 → 1 → 2 → 3 → 0。設定で 4 より上を書いていれば 0 に)。
pub fn next_row_extra_lines(n: u8) -> u8 {
    if n >= ROW_EXTRA_CYCLE {
        0
    } else {
        n + 1
    }
}

impl Display {
    /// 項目を出すか(タブは never でなければ出す。auto の1つだけのときは画面の側が隠す)。
    pub fn get(&self, item: Item) -> bool {
        match item {
            Item::RowNumbers => self.row_numbers,
            Item::Zebra => self.zebra,
            Item::ColumnLines => self.column_lines,
            Item::GroupGap => self.group_gap,
            Item::Tabs => self.tabs != TabsMode::Never,
            Item::SearchBar => self.search_bar,
            Item::Chips => self.chips,
            Item::Buttons => self.buttons == ButtonsMode::Box,
            Item::BarPadding => self.bar_padding.0 > 0,
            Item::TopPadding => self.top_padding.0 > 0,
            Item::RowExtraLines => self.row_extra_lines > 0,
        }
    }
}

/// 切り替えられる見せ方(ビューの設定の画面の「表示」の節の並び)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    RowNumbers,
    Zebra,
    ColumnLines,
    GroupGap,
    Tabs,
    SearchBar,
    /// ビューの帯のボタンの描き方(NV-23。`buttons`。plain・box を切り替える)。
    Buttons,
    /// ビューの帯の上下の余白(NV-23。`bar_padding`。0〜2 を 0.5 刻みで順に切り替える)。
    BarPadding,
    /// ヘッダーの上の余白(SR-1。`top_padding`。0〜2 を 0.5 刻みで順に切り替える)。
    TopPadding,
    Chips,
    /// 行を伸ばす数(CV-8。`row_extra_lines`。0〜3 を順に切り替える)。
    RowExtraLines,
}

impl Item {
    /// 項目の名前の文言(SR-23)。
    pub const fn msg(self) -> Msg {
        match self {
            Item::RowNumbers => Msg::DisplayRowNumbers,
            Item::Zebra => Msg::DisplayZebra,
            Item::ColumnLines => Msg::DisplayColumnLines,
            Item::GroupGap => Msg::DisplayGroupGap,
            Item::Tabs => Msg::DisplayTabs,
            Item::SearchBar => Msg::DisplaySearchBar,
            Item::Chips => Msg::DisplayChips,
            Item::Buttons => Msg::DisplayButtons,
            Item::BarPadding => Msg::DisplayBarPadding,
            Item::TopPadding => Msg::DisplayTopPadding,
            Item::RowExtraLines => Msg::DisplayRowExtraLines,
        }
    }

    /// 今の言語の項目の名前(画面に出す)。
    pub fn label(self) -> &'static str {
        self.msg().text()
    }
}

/// 「表示」の節の項目と名前(この順で並べる)。名前は日本語の文。画面には今の言語の `Item::label` を出す(SR-23)。
pub const ITEMS: [(Item, &str); 11] = [
    (Item::RowNumbers, Item::RowNumbers.msg().ja()),
    (Item::Zebra, Item::Zebra.msg().ja()),
    (Item::ColumnLines, Item::ColumnLines.msg().ja()),
    (Item::GroupGap, Item::GroupGap.msg().ja()),
    (Item::Tabs, Item::Tabs.msg().ja()),
    (Item::SearchBar, Item::SearchBar.msg().ja()),
    (Item::Buttons, Item::Buttons.msg().ja()),
    (Item::BarPadding, Item::BarPadding.msg().ja()),
    (Item::TopPadding, Item::TopPadding.msg().ja()),
    (Item::RowExtraLines, Item::RowExtraLines.msg().ja()),
    (Item::Chips, Item::Chips.msg().ja()),
];

/// 上書き(どの範囲でも `[display]`。ビューの設定の `display` も)。書いた項目だけ Some。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_numbers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zebra: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_lines: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_gap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<TabsMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_bar: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chips: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<ButtonsMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_padding: Option<Pad>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_padding: Option<Pad>,
    /// 範囲(0〜9)の外・整数でない値は持たない(既定にする。警告は読む側。views.rs・profile.rs)。
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "row_extra_lines_in_range"
    )]
    pub row_extra_lines: Option<u8>,
}

/// 行を伸ばす数の値(CV-8)か: 0〜9 の整数。
pub fn row_extra_lines_ok(v: &toml::Value) -> Option<u8> {
    v.as_integer()
        .filter(|n| (0..=i64::from(ROW_EXTRA_MAX)).contains(n))
        .map(|n| n as u8)
}

/// ビューの上書きの `row_extra_lines` を読む(CV-8)。範囲の外・整数でない値は None(ビューごと捨てない)。
fn row_extra_lines_in_range<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<u8>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        I(i64),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Raw::deserialize(d)? {
        Raw::I(n) if (0..=i64::from(ROW_EXTRA_MAX)).contains(&n) => Some(n as u8),
        _ => None,
    })
}

/// 上書きの項目の名前(知らない項目の判定)。
pub const OVERRIDE_KEYS: &[&str] = &[
    "row_numbers",
    "zebra",
    "column_lines",
    "group_gap",
    "tabs",
    "search_bar",
    "chips",
    "buttons",
    "bar_padding",
    "top_padding",
    "row_extra_lines",
];

impl DisplayOverride {
    pub fn is_empty(&self) -> bool {
        *self == DisplayOverride::default()
    }

    /// 全部の項目を書いた上書き(決まった値の書き出し)。
    pub fn all(d: &Display) -> DisplayOverride {
        DisplayOverride {
            row_numbers: Some(d.row_numbers),
            zebra: Some(d.zebra),
            column_lines: Some(d.column_lines),
            group_gap: Some(d.group_gap),
            tabs: Some(d.tabs),
            search_bar: Some(d.search_bar),
            chips: Some(d.chips),
            buttons: Some(d.buttons),
            bar_padding: Some(d.bar_padding),
            top_padding: Some(d.top_padding),
            row_extra_lines: Some(d.row_extra_lines),
        }
    }

    /// 真偽の項目の名前の欄(タブとボタンの描き方は別)。知らない名前は None。
    pub fn bool_mut(&mut self, name: &str) -> Option<&mut Option<bool>> {
        Some(match name {
            "row_numbers" => &mut self.row_numbers,
            "zebra" => &mut self.zebra,
            "column_lines" => &mut self.column_lines,
            "group_gap" => &mut self.group_gap,
            "search_bar" => &mut self.search_bar,
            "chips" => &mut self.chips,
            _ => return None,
        })
    }

    /// 書いた項目を `d` に当て、当てた項目の名前を返す。
    pub fn apply_to(&self, d: &mut Display) -> Vec<&'static str> {
        let mut out = Vec::new();
        macro_rules! put {
            ($f:ident) => {
                if let Some(v) = self.$f {
                    d.$f = v;
                    out.push(stringify!($f));
                }
            };
        }
        put!(row_numbers);
        put!(zebra);
        put!(column_lines);
        put!(group_gap);
        put!(tabs);
        put!(search_bar);
        put!(chips);
        put!(buttons);
        put!(bar_padding);
        put!(top_padding);
        put!(row_extra_lines);
        out
    }

    /// 決まった値(上書きを `base` に重ねたもの)。
    pub fn over(&self, base: &Display) -> Display {
        let mut d = *base;
        self.apply_to(&mut d);
        d
    }

    /// 決まった値で項目を出すか。
    pub fn resolve(&self, item: Item, base: &Display) -> bool {
        self.over(base).get(item)
    }

    /// 真偽の項目を `on` にする。土台と同じなら上書きを持たない(違う項目だけ持つ)。
    /// タブは出す(土台が never なら always)か出さない(never)にする。
    pub fn set(&mut self, item: Item, on: bool, base: &Display) {
        let same = base.get(item) == on;
        match item {
            Item::RowNumbers => self.row_numbers = (!same).then_some(on),
            Item::Zebra => self.zebra = (!same).then_some(on),
            Item::ColumnLines => self.column_lines = (!same).then_some(on),
            Item::GroupGap => self.group_gap = (!same).then_some(on),
            Item::SearchBar => self.search_bar = (!same).then_some(on),
            Item::Chips => self.chips = (!same).then_some(on),
            Item::Buttons => {
                let m = if on {
                    ButtonsMode::Box
                } else {
                    ButtonsMode::Plain
                };
                self.set_buttons(m, base);
            }
            Item::BarPadding => {
                let n = match (on, base.bar_padding) {
                    (false, _) => Pad(0),
                    (true, Pad(0)) => Pad::DEFAULT,
                    (true, b) => b,
                };
                self.set_bar_padding(n, base);
            }
            Item::TopPadding => {
                let n = match (on, base.top_padding) {
                    (false, _) => Pad(0),
                    (true, Pad(0)) => Pad::DEFAULT,
                    (true, b) => b,
                };
                self.set_top_padding(n, base);
            }
            Item::RowExtraLines => {
                let n = match (on, base.row_extra_lines) {
                    (false, _) => 0,
                    (true, 0) => 1,
                    (true, b) => b,
                };
                self.set_row_extra_lines(n, base);
            }
            Item::Tabs => {
                let m = if !on {
                    TabsMode::Never
                } else if base.tabs == TabsMode::Never {
                    TabsMode::Always
                } else {
                    base.tabs
                };
                self.set_tabs(m, base);
            }
        }
    }

    /// ボタンの描き方を `m` にする(土台と同じなら上書きを持たない。NV-23)。
    pub fn set_buttons(&mut self, m: ButtonsMode, base: &Display) {
        self.buttons = (m != base.buttons).then_some(m);
    }

    /// 帯の余白の行の数を `n` にする(土台と同じなら上書きを持たない。NV-23)。
    pub fn set_bar_padding(&mut self, n: Pad, base: &Display) {
        self.bar_padding = (n != base.bar_padding).then_some(n);
    }

    /// ヘッダーの上の余白を `n` にする(土台と同じなら上書きを持たない。SR-1)。
    pub fn set_top_padding(&mut self, n: Pad, base: &Display) {
        self.top_padding = (n != base.top_padding).then_some(n);
    }

    /// 行を伸ばす数を `n` にする(土台と同じなら上書きを持たない。CV-8)。
    pub fn set_row_extra_lines(&mut self, n: u8, base: &Display) {
        self.row_extra_lines = (n != base.row_extra_lines).then_some(n);
    }

    /// タブの行の値を `m` にする(土台と同じなら上書きを持たない)。
    pub fn set_tabs(&mut self, m: TabsMode, base: &Display) {
        self.tabs = (m != base.tabs).then_some(m);
    }
}
