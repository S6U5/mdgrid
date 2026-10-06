//! その場の操作の一覧(SR-24)・前置きのキーの案内(SR-25)・その英語の画面(SR-23)の、照合で見つけた
//! 試験の欠けを埋める試験。記録: specs/_changes/2026-10-05-action-menu.md のタスク #2。
//! 道具は test_action_menu のもの(窓の枠の内側の桁だけを一覧の行として読む)。

use super::keymap::{self, Action, Mode};
use super::startup::Startup;
use super::test_action_menu::{band, has_section, menu_lines, menu_mode, shot, Shot, NOTES};
use super::test_screen::{ch, col_named, make, screen, Tmp};
use super::*;
use mdgrid::config;
use mdgrid::i18n::{scoped, Lang, Msg};
use mdgrid::source::markdown::Markdown;

/// 英字だけの保管庫(英語の画面の試験)。
const EN_NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: meeting\nstatus: doing\n---\n"),
    ("b.md", "---\ntitle: shopping\nstatus: done\n---\n"),
];

/// main と同じ道筋(`App::new` → `start` → 読み込み)で、設定 `toml` と `--readonly` の有無で開く。
fn boot(name: &str, notes: &[(&str, &str)], toml: &str, readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let (config, warnings) = config::parse(toml).expect("読める設定");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings,
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// 行 `row` の status のセルを選んで `x` で一覧を開く(100×50)。一覧の行(窓の枠の内側)と開いた後の画面。
fn open_lines(a: &mut App, row: usize) -> (Vec<String>, Shot) {
    a.row = row;
    col_named(a, "status");
    let before = shot(a, 100, 50);
    ch(a, 'x');
    let after = shot(a, 100, 50);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない:\n{after}");
    (menu_lines(&before, &after), after)
}

/// 節「ビューとファイル」の項目(SR-24 の文の順)。
fn view_items() -> [&'static str; 4] {
    [
        Msg::KeyNewNote.text(),
        Msg::KeyViewSettings.text(),
        Msg::CmdExportBase.text(),
        Msg::KeySave.text(),
    ]
}

/// 節の見出し `head` から次の見出し(または末尾)までの行。
fn section_rows<'a>(lines: &'a [String], head: &str, heads: &[&str]) -> Vec<&'a String> {
    let bare = |l: &str| {
        l.trim_matches(|c: char| c.is_whitespace() || "|+-".contains(c))
            .to_string()
    };
    let Some(at) = lines.iter().position(|l| bare(l) == head) else {
        return Vec::new();
    };
    lines[at + 1..]
        .iter()
        .take_while(|l| !heads.contains(&bare(l).as_str()))
        .collect()
}

// ---- SR-24: 節「ビューとファイル」 ----

fn check_view_section(lang: &str, notes: &[(&str, &str)]) {
    let (_t, mut a) = make(&format!("sr24gapsview{lang}"), notes);
    let (lines, after) = open_lines(&mut a, 0);
    let head = Msg::MenuSecView.text();
    assert!(
        has_section(&lines, head),
        "節「{head}」の見出しが無い:\n{after}"
    );
    let heads = [
        Msg::MenuSecCell.text(),
        Msg::MenuSecColumn.text(),
        Msg::MenuSecRow.text(),
        head,
    ];
    let rows = section_rows(&lines, head, &heads);
    for item in view_items() {
        assert!(
            rows.iter().any(|l| l.contains(item)),
            "節「{head}」に「{item}」が無い:\n{after}"
        );
    }
}

#[test]
fn test_sr_24_gaps_view_section_ja() {
    // [SR-24] status のセルで `x` → 節「ビューとファイル」の見出しがあり、その下に
    // 新しいノート・ビューの設定・.base に書き出す・保存が並ぶ。
    check_view_section("ja", NOTES);
}

#[test]
fn test_sr_24_gaps_view_section_en() {
    // [SR-24] [SR-23] 英語の画面では節の見出しが "View and file" で、その下に New note・View settings・
    // Export to .base・Save が並ぶ。
    let _g = scoped(Lang::En);
    assert_eq!(Msg::MenuSecView.text(), "View and file");
    check_view_section("en", EN_NOTES);
}

// ---- SR-24: 読むだけの起動 ----

#[test]
fn test_sr_24_gaps_readonly_launch_hides_save_new_export() {
    // [SR-24] [WB-15] 読むだけの起動(--readonly)で `x` → 一覧に保存・新しいノート・.base に書き出すが無い
    // (ビューの設定は残る)。対照: 読むだけでない起動では3つともある。
    let [new_note, view, export, save] = view_items();

    let (_t, mut a) = boot("sr24gapsrw", NOTES, "", false);
    assert!(!a.readonly, "材料: 読むだけになった");
    let (lines, after) = open_lines(&mut a, 0);
    for item in [save, new_note, export, view] {
        assert!(
            lines.iter().any(|l| l.contains(item)),
            "読むだけでない起動で「{item}」が無い:\n{after}"
        );
    }

    let (_t, mut a) = boot("sr24gapsro", NOTES, "", true);
    assert!(a.readonly, "材料: --readonly で読むだけにならない");
    let (lines, after) = open_lines(&mut a, 0);
    assert!(
        lines.iter().any(|l| l.contains(view)),
        "材料: 読むだけの起動で一覧が出ていない:\n{after}"
    );
    for item in [save, new_note, export] {
        assert!(
            !lines.iter().any(|l| l.contains(item)),
            "読むだけの起動で「{item}」がある:\n{after}"
        );
    }
}

// ---- SR-25: 一覧の上の前置き ----

#[test]
fn test_sr_25_gaps_menu_ignores_prefix_without_menu_item() {
    // [SR-25] [SR-24] 設定で `[keys.table] "z z" = "bottom"` を足す。表の上では `z` で帯に「z →」の案内が出る。
    // 一覧の上では、末尾の行は一覧の項目に無いので `z` は前置きとして受けない: 一覧のまま、帯に「z →」が出ず、
    // `z` `z` で末尾の行へ動かない。
    let toml = "[keys.table]\n\"z z\" = \"bottom\"\n";
    let (_t, mut a) = boot("sr25gapsz", NOTES, toml, false);
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "z z"),
        Some(Action::Bottom),
        "材料: 設定の z z が bottom にならない"
    );
    let hint = format!("{} →", keymap::display("z"));

    // 対照: 表の上
    ch(&mut a, 'z');
    let b = band(&screen(&a));
    assert!(b.contains(&hint), "表で z のあと帯に「{hint}」が無い: {b}");
    ch(&mut a, 'z');
    assert_eq!(
        a.row,
        a.rows.len() - 1,
        "材料: 表で z z で末尾の行へ動かない"
    );

    // 一覧の上
    a.row = 0;
    col_named(&mut a, "status");
    ch(&mut a, 'x');
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    ch(&mut a, 'z');
    assert_eq!(a.mode, menu_mode(), "一覧で z のあと一覧が閉じた");
    let b = band(&screen(&a));
    assert!(
        !b.contains(&hint),
        "一覧で z のあと帯に「{hint}」が出た: {b}"
    );
    ch(&mut a, 'z');
    assert_eq!(a.row, 0, "一覧で z z で動いた");
    assert_eq!(a.mode, menu_mode(), "一覧で z z のあと一覧が閉じた");
}

// ---- SR-25・SR-23: 英語の前置きの案内 ----

#[test]
fn test_sr_25_gaps_english_prefix_hint() {
    // [SR-25] [SR-23] 英語の画面で、表で `g` → 下の帯が「g →」と英語の表示名 "First row" を含む。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr25gapsen", EN_NOTES);
    ch(&mut a, 'g');
    let b = band(&screen(&a));
    assert!(
        b.contains("g →"),
        "英語の画面で g のあと帯に「g →」が無い: {b}"
    );
    assert!(
        b.contains("First row"),
        "英語の画面で g のあと帯に \"First row\" が無い: {b}"
    );
}

// ---- SR-24: 一覧を開いたまま端末が低くなる ----

#[test]
fn test_sr_24_gaps_resize_low_closes_menu() {
    // [SR-24] 一覧を開いたまま端末を低くして(80×8)窓が入らなくなる → 一覧を閉じて表に戻り、開くときと
    // 同じ理由をメッセージ行に出す。そのあとのキー(`j`)は表のキーとして効く(見えない一覧に取られない)。
    let (_t, mut a) = make("sr24gapslow", NOTES);
    a.row = 0;
    col_named(&mut a, "status");
    ch(&mut a, 'x');
    assert_eq!(a.mode, menu_mode(), "80×24 で x で一覧が開かない");
    a.resize(80, 8);
    assert_eq!(a.mode, Mode::Table, "80×8 にしても一覧のモードのまま");
    assert_eq!(
        a.message.as_deref(),
        Some(Msg::MenuNoRoom.text()),
        "低くしたときに理由が出ない"
    );
    let s = shot(&mut a, 80, 8);
    let head: String = Msg::MenuNoRoom.text().chars().take(6).collect();
    assert!(s.text.contains(&head), "メッセージ行に理由が無い:\n{s}");
    ch(&mut a, 'j');
    assert_eq!(a.row, 1, "閉じたあとの j が表で効かない");
}

#[test]
fn test_sr_24_gaps_resize_still_fits_keeps_menu() {
    // [SR-24] 対照: 一覧を開いたまま端末の大きさが変わっても、窓が入るうち(100×50)は一覧のまま。
    let (_t, mut a) = make("sr24gapsfit", NOTES);
    col_named(&mut a, "status");
    ch(&mut a, 'x');
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    a.resize(100, 50);
    assert_eq!(a.mode, menu_mode(), "窓が入るのに一覧が閉じた");
    assert_eq!(a.message, None);
}
