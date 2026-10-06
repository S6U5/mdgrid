//! その場の操作の一覧(SR-24)と、前置きのキーの続きの案内(SR-25)の受け入れの試験。
//! 記録: specs/_changes/2026-10-05-action-menu.md。
//!
//! 実装より先に、実装を見ずに書いた。まだ無い動作とモードは名前で引く(`Action::by_name("action_menu")`・
//! `Mode::by_name("menu")`)。一覧の中身は、窓の枠(開く前と後で変わった桁のうち、四隅が角の文字の矩形)の
//! 内側の桁の文字で確かめる。窓の外の表の行は、開く前と同じ文字が残る。
//! 一覧は4つの節で20行ほどになるので、一覧の試験は 100×50 の画面で描く。
//! セルの右クリック(SR-24)は App に右クリックの口がまだ無いので、本物の実行ファイルで
//! tests/test_action_menu_e2e.rs が確かめる。
//! 試験の環境は日本語に固定されている(.cargo/config.toml)。英語は `scoped(Lang::En)` で確かめる。

use super::keymap::{self, Action, Mode};
use super::startup::Startup;
use super::test_grid::labels;
use super::test_screen::{ch, col_named, make, press, screen, text, Tmp};
use super::*;
use mdgrid::config;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// status の値の順が名前の順と違う保管庫(`s` で並びが変わる)。
pub(super) const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: alpha\nstatus: c\n---\n"),
    ("b.md", "---\ntitle: beta\nstatus: a\n---\n"),
    ("c.md", "---\ntitle: gamma\nstatus: b\n---\n"),
];

/// 英字だけの保管庫(英語の画面の試験)。
const EN_NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: meeting\nstatus: doing\n---\n"),
    ("b.md", "---\ntitle: shopping\nstatus: done\n---\n"),
];

const W: u16 = 100;
const H: u16 = 50;

/// 80×24 の下の帯の行(添字)。
const BAND: usize = 21;

// ---- 道具 ----

/// 一覧のモード(まだ無ければ落ちる)。
pub(super) fn menu_mode() -> Mode {
    Mode::by_name("menu").expect("モード menu が無い(SR-24)")
}

/// 一覧を開く動作(まだ無ければ落ちる)。
pub(super) fn menu_action() -> Action {
    Action::by_name("action_menu").expect("動作 action_menu が無い(SR-24)")
}

/// 描いた画面(バッファと、行ごとの文字)。`{}` で文字を出す(落ちたときの説明)。
pub(super) struct Shot {
    pub(super) buf: Buffer,
    pub(super) text: String,
}

impl std::fmt::Display for Shot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

/// `w`×`h` で描いた画面。
pub(super) fn shot(app: &mut App, w: u16, h: u16) -> Shot {
    app.resize(w, h);
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, app)).unwrap();
    let buf = term.backend().buffer().clone();
    let text = text(&buf);
    Shot { buf, text }
}

/// 100×50 で描いた画面。
fn tall(app: &mut App) -> Shot {
    shot(app, W, H)
}

/// 窓の角の文字。
const CORNERS: &str = "+┌┐└┘╭╮╰╯";

fn is_corner(buf: &Buffer, x: u16, y: u16) -> bool {
    CORNERS.contains(buf[(x, y)].symbol())
}

/// 窓の枠(左の桁, 上の行, 右の桁, 下の行)。開く前と後で変わった桁のうち、四隅が角の文字になる矩形。
pub(super) fn frame(before: &Shot, after: &Shot) -> Option<(u16, u16, u16, u16)> {
    let (b, a) = (&before.buf, &after.buf);
    let (w, h) = (a.area.width, a.area.height);
    let changed = |x: u16, y: u16| b[(x, y)].symbol() != a[(x, y)].symbol();
    for y in 0..h {
        for x in 0..w {
            if !(is_corner(a, x, y) && changed(x, y)) {
                continue;
            }
            for x2 in (x + 1..w).rev() {
                if !(is_corner(a, x2, y) && changed(x2, y)) {
                    continue;
                }
                for y2 in y + 1..h {
                    if is_corner(a, x, y2) && is_corner(a, x2, y2) {
                        return Some((x, y, x2, y2));
                    }
                }
            }
        }
    }
    None
}

/// バッファの行 `y` の、桁 `x0..x1` の文字(幅2の文字の後ろの空きの桁は飛ばす)。
pub(super) fn cells(buf: &Buffer, y: u16, x0: u16, x1: u16) -> String {
    let mut out = String::new();
    let mut x = x0;
    while x < x1 {
        let s = buf[(x, y)].symbol();
        out.push_str(s);
        x += width::width(s).max(1) as u16;
    }
    out
}

/// 一覧の行: 窓の枠の内側の桁だけ(上の枠の行の題も含む)。窓が無ければ空。
pub(super) fn menu_lines(before: &Shot, after: &Shot) -> Vec<String> {
    let Some((x0, y0, x1, y1)) = frame(before, after) else {
        return Vec::new();
    };
    (y0..y1)
        .map(|y| cells(&after.buf, y, x0 + 1, x1).trim_end().to_string())
        .collect()
}

/// 枠と空白を除いた中身。
fn bare(line: &str) -> String {
    line.trim_matches(|c: char| c.is_whitespace() || "│┃|─━-+┌┐└┘╭╮╰╯├┤".contains(c))
        .to_string()
}

/// 一覧の中に、中身が `name` だけの行(節の見出し)がある。
pub(super) fn has_section(lines: &[String], name: &str) -> bool {
    lines.iter().any(|l| bare(l) == name)
}

/// 一覧の中に、`label` と、語として `key` を含む行がある。
fn has_item(lines: &[String], label: &str, key: &str) -> bool {
    lines
        .iter()
        .any(|l| l.contains(label) && l.split_whitespace().any(|w| w == key))
}

/// 表のモードの動作の今の表示名(キーの表の表示名)。
pub(super) fn label_of(a: &App, action: Action) -> String {
    a.keys
        .iter()
        .find(|b| b.mode == Mode::Table && b.action == action)
        .map(|b| b.text().to_string())
        .unwrap_or_else(|| panic!("表のモードに {action:?} が無い"))
}

/// status のセルを選んで、`x` で一覧を開く。開く前と後の 100×50 の画面を返す。
fn open_menu(a: &mut App, row: usize) -> (Shot, Shot) {
    a.row = row;
    col_named(a, "status");
    let before = tall(a);
    ch(a, 'x');
    let after = tall(a);
    (before, after)
}

/// 同じ保管庫で `s` を押した並び(SR-24 の「`s` と同じ」の比べる相手)。
pub(super) fn sorted_by_s(name: &str) -> Vec<String> {
    let (_t, mut b) = make(name, NOTES);
    col_named(&mut b, "status");
    ch(&mut b, 's');
    labels(&b)
}

/// main と同じ道筋(`App::new` → `start` → 読み込み)で、設定 `toml` で開く。
fn boot_toml(tmp: &Tmp, toml: &str) -> App {
    let (config, warnings) = config::parse(toml).expect("読める設定");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings,
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn is_ja(c: char) -> bool {
    matches!(c,
        '\u{3001}'..='\u{303F}'
        | '\u{3040}'..='\u{309F}'
        | '\u{30A0}'..='\u{30FF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
    )
}

pub(super) fn band(s: &str) -> String {
    s.lines().nth(BAND).unwrap_or("").to_string()
}

// ---- SR-24 ----

#[test]
fn test_sr_24_x_opens_menu_with_sections_and_keys() {
    // [SR-24] 表のモードの既定 `x` = action_menu。status のセルで `x` → 一覧のモードになり、
    // 「セル」「列」「行」の節の見出しと、「編集  Enter」「この列で並べ替え  s」の行が出る。
    let (_t, mut a) = make("sr24open", NOTES);
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "x"),
        Some(menu_action()),
        "表で x が action_menu でない"
    );
    let (before, after) = open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    for sec in ["セル", "列", "行"] {
        assert!(
            has_section(&lines, sec),
            "節「{sec}」の見出しが無い:\n{after}"
        );
    }
    let edit = label_of(&a, Action::by_name("edit").unwrap());
    assert!(
        has_item(&lines, "編集", &keymap::display("Enter")) || has_item(&lines, &edit, "Enter"),
        "「編集  Enter」が無い:\n{after}"
    );
    let sort = label_of(&a, Action::by_name("sort_column").unwrap());
    assert!(
        has_item(&lines, "この列で並べ替え", "s") || has_item(&lines, &sort, "s"),
        "「この列で並べ替え  s」が無い:\n{after}"
    );
}

#[test]
fn test_sr_24_table_rows_stay_outside_window() {
    // [SR-24] 一覧は表の上に重ねる窓で、窓の外の表は消さない: 窓の行の、窓より左の桁には開く前と同じ
    // 表の文字(b.md・c.md の行)が残る。
    let (_t, mut a) = make("sr24outside", NOTES);
    let (before, after) = open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let (x0, y0, _, y1) =
        frame(&before, &after).unwrap_or_else(|| panic!("窓の枠が無い:\n{after}"));
    let mut left = String::new();
    for y in y0..=y1 {
        let b = cells(&before.buf, y, 0, x0);
        let n = cells(&after.buf, y, 0, x0);
        assert_eq!(n, b, "窓の左の表の行 {y} が変わった:\n{after}");
        left.push_str(&b);
    }
    assert!(
        left.contains(" b") && left.contains(" c"),
        "材料: 窓の行の左に表の行が無い:\n{before}"
    );
}

#[test]
fn test_sr_24_enter_on_sort_item_sorts_like_s() {
    // [SR-24] 一覧の並びは SR-24 の文の順(セル: 編集・空にする・コピー・エディタで開く・詳細、
    // 列: この列で並べ替え …)。書けるセルでは先頭から ↓ 5回で「この列で並べ替え」。
    // Enter → 一覧が閉じ、`s` を直接押したのと同じ並びになる。
    let want = sorted_by_s("sr24enter-s");
    let (_t, mut a) = make("sr24enter", NOTES);
    let start = labels(&a);
    assert_ne!(start, want, "材料: s で並びが変わらない");
    open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    for _ in 0..5 {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "Enter で一覧が閉じない");
    assert_eq!(labels(&a), want, "一覧の並べ替えが s と違う");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_sr_24_jk_move_in_menu() {
    // [SR-24] 一覧の中は `j` `k` でも動く: `j` 6回・`k` 1回で「この列で並べ替え」→ Enter で `s` と同じ。
    let want = sorted_by_s("sr24jk-s");
    let (_t, mut a) = make("sr24jk", NOTES);
    open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    for _ in 0..6 {
        ch(&mut a, 'j');
    }
    ch(&mut a, 'k');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "Enter で一覧が閉じない");
    assert_eq!(labels(&a), want, "j k で選んだ並べ替えが s と違う");
}

#[test]
fn test_sr_24_item_key_runs_action() {
    // [SR-24] 一覧で行に添えたキー `s` を押す → 「この列で並べ替え」を実行して閉じる(`s` と同じ並び)。
    let want = sorted_by_s("sr24key-s");
    let (_t, mut a) = make("sr24key", NOTES);
    open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    ch(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "s で一覧が閉じない");
    assert_eq!(labels(&a), want, "一覧の s が表の s と違う");
}

#[test]
fn test_sr_24_readonly_cell_hides_edit_and_clear() {
    // [SR-24] [WB-5] 読むだけのセル(BOM のノート)で `x` → 一覧に「編集」「空にする」が無い。
    // 対照: 書けるノートのセルでは両方ある。
    let (_t, mut a) = make(
        "sr24ro",
        &[
            ("bom.md", "\u{FEFF}---\nstatus: draft\n---\nbody\n"),
            ("ok.md", "---\nstatus: todo\n---\nok\n"),
        ],
    );
    let ok = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "ok.md")
        .unwrap();
    let bom = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "bom.md")
        .unwrap();
    assert!(
        a.src.get(&a.rows[bom], "status").lock.is_some(),
        "材料: BOM のノートのセルが読むだけでない"
    );

    let (before, after) = open_menu(&mut a, ok);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(
        lines.iter().any(|l| l.contains("編集")),
        "書けるセルに「編集」が無い:\n{after}"
    );
    assert!(
        lines.iter().any(|l| l.contains("空にする")),
        "書けるセルに「空にする」が無い:\n{after}"
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);

    let (before, after) = open_menu(&mut a, bom);
    assert_eq!(a.mode, menu_mode(), "読むだけのセルで x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(
        !lines.iter().any(|l| l.contains("編集")),
        "読むだけのセルに「編集」がある:\n{after}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("空にする")),
        "読むだけのセルに「空にする」がある:\n{after}"
    );
}

#[test]
fn test_sr_24_esc_closes_without_doing_anything() {
    // [SR-24] Esc → 何もせず閉じて表に戻る(ためる変更 0、並びも位置も同じ、表の画面も同じ)。
    let (_t, mut a) = make("sr24esc", NOTES);
    col_named(&mut a, "status");
    a.row = 1;
    let rows = labels(&a);
    let before = screen(&a);
    ch(&mut a, 'x');
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "Esc で一覧が閉じない");
    assert_eq!(a.changes.count(), 0);
    assert_eq!(labels(&a), rows);
    assert_eq!(a.row, 1);
    let after = screen(&a);
    // メッセージ行(下の帯の下)は比べない。
    let head = |s: &str| s.lines().take(BAND + 1).collect::<Vec<_>>().join("\n");
    assert_eq!(head(&after), head(&before), "Esc のあと表が変わった");
}

#[test]
fn test_sr_24_rebound_key_opens_menu() {
    // [SR-24] [SR-4] 設定 `[keys.table] "m" = "action_menu"` → `m` で一覧が開く。
    let tmp = Tmp::new("sr24rebind");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let mut a = boot_toml(&tmp, "[keys.table]\n\"m\" = \"action_menu\"\n");
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "m"),
        Some(menu_action()),
        "設定の m が action_menu にならない"
    );
    col_named(&mut a, "status");
    ch(&mut a, 'm');
    assert_eq!(a.mode, menu_mode(), "m で一覧が開かない");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_24_bulk_item_only_with_marks() {
    // [SR-24] 印の無いとき「選んだ行にまとめて入れる」は無く、2行に印(Space)を付けると出る。
    let (_t, mut a) = make("sr24bulk", NOTES);
    let (before, after) = open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(
        !lines.iter().any(|l| l.contains("まとめて")),
        "印の無いときに「まとめて入れる」がある:\n{after}"
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);

    a.row = 0;
    col_named(&mut a, "status");
    ch(&mut a, ' ');
    ch(&mut a, ' ');
    let row = a.row;
    let (before, after) = open_menu(&mut a, row);
    assert_eq!(a.mode, menu_mode(), "印のあとに x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(
        lines.iter().any(|l| l.contains("まとめて")),
        "2行に印があるのに「まとめて入れる」が無い:\n{after}"
    );
}

#[test]
fn test_sr_24_english_menu_has_no_japanese() {
    // [SR-24] [SR-23] 英語の画面で `x` → 一覧に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr24en", EN_NOTES);
    let (before, after) = open_menu(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(lines.len() >= 4, "一覧が出ていない:\n{after}");
    for l in &lines {
        assert!(
            !l.chars().any(is_ja),
            "英語の一覧に日本語の文字: {l:?}\n{after}"
        );
    }
}

// ---- SR-25 ----

#[test]
fn test_sr_25_prefix_g_shows_next_keys_then_moves() {
    // [SR-25] 表で `g` → 下の帯が続きのキーの案内(`g` と「先頭の行」)になる。続けて `g` → 先頭の行へ動き、
    // 帯が元(「Enter 編集」などの形)に戻る。
    let (_t, mut a) = make("sr25g", NOTES);
    a.row = 2;
    a.scroll_into_view();
    let top = label_of(&a, Action::Top);
    let normal = band(&screen(&a));
    assert!(
        normal.contains("Enter 編集"),
        "材料: 元の帯の形が違う: {normal}"
    );

    ch(&mut a, 'g');
    let b = band(&screen(&a));
    assert!(
        b.contains("先頭の行") || b.contains(&top),
        "g のあとの帯に「先頭の行」が無い: {b}"
    );
    assert!(
        b.split_whitespace().any(|w| w == "g"),
        "g のあとの帯に続きのキー g が無い: {b}"
    );
    assert!(!b.contains("Enter 編集"), "g のあとも帯が元のまま: {b}");
    assert_eq!(a.row, 2, "g だけで動いた");

    ch(&mut a, 'g');
    assert_eq!(a.row, 0, "g g で先頭の行へ動かない");
    let b = band(&screen(&a));
    assert!(b.contains("Enter 編集"), "g g のあと帯が戻らない: {b}");
    assert!(
        !b.contains("先頭の行") && !b.contains(&top),
        "g g のあとも案内が残る: {b}"
    );
}

#[test]
fn test_sr_25_prefix_then_esc_restores_band() {
    // [SR-25] `g` のあと Esc → 動かず、帯が元に戻る。
    let (_t, mut a) = make("sr25esc", NOTES);
    a.row = 2;
    a.scroll_into_view();
    let top = label_of(&a, Action::Top);
    ch(&mut a, 'g');
    let b = band(&screen(&a));
    assert!(
        b.contains("先頭の行") || b.contains(&top),
        "g のあとの帯に「先頭の行」が無い: {b}"
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.row, 2, "g のあとの Esc で動いた");
    assert_eq!(a.mode, Mode::Table);
    let b = band(&screen(&a));
    assert!(b.contains("Enter 編集"), "Esc のあと帯が戻らない: {b}");
    assert!(
        !b.contains("先頭の行") && !b.contains(&top),
        "Esc のあとも案内が残る: {b}"
    );
}

#[test]
fn test_sr_25_configured_prefix_shows_hint() {
    // [SR-25] 設定で `[keys.table] "z z" = "bottom"` を足す → `z` で帯に `z` と「末尾の行」。続けて `z` → 末尾の行。
    let tmp = Tmp::new("sr25z");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let mut a = boot_toml(&tmp, "[keys.table]\n\"z z\" = \"bottom\"\n");
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "z z"),
        Some(Action::Bottom),
        "材料: 設定の z z が bottom にならない"
    );
    let bottom = label_of(&a, Action::Bottom);
    assert_eq!(a.row, 0);
    ch(&mut a, 'z');
    let b = band(&screen(&a));
    assert!(
        b.contains("末尾の行") || b.contains(&bottom),
        "z のあとの帯に「末尾の行」が無い: {b}"
    );
    assert!(
        b.split_whitespace().any(|w| w == "z"),
        "z のあとの帯に続きのキー z が無い: {b}"
    );
    ch(&mut a, 'z');
    assert_eq!(a.row, a.rows.len() - 1, "z z で末尾の行へ動かない");
    let b = band(&screen(&a));
    assert!(b.contains("Enter 編集"), "z z のあと帯が戻らない: {b}");
}
