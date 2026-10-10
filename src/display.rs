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
        }
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
    Chips,
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
        }
    }

    /// 今の言語の項目の名前(画面に出す)。
    pub fn label(self) -> &'static str {
        self.msg().text()
    }
}

/// 「表示」の節の項目と名前(この順で並べる)。名前は日本語の文。画面には今の言語の `Item::label` を出す(SR-23)。
pub const ITEMS: [(Item, &str); 7] = [
    (Item::RowNumbers, Item::RowNumbers.msg().ja()),
    (Item::Zebra, Item::Zebra.msg().ja()),
    (Item::ColumnLines, Item::ColumnLines.msg().ja()),
    (Item::GroupGap, Item::GroupGap.msg().ja()),
    (Item::Tabs, Item::Tabs.msg().ja()),
    (Item::SearchBar, Item::SearchBar.msg().ja()),
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
        }
    }

    /// 真偽の項目の名前の欄(タブは別)。知らない名前は None。
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

    /// タブの行の値を `m` にする(土台と同じなら上書きを持たない)。
    pub fn set_tabs(&mut self, m: TabsMode, base: &Display) {
        self.tabs = (m != base.tabs).then_some(m);
    }
}
