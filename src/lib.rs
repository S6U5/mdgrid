//! Internals of the `mdgrid` command-line tool (install it with `cargo install mdgrid`).
//!
//! This library target exists so the binary and its tests can share code. It is not a supported
//! API: names and behavior may change in any release, including patch releases.
//!
//! mdgrid の核(画面に依存しない)。設計は docs/design.md、仕様は specs/。

pub mod base;
pub mod changes;
pub mod clock;
pub mod config;
pub mod display;
pub mod expr;
pub mod frontmatter;
pub mod i18n;
pub mod links;
pub(crate) mod mdtext;
pub mod newnote;
pub mod print;
pub mod settings;
pub mod source;
pub mod summary;
pub mod theme;
pub mod types;
pub mod vault;
pub mod views;
pub mod writeback;
pub(crate) mod yaml_guard;
