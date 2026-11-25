//! 行を絞る検索(`\`)の当てる範囲(navigation.search-scope)の試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った: 人の表で、kana を出していないビューでも、kana の列を隠しても、「あんどう」と
//! 打てば「安藤」の行が残る。頭を隠したノートの名前は全部の文字で当てる。

use super::app::{App, ColorMode};
use super::keymap::Action;
use super::startup::Startup;
use super::test_screen::{ch, press, typing, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("安藤.md", "---\nkana: あんどう\norg: 北の会社\n---\n"),
    ("井上.md", "---\nkana: いのうえ\norg: 南の会社\n---\n"),
    ("上田.md", "---\nkana: うえだ\norg: 北の会社\n---\n"),
];

/// `NOTES` と、ビュー V(列 `order`)の `.base` を置いて、`.base` で開く。
fn boot(tag: &str, order: &[&str], config: Config) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let mut yaml = String::from("views:\n  - type: table\n    name: V\n    order:\n");
    for c in order {
        yaml.push_str(&format!("      - {c}\n"));
    }
    tmp.write("P.base", &yaml);
    let path = tmp.notes().join("P.base");
    let base = mdgrid::base::parse(&yaml).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: path,
        base: Some((base, "P.base".into(), None)),
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// `\` で `word` を打って決めたあとの行の名前(ファイルの名前)。
fn search(a: &mut App, word: &str) -> Vec<String> {
    ch(a, '\\');
    typing(a, word);
    press(a, KeyCode::Enter);
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

#[test]
fn test_search_matches_column_not_in_view() {
    // [utest -> req~navigation.search-scope~1] kana を出していないビュー(列は org だけ)で `\` で「あんどう」と打つと、
    // 「安藤」の行が残り、ほかの行は消える。
    let (_t, mut a) = boot("ss_not_in_view", &["org"], Config::default());
    assert!(!a.cols.iter().any(|c| c == "kana"), "{:?}", a.cols);
    assert_eq!(search(&mut a, "あんどう"), ["安藤.md"]);
}

#[test]
fn test_search_matches_hidden_column() {
    // [utest -> req~navigation.search-scope~1] kana を出したビューで kana の列を隠しても、「あんどう」で「安藤」の行が残る。
    let (_t, mut a) = boot("ss_hidden", &["kana", "org"], Config::default());
    a.col = a.cols.iter().position(|c| c == "kana").unwrap();
    a.apply(Action::HideColumn);
    assert!(!a.cols.iter().any(|c| c == "kana"), "{:?}", a.cols);
    assert_eq!(search(&mut a, "あんどう"), ["安藤.md"]);
}

#[test]
fn test_search_matches_whole_note_name_with_hidden_head() {
    // [utest -> req~navigation.search-scope~1] note_name で頭を隠したノート 20261015_資料を作る.md は、`\` で 20261015 と打つと残る。
    let tmp = Tmp::new("ss_head");
    tmp.write("20261015_資料を作る.md", "---\nkana: しりょう\n---\n");
    tmp.write("20261016_会議.md", "---\nkana: かいぎ\n---\n");
    let (config, w) = mdgrid::config::parse("[display]\nnote_name = '^\\d{8}_'\n").expect("設定");
    assert!(w.is_empty(), "{w:?}");
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    assert_eq!(search(&mut a, "20261015"), ["20261015_資料を作る.md"]);
}
