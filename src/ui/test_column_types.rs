//! 列の型を設定 `[types]` と見出しの型の印から選び直す(CE-35・CE-2)の受け入れ試験。
//! 記録: specs/_changes/2026-10-13-column-types.md(タスク #1)。
//!
//! 仮定(実装役に渡す): パレットの命令の名前は `column_type`。型の窓のモードの名前は `ColumnType`。
//! 窓の項目は「自動(推定の型)」・テキスト・数・チェック・日付・日時・リストの順。選んだ型は views.toml の
//! 開いた表の `[[table]]` の `types` に書く。

use super::startup::Startup;
use super::test_screen::{ch, press, text, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

fn boot(name: &str, toml: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ndue: 2026-10-01\nn: 3\n---\n");
    tmp.write("b.md", "---\ndue: 2026-10-05\nn: 5\n---\n");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), color);
    a.resize(100, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: color == ColorMode::None,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.col = a.cols.iter().position(|c| c == "due").unwrap();
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn views_toml(t: &Tmp) -> String {
    std::fs::read_to_string(t.0.join("config/views.toml")).unwrap_or_default()
}

fn open_menu(a: &mut App) {
    ch(a, ':');
    typing(a, "column_type");
    press(a, KeyCode::Enter);
    assert_eq!(format!("{:?}", a.mode), "ColumnType", "{}", screen(a));
}

/// 窓の中の `name` の項目まで ↓ で動いて Enter(選んでいる行は `>` で始まる)。
fn choose(a: &mut App, name: &str) {
    // 一番上へ戻してから、下へ順に見る。
    for _ in 0..10 {
        press(a, KeyCode::Up);
    }
    for _ in 0..10 {
        let s = screen(a);
        let hit = s.lines().any(|l| {
            l.split_once("> ")
                .is_some_and(|(_, rest)| rest.trim_start_matches(['●', ' ']).starts_with(name))
        });
        if hit {
            press(a, KeyCode::Enter);
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("「{name}」を選べない:\n{}", screen(a));
}

#[test]
fn test_ce_35_types_setting_wins_over_inference() {
    // [CE-35][CE-2] `[types] due = "number"` → 推定(日付)より設定が勝ち、日付の値に `!`。
    let (_t, a) = boot("ce35_cfg", "[types]\ndue = \"number\"\n", ColorMode::None);
    assert_eq!(a.kind_of("due"), mdgrid::types::Kind::Number);
    assert!(screen(&a).contains("!2026-10-01"), "{}", screen(&a));
}

#[test]
fn test_ce_35_unknown_type_warns() {
    // [CE-35] 知らない型の名前 → 警告して、その列は推定のまま。
    let (c, w) = mdgrid::config::parse("[types]\ndue = \"when\"\n").unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("types.due"), "{w:?}");
    let _ = c;
}

#[test]
fn test_ce_35_menu_lists_types_and_writes_table_scope() {
    // [CE-35] パレットの column_type → 型の窓。「数」を選ぶ → views.toml の表に due = "number"、表に効く。
    let (t, mut a) = boot("ce35_menu", "", ColorMode::None);
    assert_eq!(a.kind_of("due"), mdgrid::types::Kind::Date);
    open_menu(&mut a);
    let s = screen(&a);
    for item in [
        "自動",
        "テキスト",
        "数",
        "チェック",
        "日付",
        "日時",
        "リスト",
    ] {
        assert!(s.contains(item), "窓に「{item}」:\n{s}");
    }
    choose(&mut a, "数");
    assert_eq!(format!("{:?}", a.mode), "Table");
    assert_eq!(a.kind_of("due"), mdgrid::types::Kind::Number);
    let v = views_toml(&t);
    assert!(v.contains("due = \"number\""), "{v}");
    assert!(screen(&a).contains("!2026-10-01"), "{}", screen(&a));
    // ノートは書き換えない。
    assert_eq!(
        std::fs::read_to_string(t.notes().join("a.md")).unwrap(),
        "---\ndue: 2026-10-01\nn: 3\n---\n"
    );
}

#[test]
fn test_ce_35_auto_returns_to_inference() {
    // [CE-35] 「自動」を選ぶ → 推定の型に戻り、views.toml から due が消える。
    let (t, mut a) = boot("ce35_auto", "", ColorMode::None);
    open_menu(&mut a);
    choose(&mut a, "数");
    open_menu(&mut a);
    choose(&mut a, "自動");
    assert_eq!(a.kind_of("due"), mdgrid::types::Kind::Date);
    assert!(!views_toml(&t).contains("due ="), "{}", views_toml(&t));
}

#[test]
fn test_ce_35_header_mark_click_opens_menu() {
    // [CE-35][SR-35] 色を使う表示の見出しの型の印(日付の ◷)をクリック → 型の窓。見出しの名前のクリックは
    // 今までどおり並べ替え(NV-3)。
    let (_t, mut a) = boot("ce35_click", "[look.style]\nicons = true\n", ColorMode::Rgb);
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("◷ due"))
        .unwrap_or_else(|| panic!("見出しの型の印が無い:\n{s}"));
    let x = width::width(&line[..line.find("◷ due").unwrap()]);
    a.click(x as u16, y as u16);
    assert_eq!(format!("{:?}", a.mode), "ColumnType", "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    assert_eq!(format!("{:?}", a.mode), "Table");
    a.click(x as u16 + 3, y as u16);
    assert_eq!(format!("{:?}", a.mode), "Table", "名前のクリックは並べ替え");
    assert!(!a.settings.sorts.is_empty(), "並べ替えが効く");
}

#[test]
fn test_ce_35_not_for_computed_columns() {
    // [CE-35] file.* の列では型の窓を出さない(理由を出す)。
    let (_t, mut a) = boot("ce35_file", "", ColorMode::None);
    a.cols.push("file.name".into());
    a.kinds.push(mdgrid::types::Kind::Text);
    a.col = a.cols.len() - 1;
    ch(&mut a, ':');
    typing(&mut a, "column_type");
    press(&mut a, KeyCode::Enter);
    assert_eq!(format!("{:?}", a.mode), "Table");
    assert!(a.message.is_some(), "理由を出す");
}
