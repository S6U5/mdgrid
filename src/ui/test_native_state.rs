//! mdgrid のビューの見た目の状態(BV-20・SR-12): 列の幅と畳んだまとまりを、mdgrid のビューにも名前ごとに当てる。
//! 置き場は一時フォルダ(利用者の ~/.config・~/.local/state には書かない)。

use super::settings::{Sec, VIEW_BUTTONS};
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::settings::{Cond, Dir, Group, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::views::{load_views, save_views, NativeView};
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ntitle: 会議の準備\n種別: 会議\n---\n");
    tmp.write("b.md", "---\ntitle: 読んだ本\n種別: 本\n---\n");
    tmp.write("c.md", "---\ntitle: 次の本\n種別: 本\n---\n");
    let by_kind = Settings {
        group: Group::By {
            col: "種別".into(),
            dir: Dir::Asc,
            hide_empty: false,
        },
        ..Default::default()
    };
    let views = [
        NativeView {
            name: "まとめ".into(),
            settings: by_kind,
            ..Default::default()
        },
        NativeView {
            name: "全部".into(),
            ..Default::default()
        },
    ];
    save_views(&tmp.0.join("config"), &tmp.notes(), &views).unwrap();
    tmp
}

fn boot(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
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

/// `]` でタブ `name` まで動かす。
fn goto(a: &mut App, name: &str) {
    for _ in 0..6 {
        if a.native_view().map(|v| v.name.as_str()) == Some(name) {
            return;
        }
        ch(a, ']');
    }
    panic!("タブ「{name}」に移れない");
}

fn goto_default(a: &mut App) {
    for _ in 0..6 {
        if a.nv.at.is_none() {
            return;
        }
        ch(a, ']');
    }
    panic!("既定の表に移れない");
}

fn title_width(a: &App) -> Option<usize> {
    a.widths.get("title").copied()
}

fn widen_title(a: &mut App, times: usize) {
    a.col = a.cols.iter().position(|c| c == "title").unwrap();
    for _ in 0..times {
        ch(a, '>');
    }
}

#[test]
fn test_bv_20_column_widths_per_native_view() {
    // [BV-20][SR-12] mdgrid のビュー「全部」で title の幅を変える → ほかのビュー(既定の表・「まとめ」)には
    // 当たらず、「全部」に戻ると同じ幅。終了して開き直しても同じ。
    let tmp = vault("nvstate_width");
    let mut a = boot(&tmp);
    goto(&mut a, "全部");
    widen_title(&mut a, 3);
    let w = title_width(&a).expect("手で決めた幅");
    goto(&mut a, "まとめ");
    assert_eq!(
        title_width(&a),
        None,
        "ほかの mdgrid のビューには当たらない"
    );
    goto_default(&mut a);
    assert_eq!(title_width(&a), None, "既定の表には当たらない");
    goto(&mut a, "全部");
    assert_eq!(title_width(&a), Some(w), "戻ると同じ幅");
    ch(&mut a, 'q');
    assert!(a.quit);

    let mut b = boot(&tmp);
    assert_eq!(title_width(&b), None, "起動は既定の表");
    goto(&mut b, "全部");
    assert_eq!(title_width(&b), Some(w), "開き直しても同じ幅");
}

#[test]
fn test_bv_20_folded_groups_per_native_view() {
    // [BV-20][SR-12] mdgrid のビュー「まとめ」で先頭のまとまりを畳む → ほかのビューに移って戻っても、
    // 開き直しても畳んだまま。ビューの設定(グループ)は定義から当たる。
    let tmp = vault("nvstate_fold");
    let mut a = boot(&tmp);
    goto(&mut a, "まとめ");
    let heads: Vec<String> = a.groups.iter().map(|(h, _)| h.clone()).collect();
    assert_eq!(heads, ["会議", "本"], "定義のグループで組む");
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    assert!(a.folded.contains("会議"), "畳んだ");
    goto(&mut a, "全部");
    assert!(a.folded.is_empty(), "ほかのビューには当たらない");
    goto(&mut a, "まとめ");
    assert!(a.folded.contains("会議"), "戻ると畳んだまま");
    ch(&mut a, 'q');
    assert!(a.quit);

    let mut b = boot(&tmp);
    goto(&mut b, "まとめ");
    assert!(b.folded.contains("会議"), "開き直しても畳んだまま");
    assert!(!b.folded.contains("本"), "畳んでいないまとまりは開いたまま");
}

#[test]
fn test_bv_19_commands_in_help_and_palette() {
    // [BV-19][SR-5][SR-14] キーの無いコマンドもヘルプに出て、パレットで名前から引ける。
    // [WB-15] 読むだけではパレットに出さない(書くコマンド)。
    let tmp = vault("nvstate_help");
    let mut a = boot(&tmp);
    let help: Vec<String> = help::help_lines(&a, keymap::Mode::Table, 79)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    for c in keymap::COMMANDS {
        assert!(help.iter().any(|l| l.contains(c.label)), "{}", c.label);
        let cands = help::candidates(&a, c.action.name());
        assert_eq!(cands.first().map(|c| c.label.as_str()), Some(c.label));
    }
    a.readonly = true;
    for c in keymap::COMMANDS {
        let cands = help::candidates(&a, c.label);
        assert!(cands.iter().all(|x| x.label != c.label), "{}", c.label);
    }
}

#[test]
fn test_bv_20_rename_keeps_width() {
    // [BV-20][SR-12] 名前を変えても、幅はそのビューに残る(新しい名前の状態に移す)。
    let tmp = vault("nvstate_rename");
    let mut a = boot(&tmp);
    goto(&mut a, "全部");
    widen_title(&mut a, 2);
    let w = title_width(&a);
    ch(&mut a, 'o');
    a.view_button(super::settings::VIEW_BUTTONS + 2);
    for c in "ぜんぶ".chars() {
        a.key(ratatui::crossterm::event::KeyEvent::new(
            KeyCode::Char(c),
            ratatui::crossterm::event::KeyModifiers::NONE,
        ));
    }
    // 打つ前の名前「全部」の後ろに足したので「全部ぜんぶ」。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.native_view().map(|v| v.name.as_str()), Some("全部ぜんぶ"));
    ch(&mut a, 'q');
    let mut b = boot(&tmp);
    goto(&mut b, "全部ぜんぶ");
    assert_eq!(title_width(&b), w);
}

// ---- 差し戻し1回目の再現 ----

fn conf(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

fn names(tmp: &Tmp) -> Vec<String> {
    let (v, _) = load_views(&conf(tmp), &tmp.notes());
    v.into_iter().map(|v| v.name).collect()
}

/// mdgrid のビューの状態のファイルの数(状態の置き場の中の別のフォルダ)。
fn native_states(tmp: &Tmp) -> usize {
    std::fs::read_dir(tmp.0.join("state").join(native_views::STATE_DIR))
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_name().to_string_lossy().ends_with(".toml"))
                .count()
        })
        .unwrap_or(0)
}

/// 設定の画面で「名前を付けて保存」。
fn save_as(a: &mut App, name: &str) {
    ch(a, 'o');
    a.view_button(VIEW_BUTTONS);
    typing(a, name);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table, "{:?}", a.message);
}

/// 設定の画面で条件を足して「反映」。
fn apply_filter(a: &mut App, cond: Cond) {
    ch(a, 'o');
    let d = a.draft.as_mut().unwrap();
    d.s.filters.push(cond);
    d.select(Sec::Buttons, 0);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table);
}

fn keep_book() -> Cond {
    Cond {
        col: "種別".into(),
        op: Op::Keep(vec![Some("本".into())]),
    }
}

#[test]
fn test_bv_19_export_dir_of_relative_base() {
    // [BV-19] `mdgrid tasks.base` のように相対パスで開いても、置き場は今のフォルダ。
    assert_eq!(
        native_io::export_dir_of(Path::new("tasks.base")),
        Some(PathBuf::from("."))
    );
    assert_eq!(
        native_io::export_dir_of(Path::new("sub/tasks.base")),
        Some(PathBuf::from("sub"))
    );
    assert_eq!(native_io::export_dir_of(Path::new("")), None);
}

#[test]
fn test_bv_18_two_apps_keep_each_others_views() {
    // [BV-18][BV-20] 2つの mdgrid が同じ対象を開いて、それぞれ保存しても、ほかの保存したビューは消えない。
    // 読み直しで増えたビューはタブにも出る。
    let tmp = vault("nvstate_two");
    let mut a = boot(&tmp);
    let mut b = boot(&tmp);
    save_as(&mut a, "A");
    save_as(&mut b, "B");
    assert_eq!(names(&tmp), ["まとめ", "全部", "A", "B"]);
    assert!(
        b.view_names().contains(&"A".to_string()),
        "読み直した A もタブに"
    );
    // a で上書き・削除しても B は残る。
    goto(&mut a, "A");
    apply_filter(&mut a, keep_book());
    ch(&mut a, 'o');
    a.view_button(VIEW_BUTTONS + 1);
    assert_eq!(names(&tmp), ["まとめ", "全部", "A", "B"], "{:?}", a.message);
    goto(&mut a, "全部");
    ch(&mut a, 'o');
    a.view_button(VIEW_BUTTONS + 3);
    // BV-18: 消す前に確かめる(y)。
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert_eq!(names(&tmp), ["まとめ", "A", "B"]);
}

#[test]
fn test_bv_20_applied_settings_and_columns_survive_tabs_and_restart() {
    // [BV-20][NV-17] mdgrid のビューで反映した設定と、表で動かした・隠した列は、タブを行き来しても
    // 開き直しても残り、その間はタブに `*`。上書きすると定義と同じになり `*` は消える。
    let tmp = vault("nvstate_overlay");
    let mut a = boot(&tmp);
    goto(&mut a, "全部");
    assert!(!screen(&a).lines().nth(1).unwrap().contains('*'));
    apply_filter(&mut a, keep_book());
    a.col = 0;
    ch(&mut a, 'L');
    let cols = a.cols.clone();
    assert_eq!(a.rows.len(), 2);
    assert!(screen(&a).lines().nth(1).unwrap().contains("[全部]*"));
    goto(&mut a, "まとめ");
    goto(&mut a, "全部");
    assert_eq!(a.rows.len(), 2, "反映した設定が残る");
    assert_eq!(a.cols, cols, "動かした列が残る");
    ch(&mut a, 'q');

    let mut b = boot(&tmp);
    goto(&mut b, "全部");
    assert_eq!(b.rows.len(), 2, "開き直しても残る");
    assert_eq!(b.cols, cols);
    ch(&mut b, 'o');
    b.view_button(VIEW_BUTTONS + 1);
    assert!(!b.native_dirty(), "上書きで定義と同じ");
    assert!(!screen(&b).lines().nth(1).unwrap().contains('*'));
    let (v, _) = load_views(&conf(&tmp), &tmp.notes());
    let def = v.iter().find(|v| v.name == "全部").unwrap();
    assert_eq!(def.settings.filters, [keep_book()]);
    assert_eq!(def.order, cols, "動かした並びを保存する");
}

#[test]
fn test_bv_18_save_as_keeps_moved_columns() {
    // [BV-18][NV-4] 既定の表で H/L で列を動かしてから「名前を付けて保存」→ その並びで残る。
    // 動かしていなければ並びは空(あとで増えたキーも出る)。
    let tmp = vault("nvstate_order");
    let mut a = boot(&tmp);
    save_as(&mut a, "そのまま");
    goto_default(&mut a);
    a.col = 0;
    ch(&mut a, 'L');
    let moved = a.cols.clone();
    save_as(&mut a, "動かした");
    let (v, _) = load_views(&conf(&tmp), &tmp.notes());
    let get = |n: &str| v.iter().find(|v| v.name == n).unwrap().clone();
    assert!(get("そのまま").order.is_empty());
    assert_eq!(get("動かした").order, moved);
    assert_eq!(a.cols, moved, "保存したビューでも同じ並び");
}

#[test]
fn test_bv_19_import_nameless_view_gets_name() {
    // [BV-19] 名前の無い .base のビューを取り込む → 「ファイル名 番号」の名前で増える(空の名前にしない)。
    let tmp = vault("nvstate_noname");
    tmp.write("x.base", "views:\n  - type: table\n    order: [title]\n");
    let mut a = boot(&tmp);
    a.start_import();
    typing(&mut a, "x.base");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(names(&tmp), ["まとめ", "全部", "x 1"], "{:?}", a.message);
    assert_eq!(a.native_view().map(|v| v.name.as_str()), Some("x 1"));
}

#[test]
fn test_bv_20_rename_moves_and_delete_removes_state() {
    // [BV-20][SR-12] 名前の変更は状態を新しい名前へ移し(古い名前の状態は残さない)、削除はその状態を消す。
    let tmp = vault("nvstate_files");
    let mut a = boot(&tmp);
    goto(&mut a, "全部");
    widen_title(&mut a, 1);
    goto_default(&mut a);
    // 「まとめ」も通ったので、まとめ・全部の2つ。
    assert_eq!(native_states(&tmp), 2);
    goto(&mut a, "全部");
    ch(&mut a, 'o');
    a.view_button(VIEW_BUTTONS + 2);
    typing(&mut a, "2");
    press(&mut a, KeyCode::Enter);
    assert_eq!(native_states(&tmp), 2, "古い名前の状態は残さない");
    ch(&mut a, 'o');
    a.view_button(VIEW_BUTTONS + 3);
    // BV-18: 消す前に確かめる(y)。
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert_eq!(native_states(&tmp), 1, "削除したビューの状態は消す");
}

#[test]
fn test_wb_15_readonly_hides_view_buttons_and_commands() {
    // [WB-15][BV-18] 読むだけでは設定の画面の mdgrid のビューのボタンと、ヘルプの書き出し・取り込みを出さない。
    let tmp = vault("nvstate_ro");
    let mut a = boot(&tmp);
    a.readonly = true;
    ch(&mut a, 'o');
    let s = screen(&a);
    for label in ["名前を付けて保存", "上書き", "名前の変更", "削除"] {
        assert!(!s.contains(label), "{label}: {s}");
    }
    assert_eq!(a.draft.as_ref().unwrap().len(Sec::Buttons), VIEW_BUTTONS);
    let help: Vec<String> = help::help_lines(&a, keymap::Mode::Table, 79)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    for c in keymap::COMMANDS {
        assert!(!help.iter().any(|l| l.contains(c.label)), "{}", c.label);
    }
}

#[test]
fn test_nv_18_left_right_follow_button_rows() {
    // [NV-18] ←→ は画面の同じ行のボタンの中で動く。上の右は反映・取り消し、下は既定に戻すと
    // mdgrid のビューのボタン(取り消し → 既定に戻す には飛ばない)。
    let tmp = vault("nvstate_lr");
    let mut a = boot(&tmp);
    ch(&mut a, 'o');
    a.draft.as_mut().unwrap().select(Sec::Buttons, 1);
    press(&mut a, KeyCode::Right);
    assert_eq!(a.draft.as_ref().unwrap().at(Sec::Buttons), 1);
    a.draft.as_mut().unwrap().select(Sec::Buttons, 2);
    press(&mut a, KeyCode::Left);
    assert_eq!(a.draft.as_ref().unwrap().at(Sec::Buttons), 2);
    // 下の行は 既定に戻す・名前を付けて保存・上書き… の並び(←→ で隣へ)。
    a.draft.as_mut().unwrap().select(Sec::Buttons, VIEW_BUTTONS);
    press(&mut a, KeyCode::Left);
    assert_eq!(a.draft.as_ref().unwrap().at(Sec::Buttons), 2);
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Right);
    assert_eq!(a.draft.as_ref().unwrap().at(Sec::Buttons), VIEW_BUTTONS + 1);
}

#[test]
fn test_bv_20_readonly_view_buttons_say_read_only() {
    // [BV-20][WB-15] 読むだけでボタンを直に押しても、「読むだけ」と出て views.toml も状態も書かない。
    let tmp = vault("nvstate_ro_press");
    let mut a = boot(&tmp);
    a.readonly = true;
    goto(&mut a, "まとめ");
    let before = std::fs::read(conf(&tmp).join("views.toml")).unwrap();
    let states = native_states(&tmp);
    for i in VIEW_BUTTONS..VIEW_BUTTONS + 4 {
        ch(&mut a, 'o');
        a.message = None;
        a.view_button(i);
        assert!(
            a.message.as_deref().unwrap_or("").contains("読むだけ"),
            "{i}: {:?}",
            a.message
        );
        press(&mut a, KeyCode::Esc);
    }
    assert_eq!(
        std::fs::read(conf(&tmp).join("views.toml")).unwrap(),
        before
    );
    assert_eq!(native_states(&tmp), states);
}
