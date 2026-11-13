//! ボードを開いている間のレーンの残し方(BD-2。決定 specs/_decisions/2026-11-13-board-lanes-stay.md)の試験。
//! 実装した本人が書いた(独立したテスト役ではない)。値は作ったもの。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use mdgrid::source::markdown::Markdown;

fn boot(name: &str, notes: &[(&str, &str)]) -> (std::path::PathBuf, App) {
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-test-board-lanes-{name}-{}-{}",
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

/// レーンの (値, カードの数)。値の無いレーンは None。
fn lanes(app: &App) -> Vec<(Option<String>, usize)> {
    app.board
        .as_ref()
        .expect("board")
        .lanes
        .iter()
        .map(|l| (l.value.clone(), l.rows.len()))
        .collect()
}

#[test]
fn test_bd_2_emptied_none_lane_stays_until_reopened() {
    // [BD-2] 空の行が1つだけの表で、そのカードを「(なし)」から todo へ移す → 「(なし) 0」のレーンが残り、L で戻せる。
    // ボードを閉じて開き直す → 「(なし)」は出ない。
    let (dir, mut app) = boot(
        "none",
        &[
            ("a.md", "---\nstatus:\ntitle: a\n---\n"),
            ("b.md", "---\nstatus: todo\ntitle: b\n---\n"),
        ],
    );
    app.apply(Action::BoardView);
    assert_eq!(app.mode, Mode::Board);
    let todo = Some("todo".to_string());
    assert_eq!(lanes(&app), [(todo.clone(), 1), (None, 1)]);
    // 「(なし)」のカードを選んで、左の todo へ移す。
    app.apply(Action::Right);
    app.apply(Action::CardLeft);
    assert_eq!(lanes(&app), [(todo.clone(), 2), (None, 0)]);
    assert_eq!(app.changes.count(), 1);
    // 戻せる(ためた変更は外れる)。
    app.apply(Action::CardRight);
    assert_eq!(lanes(&app), [(todo.clone(), 1), (None, 1)]);
    assert_eq!(app.changes.count(), 0);
    // もう一度移して、閉じて開き直すと、空の行が無いので「(なし)」は出ない。
    app.apply(Action::CardLeft);
    app.apply(Action::Close);
    assert_eq!(app.mode, Mode::Table);
    app.col = app.cols.iter().position(|c| c == "status").unwrap();
    app.apply(Action::BoardView);
    assert_eq!(lanes(&app), [(todo, 2)]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_bd_2_emptied_value_lane_stays_and_keeps_its_place() {
    // [BD-2] 決まりに無い値のレーンも、最後のカードを移して空になっても、開いている間は同じ場所に残る。
    let (dir, mut app) = boot(
        "value",
        &[
            ("a.md", "---\nstatus: doing\n---\n"),
            ("b.md", "---\nstatus: todo\n---\n"),
        ],
    );
    app.apply(Action::BoardView);
    let (doing, todo) = (Some("doing".to_string()), Some("todo".to_string()));
    assert_eq!(lanes(&app), [(doing.clone(), 1), (todo.clone(), 1)]);
    app.apply(Action::Left);
    app.apply(Action::CardRight);
    assert_eq!(lanes(&app), [(doing.clone(), 0), (todo.clone(), 2)]);
    app.apply(Action::CardLeft);
    assert_eq!(lanes(&app), [(doing, 1), (todo, 1)]);
    assert_eq!(app.changes.count(), 0);
    let _ = std::fs::remove_dir_all(dir);
}
