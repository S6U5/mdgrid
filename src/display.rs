//! 表の見せ方の切り替え(SR-20・SR-21)の核。画面に依存しない。
//!
//! 設定の `[display]`(`Display`)が既定を決め、ビューの設定(`Settings::display`。`DisplayOverride`)が
//! 設定と違う項目だけを持って、ビューごとに上書きする。検索の欄は設定の最上位の `search_bar` を既定にする
//! (`[display]` には足さない)。

use crate::i18n::Msg;
use serde::{Deserialize, Serialize};

/// 設定の `[display]`(SR-21)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Display {
    /// 表の左に1始まりの行番号(既定 false)。
    pub row_numbers: bool,
    /// 一行おきの色(既定 false。色を使わない表示では付けない)。
    pub zebra: bool,
    /// 列の間に `│` を引く(既定 false)。
    pub column_lines: bool,
    /// ビューのタブの帯を出す(既定 true)。
    pub tabs: bool,
    /// 設定の帯を出す(既定 true)。
    pub chips: bool,
}

impl Default for Display {
    fn default() -> Self {
        Display {
            row_numbers: false,
            zebra: false,
            column_lines: false,
            tabs: true,
            chips: true,
        }
    }
}

/// `[display]` の項目の名前(設定の読み取りが使う)。
pub const NAMES: [&str; 5] = ["row_numbers", "zebra", "column_lines", "tabs", "chips"];

impl Display {
    /// 名前の項目への参照。知らない名前は None。
    pub fn field_mut(&mut self, name: &str) -> Option<&mut bool> {
        Some(match name {
            "row_numbers" => &mut self.row_numbers,
            "zebra" => &mut self.zebra,
            "column_lines" => &mut self.column_lines,
            "tabs" => &mut self.tabs,
            "chips" => &mut self.chips,
            _ => return None,
        })
    }

    /// 項目の設定の値。検索の欄は設定の最上位の `search_bar`。
    pub fn get(&self, item: Item, search_bar: bool) -> bool {
        match item {
            Item::RowNumbers => self.row_numbers,
            Item::Zebra => self.zebra,
            Item::ColumnLines => self.column_lines,
            Item::Tabs => self.tabs,
            Item::SearchBar => search_bar,
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
pub const ITEMS: [(Item, &str); 6] = [
    (Item::RowNumbers, Item::RowNumbers.msg().ja()),
    (Item::Zebra, Item::Zebra.msg().ja()),
    (Item::ColumnLines, Item::ColumnLines.msg().ja()),
    (Item::Tabs, Item::Tabs.msg().ja()),
    (Item::SearchBar, Item::SearchBar.msg().ja()),
    (Item::Chips, Item::Chips.msg().ja()),
];

/// ビューごとの上書き(`Settings::display`)。設定と違う項目だけ Some。
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
    pub tabs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_bar: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chips: Option<bool>,
}

/// 上書きの項目の名前(views.toml の知らない項目の判定)。
pub const OVERRIDE_KEYS: &[&str] = &[
    "row_numbers",
    "zebra",
    "column_lines",
    "tabs",
    "search_bar",
    "chips",
];

impl DisplayOverride {
    pub fn is_empty(&self) -> bool {
        *self == DisplayOverride::default()
    }

    fn slot(&mut self, item: Item) -> &mut Option<bool> {
        match item {
            Item::RowNumbers => &mut self.row_numbers,
            Item::Zebra => &mut self.zebra,
            Item::ColumnLines => &mut self.column_lines,
            Item::Tabs => &mut self.tabs,
            Item::SearchBar => &mut self.search_bar,
            Item::Chips => &mut self.chips,
        }
    }

    pub fn get(&self, item: Item) -> Option<bool> {
        let mut c = *self;
        *c.slot(item)
    }

    /// 決まった値: 上書きがあればそれ、無ければ設定(`base`・`search_bar`)。
    pub fn resolve(&self, item: Item, base: &Display, search_bar: bool) -> bool {
        self.get(item).unwrap_or_else(|| base.get(item, search_bar))
    }

    /// 項目を `on` にする。設定と同じなら上書きを持たない(設定と違う項目だけ持つ)。
    pub fn set(&mut self, item: Item, on: bool, base: &Display, search_bar: bool) {
        *self.slot(item) = (on != base.get(item, search_bar)).then_some(on);
    }
}
