//! 画面の文言の言語(SR-23)の受け入れ試験(タスク4: 残りの画面のメッセージと確認の画面)。
//! 記録: specs/_changes/2026-10-03-language.md。
//! 画面の実装を見ずに、`mdgrid::i18n::scoped` でこのスレッドだけ言語を変え、80×24 の画面の文字で確かめる。
//! 試験の環境は .cargo/config.toml で日本語に固定されている。
//!
//! 保管庫の値・列の名前・ファイル名・ビューの名前は英字だけで作るので、英語では画面のどの行にも
//! 日本語の文字が出ないはず(見る行は、確認の画面・入力・選択・詳細・設定の画面は画面の全部、
//! メッセージは下の帯とメッセージ行、タブはタブの行に絞る)。

use super::keymap::Mode;
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, edit_cell, make, press, screen, typing, Tmp};
use super::test_settings_screen::{add_values, apply_btn, draft, open, section};
use super::*;
use mdgrid::config::Config;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::settings::{Cond, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use mdgrid::views::{save_views, NativeView};
use ratatui::crossterm::event::KeyCode;

const TODAY: &str = "2026-10-02";

/// 下の帯(添字 21)とメッセージ行(添字 22)。
const BAND: usize = 21;
const MSG: usize = 22;

/// 英字だけの保管庫の値。
const EN_NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: meeting\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: shopping\nstatus: done\n---\n"),
];

/// 英字だけの、ビューの設定の材料(status は todo・done・doing・無し)。
const EN_VAULT: &[(&str, &str)] = &[
    (
        "a.md",
        "---\ntitle: prep\nstatus: todo\nkind: meeting\npriority: 3\n---\n",
    ),
    (
        "b.md",
        "---\ntitle: read\nstatus: done\nkind: book\npriority: 1\n---\n",
    ),
    (
        "c.md",
        "---\ntitle: next\nstatus: todo\nkind: book\npriority: 5\n---\n",
    ),
    ("d.md", "---\ntitle: memo\nstatus: done\nkind: memo\n---\n"),
    (
        "e.md",
        "---\ntitle: log\nstatus: doing\nkind: meeting\npriority: 4\n---\n",
    ),
    ("f.md", "---\ntitle: shelf\nkind: book\n---\n"),
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

fn line(s: &str, y: usize) -> String {
    s.lines().nth(y).unwrap_or("").to_string()
}

/// 画面の全部の行に日本語の文字が無い。
fn assert_screen_no_ja(what: &str, a: &App) {
    let s = screen(a);
    for (y, l) in s.lines().enumerate() {
        assert!(
            !has_ja(l),
            "{what}: {} 行目に日本語の文字が残っている: {l:?}\n{s}",
            y + 1
        );
    }
}

/// 下の帯とメッセージ行に日本語の文字が無い(メッセージ行は空でない)。
fn assert_message_no_ja(what: &str, a: &App) {
    let s = screen(a);
    let m = line(&s, MSG);
    assert!(!m.trim().is_empty(), "{what}: メッセージ行が空\n{s}");
    for y in [BAND, MSG] {
        let l = line(&s, y);
        assert!(
            !has_ja(&l),
            "{what}: {} 行目に日本語の文字が残っている: {l:?}\n{s}",
            y + 1
        );
    }
    let msg = a.message.clone().unwrap_or_default();
    assert!(
        !has_ja(&msg),
        "{what}: メッセージに日本語の文字が残っている: {msg:?}"
    );
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

/// main と同じ道筋で `.base` なしのフォルダを開く(状態と設定の置き場は一時フォルダの下)。
fn boot(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.today = types::parse_date(TODAY).unwrap();
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
    app.today = types::parse_date(TODAY).unwrap();
    app
}

fn vault_of(name: &str, notes: &[(&str, &str)]) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    tmp
}

/// 入力の文字を全部消す。
fn wipe(a: &mut App) {
    press(a, KeyCode::End);
    let n = a.input.as_ref().unwrap().text.chars().count();
    for _ in 0..n {
        press(a, KeyCode::Backspace);
    }
}

/// ビューの設定の画面で、今の選び(選び手が開いていればその中、無ければ区画の中)を `to` に動かす。
fn sel_to(a: &mut App, to: usize) {
    loop {
        let d = draft(a);
        let now = match &d.pick {
            Some(p) => p.sel(),
            None => d.at(d.sec),
        };
        if now == to {
            return;
        }
        press(a, if now < to { KeyCode::Down } else { KeyCode::Up });
        let d = draft(a);
        let after = match &d.pick {
            Some(p) => p.sel(),
            None => d.at(d.sec),
        };
        assert_ne!(after, now, "{to} まで動かせない");
    }
}

// ---- 1. 保存の確認 ----

#[test]
fn test_sr_23_ui_more_save_confirm_english() {
    // [SR-23] 英語では、セルを直して Ctrl+S で開く保存の確認(差分)の画面と、保存したあとのメッセージに日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr23save", EN_NOTES);
    edit_cell(&mut a, 0, "status", "doing");
    edit_cell(&mut a, 1, "title", "groceries");
    assert_eq!(a.changes.count(), 2);
    assert_screen_no_ja("ためた変更のある表", &a);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    let s = screen(&a);
    assert!(s.contains("+status: doing"), "{s}");
    assert_screen_no_ja("保存の確認の画面", &a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    assert_message_no_ja("保存したあと", &a);
    // 変更が無いときの Ctrl+S のメッセージも。
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table);
    assert_message_no_ja("保存する変更が無い", &a);
}

// ---- 2. 終了の確認 ----

#[test]
fn test_sr_23_ui_more_quit_confirm_english() {
    // [SR-23] 英語では、未保存の変更があるときの `q` の終了の確認に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("sr23quit", EN_NOTES);
    edit_cell(&mut a, 0, "status", "doing");
    ch(&mut a, 'q');
    assert!(!a.quit);
    assert_eq!(a.mode, Mode::Quit);
    assert_screen_no_ja("終了の確認", &a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 1);
}

// ---- 3. セルの編集の入力の案内 ----

#[test]
fn test_sr_23_ui_more_input_hints_english() {
    // [SR-23] 英語では、数・日付・テキストのセルの入力の案内と、読めない値の理由に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = date_vault(
        "sr23input",
        &[
            ("a.md", "---\ntitle: meeting\nn: 5\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ntitle: shopping\nn: 7\ndue: 2026-09-01\n---\n"),
        ],
    );

    // 数。
    col_named(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_screen_no_ja("数の入力", &a);
    wipe(&mut a);
    typing(&mut a, "12a");
    assert_screen_no_ja("数の自由入力", &a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "読めない数では閉じない");
    assert_message_no_ja("数として読めない", &a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);

    // 日付。
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(screen(&a).contains("YYYY-MM-DD"), "{}", screen(&a));
    assert_screen_no_ja("日付の入力", &a);
    wipe(&mut a);
    typing(&mut a, "2026-02-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "実在しない日付では閉じない");
    assert_message_no_ja("実在しない日付", &a);
    wipe(&mut a);
    typing(&mut a, "soon");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "読めない日付では閉じない");
    assert_message_no_ja("日付として読めない", &a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);

    // テキスト(候補のリストのあと、打つと自由入力)。
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_screen_no_ja("テキストの入力", &a);
    typing(&mut a, "x");
    assert_screen_no_ja("テキストの自由入力", &a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 1);
}

// ---- 4. リストの選択 ----

#[test]
fn test_sr_23_ui_more_list_pick_english() {
    // [SR-23] 英語では、tags の列で開くリストの選択(候補・件数・新規・案内)に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make(
        "sr23pick",
        &[
            ("a.md", "---\ntitle: one\ntags: [alpha, beta]\n---\n"),
            ("b.md", "---\ntitle: two\ntags: [alpha]\n---\n"),
            ("c.md", "---\ntitle: three\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
    assert_screen_no_ja("リストの選択", &a);
    typing(&mut a, "gam");
    assert_screen_no_ja("リストの選択の新規", &a);
    press(&mut a, KeyCode::Esc);
}

// ---- 5. 詳細の表示 ----

#[test]
fn test_sr_23_ui_more_detail_english() {
    // [SR-23] 英語では、`K` の詳細の表示の見出しと行に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make(
        "sr23detail",
        &[
            (
                "a.md",
                "---\ntitle: meeting\nstatus: doing\nnote: long text\n---\n",
            ),
            ("b.md", "---\ntitle: shopping\nextra: x\n---\n"),
        ],
    );
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    assert_screen_no_ja("詳細の表示", &a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- 6. ビューの設定の画面 ----

#[test]
fn test_sr_23_ui_more_view_settings_english() {
    // [SR-23] 英語では、`o`(docs/keys.md の view_settings)のビューの設定の画面・列の選び手・値の一覧・
    // 反映のあとの表の上の帯とメッセージに日本語の文字が無い。
    let _g = scoped(Lang::En);
    let tmp = vault_of("sr23set", EN_VAULT);
    let mut a = boot(&tmp);
    open(&mut a);
    assert_screen_no_ja("ビューの設定の画面", &a);
    for sec in [
        Sec::Columns,
        Sec::Filters,
        Sec::Sorts,
        Sec::Group,
        Sec::Buttons,
    ] {
        section(&mut a, sec);
        assert_screen_no_ja(&format!("ビューの設定の区画 {sec:?}"), &a);
    }

    // フィルターの条件を足す: 列の選び手 → 条件の種類 → 値の一覧。
    section(&mut a, Sec::Filters);
    let last = draft(&a).len(Sec::Filters) - 1;
    sel_to(&mut a, last);
    press(&mut a, KeyCode::Enter);
    assert!(draft(&a).pick.is_some(), "列の選び手が開く");
    assert_screen_no_ja("列の選び手", &a);
    let i = draft(&a).keys.iter().position(|c| c == "status").unwrap();
    sel_to(&mut a, i);
    press(&mut a, KeyCode::Enter);
    assert_screen_no_ja("条件の種類", &a);
    sel_to(&mut a, 0);
    press(&mut a, KeyCode::Enter);
    assert!(
        matches!(draft(&a).pick, Some(Pick::Values { .. })),
        "値の一覧が開く"
    );
    assert_screen_no_ja("値の一覧", &a);
    // 選び手の Esc は1回で選び手を全部閉じ、設定の画面に戻る。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Settings);
    assert!(draft(&a).pick.is_none(), "選び手が閉じる");
    assert_screen_no_ja("選び手を閉じたあとの設定の画面", &a);
    // もう1回の Esc で設定を取り消して表に戻る。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_message_no_ja("ビューの設定を取り消した", &a);

    // 条件を足して反映 → 設定の画面の条件の文・表の上の帯・メッセージ。
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    assert_screen_no_ja("条件のあるビューの設定の画面", &a);
    apply_btn(&mut a);
    assert_screen_no_ja("反映のあとの表", &a);
    assert_message_no_ja("ビューの設定を反映した", &a);
}

// ---- 7. 新しいノートの名前の欄と作れない理由 ----

#[test]
fn test_sr_23_ui_more_new_note_name_english() {
    // [SR-23] 英語では、`a` で開く新しいノートの名前の欄と、`../x` で作れない理由に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let tmp = vault_of("sr23note", EN_NOTES);
    let mut a = boot(&tmp);
    ch(&mut a, 'a');
    assert_ne!(a.mode, Mode::Table, "名前の欄が開く\n{}", screen(&a));
    assert_screen_no_ja("新しいノートの名前の欄", &a);
    typing(&mut a, "../x");
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::Table, "作れなかったあとも欄は開いたまま");
    assert!(!tmp.0.join("x.md").exists(), "外に作らない");
    assert_message_no_ja("作れない理由", &a);
    assert_screen_no_ja("作れなかったあとの名前の欄", &a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

// ---- 8. 読むだけのセルの理由 ----

#[test]
fn test_sr_23_ui_more_read_only_reason_english() {
    // [SR-23] 英語では、読むだけのセル(2行にまたがるフローのリスト)を選んだときの下の帯・メッセージ行の理由と、
    // そこで Backspace を押したときの理由に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make(
        "sr23ro",
        &[
            ("a.md", "---\ntags: [a,\n  b]\n---\n"),
            ("b.md", "---\ntags: [a, b]\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    let s = screen(&a);
    assert!(!line(&s, MSG).trim().is_empty(), "理由が出る\n{s}");
    for y in [BAND, MSG] {
        let l = line(&s, y);
        assert!(!has_ja(&l), "{} 行目: {l:?}\n{s}", y + 1);
    }
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 0);
    assert_message_no_ja("空にできない", &a);
}

// ---- 9. 既定の表のタブ ----

#[test]
fn test_sr_23_ui_more_default_tab_english() {
    // [SR-23] 英語では、mdgrid のビューがあるときのタブの「既定の表」に当たる名前に日本語の文字が無い。
    let _g = scoped(Lang::En);
    let tmp = vault_of("sr23tab", EN_NOTES);
    let conf = tmp.0.join("config");
    std::fs::create_dir_all(&conf).unwrap();
    let view = NativeView {
        name: "todo".into(),
        settings: Settings {
            filters: vec![Cond {
                col: "status".into(),
                op: Op::Keep(vec![Some("todo".into())]),
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    save_views(&conf, &tmp.notes(), &[view]).unwrap();
    let mut a = boot(&tmp);
    let s = screen(&a);
    let tabs = line(&s, 1);
    assert!(
        tabs.contains("todo"),
        "タブに mdgrid のビュー: {tabs:?}\n{s}"
    );
    let before_todo = &tabs[..tabs.find("todo").unwrap()];
    assert!(
        before_todo.chars().any(|c| c.is_ascii_alphabetic()),
        "既定の表のタブが英語の名前で先に並ぶ: {tabs:?}"
    );
    assert!(!has_ja(&tabs), "タブの行: {tabs:?}\n{s}");
    // 切り替えてもタブの行に日本語が無い(知らせはタブを隠しているときだけ出る。SR-20)。
    ch(&mut a, ']');
    let tabs2 = line(&screen(&a), 1);
    assert!(tabs2.contains("[todo]"), "{tabs2:?}");
    assert!(!has_ja(&tabs2), "todo を選んだタブの行: {tabs2:?}");
    ch(&mut a, '[');
    let tabs3 = line(&screen(&a), 1);
    assert!(!tabs3.contains("[todo]"), "既定の表に戻る: {tabs3:?}");
    assert!(
        tabs3
            .trim_start()
            .strip_prefix('[')
            .and_then(|t| t.chars().next())
            .is_some_and(|c| c.is_ascii_alphabetic()),
        "既定の表のタブが英語の名前で選ばれている: {tabs3:?}"
    );
    assert!(!has_ja(&tabs3), "既定の表に戻ったタブの行: {tabs3:?}");
}

// ---- 10. 日本語は今のまま ----

#[test]
fn test_sr_23_ui_more_japanese_unchanged() {
    // [SR-23] 同じ画面を日本語で開くと今の日本語のまま(終了の確認と、ビューの設定の画面)。
    let _g = scoped(Lang::Ja);

    // 終了の確認(WB-11 と同じ文字)。
    let (_t, mut a) = make("sr23jaquit", EN_NOTES);
    edit_cell(&mut a, 0, "status", "doing");
    ch(&mut a, 'q');
    assert_eq!(a.mode, Mode::Quit);
    let s = screen(&a);
    assert!(
        s.contains("s 保存する") && s.contains("d 捨てて終わる") && s.contains("Esc 戻る"),
        "{s}"
    );
    press(&mut a, KeyCode::Esc);

    // ビューの設定の画面(NV-18 と同じ文字)。
    let tmp = vault_of("sr23jaset", EN_VAULT);
    let mut a = boot(&tmp);
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    let s = screen(&a);
    // NV-18: 左に区画の一覧(列・フィルター・並べ替え・グループ)、右に選んだ区画(フィルター)。
    for want in [
        "列",
        "フィルター",
        "並べ替え",
        "グループ",
        "status: done を除く",
        "[ 反映 ]",
        "[ 取り消し ]",
        "[ 既定に戻す ]",
    ] {
        assert!(s.contains(want), "{want} が無い: {s}");
    }
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("取り消した"),
        "{:?}",
        a.message
    );
}
