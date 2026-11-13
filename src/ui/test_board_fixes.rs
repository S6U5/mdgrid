//! ボードの画面のレビューの指摘の直しの試験(specs/_changes/2026-11-13-board-view.md の「レビューの指摘」の 1〜4)。
//! 実装した本人が書いた(独立したテスト役ではない)。値は作ったもの。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::Source;

fn boot(name: &str, notes: &[(&str, String)]) -> (std::path::PathBuf, App) {
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-test-board-fixes-{name}-{}-{}",
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
    app.col = app.cols.iter().position(|c| c == "status").unwrap();
    (dir, app)
}

fn screen(app: &App) -> String {
    let (w, h) = (
        app.size.0.saturating_sub(1) as usize,
        app.size.1.saturating_sub(1) as usize,
    );
    super::view::render(app, w, h)
        .into_iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_bd_2_long_value_keeps_count_and_marks() {
    // [BD-2][BD-5] 値が長くてレーンが狭くても、題のカードの数と、下に隠れたカードの印は切れない(値の側を切る)。
    let mut notes: Vec<(String, String)> = (0..12)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                "---\nstatus: in-progress-long\n---\n".to_string(),
            )
        })
        .collect();
    notes.push(("z.md".into(), "---\nstatus: zz\n---\n".into()));
    let refs: Vec<(&str, String)> = notes.iter().map(|(n, t)| (n.as_str(), t.clone())).collect();
    let (dir, mut app) = boot("long", &refs);
    app.resize(37, 16);
    app.apply(Action::BoardView);
    assert_eq!(app.mode, Mode::Board);
    let s = screen(&app);
    let head = s
        .lines()
        .find(|l| l.contains(" 12 v"))
        .unwrap_or_else(|| panic!("{s}"));
    assert!(head.contains("in-prog"), "{s}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_bd_3_card_name_shows_when_note_column_is_hidden() {
    // [BD-3] ノートの欄の名前を出さない表(note_path = off)でも、カードの1行目にノートの名前が出る。
    let (dir, mut app) = boot(
        "hidden",
        &[("write-spec.md", "---\nstatus: todo\n---\n".to_string())],
    );
    app.display.note_path = mdgrid::display::NotePath::Off;
    app.refresh();
    app.apply(Action::BoardView);
    let s = screen(&app);
    assert!(s.contains(">write-spec"), "{s}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_bd_4_moving_back_drops_the_pending_change_of_an_empty_text() {
    // [BD-4][WB-17] 空の文字列の値のカードを、隣へ移して元のレーン「(なし)」へ戻すと、ためた変更は外れる
    // (元の値の書き方のまま。未保存に数えない)。
    let (dir, mut app) = boot(
        "back",
        &[
            ("a.md", "---\nstatus: \"\"\n---\n".to_string()),
            ("b.md", "---\nstatus: todo\n---\n".to_string()),
            ("c.md", "---\nstatus: todo\n---\n".to_string()),
        ],
    );
    assert_eq!(app.kind_of("status"), mdgrid::types::Kind::Text);
    app.apply(Action::BoardView);
    let a = app
        .src
        .rows()
        .into_iter()
        .find(|r| app.src.label(r) == "a.md")
        .unwrap();
    for _ in 0..3 {
        if app.board.as_ref().unwrap().row() == Some(&a) {
            break;
        }
        app.apply(Action::Right);
    }
    assert_eq!(app.board.as_ref().unwrap().row(), Some(&a));
    app.apply(Action::CardLeft);
    assert_eq!(app.changes.count(), 1);
    app.apply(Action::CardRight);
    assert_eq!(app.changes.count(), 0, "{}", screen(&app));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_bd_6_click_below_the_drawn_cards_does_not_pick_a_hidden_card() {
    // [BD-6] 見えているカードより下の行のクリックは、描いていないカードを選ばない。
    let notes: Vec<(String, String)> = (0..9)
        .map(|i| (format!("n{i}.md"), "---\nstatus: todo\n---\n".to_string()))
        .collect();
    let refs: Vec<(&str, String)> = notes.iter().map(|(n, t)| (n.as_str(), t.clone())).collect();
    let (dir, mut app) = boot("click", &refs);
    app.apply(Action::BoardView);
    let s = screen(&app);
    let lines: Vec<&str> = s.lines().collect();
    // 窓の下の縁のすぐ上の行(カードの欄の最後の行)。
    let bottom = lines
        .iter()
        .rposition(|l| l.starts_with('╰') || l.starts_with('+') || l.starts_with('└'))
        .unwrap_or_else(|| panic!("{s}"));
    let before = app.board.as_ref().unwrap().card;
    for y in (bottom - 2)..bottom {
        app.click(3, y as u16);
        let b = app.board.as_ref().expect("ボードのまま");
        let shown = screen(&app);
        let name = format!("n{}", b.card);
        assert!(
            shown.contains(&name),
            "選んだカード {name} が見えない:\n{shown}"
        );
    }
    let _ = before;
    let _ = std::fs::remove_dir_all(dir);
}
