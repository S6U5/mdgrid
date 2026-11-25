//! 新しいノートの窓の欄に出す候補の窓(値の候補のリスト・カレンダー・リストの値の付け外しの窓)の置き場所
//! (cell-edit.new-note-form-popup-place)。期待値は specs/requirements/cell-edit.md の受け入れの例から。値は作ったもの。
//!
//! 窓に重ならないことは、候補の窓が出ている画面と、Esc で候補の窓だけを閉じた画面(cell-edit.new-note-form-esc-popup)
//! とで、新しいノートの窓の四角の中の字を比べて確かめる(窓の中のキーの行は、候補の窓の有無で文言が変わるので比べない)。
//! 同じ形の、詳細の表示で値を直すときの候補の窓の置き場所(navigation.detail-popup-place)もここで確かめる。

use super::keymap::Mode;
use super::popup::{beside, Rect, Spot, Want};
use super::startup::Startup;
use super::test_screen::{ch, press, text, typing, Tmp};
use super::width::width;
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 長い候補の名前(70 字)。
fn long_name() -> String {
    format!("とても長い名前の本{}", "あ".repeat(61))
}

/// source の決まりの値: 候補01〜候補28 と長い名前の29件。
fn source_values() -> Vec<String> {
    let mut v: Vec<String> = (1..=28).map(|k| format!("候補{k:02}")).collect();
    v.push(long_name());
    v
}

/// 設定の文(source の決まりと、`ask`)。
fn config(ask: &[&str]) -> String {
    let values: Vec<String> = source_values().iter().map(|v| format!("\"{v}\"")).collect();
    let ask: Vec<String> = ask.iter().map(|c| format!("\"{c}\"")).collect();
    format!(
        "[types]\ndue = \"date\"\ntags = \"list\"\n\n[rules.source]\nvalues = [{}]\n\n[rules.tags]\nvalues = [\"仕事\", \"家\", \"{}\"]\n\n[new_note]\nask = [{}]\n",
        values.join(", "),
        long_name(),
        ask.join(", ")
    )
}

/// 幅 `w`・高さ `h` で、設定 `toml` の表(ノート1件)を開く。
fn boot(tag: &str, toml: &str, w: u16, h: u16) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write(
        "a.md",
        "---\nsource: 候補01\npage: 3\nquote: 一文\ndue: 2026-10-01\ntags:\n  - 仕事\n---\n",
    );
    let (cfg, warnings): (Config, _) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(w, h);
    app.start(Startup {
        config: cfg,
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
    app.today = types::parse_date("2026-10-02").unwrap();
    (tmp, app)
}

/// 窓を開き、名前「ノートQ」を打って Enter(最初の聞く欄の候補の窓が出る)。
fn open_first_field(a: &mut App) {
    ch(a, 'a');
    assert!(a.note.flow.is_some(), "窓が開く:\n{}", screen(a));
    typing(a, "ノートQ");
    press(a, KeyCode::Enter);
}

/// 行の桁 [from, to] の字(幅2の字は始まりの桁で数える)。
fn cols(line: &str, from: usize, to: usize) -> String {
    let mut out = String::new();
    let mut x = 0;
    for c in line.chars() {
        if x >= from && x <= to {
            out.push(c);
        }
        x += width(&c.to_string());
    }
    out
}

/// `pat` の最初の位置(表示の幅で)。
fn col(line: &str, pat: &str) -> Option<usize> {
    line.find(pat).map(|i| width(&line[..i]))
}

/// 字 `set` のどれかの、最初(`last` なら最後)の位置(表示の幅で)。
fn col_of_any(line: &str, set: &[char], last: bool) -> Option<usize> {
    let mut found = None;
    let mut x = 0;
    for c in line.chars() {
        if set.contains(&c) {
            found = Some(x);
            if !last {
                return found;
            }
        }
        x += width(&c.to_string());
    }
    found
}

/// 新しいノートの窓の四角(上の縁の行・下の縁の行・左の縁の桁・右の縁の桁)。題「新しいノート」の行の角から探す。
fn form_rect(s: &str) -> (usize, usize, usize, usize) {
    let lines: Vec<&str> = s.lines().collect();
    let top = lines
        .iter()
        .position(|l| l.contains("新しいノート"))
        .unwrap_or_else(|| panic!("新しいノートの窓が無い:\n{s}"));
    let left = col_of_any(lines[top], &['┌', '╭', '┏', '+'], false).expect("左上の角");
    let right = col_of_any(lines[top], &['┐', '╮', '┓', '+'], true)
        .unwrap_or_else(|| panic!("右上の角:\n{s}"));
    let bottom = (top + 1..lines.len())
        .find(|&y| {
            cols(lines[y], left, left)
                .chars()
                .any(|c| ['└', '╰', '┗', '+'].contains(&c))
        })
        .unwrap_or_else(|| panic!("新しいノートの窓の下の縁が無い:\n{s}"));
    (top, bottom, left, right)
}

/// 候補の窓が出ている画面 `with` と、閉じた画面 `without` とで、新しいノートの窓の四角の中が同じか(キーの行と、
/// `skip` の行は比べない)。
fn assert_form_intact(with: &str, without: &str, skip: Option<usize>) {
    let (top, bottom, left, right) = form_rect(without);
    let a: Vec<&str> = with.lines().collect();
    let b: Vec<&str> = without.lines().collect();
    for y in top..=bottom {
        if y == bottom - 1 || Some(y) == skip {
            continue;
        }
        assert_eq!(
            cols(a[y], left, right),
            cols(b[y], left, right),
            "新しいノートの窓の {y} 行目が候補の窓で欠ける:\n{with}\n---- 候補の窓を閉じた画面 ----\n{without}"
        );
    }
}

/// `pat` のある最初の行と桁。
fn find(s: &str, pat: &str) -> Option<(usize, usize)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| col(l, pat).map(|x| (y, x)))
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_list_goes_right_of_the_form_when_it_fits() {
    // 幅 160・高さ 40、ask = [source, page, quote] で source の欄を開く → 29 件の候補の窓は新しいノートの窓の右にあり、
    // 窓の欄の名前と下の縁は欠けない。長い名前は窓の幅で切る。
    let (_t, mut a) = boot(
        "nfplace-right",
        &config(&["source", "page", "quote"]),
        160,
        40,
    );
    open_first_field(&mut a);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    let with = screen(&a);
    let (y, x) = find(&with, "候補28").unwrap_or_else(|| panic!("候補のリストが無い:\n{with}"));
    press(&mut a, KeyCode::Esc);
    let without = screen(&a);
    assert!(
        find(&without, "候補28").is_none(),
        "Esc で候補だけ閉じる:\n{without}"
    );
    let (top, _, _, right) = form_rect(&without);
    assert!(
        x > right + 1,
        "候補の窓の左の縁は新しいノートの窓の右の縁より右(候補の字 {x}・窓の右の縁 {right}):\n{with}"
    );
    assert!(y > top, "{with}");
    assert_form_intact(&with, &without, None);
    for label in ["source", "page", "quote"] {
        assert!(with.contains(label), "欄の名前 {label}:\n{with}");
    }
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_list_goes_below_the_form_when_the_right_is_narrow() {
    // 幅 90・高さ 40 で同じ窓 → 候補の窓の上の縁は新しいノートの窓の下の縁より下。
    let (_t, mut a) = boot(
        "nfplace-below",
        &config(&["source", "page", "quote"]),
        90,
        40,
    );
    open_first_field(&mut a);
    let with = screen(&a);
    let (y, _) = find(&with, "候補28").unwrap_or_else(|| panic!("候補のリストが無い:\n{with}"));
    press(&mut a, KeyCode::Esc);
    let without = screen(&a);
    let (_, bottom, _, _) = form_rect(&without);
    // 候補の窓は今の値の無い欄なので最後の「なし」を選んで開き、その近くの候補28 が見える。
    assert!(
        y > bottom + 1,
        "候補の窓は新しいノートの窓の下(候補28 は {y} 行目・窓の下の縁は {bottom} 行目):\n{with}"
    );
    assert_form_intact(&with, &without, None);
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_list_goes_above_the_form_when_right_and_below_are_full() {
    // 幅 90・高さ 24、欄が 14(名前と13の列)の窓(右にも下にも入らない)で、2つ目の欄の source を開く → 候補の窓の下の縁は
    // 新しいノートの窓の上の縁より上にあり、新しいノートの窓は欠けない。
    let mut ask = vec!["source".to_string()];
    ask.extend((2..=13).map(|k| format!("c{k:02}")));
    let ask: Vec<&str> = ask.iter().map(String::as_str).collect();
    let (_t, mut a) = boot("nfplace-above", &config(&ask), 90, 24);
    open_first_field(&mut a);
    let with = screen(&a);
    let (y, _) = find(&with, "候補28").unwrap_or_else(|| panic!("候補のリストが無い:\n{with}"));
    press(&mut a, KeyCode::Esc);
    let without = screen(&a);
    let (top, bottom, _, _) = form_rect(&without);
    // 下の帯より上は 21 行(高さ 24 から最下行・下の帯・メッセージの行を除く)。下には縁2行と候補3行が入らない。
    assert!(
        21usize.saturating_sub(bottom + 1) < 5,
        "前提: 窓の下に入らない(下の縁 {bottom}):\n{without}"
    );
    assert!(
        y + 1 < top,
        "候補の窓は新しいノートの窓の上(候補28 は {y} 行目・窓の上の縁は {top} 行目):\n{with}"
    );
    assert_form_intact(&with, &without, None);
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_calendar_does_not_cover_the_form() {
    // 幅 90・高さ 40、ask = [due] → due の欄のカレンダーは新しいノートの窓に重ならない(窓の下に出る)。
    let (_t, mut a) = boot("nfplace-cal", &config(&["due"]), 90, 40);
    open_first_field(&mut a);
    assert!(
        super::calendar::shown(&a),
        "カレンダーが出る:\n{}",
        screen(&a)
    );
    let with = screen(&a);
    press(&mut a, KeyCode::Esc);
    let without = screen(&a);
    assert!(!super::calendar::shown(&a), "Esc でカレンダーだけ閉じる");
    let (_, bottom, _, _) = form_rect(&without);
    let (y, _) =
        find(&with, "月  火  水").unwrap_or_else(|| panic!("カレンダーの曜日の行:\n{with}"));
    assert!(y > bottom, "カレンダーは窓の下:\n{with}");
    assert_form_intact(&with, &without, None);
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_list_select_does_not_cover_the_form() {
    // 幅 90・高さ 40、ask = [tags](リストの列)→ 付け外しの窓は新しいノートの窓に重ならない(窓の下に出る)。
    let (_t, mut a) = boot("nfplace-pick", &config(&["tags"]), 90, 40);
    open_first_field(&mut a);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
    let with = screen(&a);
    let (y, _) = find(&with, "[ 決める ]").unwrap_or_else(|| panic!("付け外しの窓:\n{with}"));
    press(&mut a, KeyCode::Esc);
    let without = screen(&a);
    assert!(find(&without, "[ 決める ]").is_none(), "{without}");
    let (top, bottom, _, _) = form_rect(&without);
    assert!(y > bottom, "付け外しの窓は窓の下:\n{with}");
    // 今の欄(tags)の行は、付け外しの窓の有無で値の見せ方が変わるので比べない。
    assert_form_intact(&with, &without, Some(top + 2));
}

// [utest -> req~cell-edit.new-note-form-popup-place~1]
#[test]
fn test_beside_overlaps_right_of_the_labels_when_nothing_fits() {
    // 新しいノートの窓が行 3〜30(下の帯より上は 32 行)・桁 10〜95(描ける幅 99)で、右にも下にも上にも入らない。今の欄は
    // 行 5・値の桁は 30 → 候補の窓は行 5 より下にあり、左の縁は桁 30 より左に出ない。値の桁から右に入る幅に縮める。
    let form = Rect {
        left: 10,
        top: 3,
        right: 95,
        bottom: 30,
    };
    let want = Want {
        n: 10,
        frame: 2,
        min_vis: 3,
        iw: 80,
        min_w: 24,
    };
    let s: Spot = beside(form, 5, 30, want, 99, 32).expect("置ける");
    assert!(s.below, "{s:?}");
    assert!(s.top > 5, "今の欄の行より下: {s:?}");
    assert!(s.x >= 30, "左の縁は値の桁より左に出ない: {s:?}");
    assert!(s.x + s.iw <= 99, "画面に収まる: {s:?}");
}

// ---- navigation.detail-popup-place: 詳細の表示で値を直すときの候補の窓 ----

// [utest -> req~navigation.detail-popup-place~1]
#[test]
fn test_detail_popups_stay_right_of_the_property_names() {
    // 幅 80 の詳細の表示で、70 字の名前を含む候補のある source と tags(リストの列)を直す → 候補のリストと付け外しの窓の
    // 左の縁は値の欄の桁より左に出ず、プロパティの名前はどれも欠けない。
    let (_t, mut a) = boot("detailplace", &config(&["source"]), 80, 30);
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail, "{:?}", a.message);
    let without = screen(&a);
    let w = usize::from(a.size.0 - 1);
    // 候補の行の字から窓の左の縁までの桁(リストは縁・選びの印・今の値の印、付け外しの窓は縁・選びの印)。
    for (prop, marker, lead) in [("source", "候補02", 3), ("tags", "[ 決める ]", 2)] {
        let d = a.detail.as_ref().unwrap();
        let k = a
            .detail_props(&d.row)
            .iter()
            .position(|p| p == prop)
            .unwrap_or_else(|| panic!("{prop} のプロパティ"));
        a.detail.as_mut().unwrap().sel = k;
        press(&mut a, KeyCode::Enter);
        let (vx, _, _) = super::detail::detail_anchor(&a, w).expect("値の欄");
        let with = screen(&a);
        let (_, x) = find(&with, marker).unwrap_or_else(|| panic!("{prop} の候補の窓:\n{with}"));
        assert!(
            x >= vx + lead,
            "{prop}: 候補の窓の左の縁({})は値の欄の桁({vx})より左に出ない:\n{with}",
            x - lead
        );
        for (y, line) in without.lines().enumerate() {
            let names = ["source", "page", "quote", "due", "tags"];
            if !names.iter().any(|n| cols(line, 0, vx - 1).contains(n)) {
                continue;
            }
            let got = with.lines().nth(y).unwrap_or("");
            // 左の端の桁は選んでいるプロパティの印 `>` なので比べない。
            assert_eq!(
                cols(got, 1, vx - 1),
                cols(line, 1, vx - 1),
                "{prop}: プロパティの名前が欠ける({y} 行目):\n{with}"
            );
        }
        press(&mut a, KeyCode::Esc);
        assert_eq!(a.mode, Mode::Detail, "{prop}: Esc で詳細の表示に戻る");
    }
}
