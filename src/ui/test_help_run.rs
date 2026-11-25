//! ヘルプの「今押せるキー」の行を選んで実行する(screen.help-run)。期待値は要求の受け入れの例から取った。
//! 選んだ項目は、反転の見た目と、項目の左の印 `>` で見せる(色だけで伝えない)。

use super::external::OpenLog;
use super::keymap::Mode;
use super::test_screen::{buffer, ch, make, press, screen, Tmp};
use super::App;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

/// 開く口に渡った URL。
type Seen = std::rc::Rc<std::cell::RefCell<Vec<String>>>;

/// `url: https://example.com/mail` の行と、URL の無い行。
const NOTES: &[(&str, &str)] = &[
    ("mail.md", "---\nurl: https://example.com/mail\n---\n"),
    ("memo.md", "---\nnote: x\n---\n"),
];

/// 表を開き、`url` のある行を選び、開く口を記録に替える。
fn boot(tag: &str) -> (Tmp, App, Seen) {
    let (t, mut a) = make(tag, NOTES);
    a.row = (0..a.slots.len())
        .find(|&i| {
            matches!(a.slots[i], super::grid::Slot::Row(r)
            if a.src.label(&a.rows[r]) == "mail.md")
        })
        .expect("mail の行");
    let log = OpenLog::default();
    let seen = log.0.clone();
    a.opener = Box::new(log);
    (t, a, seen)
}

/// ヘルプで選んでいる項目: (画面の行, 項目の始まりの桁, 項目の文字)。本文の行(見出しの行の下から下の帯の上まで)で
/// 反転した桁の続きを項目とし、その2桁左に印 `>` があること。
fn selected(a: &App) -> Option<(u16, u16, String)> {
    let b = buffer(a);
    let mut hit = None;
    // 画面の最下行と、その上の帯・メッセージ行は本文でない(描く高さは1つ少ない)。
    for y in 1..b.area.height.saturating_sub(3) {
        let xs: Vec<u16> = (0..b.area.width)
            .filter(|&x| b[(x, y)].modifier.contains(Modifier::REVERSED))
            .collect();
        let (Some(&x0), Some(&x1)) = (xs.first(), xs.last()) else {
            continue;
        };
        assert!(hit.is_none(), "選んでいる項目は1つ:\n{}", screen(a));
        // 幅2の文字の後ろの空きの桁は飛ばす(test_screen::text と同じ)。
        let mut text = String::new();
        let mut x = x0;
        while x <= x1 {
            let s = b[(x, y)].symbol();
            text.push_str(s);
            x += super::width::width(s).max(1) as u16;
        }
        assert_eq!(
            b[(x0 - 2, y)].symbol(),
            ">",
            "選んだ項目の左に印 `>`:\n{}",
            screen(a)
        );
        hit = Some((y, x0, text.trim().to_string()));
    }
    hit
}

/// ↓ で、文字に `label` を含む項目を `nth` 回目(0 から)に選ぶまで進める。
fn down_to(a: &mut App, label: &str, nth: usize) {
    let mut seen = 0;
    let mut last = None;
    for _ in 0..2000 {
        let now = selected(a).expect("ヘルプで項目を選んでいる");
        if now.2.contains(label) {
            if seen == nth {
                return;
            }
            seen += 1;
        }
        // 末尾で止まったら、それより先は無い。
        assert_ne!(last.as_ref(), Some(&now), "{label:?} の {nth} 回目が無い");
        last = Some(now);
        press(a, KeyCode::Down);
    }
    panic!("{label:?} の {nth} 回目に届かない");
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_enter_on_now_line_opens_url() {
    // `url` のある行を選んだ表で `?` → 「今押せるキー」の「URL を開く」の行を選んで Enter →
    // ヘルプが閉じ、その行の URL を開く(`O` を押したのと同じ)。
    let (_t, mut a, log) = boot("help-run-url");
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    // 開いたときは先頭の項目を選んでいる。
    let first = selected(&a).expect("先頭の項目を選んでいる");
    assert!(first.2.starts_with("Enter"), "{first:?}");
    down_to(&mut a, "URL を開く", 0);
    let s = screen(&a);
    let (y, _, text) = selected(&a).unwrap();
    assert!(text.starts_with('O'), "{text:?}");
    // 選んだのは「今押せるキー」の節の行(その見出しより下で、次の節の見出しより上)。
    let lines: Vec<&str> = s.lines().collect();
    let now = lines.iter().position(|l| l.contains("今押せるキー(表)"));
    assert!(now.is_some_and(|h| h < y as usize), "{s}");
    assert!(
        lines[now.unwrap() + 1..y as usize]
            .iter()
            .all(|l| !l.trim().is_empty()),
        "「今押せるキー」の節の中:\n{s}"
    );
    assert!(log.borrow().is_empty(), "選んだだけでは開かない");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.help.is_none());
    assert_eq!(
        *log.borrow(),
        vec!["https://example.com/mail".to_string()],
        "{:?}",
        a.message
    );
    // `O` と同じく、メッセージ行に URL。
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("https://example.com/mail"), "{m}");
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_enter_below_now_does_nothing() {
    // ヘルプを開いた → 「今押せるキー」より下の節の行(全部のキー)で Enter → 何もせず、ヘルプは開いたまま。
    // 同じ「URL を開く」でも、下の節の行では開かない。
    let (_t, mut a, log) = boot("help-run-below");
    let (row, col) = (a.row, a.col);
    ch(&mut a, '?');
    down_to(&mut a, "URL を開く", 1);
    let before = selected(&a).unwrap();
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Help);
    assert!(a.help.is_some());
    assert_eq!(selected(&a), Some(before), "選びも流れもそのまま");
    assert!(log.borrow().is_empty());
    assert_eq!((a.row, a.col), (row, col));
    assert_eq!(a.changes.count(), 0);
    // 印の意味の節の行(末尾)でも同じ。
    press(&mut a, KeyCode::Char('G'));
    let end = selected(&a).expect("末尾の項目を選んでいる");
    assert!(end.2.contains('~'), "末尾は行の印 ~ の行: {end:?}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Help);
    assert_eq!(selected(&a), Some(end));
    assert!(log.borrow().is_empty());
    assert_eq!((a.row, a.col), (row, col));
    assert_eq!(a.changes.count(), 0);
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_esc_closes_without_running() {
    // ヘルプで行を選んだ → Esc → 何もせずにヘルプを閉じる。
    let (_t, mut a, log) = boot("help-run-esc");
    let (row, col) = (a.row, a.col);
    ch(&mut a, '?');
    down_to(&mut a, "URL を開く", 0);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.help.is_none());
    assert!(log.borrow().is_empty());
    assert_eq!((a.row, a.col), (row, col));
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_none(), "{:?}", a.message);
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_click_selects_now_line() {
    // 「今押せるキー」の「URL を開く」の行をクリック → その行を選ぶ(まだ開かない)。Enter → ヘルプを閉じて URL を開く。
    let (_t, mut a, log) = boot("help-run-click");
    ch(&mut a, '?');
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("URL を開く"))
        .expect("最初の画面に「URL を開く」");
    let x = super::width::width(&line[..line.find("URL を開く").unwrap()]);
    a.click(x as u16, y as u16);
    assert_eq!(a.mode, Mode::Help);
    let (sy, _, text) = selected(&a).expect("クリックした項目を選ぶ");
    assert_eq!(usize::from(sy), y);
    assert!(text.contains("URL を開く"), "{text:?}");
    assert!(log.borrow().is_empty(), "クリックで選ぶだけ");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(*log.borrow(), vec!["https://example.com/mail".to_string()]);
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_click_on_selected_runs() {
    // 選んでいる項目をもう一度クリック → Enter と同じ(表のセルの2段階のクリックと同じ)。下の節の行では何もしない。
    let (_t, mut a, log) = boot("help-run-click2");
    ch(&mut a, '?');
    down_to(&mut a, "URL を開く", 0);
    let (y, x, _) = selected(&a).unwrap();
    a.click(x + 2, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(*log.borrow(), vec!["https://example.com/mail".to_string()]);
    // 下の節の行は、2度クリックしても開いたまま。
    ch(&mut a, '?');
    down_to(&mut a, "URL を開く", 1);
    let (y, x, _) = selected(&a).unwrap();
    a.click(x + 2, y);
    a.click(x + 2, y);
    assert_eq!(a.mode, Mode::Help);
    assert_eq!(log.borrow().len(), 1);
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_run_digit_group_runs_first_key() {
    // 数字のキーを1行にまとめた行(ビューの設定の「1-9 番号の区画へ」)は、先頭のキー(1)と同じにする。
    let (_t, mut a) = super::test_settings_screen::folder("help-run-digits");
    super::test_settings_screen::open(&mut a);
    ch(&mut a, '3');
    assert_eq!(
        super::test_settings_screen::draft(&a).sec,
        super::settings::Sec::Sorts
    );
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    down_to(&mut a, "番号の区画へ", 0);
    assert!(selected(&a).unwrap().2.starts_with("1-9"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert_eq!(
        super::test_settings_screen::draft(&a).sec,
        super::settings::Sec::Columns
    );
}

// [utest -> req~screen.help-run~1]
#[test]
fn test_help_selection_follows_and_wheel_scrolls() {
    // ↓ で選びを進めると、選んだ行が見えるように流れる。ホイールはヘルプを流し(screen.mouse-shift-wheel-other)、
    // 選んだ項目が見えなくなれば見えている項目へ移す。
    let (_t, mut a, _log) = boot("help-run-follow");
    ch(&mut a, '?');
    for _ in 0..60 {
        press(&mut a, KeyCode::Down);
        assert!(
            selected(&a).is_some(),
            "選んだ項目が見える:\n{}",
            screen(&a)
        );
    }
    assert!(a.help.as_ref().unwrap().top > 0, "流れた");
    press(&mut a, KeyCode::Char('g'));
    press(&mut a, KeyCode::Char('g'));
    assert_eq!(a.help.as_ref().unwrap().top, 0);
    assert!(selected(&a).unwrap().2.starts_with("Enter"));
    for _ in 0..5 {
        a.wheel(true);
    }
    assert_eq!(a.help.as_ref().unwrap().top, 5);
    let (y, _, _) = selected(&a).expect("ホイールで流しても、選んだ項目は見える");
    assert_eq!(y, 1, "見えている先頭の項目へ移る:\n{}", screen(&a));
}
