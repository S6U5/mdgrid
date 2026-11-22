//! ビューの設定の画面の左の区画の一覧をキーで動かす試験(navigation.settings-sections-keys・
//! navigation.settings-sections-digits・navigation.settings-sections-focus)。期待は要求の受け入れの例から。
//! 画面の1行は「左の一覧 │ 右の欄」。選びの印は、左では行の頭の `> `、右の項目では頭の `  >`。

use super::keymap::Mode;
use super::settings::{Pick, Sec};
use super::test_screen::{ch, press, screen, typing};
use super::test_settings_screen::{draft, folder, open, section};
use super::App;
use ratatui::crossterm::event::KeyCode;

/// 欄の行(見出しと反映の行・空行の下から、下の空行の上まで)を、左と右に分ける。
fn panes(s: &str) -> Vec<(String, String)> {
    s.lines()
        .skip(2)
        .filter_map(|l| l.split_once('│'))
        .map(|(l, r)| {
            let r = r.strip_prefix(' ').unwrap_or(r);
            (l.trim_end().to_string(), r.trim_end().to_string())
        })
        .collect()
}

/// 左の一覧で選びの印 `>` の付いた行(印を除いた区画の名前と状態)。
fn left_marked(s: &str) -> Vec<String> {
    panes(s)
        .into_iter()
        .filter_map(|(l, _)| l.strip_prefix("> ").map(str::to_string))
        .collect()
}

/// 右の項目で選びの印 `>` の付いた行(印を除いた項目の文字)。
fn right_marked(s: &str) -> Vec<String> {
    panes(s)
        .into_iter()
        .filter_map(|(_, r)| r.strip_prefix("  >").map(str::to_string))
        .collect()
}

/// 右の欄の見出し(1行目。キーが右に効いていれば頭に `>`)。
fn right_head(s: &str) -> String {
    panes(s).first().map(|(_, r)| r.clone()).unwrap_or_default()
}

/// 左の一覧の区画の行(印と状態を除いた「番号 名前」)。
fn left_names(s: &str) -> Vec<String> {
    panes(s)
        .into_iter()
        .map(|(l, _)| l.chars().skip(2).collect::<String>())
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            // 名前のあとの状態(右寄せ)を落とす: 「1 列           5/5」→「1 列」。
            let mut words = l.split_whitespace();
            format!(
                "{} {}",
                words.next().unwrap_or_default(),
                words.next().unwrap_or_default()
            )
        })
        .collect()
}

/// 列の区画の項目にいる(`o` で開き、Tab で列の区画へ)。
fn at_columns(a: &mut App) {
    open(a);
    section(a, Sec::Columns);
    assert!(!draft(a).nav, "開いたときは右の項目にキーが効く");
}

#[test]
fn test_sections_keys_list_and_items() {
    // [navigation.settings-sections-keys] 要求の確かめは stest(tests/test_settings_sections_e2e.rs)。ここは画面の中の状態と、
    // 矢印・Enter も同じことを手早く見る(印は付けない)。列の区画の項目にいる → `h` → 左の「列」が選ばれ、選びの印が左に移る。
    // `j` を2回 → 左は「並べ替え」で、右が並べ替えの中身。`l` → 右の並べ替えの1つ目の項目。Tab と Shift+Tab はこれまでどおり区画を巡る。
    let (_t, mut a) = folder("secs-keys");
    at_columns(&mut a);
    ch(&mut a, 'h');
    let s = screen(&a);
    assert_eq!(draft(&a).sec, Sec::Columns);
    assert!(draft(&a).nav);
    assert_eq!(left_marked(&s).len(), 1, "{s}");
    assert!(left_marked(&s)[0].starts_with("1 列"), "{s}");
    assert!(right_marked(&s).is_empty(), "印は左だけ:\n{s}");
    ch(&mut a, 'j');
    ch(&mut a, 'j');
    let s = screen(&a);
    assert_eq!(draft(&a).sec, Sec::Sorts);
    assert!(left_marked(&s)[0].starts_with("3 並べ替え"), "{s}");
    assert_eq!(right_head(&s).trim(), "並べ替え", "{s}");
    assert!(
        s.contains("上の決まりが先。値が同じなら次の決まりで並べる"),
        "右が並べ替えの中身:\n{s}"
    );
    ch(&mut a, 'l');
    let s = screen(&a);
    assert!(!draft(&a).nav);
    assert_eq!(draft(&a).sec, Sec::Sorts);
    assert_eq!(draft(&a).at(Sec::Sorts), 0, "1つ目の項目");
    assert_eq!(right_marked(&s), ["+ 並べ替えを足す"], "{s}");
    assert!(left_marked(&s).is_empty(), "印は右へ:\n{s}");
    // Tab と Shift+Tab は、これまでどおり区画を巡る(並べ替え → グループ → 並べ替え)。
    press(&mut a, KeyCode::Tab);
    assert_eq!(draft(&a).sec, Sec::Group);
    press(&mut a, KeyCode::BackTab);
    assert_eq!(draft(&a).sec, Sec::Sorts);
    // ←・↓・↑・→ も同じ(左の一覧で ↓ → グループ、↑ → 並べ替え、→ で項目へ)。
    press(&mut a, KeyCode::Left);
    assert!(draft(&a).nav);
    press(&mut a, KeyCode::Down);
    assert_eq!(draft(&a).sec, Sec::Group);
    press(&mut a, KeyCode::Up);
    assert_eq!(draft(&a).sec, Sec::Sorts);
    press(&mut a, KeyCode::Right);
    assert!(!draft(&a).nav);
    // Enter も左の一覧から右の項目へ(項目は決めない)。
    ch(&mut a, 'h');
    press(&mut a, KeyCode::Enter);
    assert!(!draft(&a).nav);
    assert!(draft(&a).pick.is_none(), "Enter で項目を決めていない");
    assert_eq!(a.mode, Mode::Settings);
}

#[test]
fn test_sections_digits_numbers_and_jump() {
    // [utest -> req~navigation.settings-sections-digits~2] `o` → 左の一覧が「1 列」「2 フィルター」「3 並べ替え」…。
    // `3` → 並べ替えの区画が選ばれ、右がその中身。
    let (_t, mut a) = folder("secs-digits");
    open(&mut a);
    let s = screen(&a);
    assert_eq!(
        left_names(&s),
        [
            "1 列",
            "2 フィルター",
            "3 並べ替え",
            "4 グループ",
            "5 親子",
            "6 表示",
            "7 ビュー",
            "8 見た目"
        ],
        "{s}"
    );
    ch(&mut a, '3');
    let s = screen(&a);
    assert_eq!(draft(&a).sec, Sec::Sorts);
    assert!(right_head(&s).contains("並べ替え"), "{s}");
    assert!(s.contains("+ 並べ替えを足す"), "右が並べ替えの中身:\n{s}");
    // 左の一覧にいても同じ(`1` → 列の区画、印は左のまま)。
    ch(&mut a, 'h');
    ch(&mut a, '1');
    let s = screen(&a);
    assert_eq!(draft(&a).sec, Sec::Columns);
    assert!(left_marked(&s)[0].starts_with("1 列"), "{s}");
}

#[test]
fn test_sections_digits_beyond_count() {
    // [utest -> req~navigation.settings-sections-digits~2] 区画の数より大きい数字 → 何も変わらない。
    let (_t, mut a) = folder("secs-digits-over");
    open(&mut a);
    ch(&mut a, '3');
    let before = screen(&a);
    let sec = draft(&a).sec;
    ch(&mut a, '9');
    assert_eq!(draft(&a).sec, sec);
    assert_eq!(screen(&a), before);
    assert_eq!(a.mode, Mode::Settings);
}

#[test]
fn test_sections_digits_in_value_input() {
    // [utest -> req~navigation.settings-sections-digits~2] 値の入力の欄にいるとき(navigation.settings-filter の値の入力)の数字 → 入力の文字になる。
    let (_t, mut a) = folder("secs-digits-text");
    open(&mut a);
    ch(&mut a, '2');
    assert_eq!(draft(&a).sec, Sec::Filters);
    // 「+ 条件を足す」→ 列 title → 種類「含む」→ 値の入力。
    let last = draft(&a).len(Sec::Filters) - 1;
    for _ in 0..last {
        ch(&mut a, 'j');
    }
    press(&mut a, KeyCode::Enter);
    let title = draft(&a).keys.iter().position(|c| c == "title").unwrap();
    for _ in 0..title {
        ch(&mut a, 'j');
    }
    press(&mut a, KeyCode::Enter);
    assert!(matches!(draft(&a).pick, Some(Pick::Kind { .. })));
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "31");
    assert_eq!(draft(&a).text.as_ref().unwrap().text, "31");
    assert_eq!(draft(&a).sec, Sec::Filters, "区画は移らない");
    assert_eq!(a.mode, Mode::SettingsText);
}

#[test]
fn test_sections_focus_marker() {
    // [utest -> req~navigation.settings-sections-focus~2] 左の一覧にいる → 左の選んだ区画の頭に選びの印、右の項目には印が無い。
    // `l` → 印が右の項目へ移る。
    let (_t, mut a) = folder("secs-focus");
    open(&mut a);
    ch(&mut a, 'h');
    let s = screen(&a);
    assert_eq!(left_marked(&s).len(), 1, "{s}");
    assert!(left_marked(&s)[0].starts_with("2 フィルター"), "{s}");
    assert!(right_marked(&s).is_empty(), "右の項目に印が無い:\n{s}");
    assert!(
        !right_head(&s).starts_with('>'),
        "右の見出しにも印が無い:\n{s}"
    );
    ch(&mut a, 'l');
    let s = screen(&a);
    assert!(left_marked(&s).is_empty(), "左の印は消える:\n{s}");
    assert_eq!(right_marked(&s), ["+ 条件を足す"], "{s}");
}

#[test]
fn test_sections_digit_keys_in_help() {
    // [utest -> req~screen.key-table~1] 表に足した数字のキー(1〜9)は、ヘルプのビューの設定の節に1行で出る(`1-9`)。
    let (_t, mut a) = folder("secs-help");
    open(&mut a);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let lines = super::help::help_lines(&a, Mode::Settings, 78);
    let hit: Vec<&String> = lines
        .iter()
        .map(|(l, _)| l)
        .filter(|l| l.contains("番号の区画へ"))
        .collect();
    assert!(!hit.is_empty(), "{lines:?}");
    assert!(hit.iter().all(|l| l.contains("1-9")), "{hit:?}");
}
