//! 列の値の頻度表(NV-9)の受け入れの試験。記録: specs/_changes/2026-10-06-frequency-table.md のタスク #1。
//!
//! 実装より先に、実装を見ずに書いた。前提にしたもの(記録の「設計」):
//! モード `Mode::Freq`(設定の名前 `freq`)、動作の名前 `frequency`、表のモードの既定のキー `%`。
//! 窓は操作の一覧(SR-24)と同じ形(表の上に重ねる枠の窓)なので、窓の中身は test_action_menu の道具
//! (開く前と後で変わった四隅の角の矩形の内側)で読む。
//! 試験の環境は日本語に固定されている(.cargo/config.toml)。英語は `scoped(Lang::En)` で確かめる。

use super::keymap::{self, Action, Mode};
use super::test_action_menu::{has_section, label_of, menu_lines, menu_mode, shot, Shot};
use super::test_screen::{ch, col_named, make, press, screen, typing};
use super::*;
use mdgrid::config;
use mdgrid::i18n::{scoped, Lang, Msg};
use ratatui::crossterm::event::KeyCode;

/// tags の列(リストの値): 会議 3・家 2・値の無い行 1(5行)。
const TAGS: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\ntags: [会議, 家]\n---\n"),
    ("b.md", "---\ntitle: b\ntags: [会議]\n---\n"),
    ("c.md", "---\ntitle: c\ntags: [家]\n---\n"),
    ("d.md", "---\ntitle: d\ntags: [会議]\n---\n"),
    ("e.md", "---\ntitle: e\n---\n"),
];

/// 英字だけの tags(英語の画面の試験): meeting 3・home 2・値の無い行 1。
const EN_TAGS: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\ntags: [meeting, home]\n---\n"),
    ("b.md", "---\ntitle: b\ntags: [meeting]\n---\n"),
    ("c.md", "---\ntitle: c\ntags: [home]\n---\n"),
    ("d.md", "---\ntitle: d\ntags: [meeting]\n---\n"),
    ("e.md", "---\ntitle: e\n---\n"),
];

/// status の列(普通の値): done 2・todo 1。値の順(todo が先の名前の順)と件数の順が違う。
const STATUS: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: b\nstatus: done\n---\n"),
    ("c.md", "---\ntitle: c\nstatus: done\n---\n"),
];

const W: u16 = 80;
const H: u16 = 24;

// ---- 道具 ----

/// 頻度表を開く動作(まだ無ければ落ちる)。
fn freq_action() -> Action {
    Action::by_name("frequency").expect("動作 frequency が無い(NV-9)")
}

/// 行の名前の並び。
fn names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// 列 `col` を選んで `key` で頻度表を開く。窓の行(枠の内側)と開いた後の画面。
fn open_with(a: &mut App, col: &str, key: char) -> (Vec<String>, Shot) {
    a.row = 0;
    col_named(a, col);
    let before = shot(a, W, H);
    ch(a, key);
    let after = shot(a, W, H);
    assert_eq!(
        a.mode,
        Mode::Freq,
        "{key} で頻度表が開かない(mode={:?}):\n{after}",
        a.mode
    );
    (menu_lines(&before, &after), after)
}

fn open(a: &mut App, col: &str) -> (Vec<String>, Shot) {
    open_with(a, col, '%')
}

/// 行の語(空白で分け、前後の括弧と枠の文字を除く)。
fn words(line: &str) -> Vec<String> {
    line.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| "()（）[]│┃|".contains(c))
                .to_string()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// 窓の中で、値 `value` を含み、語として件数 `count` を持つ行の位置。
fn item_at(lines: &[String], value: &str, count: usize) -> Option<usize> {
    let n = count.to_string();
    lines
        .iter()
        .position(|l| l.contains(value) && words(l).iter().any(|w| *w == n))
}

/// 窓の中で、値 `value` の行に割合 `pct`(`60%` の形)が添えてある。
fn has_pct(lines: &[String], value: &str, pct: &str) -> bool {
    lines
        .iter()
        .any(|l| l.contains(value) && words(l).iter().any(|w| w == pct))
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

/// 表の上の帯の行(ヘッダー・タブ・検索の欄・設定の帯。0〜3行目)。
fn top(s: &str) -> String {
    s.lines().take(4).collect::<Vec<_>>().join("\n")
}

/// 下の帯より上(メッセージ行を除く)。
fn above_message(s: &str) -> String {
    s.lines().take(22).collect::<Vec<_>>().join("\n")
}

// ---- NV-9: 開く・並び・件数・割合 ----

#[test]
fn test_nv_9_percent_key_is_frequency() {
    // [NV-9] [SR-16] 表のモードの既定 `%` = frequency。
    let (_t, a) = make("nv9key", TAGS);
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "%"),
        Some(freq_action()),
        "表で % が frequency でない"
    );
    assert_eq!(
        Mode::by_name("freq"),
        Some(Mode::Freq),
        "モードの設定の名前が freq でない"
    );
}

#[test]
fn test_nv_9_tags_counts_order_and_percent() {
    // [NV-9] tags の列で `%` → 窓に「会議 3 (60%)」「家 2 (40%)」「(空) 1 (20%)」が多い順。
    // リストの値は要素ごとに数え、値の無い行は「(空)」。割合は全行(5行)に対する。
    let (_t, mut a) = make("nv9tags", TAGS);
    assert_eq!(names(&a).len(), 5);
    let (lines, after) = open(&mut a, "tags");
    assert!(!lines.is_empty(), "窓が無い:\n{after}");
    let meeting =
        item_at(&lines, "会議", 3).unwrap_or_else(|| panic!("「会議 3」が無い:\n{after}"));
    let home = item_at(&lines, "家", 2).unwrap_or_else(|| panic!("「家 2」が無い:\n{after}"));
    let empty = item_at(&lines, "(空)", 1).unwrap_or_else(|| panic!("「(空) 1」が無い:\n{after}"));
    assert!(meeting < home, "「会議」が「家」より上でない:\n{after}");
    assert!(home < empty, "「家」が「(空)」より上でない:\n{after}");
    assert!(
        has_pct(&lines, "会議", "60%"),
        "会議に 60% が無い:\n{after}"
    );
    assert!(has_pct(&lines, "家", "40%"), "家に 40% が無い:\n{after}");
    assert!(
        has_pct(&lines, "(空)", "20%"),
        "(空) に 20% が無い:\n{after}"
    );
}

#[test]
fn test_nv_9_window_keeps_table_outside() {
    // [NV-9] 窓は表の上に重ねる: 窓を開いても表の行(ノートの名前)の一部は見えたまま。
    let (_t, mut a) = make("nv9outside", TAGS);
    let (_, after) = open(&mut a, "tags");
    // SR-29: 左の欄の名前は `.md` を除いたもの。
    let visible = ["a", "b", "c", "d", "e"]
        .iter()
        .filter(|n| {
            after
                .text
                .lines()
                .any(|l| l.get(1..).is_some_and(|r| r.starts_with(&format!("{n} "))))
        })
        .count();
    assert!(visible > 0, "窓の外の表が見えない:\n{after}");
}

#[test]
fn test_nv_9_status_plain_values() {
    // [NV-9] 普通の値の列(status)でも同じ: done 2 が todo 1 より上(件数の多い順)。
    let (_t, mut a) = make("nv9status", STATUS);
    let (lines, after) = open(&mut a, "status");
    let done = item_at(&lines, "done", 2).unwrap_or_else(|| panic!("「done 2」が無い:\n{after}"));
    let todo = item_at(&lines, "todo", 1).unwrap_or_else(|| panic!("「todo 1」が無い:\n{after}"));
    assert!(done < todo, "done が todo より上でない:\n{after}");
    assert!(
        has_pct(&lines, "done", "67%") || has_pct(&lines, "done", "66%"),
        "done に割合(2/3)が無い:\n{after}"
    );
    assert!(
        has_pct(&lines, "todo", "33%"),
        "todo に割合(1/3)が無い:\n{after}"
    );
}

#[test]
fn test_nv_9_same_count_in_value_order() {
    // [NV-9] 同じ件数は値の順: x・y・z が1件ずつ → x, y, z の順。
    let (_t, mut a) = make(
        "nv9tie",
        &[
            ("a.md", "---\ntitle: a\nstatus: zz\n---\n"),
            ("b.md", "---\ntitle: b\nstatus: xx\n---\n"),
            ("c.md", "---\ntitle: c\nstatus: yy\n---\n"),
        ],
    );
    let (lines, after) = open(&mut a, "status");
    let x = item_at(&lines, "xx", 1).unwrap_or_else(|| panic!("xx が無い:\n{after}"));
    let y = item_at(&lines, "yy", 1).unwrap_or_else(|| panic!("yy が無い:\n{after}"));
    let z = item_at(&lines, "zz", 1).unwrap_or_else(|| panic!("zz が無い:\n{after}"));
    assert!(x < y && y < z, "同じ件数が値の順でない:\n{after}");
}

#[test]
fn test_nv_9_counts_rows_after_view_filter() {
    // [NV-9] 数えるのは今のビューの絞り込みのあとの行: `,` で done だけにしてから `%` → done 2(100%)、
    // todo は無い。
    let (_t, mut a) = make("nv9filtered", STATUS);
    a.row = names(&a).iter().position(|n| n == "b.md").unwrap();
    col_named(&mut a, "status");
    ch(&mut a, ',');
    assert_eq!(
        names(&a),
        ["b.md", "c.md"],
        "材料: , で done だけにならない"
    );
    let (lines, after) = open(&mut a, "status");
    assert!(
        item_at(&lines, "done", 2).is_some(),
        "「done 2」が無い:\n{after}"
    );
    assert!(
        has_pct(&lines, "done", "100%"),
        "done が 100% でない:\n{after}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("todo")),
        "絞り込みで外れた todo が数えられている:\n{after}"
    );
}

// ---- NV-9: 選んで絞る ----

#[test]
fn test_nv_9_enter_on_first_filters_to_value() {
    // [NV-9] [NV-16] 窓の最初の項目(会議)で Enter → 窓が閉じて表のモード、tags に会議を持つ行だけになり、
    // 上の設定の帯に条件(会議)が出る。
    let (_t, mut a) = make("nv9enter", TAGS);
    let before_top = top(&screen(&a));
    assert!(
        !before_top.contains("会議"),
        "材料: 開く前から帯に会議:\n{before_top}"
    );
    open(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "Enter で窓が閉じない");
    assert_eq!(
        names(&a),
        ["a.md", "b.md", "d.md"],
        "会議を持つ行だけでない"
    );
    let s = screen(&a);
    assert!(top(&s).contains("会議"), "上の帯に条件(会議)が無い:\n{s}");
    assert_eq!(a.changes.count(), 0, "絞り込みでノートを書いた");
}

#[test]
fn test_nv_9_down_enter_filters_to_second_value() {
    // [NV-9] [NV-16] ↓ で2つ目(家)を選んで Enter → 家を持つ行だけ、上の帯に条件(家)。
    let (_t, mut a) = make("nv9down", TAGS);
    open(&mut a, "tags");
    press(&mut a, KeyCode::Down);
    assert_eq!(a.mode, Mode::Freq, "↓ で窓が閉じた");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "Enter で窓が閉じない");
    assert_eq!(names(&a), ["a.md", "c.md"], "家を持つ行だけでない");
    let s = screen(&a);
    assert!(top(&s).contains("家"), "上の帯に条件(家)が無い:\n{s}");
}

#[test]
fn test_nv_9_jk_move_in_window() {
    // [NV-9] 窓の中は `j` `k` でも動く: `j` 2回・`k` 1回で2つ目(家)→ Enter で家の行だけ。
    let (_t, mut a) = make("nv9jk", TAGS);
    open(&mut a, "tags");
    ch(&mut a, 'j');
    ch(&mut a, 'j');
    ch(&mut a, 'k');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(names(&a), ["a.md", "c.md"], "j k で選んだ値で絞れていない");
}

#[test]
fn test_nv_9_empty_item_filters_rows_without_value() {
    // [NV-9] 「(空)」(3つ目)で Enter → 値の無い行(e.md)だけ。
    let (_t, mut a) = make("nv9empty", TAGS);
    open(&mut a, "tags");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(names(&a), ["e.md"], "値の無い行だけでない");
}

#[test]
fn test_nv_9_status_enter_same_as_comma() {
    // [NV-9] [NV-8] status の頻度表で done(先頭)を Enter → `,` を done のセルで押したのと同じ行と同じ帯。
    let (_t, mut want) = make("nv9comma-want", STATUS);
    want.row = names(&want).iter().position(|n| n == "b.md").unwrap();
    col_named(&mut want, "status");
    ch(&mut want, ',');
    assert_eq!(
        names(&want),
        ["b.md", "c.md"],
        "材料: , で done だけにならない"
    );

    let (_t, mut a) = make("nv9comma", STATUS);
    open(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(names(&a), names(&want), "頻度表の絞り込みが , と違う");
    for app in [&mut a, &mut want] {
        app.row = 0;
        col_named(app, "status");
        app.message = None;
    }
    assert_eq!(
        top(&screen(&a)),
        top(&screen(&want)),
        "上の帯が , の絞り込みと違う"
    );
}

// ---- NV-9: Esc ----

#[test]
fn test_nv_9_esc_closes_without_change() {
    // [NV-9] Esc → 何もせず閉じる: 表のモード、行の数・並び・位置・表の画面が同じ。
    let (_t, mut a) = make("nv9esc", TAGS);
    col_named(&mut a, "tags");
    a.row = 1;
    let rows = names(&a);
    let before = screen(&a);
    ch(&mut a, '%');
    assert_eq!(a.mode, Mode::Freq, "% で頻度表が開かない");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "Esc で窓が閉じない");
    assert_eq!(names(&a), rows, "Esc で行が変わった");
    assert_eq!(a.rows.len(), 5);
    assert_eq!(a.row, 1);
    assert_eq!(a.changes.count(), 0);
    assert_eq!(
        above_message(&screen(&a)),
        above_message(&before),
        "Esc のあと表が変わった"
    );
}

// ---- NV-9・SR-23: 英語の画面 ----

#[test]
fn test_nv_9_english_window_has_no_japanese() {
    // [NV-9] [SR-23] 英語の画面で `%` → 窓の見出しと「(空)」に当たる文字に日本語が無い。
    // 値の無い行の項目(1件・20%)はある。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("nv9en", EN_TAGS);
    let (lines, after) = open(&mut a, "tags");
    assert!(lines.len() >= 3, "窓が出ていない:\n{after}");
    for l in &lines {
        assert!(
            !l.chars().any(is_ja),
            "英語の窓に日本語の文字: {l:?}\n{after}"
        );
    }
    let meeting =
        item_at(&lines, "meeting", 3).unwrap_or_else(|| panic!("meeting 3 が無い:\n{after}"));
    let home = item_at(&lines, "home", 2).unwrap_or_else(|| panic!("home 2 が無い:\n{after}"));
    assert!(meeting < home, "meeting が home より上でない:\n{after}");
    let empty = lines
        .iter()
        .position(|l| {
            !l.contains("meeting")
                && !l.contains("home")
                && words(l).iter().any(|w| w == "1")
                && words(l).iter().any(|w| w == "20%")
        })
        .unwrap_or_else(|| panic!("値の無い行の項目(1・20%)が無い:\n{after}"));
    assert!(home < empty, "空の項目が home より上:\n{after}");
    // 選んで絞ったあとの帯・メッセージも日本語でない。
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(
        !top(&s).chars().any(is_ja),
        "英語の画面で上の帯に日本語:\n{s}"
    );
    if let Some(m) = &a.message {
        assert!(!m.chars().any(is_ja), "英語の画面でメッセージに日本語: {m}");
    }
}

#[test]
fn test_nv_9_english_action_label_not_japanese() {
    // [NV-9] [SR-23] 英語の画面で、frequency の表示名(キーの表・パレット・一覧に出る名前)が日本語でない。
    let _g = scoped(Lang::En);
    let (_t, a) = make("nv9enlabel", EN_TAGS);
    let label = label_of(&a, freq_action());
    assert!(!label.is_empty());
    assert!(!label.chars().any(is_ja), "英語の表示名に日本語: {label:?}");
}

// ---- NV-9・SR-4: 割り当て直し ----

#[test]
fn test_nv_9_rebound_key_opens() {
    // [NV-9] [SR-4] 設定 `[keys.table] "!" = "frequency"` → `!` でも開く。
    let (_t, mut a) = make("nv9rebind", TAGS);
    let (cfg, _) = config::parse("[keys.table]\n\"!\" = \"frequency\"\n").expect("読める設定");
    a.configure(&cfg);
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "!"),
        Some(freq_action()),
        "設定の ! が frequency にならない"
    );
    let (lines, after) = open_with(&mut a, "tags", '!');
    assert!(
        item_at(&lines, "会議", 3).is_some(),
        "「会議 3」が無い:\n{after}"
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- NV-9・SR-14: パレット ----

#[test]
fn test_nv_9_palette_opens() {
    // [NV-9] [SR-14] パレットで frequency を実行 → 頻度表が開く。
    let (_t, mut a) = make("nv9pal", TAGS);
    col_named(&mut a, "tags");
    ch(&mut a, ':');
    typing(&mut a, "frequency");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Freq, "パレットから頻度表が開かない");
}

// ---- NV-9・SR-24: 操作の一覧 ----

/// 一覧の項目(題の行と節の見出しと空の行を除いた行)。
fn menu_items(lines: &[String]) -> Vec<String> {
    let heads = [
        Msg::MenuSecCell.text(),
        Msg::MenuSecColumn.text(),
        Msg::MenuSecRow.text(),
        Msg::MenuSecView.text(),
    ];
    lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty() && !heads.iter().any(|h| has_section(&[l.to_string()], h)))
        .cloned()
        .collect()
}

#[test]
fn test_nv_9_action_menu_column_section_has_item() {
    // [NV-9] [SR-24] tags のセルで `x` → 「列」の節に頻度表の項目(キー `%` を添える)。Enter で開ける。
    let (_t, mut a) = make("nv9menu", TAGS);
    let label = label_of(&a, freq_action());
    a.row = 0;
    col_named(&mut a, "tags");
    let before = shot(&mut a, 100, 50);
    ch(&mut a, 'x');
    let after = shot(&mut a, 100, 50);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);

    // 「列」の節の中(次の節の見出しまで)にある。
    let col_head = Msg::MenuSecColumn.text();
    let at = lines
        .iter()
        .position(|l| has_section(&[l.clone()], col_head))
        .unwrap_or_else(|| panic!("節「{col_head}」が無い:\n{after}"));
    let next = [Msg::MenuSecRow.text(), Msg::MenuSecView.text()];
    let section: Vec<&String> = lines[at + 1..]
        .iter()
        .take_while(|l| !next.iter().any(|h| has_section(&[l.to_string()], h)))
        .collect();
    let pct = keymap::display("%");
    assert!(
        section
            .iter()
            .any(|l| l.contains(&label) && l.split_whitespace().any(|w| w == pct || w == "%")),
        "節「{col_head}」に「{label}  %」が無い:\n{after}"
    );

    // その項目まで ↓ で動いて Enter → 頻度表が開く。
    let items = menu_items(&lines);
    let idx = items
        .iter()
        .position(|l| l.contains(&label))
        .unwrap_or_else(|| panic!("一覧の項目に「{label}」が無い:\n{after}"));
    for _ in 0..idx {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Freq, "一覧の頻度表の項目で Enter で開かない");
}

#[test]
fn test_nv_9_action_menu_item_key_opens() {
    // [NV-9] [SR-24] 一覧で行に添えたキー `%` を押す → 頻度表が開く。
    let (_t, mut a) = make("nv9menukey", TAGS);
    col_named(&mut a, "tags");
    ch(&mut a, 'x');
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    ch(&mut a, '%');
    assert_eq!(a.mode, Mode::Freq, "一覧で % で頻度表が開かない");
}
