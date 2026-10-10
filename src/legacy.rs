//! 0.3.0 までの設定の書き方を今の形に写す(CLI-20)。対応の表は `schema::LEGACY`(文書の表と突き合わせる)。
//! 新しい道筋にも書いてあれば新しいほうを使い、旧い名前ごとに「旧い名前は新しい道筋に移った」と警告する。

use crate::i18n::Msg;
use crate::profile::warn;

/// 道筋 `path` に値を置く(既にあれば置かない。途中が表でなければ置かない)。
fn put(t: &mut toml::Table, path: &str, v: toml::Value) {
    if let Some((tab, last)) = parent(t, path) {
        tab.entry(last).or_insert(v);
    }
}

fn moved(w: &mut Vec<String>, file: &str, old: &str, new: &str) {
    warn(w, file, old, Msg::RsnMoved.fill(&[&new]));
}

/// 最上位の旧い名前を新しい道筋へ写した表を返す。
pub fn lift(mut t: toml::Table, file: &str, w: &mut Vec<String>) -> toml::Table {
    // 名前だけが変わった項目。
    for (old, new) in [
        ("color", "terminal.color"),
        ("ambiguous_wide", "terminal.ambiguous_wide"),
        ("nerd_font", "terminal.nerd_font"),
        ("workspace_detect", "workspace.detect"),
        ("search_bar", "display.search_bar"),
        ("date_format", "dates.format"),
        ("week_start", "dates.week_start"),
        ("candidates", "edit.candidates"),
        ("add_frontmatter", "edit.add_frontmatter"),
    ] {
        if let Some(v) = t.remove(old) {
            put(&mut t, new, v);
            moved(w, file, old, new);
        }
    }
    // 旧い `look = "modern"|"classic"` は文字列(今の `[look]` は表)。
    if t.get("look").is_some_and(|v| v.is_str()) {
        let v = t.remove("look").unwrap_or(toml::Value::Boolean(false));
        put(&mut t, "look.mode", v);
        moved(w, file, "look", "look.mode");
    }
    // テーマ: `"auto"` に明暗のどちらかを書いていれば明暗の表。
    let theme = t.remove("theme");
    let light = t.remove("theme_light");
    let dark = t.remove("theme_dark");
    if let Some(th) = theme {
        let v = match (th.as_str(), &light, &dark) {
            (Some("auto"), l, d) if l.is_some() || d.is_some() => {
                let mut pair = toml::Table::new();
                pair.insert(
                    "light".into(),
                    l.clone()
                        .unwrap_or_else(|| toml::Value::String("saas".into())),
                );
                pair.insert(
                    "dark".into(),
                    d.clone()
                        .unwrap_or_else(|| toml::Value::String("sumi".into())),
                );
                toml::Value::Table(pair)
            }
            _ => th,
        };
        put(&mut t, "look.theme", v);
        moved(w, file, "theme", "look.theme");
    }
    if light.is_some() {
        moved(w, file, "theme_light", "look.theme");
    }
    if dark.is_some() {
        moved(w, file, "theme_dark", "look.theme");
    }
    // 窓の枠: ascii だけが意味を持つ(rounded は組のまま)。
    if let Some(b) = t.remove("borders") {
        if b.as_str() == Some("ascii") {
            put(&mut t, "look.style.frames", b);
        } else if b.as_str() != Some("rounded") {
            // 読めない値は新しい道筋の型の警告に回す。
            put(&mut t, "look.style.frames", b);
        }
        moved(w, file, "borders", "look.style.frames");
    }
    // 部品の形: preset は look.preset、ほかは look.style。
    if let Some(s) = t.remove("style") {
        match s {
            toml::Value::Table(st) => {
                for (k, v) in st {
                    if k == "preset" {
                        put(&mut t, "look.preset", v);
                        moved(w, file, "style.preset", "look.preset");
                    } else {
                        put(&mut t, &format!("look.style.{k}"), v);
                    }
                }
                moved(w, file, "style", "look.style");
            }
            other => {
                put(&mut t, "look.style", other);
                moved(w, file, "style", "look.style");
            }
        }
    }
    // セルの部品: 文字列は look.cells、表は全体の切り替え・部品の形の plain・列ごと。
    if let Some(c) = t.remove("cells") {
        match c {
            toml::Value::Table(ct) => {
                for (k, v) in ct {
                    let path = format!("cells.{k}");
                    let off = v.as_bool() == Some(false);
                    match k.as_str() {
                        "style" => {
                            put(&mut t, "look.cells", v);
                        }
                        "checkbox" | "chips" | "select" | "links" => {
                            let (to, plain) = match k.as_str() {
                                "checkbox" => ("look.style.check", "text"),
                                "chips" => ("look.style.tags", "plain"),
                                "select" => ("look.style.status", "plain"),
                                _ => ("look.style.links", "plain"),
                            };
                            if off {
                                set(&mut t, to, toml::Value::String(plain.into()));
                            }
                            moved(w, file, &path, to);
                            continue;
                        }
                        "icons" => {
                            if off {
                                set(&mut t, "look.style.icons", v);
                            }
                            moved(w, file, &path, "look.style.icons");
                            continue;
                        }
                        "columns" => {
                            put(&mut t, "look.columns", v);
                        }
                        _ => {
                            put(&mut t, &format!("look.{path}"), v);
                        }
                    }
                    let to = if k == "style" {
                        "look.cells"
                    } else {
                        "look.columns"
                    };
                    moved(w, file, &path, to);
                }
            }
            other => {
                put(&mut t, "look.cells", other);
                moved(w, file, "cells", "look.cells");
            }
        }
    }
    if let Some(c) = t.remove("colors") {
        put(&mut t, "look.colors", c);
        moved(w, file, "colors", "look.colors");
    }
    // タブの行: auto だけが意味を持つ(always は既定)。
    if let Some(v) = t.remove("view_tabs") {
        if v.as_str() != Some("always") {
            put(&mut t, "display.tabs", v);
        }
        moved(w, file, "view_tabs", "display.tabs");
    }
    t
}

/// 道筋の親の表(無ければ作る。途中が表でなければ None)。
fn parent<'a>(t: &'a mut toml::Table, path: &str) -> Option<(&'a mut toml::Table, String)> {
    let mut parts: Vec<&str> = path.split('.').collect();
    let last = parts.pop().unwrap_or_default().to_string();
    let mut cur = t;
    for p in parts {
        cur = cur
            .entry(p.to_string())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()?;
    }
    Some((cur, last))
}

/// 道筋に値を置く(旧い `[cells]` の「使わない」は部品の形より強いので、あっても置き換える)。
fn set(t: &mut toml::Table, path: &str, v: toml::Value) {
    if let Some((tab, last)) = parent(t, path) {
        tab.insert(last, v);
    }
}
