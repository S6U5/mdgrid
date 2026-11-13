//! ボードの画面(BD-1〜BD-6)の画面の単体試験。仕様: specs/board/spec.md。
//! 記録: specs/_changes/2026-11-13-board-view.md(タスク #2〜#4。モードと動作の名前・既定のキー・画面の文言)。
//! 実装を見ずに書いた。値はどれも作ったもの。
//!
//! 画面は行の番号や桁でなく中身で探す: レーンの見出しは「値 数」、選んでいるカードは1行目の頭の `>`、
//! ためた変更の印は `*`。ボードの状態の欄(`App` の中)には頼らず、キー・クリック・描いた画面の文字・
//! ファイルの中身で確かめる。使う名前は `Mode::Board`・`Action::BoardView`・`Action::CardLeft`・`Action::CardRight` だけ。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, edit_cell, press, read, text, typing, Tmp};
use super::width::width;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use std::path::PathBuf;

// ---------------------------------------------------------------- 材料

/// BD-1 の例の表: status が todo(2行)・doing(1行)・done(1行)。estimate は数、due は日付、tags はリスト、
/// flag はチェックボックス。名前の順は draft-api・fix-login・ship-docs・write-spec。
const TASKS: &[(&str, &str)] = &[
    (
        "write-spec.md",
        "---\nstatus: todo\ndue: 2026-11-20\nowner: mika\ntags: [urgent, docs]\nestimate: 3\nflag: true\n---\n",
    ),
    (
        "fix-login.md",
        "---\nstatus: todo\ndue: 2026-11-18\nowner: ren\ntags: [web]\nestimate: 5\nflag: false\n---\n",
    ),
    (
        "draft-api.md",
        "---\nstatus: doing\ndue: 2026-11-25\nowner: sora\ntags: [web]\nestimate: 8\nflag: false\n---\n",
    ),
    (
        "ship-docs.md",
        "---\nstatus: done\ndue: 2026-11-10\nowner: yui\ntags: [docs]\nestimate: 2\nflag: true\n---\n",
    ),
];

const NAMES: &[&str] = &["write-spec", "fix-login", "draft-api", "ship-docs"];

/// 列の決まり(CE-36): status の値の候補と順。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n";

const REASON_JA: &str = "この列ではボードにできない";
const REASON_EN: &str = "this column cannot make a board";

fn startup(tmp: &Tmp, target: PathBuf, config: &str, readonly: bool) -> Startup {
    let (config, warnings) = mdgrid::config::parse(config).expect("設定が読める");
    assert!(warnings.is_empty(), "{warnings:?}");
    Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base: None,
    }
}

/// `notes` をフォルダの表として開く(幅 100・高さ 30)。`config` は全体の設定の文。
fn boot_with(tag: &str, notes: &[(&str, &str)], config: &str, readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
    app.start(startup(&tmp, dir, config, readonly));
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

fn boot(tag: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    boot_with(tag, notes, "", false)
}

/// `notes` を、`.base`(`base` の文)のビューで開く(絞り込み・列の並び・並べ替えを決める)。
fn boot_base(tag: &str, notes: &[(&str, &str)], base: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let path = tmp.notes().join("tasks.base");
    std::fs::write(&path, base).unwrap();
    let t = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 30);
    let mut s = startup(&tmp, path, "", false);
    s.base = t.base;
    app.start(s);
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// status の列にカーソルを置いて `B` で、ボードの画面にする。
fn open_board(a: &mut App) {
    col_named(a, "status");
    ch(a, 'B');
    assert_eq!(a.mode, Mode::Board, "{:?}\n{}", a.message, scr(a));
}

// ---------------------------------------------------------------- 画面を読む道具

/// App の大きさで描いた画面の文字。
fn scr(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// 行 `l` で、語 `name` が(前後が語の字でなく)立っている所のバイトの位置。
fn word_in(l: &str, name: &str) -> Option<usize> {
    l.match_indices(name)
        .map(|(i, _)| i)
        .find(|&i| !l[..i].ends_with(is_word) && !l[i + name.len()..].starts_with(is_word))
}

/// 画面で語 `name` の最初の字の桁と行。
fn find(s: &str, name: &str) -> Option<(u16, u16)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| word_in(l, name).map(|i| (width(&l[..i]) as u16, y as u16)))
}

/// 語 `name` のある行。
fn line_of(s: &str, name: &str) -> Option<String> {
    s.lines()
        .find(|l| word_in(l, name).is_some())
        .map(str::to_string)
}

/// レーンの見出し「値 数」。
struct Head {
    x: u16,
    y: u16,
    count: usize,
    /// 数のすぐあとの字(上下に隠れたカードの印 `^`・`v` が来る所。4字まで)。
    tail: String,
}

/// 値 `value` のレーンの見出し(値のあとに空白を挟んで数)。見えていなければ None。
fn head(s: &str, value: &str) -> Option<Head> {
    s.lines().enumerate().find_map(|(y, l)| {
        l.match_indices(value).find_map(|(i, _)| {
            if l[..i].ends_with(is_word) {
                return None;
            }
            let rest = &l[i + value.len()..];
            let t = rest.trim_start_matches(' ');
            if t.len() == rest.len() {
                return None;
            }
            let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
            let count = digits.parse().ok()?;
            Some(Head {
                x: width(&l[..i]) as u16,
                y: y as u16,
                count,
                tail: t[digits.len()..].chars().take(4).collect(),
            })
        })
    })
}

/// 値 `value` のレーンのカードの数(見出しの数)。レーンが見えていなければ None。
fn count(a: &App, value: &str) -> Option<usize> {
    head(&scr(a), value).map(|h| h.count)
}

/// 見えているレーンを、左からの順で(`values` のうち見出しのあるもの)。
fn lanes_shown(a: &App, values: &[&str]) -> Vec<String> {
    let s = scr(a);
    let mut v: Vec<(u16, String)> = values
        .iter()
        .filter_map(|v| head(&s, v).map(|h| (h.x, v.to_string())))
        .collect();
    v.sort();
    v.into_iter().map(|(_, v)| v).collect()
}

/// 見出しの行(枠の線と前後の空白を除く)。
fn head_row(a: &App, value: &str) -> String {
    let s = scr(a);
    let h = head(&s, value).unwrap_or_else(|| panic!("見出し {value}:\n{s}"));
    s.lines()
        .nth(h.y as usize)
        .unwrap()
        .trim_matches(|c: char| c == ' ' || c == '│' || c == '|')
        .to_string()
}

/// カード `name` が選ばれているか(1行目の頭に `>`。`>` と名前の間は空白と `*` だけ)。
fn selected(s: &str, name: &str) -> bool {
    s.lines().any(|l| {
        word_in(l, name).is_some_and(|i| {
            l[..i]
                .trim_end_matches(|c: char| c == ' ' || c == '*')
                .ends_with('>')
        })
    })
}

/// 選ばれているカードの名前(`names` のうち)。
fn selected_of(a: &App, names: &[&str]) -> Vec<String> {
    let s = scr(a);
    names
        .iter()
        .filter(|n| selected(&s, n))
        .map(|n| n.to_string())
        .collect()
}

/// カード `name` の1行目にためた変更の印 `*` があるか。
fn starred(a: &App, name: &str) -> bool {
    let s = scr(a);
    line_of(&s, name)
        .unwrap_or_else(|| panic!("カード {name}:\n{s}"))
        .contains('*')
}

/// キーだけで、カード `name` を選ぶ(左の端のレーンから順に、上のカードから探す)。
fn select_card(a: &mut App, name: &str) {
    for _ in 0..10 {
        ch(a, 'h');
    }
    for _ in 0..10 {
        for _ in 0..12 {
            ch(a, 'k');
        }
        for _ in 0..12 {
            if selected(&scr(a), name) {
                return;
            }
            ch(a, 'j');
        }
        ch(a, 'l');
    }
    panic!("カード {name} を選べない:\n{}", scr(a));
}

/// カード `name` が選ばれるまで、キー `key` を押す(端で止まっても回っても同じ所に着く)。
fn press_until(a: &mut App, key: char, name: &str) {
    for _ in 0..60 {
        if selected(&scr(a), name) {
            return;
        }
        ch(a, key);
    }
    panic!("{key} でカード {name} に着かない:\n{}", scr(a));
}

/// 表の画面で選んでいる行のノートの名前(拡張子なし)。
fn table_row(a: &App) -> String {
    let r = a.cur_row().expect("選んでいる行");
    a.src.label(&r).trim_end_matches(".md").to_string()
}

fn shift(a: &mut App, code: KeyCode) {
    a.key(KeyEvent::new(code, KeyModifiers::SHIFT));
}

fn msg(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

/// 描いた行が幅 `w - 1` に収まる(SR-9 と同じ確かめ方)。端末にも描けて落ちない。
fn fits(a: &mut App, w: u16, h: u16) {
    a.resize(w, h);
    for l in super::view::render(a, (w - 1) as usize, (h - 1) as usize) {
        let lw: usize = l.spans.iter().map(|s| width(&s.content)).sum();
        assert!(lw <= (w - 1) as usize, "幅 {w}・高さ {h} で {lw}: {l:?}");
    }
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
}

/// ヘッダーの画面の型のメニューを開いて「ボード」を選ぶ(クリック)。
fn pick_board_in_menu(a: &mut App) {
    let s = scr(a);
    let (x, y) = s
        .lines()
        .enumerate()
        .find_map(|(y, l)| l.find("▦ 表 ▾").map(|i| (width(&l[..i]) as u16, y as u16)))
        .unwrap_or_else(|| panic!("ヘッダーのメニュー:\n{s}"));
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::ScreenMenu, "{s}");
    let s = scr(a);
    let (x, y) = s
        .lines()
        .enumerate()
        .skip(y as usize + 1)
        .find_map(|(yy, l)| l.find("ボード").map(|i| (width(&l[..i]) as u16, yy as u16)))
        .unwrap_or_else(|| panic!("窓に「ボード」:\n{s}"));
    a.click(x + 1, y);
}

// ---------------------------------------------------------------- BD-1

#[test]
fn test_bd_1_key_switches_and_shows_lanes_with_cards() {
    // [BD-1] status の列にカーソルを置いて B → todo(2)・doing(1)・done(1) のレーンに4枚のカード。
    let (_t, mut a) = boot("bd1key", TASKS);
    col_named(&mut a, "status");
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Board, "{:?}", a.message);
    let s = scr(&a);
    assert_eq!(count(&a, "todo"), Some(2), "{s}");
    assert_eq!(count(&a, "doing"), Some(1), "{s}");
    assert_eq!(count(&a, "done"), Some(1), "{s}");
    for n in NAMES {
        assert!(find(&s, n).is_some(), "カード {n}:\n{s}");
    }
    // 画面の型のボタンは「ボード」。
    assert!(s.contains("ボード ▾"), "{s}");
    assert!(!s.contains("▦ 表 ▾"), "{s}");
    // ちょうど1枚のカードが選ばれている。
    assert_eq!(selected_of(&a, NAMES).len(), 1, "{s}");
}

#[test]
fn test_bd_1_action_board_view_switches() {
    // [BD-1] 動作 `board_view`(Action::BoardView)でも切り替わり、もう一度で表に戻る。
    let (_t, mut a) = boot("bd1action", TASKS);
    col_named(&mut a, "status");
    a.apply(Action::BoardView);
    assert_eq!(a.mode, Mode::Board, "{:?}", a.message);
    a.apply(Action::BoardView);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_bd_1_esc_q_and_same_key_return_to_table() {
    // [BD-1] ボードの画面で Esc・q・B → 表の画面に戻る。ためた変更は無い。
    for (i, key) in [KeyCode::Esc, KeyCode::Char('q'), KeyCode::Char('B')]
        .into_iter()
        .enumerate()
    {
        let (_t, mut a) = boot(&format!("bd1back{i}"), TASKS);
        open_board(&mut a);
        press(&mut a, key);
        assert_eq!(a.mode, Mode::Table, "{key:?}");
        let s = scr(&a);
        assert!(s.contains("▦ 表 ▾"), "{key:?}\n{s}");
        // 表の行の「todo 2026-11-18」は見出しでない(レーンの見出しなら数は 2)。
        assert!(
            head(&s, "todo").is_none_or(|h| h.count != 2),
            "{key:?}\n{s}"
        );
        assert_eq!(a.changes.count(), 0);
        // 戻ったあと、もう一度 B で開ける。
        ch(&mut a, 'B');
        assert_eq!(a.mode, Mode::Board, "{key:?}");
    }
}

#[test]
fn test_bd_1_menu_switches_to_board() {
    // [BD-1][REL-8] 「▦ 表 ▾」の窓に「ボード」。選ぶ → ボードの画面。
    let (_t, mut a) = boot("bd1menu", TASKS);
    col_named(&mut a, "status");
    pick_board_in_menu(&mut a);
    assert_eq!(a.mode, Mode::Board, "{:?}\n{}", a.message, scr(&a));
    assert_eq!(count(&a, "todo"), Some(2), "{}", scr(&a));
}

#[test]
fn test_bd_1_menu_on_number_column_stays_with_reason() {
    // [BD-1][REL-8] 数の列を選んでいても窓に「ボード」は出る。選ぶ → 理由を出して表のまま。
    let (_t, mut a) = boot("bd1menunum", TASKS);
    col_named(&mut a, "estimate");
    pick_board_in_menu(&mut a);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(msg(&a).starts_with(REASON_JA), "{:?}", a.message);
}

#[test]
fn test_bd_1_palette_switches_to_board() {
    // [BD-1] パレットで `board_view` → ボードの画面。
    let (_t, mut a) = boot("bd1palette", TASKS);
    col_named(&mut a, "status");
    ch(&mut a, ':');
    assert_eq!(a.mode, Mode::Palette);
    typing(&mut a, "board_view");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Board, "{:?}\n{}", a.message, scr(&a));
    assert_eq!(count(&a, "doing"), Some(1), "{}", scr(&a));
}

#[test]
fn test_bd_1_non_text_columns_stay_with_reason() {
    // [BD-1] 数(estimate)・日付(due)・リスト(tags)・チェックボックス(flag)の列で B → 切り替わらず理由。
    let (_t, mut a) = boot("bd1types", TASKS);
    for col in ["estimate", "due", "tags", "flag"] {
        a.message = None;
        col_named(&mut a, col);
        ch(&mut a, 'B');
        assert_eq!(a.mode, Mode::Table, "{col}");
        assert!(msg(&a).starts_with(REASON_JA), "{col}: {:?}", a.message);
        let s = scr(&a);
        assert!(s.contains(REASON_JA), "{col}:\n{s}");
        assert!(s.contains("▦ 表 ▾"), "{col}:\n{s}");
        assert_eq!(a.changes.count(), 0);
    }
    // テキストの列(owner)では切り替わる(status という名前に限らない)。
    col_named(&mut a, "owner");
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Board, "{:?}", a.message);
    assert_eq!(count(&a, "mika"), Some(1), "{}", scr(&a));
}

#[test]
fn test_bd_1_file_and_formula_columns_stay_with_reason() {
    // [BD-1] `file.*` と計算の列では切り替わらず理由。同じビューのテキストの列では切り替わる。
    let (_t, mut a) = boot_base(
        "bd1calc",
        TASKS,
        "formulas:\n  label: 'owner + \"!\"'\nviews:\n  - type: table\n    name: 全部\n    order: [file.name, formula.label, status, owner]\n",
    );
    for col in ["file.name", "formula.label"] {
        a.message = None;
        col_named(&mut a, col);
        ch(&mut a, 'B');
        assert_eq!(a.mode, Mode::Table, "{col}");
        assert!(msg(&a).starts_with(REASON_JA), "{col}: {:?}", a.message);
    }
    open_board(&mut a);
    assert_eq!(count(&a, "todo"), Some(2), "{}", scr(&a));
}

#[test]
fn test_bd_1_only_rows_through_view() {
    // [BD-1] status = done の行を絞り込みで外したビュー → done のカードもレーンも出ない。
    let (_t, mut a) = boot_base(
        "bd1view",
        TASKS,
        "views:\n  - type: table\n    name: 残り\n    filters:\n      and:\n        - status != \"done\"\n    order:\n      - status\n      - due\n      - owner\n",
    );
    open_board(&mut a);
    let s = scr(&a);
    assert!(find(&s, "ship-docs").is_none(), "{s}");
    assert!(head(&s, "done").is_none(), "{s}");
    assert_eq!(count(&a, "todo"), Some(2), "{s}");
    assert_eq!(count(&a, "doing"), Some(1), "{s}");
    for n in ["write-spec", "fix-login", "draft-api"] {
        assert!(find(&s, n).is_some(), "カード {n}:\n{s}");
    }
}

#[test]
fn test_bd_1_english_words() {
    // [BD-1][BD-2] 英語: 理由は「this column cannot make a board」で始まり、画面の型は「Board」、空のレーンは「(none)」。
    let _g = scoped(Lang::En);
    let mut notes = TASKS.to_vec();
    notes.push(("no-status.md", "---\nowner: kai\nestimate: 1\n---\n"));
    let (_t, mut a) = boot("bd1en", &notes);
    col_named(&mut a, "estimate");
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Table);
    assert!(msg(&a).starts_with(REASON_EN), "{:?}", a.message);
    open_board(&mut a);
    let s = scr(&a);
    assert!(s.contains("Board"), "{s}");
    assert_eq!(count(&a, "(none)"), Some(1), "{s}");
    assert!(!s.contains("(なし)") && !s.contains("ボード"), "{s}");
}

// ---------------------------------------------------------------- BD-2

/// BD-2 の例の行: status が done・todo・review・空。
const MIXED: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: done\nowner: mika\n---\n"),
    ("bravo.md", "---\nstatus: todo\nowner: ren\n---\n"),
    ("charlie.md", "---\nstatus: review\nowner: sora\n---\n"),
    ("delta.md", "---\nstatus:\nowner: yui\n---\n"),
];

const MIXED_LANES: &[&str] = &["todo", "doing", "done", "review", "(なし)"];

#[test]
fn test_bd_2_rule_values_order_with_empty_lane_and_none_last() {
    // [BD-2] 決まり todo・doing・done と、done・todo・review・空 の4行 →
    // todo(1)・doing(0)・done(1)・review(1)・(なし)(1) の順。見出しは「値 数」。
    let (_t, mut a) = boot_with("bd2rules", MIXED, RULES, false);
    open_board(&mut a);
    let s = scr(&a);
    assert_eq!(lanes_shown(&a, MIXED_LANES), MIXED_LANES, "{s}");
    for (v, n) in [
        ("todo", 1),
        ("doing", 0),
        ("done", 1),
        ("review", 1),
        ("(なし)", 1),
    ] {
        assert_eq!(count(&a, v), Some(n), "{v}:\n{s}");
    }
    // 見出しは同じ行に横に並ぶ。
    let ys: Vec<u16> = MIXED_LANES.iter().map(|v| head(&s, v).unwrap().y).collect();
    assert!(ys.windows(2).all(|w| w[0] == w[1]), "{ys:?}\n{s}");
}

#[test]
fn test_bd_2_no_rule_text_order_and_no_none_lane() {
    // [BD-2] 決まりが無い → doing・done・todo の文字の順。空の行が無い → 「(なし)」は出ない。
    let (_t, mut a) = boot("bd2plain", TASKS);
    open_board(&mut a);
    let s = scr(&a);
    assert_eq!(
        lanes_shown(&a, &["todo", "doing", "done", "(なし)"]),
        ["doing", "done", "todo"],
        "{s}"
    );
    assert!(!s.contains("(なし)"), "{s}");
}

#[test]
fn test_bd_2_missing_null_and_empty_text_share_none_lane() {
    // [BD-2] キーが無い・null・空の文字 の行は、どれも「(なし)」のレーン(最後)。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("bravo.md", "---\nowner: ren\n---\n"),
        ("charlie.md", "---\nstatus:\nowner: sora\n---\n"),
        ("delta.md", "---\nstatus: \"\"\nowner: yui\n---\n"),
    ];
    let (_t, mut a) = boot("bd2none", notes);
    open_board(&mut a);
    let s = scr(&a);
    assert_eq!(count(&a, "(なし)"), Some(3), "{s}");
    assert_eq!(count(&a, "todo"), Some(1), "{s}");
    assert_eq!(
        lanes_shown(&a, &["(なし)", "todo"]),
        ["todo", "(なし)"],
        "{s}"
    );
    for n in ["alpha", "bravo", "charlie", "delta"] {
        assert!(find(&s, n).is_some(), "カード {n}:\n{s}");
    }
}

#[test]
fn test_bd_2_pending_value_decides_lane() {
    // [BD-2] ためた値があればその値でレーンが決まる: 表で draft-api の status を review に直す(保存しない)→
    // ボードに review(1) が出て、doing のレーンは無い(決まりが無いので、行の無い値のレーンは出ない)。
    let (t, mut a) = boot("bd2pending", TASKS);
    let before = read(&t, "draft-api.md");
    let row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "draft-api.md")
        .unwrap();
    edit_cell(&mut a, row, "status", "review");
    assert_eq!(a.changes.count(), 1);
    open_board(&mut a);
    let s = scr(&a);
    assert_eq!(count(&a, "review"), Some(1), "{s}");
    assert_eq!(count(&a, "doing"), None, "{s}");
    assert_eq!(count(&a, "todo"), Some(2), "{s}");
    // [BD-3] ためた変更のある行のカードに `*`。
    assert!(starred(&a, "draft-api"), "{s}");
    assert_eq!(read(&t, "draft-api.md"), before);
}

// ---------------------------------------------------------------- BD-3

const VIEW_COLS: &str =
    "views:\n  - type: table\n    name: 全部\n    order:\n      - status\n      - due\n      - owner\n      - tags\n";

#[test]
fn test_bd_3_card_name_and_first_two_columns() {
    // [BD-3] ビューの列が status・due・owner・tags → write-spec のカードの1行目に名前(.md なし)、
    // 2行目に due(2026-11-20)と owner(mika)。3つ目の列(tags)の値は出ない。
    let (_t, mut a) = boot_base("bd3card", TASKS, VIEW_COLS);
    open_board(&mut a);
    let s = scr(&a);
    let (_, y) = find(&s, "write-spec").unwrap_or_else(|| panic!("{s}"));
    assert!(!s.contains("write-spec.md"), "{s}");
    let second = s.lines().nth(y as usize + 1).unwrap();
    assert!(
        second.contains("2026-11-20") && second.contains("mika"),
        "2行目: {second:?}\n{s}"
    );
    let first = s.lines().nth(y as usize).unwrap();
    assert!(!first.contains("2026-11-20"), "1行目: {first:?}");
    assert!(!s.contains("urgent"), "tags は出さない:\n{s}");
    // どのカードも同じ形。
    for (n, due, owner) in [
        ("fix-login", "2026-11-18", "ren"),
        ("draft-api", "2026-11-25", "sora"),
        ("ship-docs", "2026-11-10", "yui"),
    ] {
        let (_, y) = find(&s, n).unwrap();
        let second = s.lines().nth(y as usize + 1).unwrap();
        assert!(
            second.contains(due) && second.contains(owner),
            "{n}: {second:?}"
        );
    }
}

#[test]
fn test_bd_3_second_line_skips_grouping_column() {
    // [BD-3] まとめる列がビューの先頭でなくても、2行目は「まとめる列を除いた先頭の2つ」(owner・due)。
    let (_t, mut a) = boot_base(
        "bd3skip",
        TASKS,
        "views:\n  - type: table\n    name: 全部\n    order:\n      - owner\n      - status\n      - due\n      - estimate\n",
    );
    open_board(&mut a);
    let s = scr(&a);
    let (_, y) = find(&s, "write-spec").unwrap_or_else(|| panic!("{s}"));
    let second = s.lines().nth(y as usize + 1).unwrap();
    assert!(
        second.contains("mika") && second.contains("2026-11-20"),
        "2行目: {second:?}\n{s}"
    );
}

#[test]
fn test_bd_3_pending_mark_on_cards_with_changes() {
    // [BD-3][SR-15] ためた変更のある行のカードにだけ `*`(まとめる列でない列の変更でも)。
    let (_t, mut a) = boot("bd3star", TASKS);
    let row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "fix-login.md")
        .unwrap();
    edit_cell(&mut a, row, "owner", "nao");
    open_board(&mut a);
    assert!(starred(&a, "fix-login"), "{}", scr(&a));
    // ほかの行のカードの1行目に `*` は無い(同じ行に fix-login が並ぶ所は見ない)。
    let s = scr(&a);
    for n in ["write-spec", "draft-api", "ship-docs"] {
        let l = line_of(&s, n).unwrap();
        if word_in(&l, "fix-login").is_none() {
            assert!(!l.contains('*'), "{n}: {l:?}");
        }
    }
}

#[test]
fn test_bd_3_cards_follow_view_sort() {
    // [BD-3][NV-3] due の昇順のビュー → レーンの中も due の昇順(名前の順ではない)。
    let notes: &[(&str, &str)] = &[
        ("apple.md", "---\nstatus: todo\ndue: 2026-12-03\n---\n"),
        ("banana.md", "---\nstatus: todo\ndue: 2026-12-01\n---\n"),
        ("cherry.md", "---\nstatus: todo\ndue: 2026-12-02\n---\n"),
        ("grape.md", "---\nstatus: done\ndue: 2026-12-09\n---\n"),
        ("lemon.md", "---\nstatus: done\ndue: 2026-12-05\n---\n"),
    ];
    let (_t, mut a) = boot_base(
        "bd3sort",
        notes,
        "views:\n  - type: table\n    name: 期限の順\n    order:\n      - status\n      - due\n    sort:\n      - property: due\n        direction: ASC\n",
    );
    open_board(&mut a);
    let s = scr(&a);
    let y = |n: &str| find(&s, n).unwrap_or_else(|| panic!("{n}:\n{s}")).1;
    assert!(
        y("banana") < y("cherry") && y("cherry") < y("apple"),
        "todo のレーンは due の昇順:\n{s}"
    );
    assert!(y("lemon") < y("grape"), "done のレーンも:\n{s}");
    // 同じレーンのカードは同じ桁に縦に並ぶ。
    let x = |n: &str| find(&s, n).unwrap().0;
    assert!(x("banana").abs_diff(x("apple")) <= 2, "{s}");
}

#[test]
fn test_bd_3_enter_goes_to_table_row() {
    // [BD-3] カードを選んで Enter → 表の画面に戻って、その行が選ばれている。
    for name in ["fix-login", "ship-docs"] {
        let (_t, mut a) = boot(&format!("bd3enter-{name}"), TASKS);
        open_board(&mut a);
        select_card(&mut a, name);
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Table, "{name}");
        assert_eq!(table_row(&a), name);
        assert!(scr(&a).contains("▦ 表 ▾"));
    }
}

#[test]
fn test_bd_3_keys_move_between_lanes_and_cards() {
    // [BD-3] h・l(←・→)でレーン、j・k(↓・↑)でカードを選ぶ。選んでいるカードはいつも1枚。
    let (_t, mut a) = boot_with("bd3keys", TASKS, RULES, false);
    open_board(&mut a);
    // 左の端のレーン(todo)の先頭のカード。
    select_card(&mut a, "fix-login");
    assert_eq!(selected_of(&a, NAMES), ["fix-login"], "{}", scr(&a));
    ch(&mut a, 'j');
    assert_eq!(selected_of(&a, NAMES), ["write-spec"], "{}", scr(&a));
    press(&mut a, KeyCode::Up);
    assert_eq!(selected_of(&a, NAMES), ["fix-login"]);
    press(&mut a, KeyCode::Down);
    assert_eq!(selected_of(&a, NAMES), ["write-spec"]);
    ch(&mut a, 'l');
    assert_eq!(selected_of(&a, NAMES), ["draft-api"], "{}", scr(&a));
    press(&mut a, KeyCode::Right);
    assert_eq!(selected_of(&a, NAMES), ["ship-docs"]);
    press(&mut a, KeyCode::Left);
    assert_eq!(selected_of(&a, NAMES), ["draft-api"]);
    assert_eq!(a.mode, Mode::Board);
    assert_eq!(a.changes.count(), 0, "選ぶだけでは変更をためない");
}

// ---------------------------------------------------------------- BD-4

#[test]
fn test_bd_4_move_right_pends_one_change_and_keeps_file() {
    // [BD-4] todo のカード write-spec で L → doing へ移り、カードに `*`、「未保存 1」、ファイルは変わらない。
    // 移したあとも同じカードを選んだまま。メッセージは「… へ移した」。
    let (t, mut a) = boot_with("bd4right", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "write-spec");
    assert!(scr(&a).contains("未保存 0"), "{}", scr(&a));
    ch(&mut a, 'L');
    assert_eq!(a.mode, Mode::Board);
    let s = scr(&a);
    assert_eq!(count(&a, "todo"), Some(1), "{s}");
    assert_eq!(count(&a, "doing"), Some(2), "{s}");
    assert_eq!(count(&a, "done"), Some(1), "{s}");
    assert!(starred(&a, "write-spec"), "{s}");
    assert!(s.contains("未保存 1"), "{s}");
    assert_eq!(a.changes.count(), 1);
    assert_eq!(selected_of(&a, NAMES), ["write-spec"], "{s}");
    let m = msg(&a);
    assert!(m.contains("doing") && m.contains("へ移した"), "{m:?}");
    // doing のレーンの中に並ぶ(doing の見出しの下、draft-api と同じ桁)。
    let (x, y) = find(&s, "write-spec").unwrap();
    let (dx, _) = find(&s, "draft-api").unwrap();
    assert!(x.abs_diff(dx) <= 2, "{s}");
    assert!(y > head(&s, "doing").unwrap().y, "{s}");
    for (n, text) in TASKS {
        assert_eq!(read(&t, n), *text, "{n} は変わらない");
    }
}

#[test]
fn test_bd_4_card_actions_move_left_and_right() {
    // [BD-4] 動作 `card_right`・`card_left`(Action::CardRight・CardLeft)でも移る。
    let (_t, mut a) = boot_with("bd4action", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "draft-api");
    a.apply(Action::CardRight);
    assert_eq!(count(&a, "done"), Some(2), "{}", scr(&a));
    assert_eq!(count(&a, "doing"), Some(0), "{}", scr(&a));
    a.apply(Action::CardLeft);
    a.apply(Action::CardLeft);
    assert_eq!(count(&a, "todo"), Some(3), "{}", scr(&a));
    assert_eq!(count(&a, "done"), Some(1), "{}", scr(&a));
    assert_eq!(selected_of(&a, NAMES), ["draft-api"]);
}

#[test]
fn test_bd_4_shift_arrows_move_card() {
    // [BD-4] Shift+→ で右、Shift+← で左の隣のレーンへ移す(L・H と同じ)。
    let (_t, mut a) = boot_with("bd4shift", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "fix-login");
    shift(&mut a, KeyCode::Right);
    assert_eq!(count(&a, "todo"), Some(1), "{}", scr(&a));
    assert_eq!(count(&a, "doing"), Some(2), "{}", scr(&a));
    assert!(starred(&a, "fix-login"));
    shift(&mut a, KeyCode::Right);
    assert_eq!(count(&a, "doing"), Some(1), "{}", scr(&a));
    assert_eq!(count(&a, "done"), Some(2), "{}", scr(&a));
    shift(&mut a, KeyCode::Left);
    assert_eq!(count(&a, "doing"), Some(2), "{}", scr(&a));
    assert_eq!(count(&a, "done"), Some(1), "{}", scr(&a));
    assert_eq!(selected_of(&a, NAMES), ["fix-login"]);
    // H でも左へ。
    select_card(&mut a, "ship-docs");
    ch(&mut a, 'H');
    assert_eq!(count(&a, "doing"), Some(3), "{}", scr(&a));
    assert_eq!(count(&a, "done"), Some(0), "{}", scr(&a));
}

#[test]
fn test_bd_4_edges_do_not_move() {
    // [BD-4] 左の端のレーンで H、右の端のレーンで L → 動かず、変更もためない。
    let (t, mut a) = boot_with("bd4edge", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "write-spec");
    ch(&mut a, 'H');
    shift(&mut a, KeyCode::Left);
    assert_eq!(a.mode, Mode::Board);
    assert_eq!(a.changes.count(), 0);
    select_card(&mut a, "ship-docs");
    ch(&mut a, 'L');
    shift(&mut a, KeyCode::Right);
    assert_eq!(a.changes.count(), 0);
    let s = scr(&a);
    assert_eq!(
        (count(&a, "todo"), count(&a, "doing"), count(&a, "done")),
        (Some(2), Some(1), Some(1)),
        "{s}"
    );
    assert!(s.contains("未保存 0"), "{s}");
    assert_eq!(selected_of(&a, NAMES), ["ship-docs"], "{s}");
    for (n, text) in TASKS {
        assert_eq!(read(&t, n), *text);
    }
}

#[test]
fn test_bd_4_moves_into_rule_lane_without_rows() {
    // [BD-4][BD-2] 決まりにあって行の無いレーン(doing 0)へも移せる。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("bravo.md", "---\nstatus: done\nowner: ren\n---\n"),
    ];
    let (_t, mut a) = boot_with("bd4empty", notes, RULES, false);
    open_board(&mut a);
    assert_eq!(count(&a, "doing"), Some(0), "{}", scr(&a));
    select_card(&mut a, "alpha");
    ch(&mut a, 'L');
    let s = scr(&a);
    assert_eq!(count(&a, "todo"), Some(0), "{s}");
    assert_eq!(count(&a, "doing"), Some(1), "{s}");
    assert_eq!(selected_of(&a, &["alpha", "bravo"]), ["alpha"], "{s}");
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_bd_4_undo_and_redo_in_board() {
    // [BD-4][WB-10] 移して u → todo に戻って「未保存 0」。やり直し(Ctrl+r・U)→ また doing。Ctrl+z でも取り消し。
    let (_t, mut a) = boot_with("bd4undo", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "write-spec");
    ch(&mut a, 'L');
    assert_eq!(count(&a, "doing"), Some(2));
    let moved = |a: &App, yes: bool| {
        let s = scr(a);
        let want = if yes { (1, 2, 1) } else { (2, 1, 0) };
        assert_eq!(count(a, "todo"), Some(want.0), "{s}");
        assert_eq!(count(a, "doing"), Some(want.1), "{s}");
        assert!(s.contains(&format!("未保存 {}", want.2)), "{s}");
        assert_eq!(a.changes.count(), want.2);
        assert_eq!(starred(a, "write-spec"), yes, "{s}");
        assert_eq!(a.mode, Mode::Board);
    };
    ch(&mut a, 'u');
    moved(&a, false);
    ctrl(&mut a, 'r');
    moved(&a, true);
    ctrl(&mut a, 'z');
    moved(&a, false);
    ch(&mut a, 'U');
    moved(&a, true);
    // 2つ隣まで移すと2手。1手ずつ戻る。
    select_card(&mut a, "write-spec");
    ch(&mut a, 'L');
    assert_eq!(count(&a, "done"), Some(2), "{}", scr(&a));
    ch(&mut a, 'u');
    moved(&a, true);
    ch(&mut a, 'u');
    moved(&a, false);
}

#[test]
fn test_bd_4_save_opens_review_from_table_and_writes() {
    // [BD-4][WB-9] Ctrl+S → 表の画面に戻って保存の確認(差分 `-status: todo`・`+status: doing`)。
    // 確定までは書かない。確定で書かれて「未保存 0」。
    let (t, mut a) = boot_with("bd4save", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "write-spec");
    ch(&mut a, 'L');
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    let s = scr(&a);
    assert!(
        s.contains("-status: todo") && s.contains("+status: doing"),
        "{s}"
    );
    assert!(s.contains("write-spec.md"), "{s}");
    assert_eq!(read(&t, "write-spec.md"), TASKS[0].1, "確定までは書かない");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(
        read(&t, "write-spec.md"),
        TASKS[0].1.replace("status: todo", "status: doing")
    );
    for (n, text) in &TASKS[1..] {
        assert_eq!(read(&t, n), *text, "{n} は変わらない");
    }
    assert_eq!(a.changes.count(), 0);
    assert!(scr(&a).contains("未保存 0"), "{}", scr(&a));
    // 保存のあとのボードは、書いた値のレーン。
    open_board(&mut a);
    assert_eq!(count(&a, "doing"), Some(2), "{}", scr(&a));
    assert!(!starred(&a, "write-spec"));
}

#[test]
fn test_bd_4_save_review_back_keeps_changes() {
    // [BD-4][WB-9] 保存の確認をやめる(Esc)→ 書かず、ためた変更は残る。
    let (t, mut a) = boot_with("bd4back", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "write-spec");
    ch(&mut a, 'L');
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Esc);
    assert_ne!(a.mode, Mode::Confirm);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "write-spec.md"), TASKS[0].1);
}

#[test]
fn test_bd_4_move_to_none_lane_empties_value() {
    // [BD-4][CE-9] レーン「(なし)」へ移す → 値を空にする変更。保存で `status:`(キーは残す)。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("bravo.md", "---\nstatus: done\nowner: ren\n---\n"),
        ("charlie.md", "---\nowner: sora\n---\n"),
    ];
    let (t, mut a) = boot_with("bd4none", notes, RULES, false);
    open_board(&mut a);
    let names = ["alpha", "bravo", "charlie"];
    assert_eq!(count(&a, "(なし)"), Some(1), "{}", scr(&a));
    select_card(&mut a, "bravo");
    ch(&mut a, 'L');
    let s = scr(&a);
    assert_eq!(count(&a, "done"), Some(0), "{s}");
    assert_eq!(count(&a, "(なし)"), Some(2), "{s}");
    assert_eq!(selected_of(&a, &names), ["bravo"], "{s}");
    assert!(starred(&a, "bravo"), "{s}");
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "bravo.md"), notes[1].1);
    // 「(なし)」は右の端: もう一度 L では動かない。
    ch(&mut a, 'L');
    assert_eq!(a.changes.count(), 1);
    assert_eq!(count(&a, "(なし)"), Some(2));
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    let s = scr(&a);
    assert!(s.contains("-status: done") && s.contains("+status:"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "bravo.md"), "---\nstatus:\nowner: ren\n---\n");
    assert_eq!(read(&t, "charlie.md"), notes[2].1);
}

#[test]
fn test_bd_4_move_out_of_none_lane_sets_value() {
    // [BD-4] 「(なし)」のカードを H で左の隣(done)へ → その値にする変更。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("charlie.md", "---\nstatus:\nowner: sora\n---\n"),
    ];
    let (t, mut a) = boot_with("bd4fromnone", notes, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "charlie");
    ch(&mut a, 'H');
    let s = scr(&a);
    assert_eq!(count(&a, "done"), Some(1), "{s}");
    assert_eq!(a.changes.count(), 1);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        read(&t, "charlie.md"),
        "---\nstatus: done\nowner: sora\n---\n"
    );
}

#[test]
fn test_bd_4_read_only_cell_stays_with_reason() {
    // [BD-4][CE-8] 読むだけのセル(タグつきの値)の行のカードは、移さずに理由を出す。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("bravo.md", "---\nstatus: doing\nowner: ren\n---\n"),
        ("locked.md", "---\nstatus: !!str doing\nowner: sora\n---\n"),
        ("zulu.md", "---\nstatus: done\nowner: yui\n---\n"),
    ];
    let (t, mut a) = boot_with("bd4lock", notes, RULES, false);
    open_board(&mut a);
    let names = ["alpha", "bravo", "locked", "zulu"];
    let before = scr(&a);
    let counts = |a: &App| {
        ["todo", "doing", "done"]
            .iter()
            .map(|v| count(a, v))
            .collect::<Vec<_>>()
    };
    let c0 = counts(&a);
    select_card(&mut a, "locked");
    let mut reasons = Vec::new();
    for key in ['L', 'H'] {
        a.message = None;
        ch(&mut a, key);
        reasons.push(msg(&a));
        assert_eq!(a.mode, Mode::Board, "{key}");
        assert_eq!(a.changes.count(), 0, "{key}");
        assert_eq!(counts(&a), c0, "{key}:\n{}", scr(&a));
        assert_eq!(selected_of(&a, &names), ["locked"], "{key}");
        assert!(!starred(&a, "locked"), "{key}");
    }
    assert!(
        reasons
            .iter()
            .any(|m| !m.is_empty() && !m.contains("へ移した")),
        "理由が出る: {reasons:?}\n{before}"
    );
    assert!(
        reasons
            .iter()
            .any(|m| m.contains("読むだけ") || m.contains("タグ")),
        "読むだけの理由: {reasons:?}"
    );
    assert_eq!(read(&t, "locked.md"), notes[2].1);
}

#[test]
fn test_bd_4_unreadable_frontmatter_card_does_not_move() {
    // [BD-4][WB-5] 読めないフロントマター(閉じが無い)のノートのカードで L・H → 移らず理由。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("broken.md", "---\nstatus: doing\nowner: ren\n本文\n"),
        ("zulu.md", "---\nstatus: done\nowner: yui\n---\n"),
    ];
    let (t, mut a) = boot_with("bd4broken", notes, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "broken");
    let mut reasons = Vec::new();
    for key in ['L', 'H'] {
        a.message = None;
        ch(&mut a, key);
        reasons.push(msg(&a));
        assert_eq!(a.changes.count(), 0, "{key}");
        assert!(!starred(&a, "broken"), "{key}");
    }
    assert!(
        reasons
            .iter()
            .any(|m| !m.is_empty() && !m.contains("へ移した")),
        "理由が出る: {reasons:?}"
    );
    assert_eq!(read(&t, "broken.md"), notes[1].1);
}

#[test]
fn test_bd_4_readonly_start_does_not_move() {
    // [BD-4][WB-15] 読むだけの起動: ボードは見られるが、L・H・Shift+矢印でカードは移らない。
    let (t, mut a) = boot_with("bd4ro", TASKS, RULES, true);
    open_board(&mut a);
    select_card(&mut a, "draft-api");
    ch(&mut a, 'L');
    ch(&mut a, 'H');
    shift(&mut a, KeyCode::Right);
    shift(&mut a, KeyCode::Left);
    a.apply(Action::CardRight);
    a.apply(Action::CardLeft);
    assert_eq!(a.changes.count(), 0);
    let s = scr(&a);
    assert_eq!(
        (count(&a, "todo"), count(&a, "doing"), count(&a, "done")),
        (Some(2), Some(1), Some(1)),
        "{s}"
    );
    assert!(!s.contains("未保存 1"), "{s}");
    for n in NAMES {
        assert!(!starred(&a, n), "{n}:\n{s}");
    }
    for (n, text) in TASKS {
        assert_eq!(read(&t, n), *text);
    }
}

#[test]
fn test_bd_4_same_card_stays_selected_across_moves() {
    // [BD-4] 続けて移しても同じカードを選んだまま(移した先のレーンで j・k が効く)。
    let (_t, mut a) = boot_with("bd4keep", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "fix-login");
    ch(&mut a, 'L');
    assert_eq!(selected_of(&a, NAMES), ["fix-login"], "{}", scr(&a));
    ch(&mut a, 'L');
    assert_eq!(selected_of(&a, NAMES), ["fix-login"], "{}", scr(&a));
    assert_eq!(count(&a, "done"), Some(2));
    // Enter → 表のその行(移したカードのまま)。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(table_row(&a), "fix-login");
    // 2回移しても、ためた変更は同じセルの1つ(値は done)。
    assert_eq!(a.changes.count(), 1);
}

// ---------------------------------------------------------------- BD-5

const FIVE: &[&str] = &["aaa", "bbb", "ccc", "ddd", "eee"];
const FIVE_CARDS: &[&str] = &["note-1", "note-2", "note-3", "note-4", "note-5"];

/// レーンが5つ(値 aaa〜eee。カードは1枚ずつ note-1〜note-5)。
fn five() -> Vec<(String, String)> {
    FIVE.iter()
        .zip(FIVE_CARDS)
        .map(|(v, n)| {
            (
                format!("{n}.md"),
                format!("---\nstatus: {v}\nowner: owner-{v}\n---\n"),
            )
        })
        .collect()
}

fn refs(v: &[(String, String)]) -> Vec<(&str, &str)> {
    v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

#[test]
fn test_bd_5_width_100_shows_all_five_lanes() {
    // [BD-5] レーンが5つで幅 100 → 5つとも横に並ぶ。隣の見出しとの間は 18 桁以上で、隠れたレーンの印は無い。
    let notes = five();
    let (_t, mut a) = boot("bd5wide", &refs(&notes));
    open_board(&mut a);
    let s = scr(&a);
    assert_eq!(lanes_shown(&a, FIVE), FIVE, "{s}");
    let xs: Vec<u16> = FIVE.iter().map(|v| head(&s, v).unwrap().x).collect();
    assert!(xs.windows(2).all(|w| w[1] - w[0] >= 18), "{xs:?}\n{s}");
    let d = xs[1] - xs[0];
    assert!(
        xs.windows(2).all(|w| (w[1] - w[0]).abs_diff(d) <= 1),
        "同じ幅で分ける: {xs:?}\n{s}"
    );
    let row = head_row(&a, "aaa");
    assert!(!row.starts_with('<') && !row.ends_with('>'), "{row:?}");
    for n in FIVE_CARDS {
        assert!(find(&s, n).is_some(), "カード {n}:\n{s}");
    }
}

#[test]
fn test_bd_5_width_40_shows_two_lanes_and_scrolls_to_selected() {
    // [BD-5] 幅 40 → 2つだけ出る。l で右へ動くと選んだレーンが見える所まで送られ、左に隠れたレーンの印 `<`。
    let notes = five();
    let (_t, mut a) = boot("bd5narrow", &refs(&notes));
    open_board(&mut a);
    a.resize(40, 24);
    press_until(&mut a, 'h', "note-1");
    let s = scr(&a);
    assert_eq!(lanes_shown(&a, FIVE), ["aaa", "bbb"], "{s}");
    assert_eq!(selected_of(&a, FIVE_CARDS), ["note-1"], "{s}");
    let row = head_row(&a, "aaa");
    assert!(!row.starts_with('<'), "左に隠れたレーンは無い: {row:?}");
    assert!(row.ends_with('>'), "右に隠れたレーンの印: {row:?}\n{s}");
    // 右へ1つずつ: 選んだレーンとそのカードがいつも見えて、出るのは2つだけ。
    for (i, (v, n)) in FIVE.iter().zip(FIVE_CARDS).enumerate().skip(1) {
        ch(&mut a, 'l');
        let s = scr(&a);
        let shown = lanes_shown(&a, FIVE);
        assert_eq!(shown.len(), 2, "{v}:\n{s}");
        assert!(
            shown.iter().any(|x| x == v),
            "選んだレーン {v} が見える:\n{s}"
        );
        assert_eq!(selected_of(&a, FIVE_CARDS), [n.to_string()], "{s}");
        if i >= 2 {
            let row = head_row(&a, v);
            assert!(row.starts_with('<'), "左に隠れたレーンの印: {row:?}\n{s}");
        }
    }
    // 右の端まで来たら、右に隠れたレーンの印は無い。
    let row = head_row(&a, "eee");
    assert!(row.starts_with('<') && !row.ends_with('>'), "{row:?}");
    assert_eq!(lanes_shown(&a, FIVE), ["ddd", "eee"], "{}", scr(&a));
    // 左へ戻ると、また左のレーンが見える。
    for _ in 0..4 {
        ch(&mut a, 'h');
    }
    let shown = lanes_shown(&a, FIVE);
    assert!(shown.iter().any(|x| x == "aaa"), "{}", scr(&a));
    assert!(!head_row(&a, "aaa").starts_with('<'));
}

/// 1つのレーン(todo)にカード 30 枚(card-01〜card-30)と、隣のレーン(done)に1枚。
fn tall() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = (1..=30)
        .map(|i| {
            (
                format!("card-{i:02}.md"),
                format!("---\nstatus: todo\nowner: owner-{i:02}\n---\n"),
            )
        })
        .collect();
    v.push((
        "zz-last.md".to_string(),
        "---\nstatus: done\nowner: last\n---\n".to_string(),
    ));
    v
}

#[test]
fn test_bd_5_tall_lane_scrolls_to_selected_card() {
    // [BD-5] 1つのレーンにカード 30 枚・高さ 24 → j で下へ動くと選んだカードが見える。
    // 隠れたカードの印は、そのレーンの見出しの数のあとの `^`(上)・`v`(下)。
    let notes = tall();
    let (_t, mut a) = boot_with("bd5tall", &refs(&notes), RULES, false);
    a.resize(100, 24);
    open_board(&mut a);
    select_card(&mut a, "card-01");
    let s = scr(&a);
    assert_eq!(count(&a, "todo"), Some(30), "{s}");
    assert!(selected(&s, "card-01"), "{s}");
    assert!(find(&s, "card-30").is_none(), "入りきらない:\n{s}");
    let t = head(&s, "todo").unwrap().tail;
    assert!(
        t.contains('v') && !t.contains('^'),
        "下に隠れた印: {t:?}\n{s}"
    );
    // 隣のレーン(1枚)には印が無い。
    let t = head(&s, "done").unwrap().tail;
    assert!(!t.contains('v') && !t.contains('^'), "{t:?}\n{s}");
    for i in 2..=30 {
        ch(&mut a, 'j');
        let s = scr(&a);
        assert!(selected(&s, &format!("card-{i:02}")), "card-{i:02}:\n{s}");
        assert!(head(&s, "todo").is_some(), "見出しは残る:\n{s}");
    }
    let s = scr(&a);
    assert!(find(&s, "card-01").is_none(), "{s}");
    let t = head(&s, "todo").unwrap().tail;
    assert!(
        t.contains('^') && !t.contains('v'),
        "上に隠れた印: {t:?}\n{s}"
    );
    // 真ん中では上下の両方。
    for _ in 0..15 {
        ch(&mut a, 'k');
    }
    let s = scr(&a);
    assert!(selected(&s, "card-15"), "{s}");
    let t = head(&s, "todo").unwrap().tail;
    assert!(t.contains('^') && t.contains('v'), "{t:?}\n{s}");
}

#[test]
fn test_bd_5_lines_fit_any_width_and_height() {
    // [BD-5] どの幅と高さでも、どの行も画面の幅を超えない(レーンが多い・カードが多い・移したあとも)。
    let mut notes = five();
    notes.extend(tall());
    let (_t, mut a) = boot("bd5fit", &refs(&notes));
    open_board(&mut a);
    for w in [160u16, 100, 60, 40, 37, 20, 9, 3] {
        for h in [30u16, 24, 12, 6] {
            fits(&mut a, w, h);
            assert_eq!(a.mode, Mode::Board);
        }
    }
    // 動かしながらでも同じ。
    a.resize(40, 12);
    for key in ['l', 'j', 'j', 'l', 'L', 'j', 'h', 'H', 'k'] {
        ch(&mut a, key);
        fits(&mut a, 40, 12);
        fits(&mut a, 57, 9);
    }
}

#[test]
fn test_bd_5_long_text_stays_in_its_lane() {
    // [BD-5] 長い名前と長い値は、レーンの幅で切れて、隣のレーンにはみ出さない(隣のカードが隠れない)。
    let long_name = format!("long-{}", "n".repeat(70));
    let long_owner = "w".repeat(90);
    let notes = vec![
        (
            format!("{long_name}.md"),
            format!("---\nstatus: aaa\nowner: {long_owner}\n---\n"),
        ),
        (
            "short.md".to_string(),
            "---\nstatus: bbb\nowner: bob\n---\n".to_string(),
        ),
    ];
    let (_t, mut a) = boot("bd5long", &refs(&notes));
    open_board(&mut a);
    for w in [100u16, 40] {
        a.resize(w, 24);
        let s = scr(&a);
        let l = s
            .lines()
            .find(|l| l.contains("long-n"))
            .unwrap_or_else(|| panic!("幅 {w}:\n{s}"));
        assert!(!l.contains(&long_name), "幅 {w} で切れる: {l:?}");
        assert!(
            word_in(l, "short").is_some(),
            "幅 {w}: 隣のレーンの先頭のカードが同じ行に見える: {l:?}\n{s}"
        );
        let (_, y) = find(&s, "short").unwrap();
        let second = s.lines().nth(y as usize + 1).unwrap();
        assert!(word_in(second, "bob").is_some(), "幅 {w}: {second:?}\n{s}");
        assert!(!second.contains(&long_owner), "幅 {w}: {second:?}");
        // 隣のレーンの見出しより右に、長い字の続きが出ない。
        let bx = head(&s, "bbb").unwrap().x as usize;
        for line in [l, second] {
            let mut x = 0;
            for c in line.chars() {
                if x >= bx {
                    assert!(c != 'n' && c != 'w', "幅 {w}: はみ出し: {line:?}");
                }
                x += width(&c.to_string()).max(1);
            }
        }
        fits(&mut a, w, 24);
    }
}

// ---------------------------------------------------------------- BD-6

#[test]
fn test_bd_6_click_card_selects_then_goes_to_table_row() {
    // [BD-6] doing のレーンのカードをクリック → そのカードが選ばれる。もう一度クリック → 表の画面でその行。
    let (_t, mut a) = boot_with("bd6card", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "fix-login");
    let s = scr(&a);
    let (x, y) = find(&s, "draft-api").unwrap();
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Board, "1回目は選ぶだけ");
    assert_eq!(selected_of(&a, NAMES), ["draft-api"], "{}", scr(&a));
    assert_eq!(a.changes.count(), 0);
    let s = scr(&a);
    let (x, y) = find(&s, "draft-api").unwrap();
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(table_row(&a), "draft-api");
}

#[test]
fn test_bd_6_click_second_line_and_other_card_selects() {
    // [BD-6] カードの2行目のクリックでも選ぶ。別のカードのクリックは、選び直すだけ(表へ移らない)。
    let (_t, mut a) = boot_with("bd6other", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "ship-docs");
    let s = scr(&a);
    let (x, y) = find(&s, "write-spec").unwrap();
    a.click(x + 1, y + 1);
    assert_eq!(a.mode, Mode::Board);
    assert_eq!(selected_of(&a, NAMES), ["write-spec"], "{}", scr(&a));
    let s = scr(&a);
    let (x, y) = find(&s, "fix-login").unwrap();
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Board, "別のカードは選ぶだけ");
    assert_eq!(selected_of(&a, NAMES), ["fix-login"], "{}", scr(&a));
}

#[test]
fn test_bd_6_click_lane_head_selects_lane() {
    // [BD-6] done の見出しをクリック → done のレーンが選ばれる(そのレーンのカードが選ばれ、L は右の端で動かない)。
    let (_t, mut a) = boot_with("bd6head", TASKS, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "fix-login");
    let s = scr(&a);
    let h = head(&s, "done").unwrap();
    a.click(h.x + 1, h.y);
    assert_eq!(a.mode, Mode::Board);
    assert_eq!(selected_of(&a, NAMES), ["ship-docs"], "{}", scr(&a));
    // 行の無いレーン(見出しだけ)もクリックで選べる: そこから l で右の隣のレーンのカード。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nowner: mika\n---\n"),
        ("bravo.md", "---\nstatus: done\nowner: ren\n---\n"),
    ];
    let (_t, mut a) = boot_with("bd6emptyhead", notes, RULES, false);
    open_board(&mut a);
    select_card(&mut a, "alpha");
    let s = scr(&a);
    let h = head(&s, "doing").unwrap();
    a.click(h.x + 1, h.y);
    assert_eq!(a.mode, Mode::Board);
    assert!(
        selected_of(&a, &["alpha", "bravo"]).is_empty(),
        "空のレーンを選ぶ:\n{}",
        scr(&a)
    );
    ch(&mut a, 'l');
    assert_eq!(
        selected_of(&a, &["alpha", "bravo"]),
        ["bravo"],
        "{}",
        scr(&a)
    );
}

#[test]
fn test_bd_6_wheel_selects_cards_in_lane() {
    // [BD-6] ホイールで、選んでいるレーンのカードを上下に選ぶ(レーンは変わらない)。
    let notes = tall();
    let (_t, mut a) = boot_with("bd6wheel", &refs(&notes), RULES, false);
    a.resize(100, 24);
    open_board(&mut a);
    select_card(&mut a, "card-01");
    a.wheel(true);
    assert!(selected(&scr(&a), "card-02"), "{}", scr(&a));
    a.wheel(true);
    assert!(selected(&scr(&a), "card-03"), "{}", scr(&a));
    a.wheel(false);
    assert!(selected(&scr(&a), "card-02"), "{}", scr(&a));
    a.wheel(false);
    assert!(selected(&scr(&a), "card-01"), "{}", scr(&a));
    // 下へ送っても、選んだカードが見える(BD-5)。
    for _ in 0..29 {
        a.wheel(true);
    }
    assert!(selected(&scr(&a), "card-30"), "{}", scr(&a));
    assert_eq!(a.mode, Mode::Board);
    assert_eq!(a.changes.count(), 0);
}
