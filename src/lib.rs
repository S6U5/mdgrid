//! Internals of the `mdgrid` command-line tool (install it with `cargo install mdgrid`).
//!
//! This library target exists so the binary and its tests can share code. It is not a supported
//! API: names and behavior may change in any release, including patch releases.
//!
//! mdgrid の核(画面に依存しない)。設計は docs/design.md、仕様は specs/。

#![cfg_attr(test, allow(clippy::disallowed_methods))]

pub mod base;
pub mod cells;
pub mod changes;
pub mod chart;
pub mod clock;
pub mod colors;
pub mod condition;
pub mod config;
pub mod csvfile;
pub mod datewords;
pub mod display;
pub mod expr;
pub mod favorites;
pub mod folder_tree;
pub mod frontmatter;
pub mod gantt;
pub mod gitsync;
pub mod i18n;
pub mod kanban;
pub mod ledger;
pub mod legacy;
pub mod links;
pub(crate) mod mdtext;
pub mod newnote;
pub mod notice;
pub mod notify;
pub mod numfmt;
pub mod page;
pub mod places;
pub mod print;
pub mod profile;
pub mod recent;
pub mod recurrence;
pub mod relations;
pub mod relmap;
pub mod resume;
pub mod routine;
pub mod rules;
pub mod schema;
pub mod session;
pub mod settings;
pub mod source;
pub mod starter;
pub mod style;
pub mod summary;
pub mod textfile;
pub mod theme;
pub mod trash;
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
#[path = "test_condition_unit.rs"]
mod test_condition_unit;

#[cfg(test)]
#[path = "test_notify_unit.rs"]
mod test_notify_unit;

#[cfg(test)]
#[path = "test_routine_unit.rs"]
mod test_routine_unit;

#[cfg(test)]
#[path = "test_locale_pick_unit.rs"]
mod test_locale_pick_unit;

#[cfg(test)]
#[path = "test_tab_prefs_unit.rs"]
mod test_tab_prefs;

#[cfg(test)]
#[path = "test_uifile_unit.rs"]
mod test_uifile;

#[cfg(test)]
#[path = "test_tree_unit.rs"]
mod test_tree;

#[cfg(test)]
#[path = "test_wbs_unit.rs"]
mod test_wbs;

#[cfg(test)]
#[path = "test_weekday_lang_unit.rs"]
mod test_weekday_lang;

#[cfg(test)]
#[path = "test_csvfile_unit.rs"]
mod test_csvfile;

#[cfg(test)]
#[path = "test_csv_edges_unit.rs"]
mod test_csv_edges;

#[cfg(test)]
#[path = "test_csv_identity_unit.rs"]
mod test_csv_identity;

#[cfg(test)]
#[path = "test_body_path_unit.rs"]
mod test_body_path;

#[cfg(test)]
#[path = "test_chart_unit.rs"]
mod test_chart;

#[cfg(test)]
#[path = "test_datewords_unit.rs"]
mod test_datewords;
