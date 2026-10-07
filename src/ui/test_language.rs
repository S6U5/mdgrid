//! 画面の文言の言語(SR-23)の受け入れ試験(タスク3: キーの表示名・下の帯・ヘルプ・パレット・「+ 新規」・カレンダー)。
//! 記録: specs/_changes/2026-10-03-language.md。
//! 画面の実装を見ずに、`mdgrid::i18n::scoped` でこのスレッドだけ言語を変え、80×24 の画面の文字で確かめる。
//! 試験の環境は .cargo/config.toml で日本語に固定されている。
//!
//! 見る行は絞る(ノートの値が混ざらないように): 下の帯は 22 行目(添字 21)、ヘッダーは 1 行目の
//! 右の端の「+ …」、パレットは入力の下の候補の行、カレンダーはその枠の桁から右だけ。
//! 英語の試験の保管庫の値と列の名前は英字で作る。

use super::help::candidates;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{buffer, ch, col_named, ctrl, golden, make, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;

const TODAY: &str = "2026-10-02";

/// 下の帯の行(80×24 で添字 21)。
const BAND: usize = 21;

/// 英字だけの保管庫の値。
const EN_NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: meeting\nstatus: doing\n---\n"),
    ("b.md", "---\ntitle: shopping\nstatus: done\n---\n"),
];

// ---- 道具 ----

/// 日本語の文字(ひらがな・カタカナ・漢字・「・」「「」「」」「、」「。」などの和文の記号)。
fn is_ja(c: char) -> bool {
    matches!(c,
        '\u{3001}'..='\u{303F}' // 、。「」 など
        | '\u{3040}'..='\u{309F}' // ひらがな
        | '\u{30A0}'..='\u{30FF}' // カタカナ・「・」
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}' // 漢字
    )
}

fn has_ja(s: &str) -> bool {
    s.chars().any(is_ja)
}

fn assert_no_ja(what: &str, line: &str, s: &str) {
    assert!(
        !has_ja(line),
        "{what} に日本語の文字が残っている: {line:?}\n{s}"
    );
}

fn line(s: &str, y: usize) -> String {
    s.lines().nth(y).unwrap_or("").to_string()
}

/// 空白で区切った語のうち、`key` の次の語が英字で始まる(「Enter 編集」でなく「Enter Edit」)。
fn word_after_key(line: &str, key: &str) -> bool {
    let toks: Vec<&str> = line.split_whitespace().collect();
    toks.windows(2)
        .any(|w| w[0] == key && w[1].chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
}

/// バッファの行 `y` の、桁 `x0` から右の文字(幅2の文字の後ろの空きの桁は飛ばす)。
fn cells_from(buf: &Buffer, y: u16, x0: u16) -> String {
    let mut out = String::new();
    let mut x = x0;
    while x < buf.area.width {
        let s = buf[(x, y)].symbol();
        out.push_str(s);
        x += width::width(s).max(1) as u16;
    }
    out.trim_end().to_string()
}

/// main と同じ道筋で `.base` なしのフォルダを開く(ヘッダーに「+ 新規」が出る形)。
fn boot(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// types.json(due は date)のある保管庫。今日は 2026-10-02。
fn date_vault(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    a.today = types::parse_date(TODAY).unwrap();
    (tmp, a)
}

/// ヘッダーの行の右の端の「+ …」の文字と、その始まりの桁。
fn header_button(a: &App) -> (String, u16) {
    let s = screen(a);
    let h = line(&s, 0);
    let i = h
        .rfind("+ ")
        .unwrap_or_else(|| panic!("ヘッダーに「+ …」が無い: {h:?}\n{s}"));
    let label = h[i..].trim_end().to_string();
    (label, width::width(&h[..i]) as u16)
}

/// 日付のセルの下に出たカレンダーの枠の行(枠の左の桁から右だけ)。
fn calendar_box(a: &App) -> Vec<String> {
    let buf = buffer(a);
    let s = screen(a);
    let (_, cy) = view::cursor(a).expect("入力のカーソルがある");
    let top = cy + 1;
    let head = line(&s, top as usize);
    let i = head
        .find("+-")
        .unwrap_or_else(|| panic!("入力の下にカレンダーの見出しが無い\n{s}"));
    let x0 = width::width(&head[..i]) as u16;
    let mut out = Vec::new();
    for y in top..buf.area.height {
        let l = cells_from(&buf, y, x0);
        if !(l.starts_with('+') || l.starts_with('|')) {
            break;
        }
        out.push(l);
    }
    assert!(out.len() >= 4, "カレンダーの枠が短い: {out:?}\n{s}");
    out
}

// ---- 1. 下の帯 ----

#[test]
fn test_sr_23_ui_band_english() {
    // [SR-23] 英語では、表の下の帯(モード・位置・未保存・今押せるキー)に日本語の文字が無く、キーの横は英語の語。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr23band", EN_NOTES);
    let s = screen(&a);
    let band = line(&s, BAND);
    assert!(band.contains("Enter"), "{s}");
    assert_no_ja("表の下の帯", &band, &s);
    assert!(word_after_key(&band, "Enter"), "Enter の横が英語: {band:?}");
    assert!(word_after_key(&band, "q"), "q の横が英語: {band:?}");
    // 編集のモードの帯も英語(モードの表示を含む)。
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let s = screen(&a);
    let band = line(&s, BAND);
    assert_no_ja("編集の下の帯", &band, &s);
    assert!(word_after_key(&band, "Esc"), "Esc の横が英語: {band:?}");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- 2. ヘルプ ----

#[test]
fn test_sr_23_ui_help_english() {
    // [SR-23] 英語では、ヘルプ(`?`)の見出しと各行の説明に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr23help", EN_NOTES);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let s = screen(&a);
    for y in 0..=BAND {
        assert_no_ja(&format!("ヘルプの {} 行目", y + 1), &line(&s, y), &s);
    }
    assert!(
        line(&s, 1).chars().any(|c| c.is_ascii_alphabetic()),
        "先頭の見出しが英語: {s}"
    );
    assert!(word_after_key(&line(&s, 2), "Enter"), "{s}");
    // 流した先の節も全部。
    let all = help::help_lines(&a, Mode::Table, 79);
    assert!(all.iter().any(|(_, h)| *h), "見出しの行がある");
    for (l, h) in &all {
        let what = if *h {
            "ヘルプの見出し"
        } else {
            "ヘルプの行"
        };
        assert_no_ja(what, l, &s);
    }
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- 3. パレット ----

/// パレットの候補の行(入力の行の下から、最初の空の行まで)。
fn palette_lines(s: &str) -> Vec<String> {
    s.lines()
        .skip(2)
        .take_while(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn test_sr_23_ui_palette_english() {
    // [SR-23] 英語では、パレット(`:`)の候補の説明に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr23pal", EN_NOTES);
    ch(&mut a, ':');
    assert_eq!(a.mode, Mode::Palette);
    let n = candidates(&a, "").len();
    assert!(n > 10, "候補がある: {n}");
    // ↓ で全部の候補を流して、見えている候補の行を全部見る。
    for _ in 0..=n {
        let s = screen(&a);
        let cands = palette_lines(&s);
        assert!(!cands.is_empty(), "候補の行が見える\n{s}");
        for l in &cands {
            assert_no_ja("パレットの候補", l, &s);
        }
        assert_no_ja("パレットの下の帯", &line(&s, BAND), &s);
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Esc);
    // save と打つと、先頭の候補は英語の説明・save・^S。
    ch(&mut a, ':');
    typing(&mut a, "save");
    let s = screen(&a);
    let first = line(&s, 2);
    assert!(
        first.starts_with('>') && first.contains("save") && first.contains("^S"),
        "{s}"
    );
    assert_no_ja("パレットの先頭の候補", &first, &s);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- 4. ヘッダーの「+ 新規」 ----

/// ヘッダーの右の端の「+ …」を、始まり・終わりの桁でクリックすると名前の欄が開き、
/// 1つ左の桁では開かない。最後に名前を打って作る。返すのはボタンの文字。
fn header_button_clicks(name: &str) -> String {
    let tmp = Tmp::new(name);
    for (n, t) in EN_NOTES {
        tmp.write(n, t);
    }
    let mut a = boot(&tmp);
    let (label, x0) = header_button(&a);
    let w = width::width(&label) as u16;
    assert!(w > 2, "「+ 」の後ろに文字がある: {label:?}");
    let s = screen(&a);
    assert!(
        line(&s, 0).trim_end().ends_with(&label),
        "ヘッダーの右の端: {s}"
    );
    for x in [x0, x0 + w - 1] {
        a.click(x, 0);
        assert_ne!(
            a.mode,
            Mode::Table,
            "{label:?} の桁 {x} のクリックで名前の欄が開く\n{}",
            screen(&a)
        );
        press(&mut a, KeyCode::Esc);
        assert_eq!(a.mode, Mode::Table, "Esc で表に戻る");
    }
    a.click(x0 - 1, 0);
    assert_eq!(
        a.mode,
        Mode::Table,
        "{label:?} の1つ左の桁 {} では開かない\n{}",
        x0 - 1,
        screen(&a)
    );
    a.click(x0 + 1, 0);
    assert_ne!(a.mode, Mode::Table);
    typing(&mut a, "prep");
    ctrl(&mut a, 's'); // CE-26: 窓のどの欄からでも作る
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る\n{}", screen(&a));
    assert!(tmp.notes().join("prep.md").is_file(), "prep.md ができる");
    label
}

#[test]
fn test_sr_23_ui_new_note_button_english() {
    // [SR-23] 英語では、ヘッダーの「+ 新規」が英語の文字になり、そのクリックの当たりが英語の幅で合う。
    let _g = scoped(Lang::En);
    let label = header_button_clicks("sr23new");
    assert!(!has_ja(&label), "ヘッダーのボタンが英語: {label:?}");
    assert!(
        label[2..].chars().any(|c| c.is_ascii_alphabetic()),
        "{label:?}"
    );
}

// ---- 5. カレンダー ----

#[test]
fn test_sr_23_ui_calendar_english() {
    // [SR-23] 英語では、日付の列で開くカレンダーの見出し(月・曜日)と縁の案内に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = date_vault(
        "sr23cal",
        &[
            ("a.md", "---\ntitle: meeting\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ntitle: shopping\ndue: 2026-09-01\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(calendar::shown(&a));
    let s = screen(&a);
    let cal = calendar_box(&a);
    assert!(cal[0].contains("2026"), "見出しに年: {cal:?}");
    for l in &cal {
        assert_no_ja("カレンダー", l, &s);
    }
    // 曜日の見出しの行は英字。
    assert!(
        cal[1].chars().any(|c| c.is_ascii_alphabetic()),
        "曜日の見出しが英語: {cal:?}"
    );
    // 縁の案内: Shift+←→ と Shift+↑↓ の横が英語の語。
    let edge = cal.join("\n");
    for key in ["Shift+←→", "Shift+↑↓"] {
        assert!(edge.contains(key), "{key} が縁にある: {cal:?}");
        assert!(
            cal.iter().any(|l| word_after_key(l, key)),
            "{key} の横が英語: {cal:?}"
        );
    }
    assert_no_ja("編集の下の帯", &line(&s, BAND), &s);
    press(&mut a, KeyCode::Esc);
}

// ---- 6. 日本語は今のまま ----

#[test]
fn test_sr_23_ui_japanese_unchanged() {
    // [SR-23] 同じ操作を日本語で行うと、今の日本語の画面(既存の golden)のまま。
    let _g = scoped(Lang::Ja);

    // 下の帯: sr_1 と同じ。
    let (_t1, a) = make(
        "sr23ja1",
        &[
            (
                "a.md",
                "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n本文\n",
            ),
            ("b.md", "---\ntitle: 買い物\nstatus: 完了\n---\n"),
            ("c.md", "---\ntitle: 読書\ntags: [本, 秋]\n---\n"),
        ],
    );
    let s = screen(&a);
    assert!(line(&s, BAND).contains("Enter 編集"), "{s}");
    golden("sr_1", &s);

    // ヘルプ: sr_5 と同じ。パレット: sr_14 と同じ。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n"),
        ("b.md", "---\ntitle: 買い物\nstatus: 完了\n---\n"),
    ];
    let (_t2, mut a) = make("sr23ja5", notes);
    ch(&mut a, '?');
    let s = screen(&a);
    assert!(line(&s, 1).contains("今押せるキー(表)"), "{s}");
    golden("sr_5", &s);
    let (_t3, mut a) = make("sr23ja14", notes);
    ch(&mut a, ':');
    typing(&mut a, "save");
    golden("sr_14", &screen(&a));

    // ヘッダーの「+ 新規」と、その当たり。
    let label = header_button_clicks("sr23janew");
    assert_eq!(label, "+ 新規");

    // カレンダー: ce_20 と同じ。
    let (_t4, mut a) = date_vault(
        "sr23jacal",
        &[
            ("a.md", "---\ntitle: 会議\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ntitle: 買い物\ndue: 2026-09-01\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    let cal = calendar_box(&a);
    assert!(cal[0].contains(" 2026年10月 "), "{cal:?}");
    golden("ce_20", &screen(&a));
}
