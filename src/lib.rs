//! Internals of the `mdgrid` command-line tool (install it with `cargo install mdgrid`).
//!
//! This library target exists so the binary and its tests can share code. It is not a supported
//! API: names and behavior may change in any release, including patch releases.
//!
//! mdgrid の核(画面に依存しない)。設計は docs/design.md、仕様は specs/。

pub mod base;
pub mod cells;
pub mod changes;
pub mod clock;
pub mod colors;
pub mod config;
pub mod display;
pub mod expr;
pub mod frontmatter;
pub mod i18n;
pub mod legacy;
pub mod links;
pub(crate) mod mdtext;
pub mod newnote;
pub mod places;
pub mod print;
pub mod profile;
pub mod relations;
pub mod relmap;
pub mod schema;
pub mod settings;
pub mod source;
pub mod style;
pub mod summary;
pub mod theme;
pub mod tree;
pub mod types;
pub mod uifile;
pub mod vault;
pub mod views;
pub mod workspace;
pub mod writeback;
pub(crate) mod yaml_guard;

#[cfg(test)]
#[path = "test_toml_edit_safety_unit.rs"]
mod test_toml_edit_safety;

#[cfg(test)]
#[path = "test_tab_prefs_unit.rs"]
mod test_tab_prefs;

#[cfg(test)]
#[path = "test_look_unit.rs"]
mod test_look;

#[cfg(test)]
#[path = "test_tree_unit.rs"]
mod test_tree;

#[cfg(test)]
#[path = "test_wbs_unit.rs"]
mod test_wbs;

#[cfg(test)]
#[path = "test_weekday_lang_unit.rs"]
mod test_weekday_lang;
