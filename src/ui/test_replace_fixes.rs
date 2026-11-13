//! レビューの指摘の直しの試験(specs/_changes/2026-11-13-regex-replace.md の「レビューの指摘」の 1・2)。
//! 実装した本人が書いた(独立したテスト役ではない)。値は作ったもの。
use super::app::{App, ColorMode};
use super::keymap::Action;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, Source};

fn boot(name: &str, notes: &[(&str, &str)]) -> (std::path::PathBuf, App) {
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-test-replace-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    for (n, t) in notes {
        std::fs::write(dir.join(n), t).unwrap();
    }
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 24);
    while !app.loaded() {
        app.load_step(100);
    }
    let j = app.cols.iter().position(|c| c == "title").unwrap();
    app.col = j;
    (dir, app)
}

fn type_and_run(app: &mut App, text: &str) {
    for c in text.chars() {
        app.palette_insert(c);
    }
    app.apply(Action::Run);
}

fn pending(app: &App, name: &str) -> Option<String> {
    let row = app
        .src
        .rows()
        .into_iter()
        .find(|r| app.src.label(r) == name)
        .unwrap();
    match app.changes.pending(&row, "title") {
        Some(NewValue::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

#[test]
fn test_ce_15_fullwidth_text_is_kept_as_typed() {
    // [CE-15] 探す正規表現と置き換える文字の全角の字・`・`・`、` は、半角に直さず打ったまま使う。
    let (dir, mut app) = boot("wide", &[("a.md", "---\ntitle: A/B(1)\n---\n")]);
    app.apply(Action::RegexReplace);
    type_and_run(&mut app, "/");
    type_and_run(&mut app, "・");
    assert_eq!(pending(&app, "a.md").as_deref(), Some("A・B(1)"));
    app.apply(Action::RegexReplace);
    // 全角の括弧は、組の括弧(半角)にならず、字そのものとして探す(値の半角の括弧には一致しない)。
    type_and_run(&mut app, "(");
    type_and_run(&mut app, "〔");
    assert_eq!(pending(&app, "a.md").as_deref(), Some("A・B(1)"));
    app.apply(Action::RegexReplace);
    type_and_run(&mut app, "B");
    type_and_run(&mut app, "B");
    assert_eq!(pending(&app, "a.md").as_deref(), Some("A・B(1)"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_ce_15_empty_string_cell_is_untouched() {
    // [CE-15] 空の文字列のセルは、空に一致する正規表現でも変えない。
    let (dir, mut app) = boot(
        "empty",
        &[
            ("a.md", "---\ntitle: \"\"\n---\n"),
            ("b.md", "---\ntitle: x\n---\n"),
        ],
    );
    app.apply(Action::RegexReplace);
    type_and_run(&mut app, "^");
    type_and_run(&mut app, "new ");
    assert_eq!(pending(&app, "a.md"), None);
    assert_eq!(pending(&app, "b.md").as_deref(), Some("new x"));
    let _ = std::fs::remove_dir_all(dir);
}
