//! 表の見せ方の切り替え(display-options)の画面の受け入れ試験。SR-20(関係: SR-21・NV-13・NV-16・NV-23・
//! BV-17・SR-15)。記録 specs/_changes/2026-10-02-display-options.md。実装を見ずに、画面の文字・バッファのセルの
//! style・キーで確かめる。状態と mdgrid のビューの置き場は一時フォルダ(`<一時>/state`・`<一時>/config`)。
//!
//! 仮定(実装役に渡す):
//! - 設定は `mdgrid::config::parse` の `[display]`(row_numbers・zebra・column_lines・tabs・chips)を
//!   `App::configure` / `Startup.config` で当てる。試験は TOML の文字列から作る(Config の形に頼らない)。
//! - 行番号は各ノートの行の左(選択の印 `>`・`+` の前でも後でもよい)に、今の表示の並び(絞り込みと
//!   並べ替えのあと)で 1 始まりの数。グループの見出しの行には付けない(番号はグループをまたいで続く)。
//! - 一行おきの色は、表の2行目・4行目…(偶数の行)の背景の色が奇数の行と違う。色なし(ColorMode::None。
//!   `--no-color`・NO_COLOR・設定 color = false)では偶数と奇数の行の style は同じ。
//! - 列の区切り線(提案 column-lines-fix。既定 column_lines = false): 既定・false は今の画面のまま(列の間は空白
//!   1つで `│` は無い)。true で列の見出しの行と各ノートの行の列の間に `│` を引く(どの行でも同じ桁)。
//! - ビューの設定の画面(`o`)に「表示」の節(見出しを選ぶと `>表示`)。項目は `[ ] 行番号`・`[ ] 一行おきの色`・
//!   `[ ] 列の区切り線`・`[x] タブ`・`[x] 検索の欄`・`[x] 設定の帯`(今効いている値)。Tab で節に移り、↑↓ で
//!   選び(選んだ項目は `>[x] 名前`)、Space で切り替え、反映(ボタン)で表に効く。そのビューだけに効く。
//! - 設定の帯を隠しても `f` は効く(帯の項目を選ぶモード Mode::Chips に入り、Backspace で条件を外せる)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{app_of, buffer, ch, golden, make, press, screen, typing, Tmp};
use super::test_settings_screen::{apply_btn, open};
use super::*;
use mdgrid::config::{parse, Config};
use mdgrid::settings::Dir;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

// ---- 材料 ----

/// 6つのノート(status は todo・done・doing・無し、種別 は 会議・本・メモ)と types.json。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date", "priority": "number"}}"#,
    )
    .unwrap();
    tmp.write(
        "a.md",
        "---\ntitle: 会議の準備\nstatus: todo\n種別: 会議\npriority: 3\n---\n",
    );
    tmp.write(
        "b.md",
        "---\ntitle: 読んだ本\nstatus: done\n種別: 本\npriority: 1\n---\n",
    );
    tmp.write(
        "c.md",
        "---\ntitle: 次の本\nstatus: todo\n種別: 本\npriority: 5\n---\n",
    );
    tmp.write("d.md", "---\ntitle: メモ\nstatus: done\n種別: メモ\n---\n");
    tmp.write(
        "e.md",
        "---\ntitle: 会議の記録\nstatus: doing\n種別: 会議\npriority: 4\n---\n",
    );
    tmp.write("f.md", "---\ntitle: 本棚\n種別: 本\n---\n");
    tmp
}

/// ビューが2つの `.base`: status でまとめた「全部」と、doing を外した「進行」。
const BASE: &str = r#"views:
  - type: table
    name: 全部
    groupBy:
      property: status
      direction: ASC
    order: [title, status, 種別, priority]
  - type: table
    name: 進行
    filters: 'status != "doing"'
    order: [title, status, 種別, priority]
"#;

fn base_path(tmp: &Tmp) -> PathBuf {
    let p = tmp.notes().join("tasks.base");
    if !p.exists() {
        std::fs::write(&p, BASE).unwrap();
    }
    p
}

/// 設定の TOML から Config を作る(警告は問わない。振る舞いで確かめる)。
fn cfg(text: &str) -> Config {
    parse(text).expect("設定の TOML は読める").0
}

/// `.base` を main と同じ道筋で開く(状態と mdgrid のビューの置き場は一時フォルダ)。
fn boot_base(tmp: &Tmp, config: Config, color: ColorMode) -> App {
    let p = base_path(tmp);
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), color);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// `.base` なしのフォルダを main と同じ道筋で開く(mdgrid のビューの置き場あり)。
fn boot_folder(tmp: &Tmp, config: Config) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
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

/// `.base` なし・置き場なしで開き、設定を当てる。
fn folder_with(name: &str, config: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = vault(name);
    let mut app = app_of(&tmp, color);
    app.configure(&cfg(config));
    (tmp, app)
}

fn labels(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

// ---- 画面の読み取り ----

/// 表の行(列の見出しの下から、画面の slots の数だけ)の文字。
fn table_lines(app: &App) -> Vec<String> {
    let s = screen(app);
    let dy = view::data_y(app);
    let n = app.slots.len().min(app.data_height());
    s.lines().skip(dy).take(n).map(str::to_string).collect()
}

/// 行の頭の選択の印と空白を除いたあとの数(行番号)。数で始まらなければ None。
fn lead_number(line: &str) -> Option<usize> {
    let rest = line.trim_start_matches([' ', '>', '+']);
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    // 数のあとは区切り(空白か区切り線)で、ノートの名前とくっつかない。
    let after = rest[digits.len()..].chars().next();
    assert!(
        matches!(after, Some(' ') | Some('│') | Some('>') | Some('+')),
        "行番号のあとに区切りが無い: {line:?}"
    );
    digits.parse().ok()
}

/// 表のノートの行に、上から 1・2・3… と番号が付いていて、見出しの行には付いていない。
fn assert_numbered(app: &App) {
    let lines = table_lines(app);
    assert!(!lines.is_empty(), "{}", screen(app));
    let mut want = 1;
    for (k, slot) in app.slots.iter().take(lines.len()).enumerate() {
        let line = &lines[k];
        match slot {
            grid::Slot::Row(r) => {
                assert_eq!(
                    lead_number(line),
                    Some(want),
                    "{k} 行目に番号 {want}: {line:?}\n{}",
                    screen(app)
                );
                let label = app.src.label(&app.rows[*r]);
                let label = label.trim_end_matches(".md");
                assert!(line.contains(label), "番号の後ろに名前: {line:?}");
                want += 1;
            }
            grid::Slot::Head(_) => {
                assert_eq!(
                    lead_number(line),
                    None,
                    "見出しの行には番号を付けない: {line:?}"
                );
            }
            // SR-30: 見出しの上の空きにも番号を付けない。
            grid::Slot::Gap => {
                assert_eq!(
                    lead_number(line),
                    None,
                    "空きの行には番号を付けない: {line:?}"
                );
            }
        }
    }
}

/// 表のどの行にも番号が無い。
fn assert_unnumbered(app: &App) {
    for line in table_lines(app) {
        assert_eq!(
            lead_number(&line),
            None,
            "番号がある: {line:?}\n{}",
            screen(app)
        );
    }
}

/// タブの行(2行目)。
fn tab_line(a: &App) -> String {
    screen(a).lines().nth(1).unwrap_or("").to_string()
}

/// `]` で名前のタブまで動かす(選んだタブは `[名前]`)。
fn goto_tab(a: &mut App, name: &str) {
    let want = format!("[{name}]");
    for _ in 0..8 {
        if tab_line(a).contains(&want) {
            return;
        }
        ch(a, ']');
    }
    panic!("タブ「{name}」に移れない: {}", screen(a));
}

/// 画面の文字 `label` をクリックする(ボタン)。
fn click_label(a: &mut App, label: &str) {
    let s = screen(a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(label))
        .unwrap_or_else(|| panic!("画面に「{label}」が無い:\n{s}"));
    let x = width::width(&line[..line.find(label).unwrap()]);
    a.click(x as u16 + 1, y as u16);
}

fn settle(a: &mut App) {
    for _ in 0..4 {
        if a.mode == Mode::Table {
            return;
        }
        press(a, KeyCode::Esc);
    }
}

// ---- ビューの設定の画面の「表示」の節 ----

const DISPLAY_LABELS: [&str; 6] = [
    "行番号",
    "一行おきの色",
    "列の区切り線",
    "タブ",
    "検索の欄",
    "設定の帯",
];

/// 「表示」の区画に移る(Tab で、選んだ区画が「表示」になるまで。NV-18)。
fn display_section(a: &mut App) {
    for _ in 0..8 {
        if a.draft.as_ref().map(|d| d.sec) == Some(super::settings::Sec::Display) {
            return;
        }
        press(a, KeyCode::Tab);
    }
    panic!("「表示」の節に移れない:\n{}", screen(a));
}

/// 「表示」の区画の `名前 … ● オン` / `名前 … ○ オフ` の今の値(NV-18 のスイッチ)。
fn display_checked(a: &App, label: &str) -> bool {
    let s = screen(a);
    let line = s
        .lines()
        .find(|l| l.contains(label) && (l.contains("● オン") || l.contains("○ オフ")))
        .unwrap_or_else(|| panic!("「{label}」の項目が見えない:\n{s}"));
    line.contains("● オン")
}

/// 「表示」の節で `label` を選び、Space で切り替える。切り替えたあとの値を返す。
fn toggle_display(a: &mut App, label: &str) -> bool {
    display_section(a);
    let before = display_checked(a, label);
    for _ in 0..10 {
        let s = screen(a);
        if s.contains(&format!(">{label}")) {
            ch(a, ' ');
            let after = display_checked(a, label);
            assert_ne!(after, before, "Space で「{label}」が切り替わる");
            return after;
        }
        press(a, KeyCode::Down);
    }
    panic!("「{label}」を選べない:\n{}", screen(a));
}

// ---- SR-20: 行番号 ----

#[test]
fn test_sr_20_row_numbers_from_config() {
    // [SR-20][SR-21] 設定 `[display] row_numbers = true` → 各行の左に 1・2・3…。既定(設定なし)では出さない。
    let (_t, a) = folder_with("sr20num0", "", ColorMode::None);
    assert_unnumbered(&a);
    let (_t, mut a) = folder_with(
        "sr20num",
        "[display]\nrow_numbers = true\n",
        ColorMode::None,
    );
    assert_eq!(a.rows.len(), 6);
    assert_numbered(&a);
    let lines = table_lines(&a);
    assert_eq!(lead_number(&lines[0]), Some(1));
    assert_eq!(lead_number(&lines[1]), Some(2));
    assert_eq!(lead_number(&lines[2]), Some(3));
    // 列の見出しの行の名前の列(「ノート」)は番号の欄の右に残る。
    let s = screen(&a);
    let head = s.lines().nth(view::data_y(&a) - 1).unwrap();
    assert!(head.contains("ノート"), "{s}");
    // [SR-9] どの行も幅以下。
    for l in s.lines() {
        assert!(width::width(l) <= 80, "幅を超える: {l}");
    }
    // 行のクリックは番号の欄の上でもその行を選ぶ(番号の欄の分だけずれない)。
    let dy = view::data_y(&a) as u16;
    a.click(1, dy + 2);
    assert_eq!(a.row, 2);
}

#[test]
fn test_sr_20_row_numbers_follow_sort_and_filter() {
    // [SR-20] 番号は今の表示の並び(並べ替え・絞り込みのあと)の 1 始まり。
    let (_t, mut a) = folder_with(
        "sr20sort",
        "[display]\nrow_numbers = true\n",
        ColorMode::None,
    );
    a.settings.sorts.push(("priority".into(), Dir::Desc));
    a.refresh();
    assert_eq!(labels(&a)[0], "c.md", "priority の降順");
    assert_numbered(&a);
    let first = &table_lines(&a)[0];
    assert!(first.contains(" c "), "1 番は並べ替えのあとの先頭: {first}");
    // 簡易の絞り込みのあとも 1 から振り直す。
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 2, "{:?}", labels(&a));
    assert_numbered(&a);
    let lines = table_lines(&a);
    assert_eq!(lead_number(&lines[0]), Some(1));
    assert_eq!(lead_number(&lines[1]), Some(2));
}

#[test]
fn test_sr_20_row_numbers_skip_group_headings() {
    // [SR-20] グループの見出しの行には番号を付けない。ノートの行の番号は続く。
    let tmp = vault("sr20group");
    let a = boot_base(
        &tmp,
        cfg("[display]\nrow_numbers = true\n"),
        ColorMode::None,
    );
    assert!(!a.groups.is_empty(), "全部 は status でまとめる");
    assert!(
        a.slots.iter().any(|s| matches!(s, grid::Slot::Head(_))),
        "見出しの行がある"
    );
    assert_numbered(&a);
}

#[test]
fn test_sr_20_settings_screen_has_display_section() {
    // [SR-20][NV-13] ビューの設定の画面に「表示」の節と6つの項目。値は今効いているもの(設定の既定)。
    let (_t, mut a) = folder_with("sr20sec", "", ColorMode::None);
    open(&mut a);
    // NV-18: 左の一覧から「表示」の区画へ。
    display_section(&mut a);
    let s = screen(&a);
    assert!(s.contains("表示"), "{s}");
    for label in DISPLAY_LABELS {
        assert!(s.contains(label), "「{label}」が無い:\n{s}");
    }
    assert!(!display_checked(&a, "行番号"));
    assert!(!display_checked(&a, "一行おきの色"));
    assert!(!display_checked(&a, "列の区切り線"));
    assert!(display_checked(&a, "タブ"));
    assert!(display_checked(&a, "検索の欄"));
    assert!(display_checked(&a, "設定の帯"));
    for l in s.lines() {
        assert!(width::width(l) <= 80, "幅を超える: {l}");
    }
    // 設定で行番号を出していれば、画面の項目も入っている。
    let (_t, mut b) = folder_with(
        "sr20sec2",
        "[display]\nrow_numbers = true\n",
        ColorMode::None,
    );
    open(&mut b);
    display_section(&mut b);
    assert!(display_checked(&b, "行番号"));
}

#[test]
fn test_sr_20_toggle_in_settings_applies_to_that_view_only() {
    // [SR-20][NV-13] 設定で番号を出す。ビュー「全部」の設定の画面で行番号を消す → 反映の前は変わらず、
    // 反映で「全部」だけ番号が無い。「進行」には番号が残る。取り消しなら変わらない。
    let tmp = vault("sr20view");
    let mut a = boot_base(
        &tmp,
        cfg("[display]\nrow_numbers = true\n"),
        ColorMode::None,
    );
    assert!(tab_line(&a).contains("[全部]"), "{}", screen(&a));
    assert_numbered(&a);

    // 取り消し(Esc)は変えない。
    open(&mut a);
    assert!(!toggle_display(&mut a, "行番号"));
    press(&mut a, KeyCode::Esc);
    settle(&mut a);
    assert_numbered(&a);

    // 反映で、このビューだけ消える(設定の画面は全画面を覆うので、表は画面を閉じてから見る。
    // 反映の前に表を変えないことは、上の取り消しで確かめた)。
    open(&mut a);
    assert!(!toggle_display(&mut a, "行番号"));
    apply_btn(&mut a);
    assert_unnumbered(&a);
    goto_tab(&mut a, "進行");
    assert_numbered(&a);
    goto_tab(&mut a, "全部");
    assert_unnumbered(&a);
}

#[test]
fn test_sr_20_saved_native_view_keeps_display() {
    // [SR-20][BV-17] 設定で番号を出す。設定の画面で行番号を消し、mdgrid のビュー「番号なし」として保存 →
    // そのビューだけ番号が無い。終了して開き直しても同じ。
    let tmp = vault("sr20save");
    let config = "[display]\nrow_numbers = true\n";
    let mut a = boot_folder(&tmp, cfg(config));
    assert_numbered(&a);
    open(&mut a);
    assert!(!toggle_display(&mut a, "行番号"));
    click_label(&mut a, "名前を付けて保存");
    for _ in 0..40 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "番号なし");
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    goto_tab(&mut a, "番号なし");
    assert_unnumbered(&a);
    goto_tab(&mut a, "既定の表");
    assert_numbered(&a);
    settle(&mut a);
    ch(&mut a, 'q');
    assert!(a.quit, "終了できる: {}", screen(&a));

    let mut b = boot_folder(&tmp, cfg(config));
    goto_tab(&mut b, "番号なし");
    assert_unnumbered(&b);
    goto_tab(&mut b, "既定の表");
    assert_numbered(&b);
    // 設定の画面の項目も保存した値。
    goto_tab(&mut b, "番号なし");
    open(&mut b);
    display_section(&mut b);
    assert!(!display_checked(&b, "行番号"));
    assert!(display_checked(&b, "検索の欄"), "ほかの項目は設定のまま");
}

// ---- SR-20・SR-15: 一行おきの色 ----

/// 表の k 行目(0 から)の画面の y。
fn row_y(a: &App, k: usize) -> u16 {
    (view::data_y(a) + k) as u16
}

/// 比べる桁(名前の列の頭・1つ目と2つ目の値の列の頭)。どれも字の始まる桁(全角の字の右半分の隠れた
/// セルは ratatui が style を当てないので比べない)。
fn probe_xs(a: &App) -> Vec<u16> {
    let (lay, cols) = view::visible_layout(a);
    let mut xs = vec![1u16, 2];
    let mut x = lay.data_x();
    for &(_, w) in cols.iter().take(2) {
        xs.push(x as u16);
        x += w + 1;
    }
    xs
}

#[test]
fn test_sr_20_zebra_colors_even_rows() {
    // [SR-20] 一行おきの色: 2行目・4行目(偶数の行)の背景の色が、3行目・5行目(奇数の行)と違う。
    // 1行目は選んだ行なので比べない。
    let (_t, a) = folder_with("sr20zebra", "[display]\nzebra = true\n", ColorMode::Indexed);
    assert_eq!(a.row, 0);
    let buf = buffer(&a);
    for x in probe_xs(&a) {
        let even2 = buf[(x, row_y(&a, 1))].bg;
        let odd3 = buf[(x, row_y(&a, 2))].bg;
        let even4 = buf[(x, row_y(&a, 3))].bg;
        let odd5 = buf[(x, row_y(&a, 4))].bg;
        assert_ne!(even2, odd3, "x={x}: 2行目と3行目の背景が同じ");
        assert_eq!(even2, even4, "x={x}: 偶数の行どうしは同じ背景");
        assert_eq!(odd3, odd5, "x={x}: 奇数の行どうしは同じ背景");
    }
}

#[test]
fn test_sr_20_zebra_off_by_default() {
    // [SR-20] 既定(zebra = false)では、偶数と奇数の行の背景は同じ。
    let (_t, a) = folder_with("sr20zebra0", "", ColorMode::Indexed);
    let buf = buffer(&a);
    for x in probe_xs(&a) {
        assert_eq!(
            buf[(x, row_y(&a, 1))].bg,
            buf[(x, row_y(&a, 2))].bg,
            "x={x}"
        );
    }
}

#[test]
fn test_sr_20_zebra_not_without_color() {
    // [SR-20][SR-15] 色を使わない表示(`--no-color`・NO_COLOR = ColorMode::None、設定 color = false)では
    // 一行おきの色を付けない: 偶数と奇数の行の style(背景・前景・飾り)は同じ。
    let check = |a: &App| {
        let buf = buffer(a);
        for x in probe_xs(a) {
            let even = &buf[(x, row_y(a, 1))];
            let odd = &buf[(x, row_y(a, 2))];
            assert_eq!(even.bg, odd.bg, "x={x}: 背景");
            assert_eq!(even.fg, odd.fg, "x={x}: 前景");
            assert_eq!(even.modifier, odd.modifier, "x={x}: 飾り");
        }
    };
    let (_t, a) = folder_with("sr20zebranc", "[display]\nzebra = true\n", ColorMode::None);
    check(&a);
    let (_t, b) = folder_with(
        "sr20zebranc2",
        "color = false\n\n[display]\nzebra = true\n",
        ColorMode::Indexed,
    );
    check(&b);
    // `--no-color` の起動の道筋でも同じ。
    let tmp = vault("sr20zebranc3");
    let p = base_path(&tmp);
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut c = App::new(Box::new(t.src), ColorMode::Rgb);
    c.resize(80, 24);
    c.start(Startup {
        config: cfg("[display]\nzebra = true\n"),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: None,
        config_dir: None,
        target: p,
        base: t.base,
    });
    while !c.loaded() {
        c.load_step(100);
    }
    goto_tab(&mut c, "進行");
    check(&c);
}

// ---- SR-20: 列の区切り線 ----

#[test]
fn test_sr_20_column_lines_default_none_on_draws_bars() {
    // [SR-20][SR-21] 列の区切り線(既定 column_lines = false): 既定・false は今の見た目のまま(列の間は空白で
    // `│` が無い。golden sr_1 と同じ)。true → 列の見出しの行と各ノートの行の列の間に `│`(どの行でも同じ桁)。
    let notes = [
        (
            "a.md",
            "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n本文\n",
        ),
        ("b.md", "---\ntitle: 買い物\nstatus: 完了\n---\n"),
        ("c.md", "---\ntitle: 読書\ntags: [本, 秋]\n---\n"),
    ];
    // 表の部分(列の見出しの行とノートの行)。
    let table = |a: &App| -> Vec<String> {
        let s = screen(a);
        let dy = view::data_y(a);
        s.lines()
            .skip(dy - 1)
            .take(a.rows.len() + 1)
            .map(str::to_string)
            .collect()
    };

    // 既定(書かない)と false は今の見た目。
    for (name, config) in [
        ("sr20lines0", ""),
        ("sr20linesf", "[display]\ncolumn_lines = false\n"),
    ] {
        let (_t, mut a) = make(name, &notes);
        a.configure(&cfg(config));
        for l in table(&a) {
            assert!(
                !l.contains('│'),
                "{config:?} で区切り線: {l}\n{}",
                screen(&a)
            );
        }
        golden("sr_1", &screen(&a));
    }

    // true は列の間に `│`。
    let (_t, mut on) = make("sr20lines1", &notes);
    on.configure(&cfg("[display]\ncolumn_lines = true\n"));
    let lines = table(&on);
    let s = screen(&on);
    assert!(lines[0].contains("ノート"), "1行目は列の見出し:\n{s}");
    let (_, cols) = view::visible_layout(&on);
    assert_eq!(cols.len(), 3, "title・status・tags が見える:\n{s}");
    // 書記素ごとの桁で `│` の位置を取る。
    let bars = |l: &str| -> Vec<usize> {
        let mut x = 0;
        let mut out = Vec::new();
        for g in unicode_segmentation::UnicodeSegmentation::graphemes(l, true) {
            if g == "│" {
                out.push(x);
            }
            x += width::width(g);
        }
        out
    };
    let head = bars(&lines[0]);
    // ノートの列と title・title と status・status と tags の間(右端の後ろは問わない)。
    assert!(
        head.len() >= cols.len(),
        "列の見出しの行の列の間に `│`: {:?}\n{s}",
        lines[0]
    );
    for l in &lines[1..] {
        assert_eq!(bars(l), head, "ノートの行も同じ桁に `│`: {l:?}\n{s}");
    }
    // 値はそのまま見える。
    assert!(
        s.contains("会議のメモ") && s.contains("進行中") && s.contains("[本, 秋]"),
        "{s}"
    );
    for l in s.lines() {
        assert!(width::width(l) <= 80, "幅を超える: {l}");
    }
}

// ---- SR-20・NV-23: 検索の欄 ----

#[test]
fn test_sr_20_search_bar_hidden_from_settings_screen() {
    // [SR-20][NV-23] 設定の画面の「表示」で検索の欄を隠して反映 → 欄が無く表が1行広い。`\` は最下行で打てる。
    let (_t, mut a) = folder_with("sr20bar", "", ColorMode::None);
    let h_on = a.data_height();
    assert!(screen(&a).contains("検索:"));
    open(&mut a);
    assert!(!toggle_display(&mut a, "検索の欄"));
    apply_btn(&mut a);
    assert_eq!(a.data_height(), h_on + 1, "表が1行広い");
    let s = screen(&a);
    assert!(!s.contains("検索:"), "{s}");
    assert!(s.lines().nth(2).unwrap().contains("ノート"), "{s}");
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    assert_eq!(a.mode, Mode::Filter);
    assert_eq!(a.rows.len(), 2);
    let s = screen(&a);
    assert!(s.lines().nth(22).unwrap().starts_with("\\会議"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 2);
}

// ---- SR-20: タブ ----

#[test]
fn test_sr_20_tabs_hidden_but_brackets_switch() {
    // [SR-20][SR-21] 設定 `[display] tabs = false` → タブの行が無く、表が1行広い。`]` でビューは切り替わる。
    let tmp = vault("sr20tabs");
    let on = boot_base(&tmp, cfg(""), ColorMode::None);
    let h_on = on.data_height();
    let rows_all = labels(&on);
    drop(on);
    let mut a = boot_base(&tmp, cfg("[display]\ntabs = false\n"), ColorMode::None);
    assert_eq!(a.data_height(), h_on + 1, "表が1行広い");
    let s = screen(&a);
    assert!(
        !s.contains("[全部]") && !s.contains("進行"),
        "タブが無い:\n{s}"
    );
    assert!(
        s.lines().nth(1).unwrap().starts_with(" 検索:"),
        "検索の欄が1行上がる:\n{s}"
    );
    assert!((0..80u16).all(|x| view::tab_at(&a, x, 1).is_none()));
    assert_eq!(labels(&a), rows_all);
    // `]` で「進行」(doing を外す)に切り替わる。
    ch(&mut a, ']');
    let rows = labels(&a);
    assert_eq!(rows.len(), 5, "{rows:?}");
    assert!(!rows.contains(&"e.md".to_string()), "{rows:?}");
    // `[` で戻る。
    ch(&mut a, '[');
    assert_eq!(labels(&a), rows_all);
}

// ---- SR-20・NV-16: 設定の帯 ----

#[test]
fn test_sr_20_chips_hidden_but_f_works() {
    // [SR-20][NV-16][NV-22] 設定 `[display] chips = false` → 設定が効いていても帯が無い(表は下がらない)。
    // 帯の操作 `f` は効く: 帯の項目を選ぶモードに入り、Backspace で条件を外せる。
    let (_t, mut a) = folder_with("sr20chips", "[display]\nchips = false\n", ColorMode::None);
    let dy = view::data_y(&a);
    let h = a.data_height();
    a.settings.sorts.push(("priority".into(), Dir::Desc));
    a.refresh();
    assert!(!a.settings.is_default());
    let s = screen(&a);
    assert!(!s.contains("設定:"), "帯が無い:\n{s}");
    assert_eq!(view::data_y(&a), dy, "表は下がらない");
    assert_eq!(a.data_height(), h);
    assert!(s.lines().nth(dy - 1).unwrap().contains("ノート"), "{s}");
    ch(&mut a, 'f');
    assert_eq!(a.mode, Mode::Chips, "f は効く: {:?}", a.message);
    press(&mut a, KeyCode::Backspace);
    assert!(a.settings.is_default(), "帯が無くても条件を外せる");
    settle(&mut a);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_20_chips_hidden_from_settings_screen() {
    // [SR-20][NV-16] 設定の画面の「表示」で設定の帯を隠して反映 → 効いている設定があっても帯が無い。
    let (_t, mut a) = folder_with("sr20chips2", "", ColorMode::None);
    a.settings.sorts.push(("priority".into(), Dir::Desc));
    a.refresh();
    assert!(screen(&a).contains("設定:"));
    open(&mut a);
    assert!(!toggle_display(&mut a, "設定の帯"));
    apply_btn(&mut a);
    assert!(!a.settings.sorts.is_empty(), "並べ替えはそのまま");
    assert!(!screen(&a).contains("設定:"), "{}", screen(&a));
}
