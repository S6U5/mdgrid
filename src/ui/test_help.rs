//! ヘルプ(SR-5)とパレット(SR-14・SR-16・NV-7)の試験。
use super::help::{candidates, Target};
use super::keymap::{Action, Binding, Mode};
use super::test_screen::*;
use super::*;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n"),
    ("b.md", "---\ntitle: 買い物\nstatus: 完了\n---\n"),
];

#[test]
fn test_sr_5_help_golden() {
    // [SR-5] 表で `?` → ヘルプを重ね、先頭に表のモードで押せるキー、その下に全部の節。
    let (_t, mut a) = make("sr5", NOTES);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let s = screen(&a);
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[1].contains("今押せるキー(表)"), "{s}");
    // 先頭は下の帯の順位の順(Enter 編集・? ヘルプ …)。
    assert!(
        lines[2].contains("Enter") && lines[2].contains("編集"),
        "{s}"
    );
    golden("sr_5", &s);
    // 流すと、下の節も見える(最後は印の節。help-markers)。
    press(&mut a, KeyCode::Char('G'));
    let s = screen(&a);
    assert!(s.contains(" 印") && s.contains("行の左"), "{s}");
    // [SR-9] ヘルプも全行が幅以下。
    assert_fits(&mut a);
    // Esc で閉じて表へ戻る。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_5_help_from_review_shows_review_keys() {
    // [SR-5] 保存の確認で `?` → 先頭はそのモードのキー。閉じると保存の確認に戻る。
    let (_t, mut a) = make("sr5r", NOTES);
    press(&mut a, KeyCode::Backspace);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let s = screen(&a);
    assert!(
        s.lines()
            .nth(1)
            .unwrap()
            .contains("今押せるキー(保存の確認)"),
        "{s}"
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Confirm);
    assert!(a.review.is_some());
}

#[test]
fn test_sr_14_palette_save_first_golden() {
    // [SR-14] `:` で「save」と打つ → 保存が候補の先頭で、横に ^S。
    let (_t, mut a) = make("sr14", NOTES);
    ch(&mut a, ':');
    assert_eq!(a.mode, Mode::Palette);
    typing(&mut a, "save");
    let c = candidates(&a, "save");
    assert_eq!(c[0].target, Target::Run(Action::Save));
    assert_eq!(c[0].keys, "^S");
    let s = screen(&a);
    let line = s.lines().nth(2).unwrap();
    assert!(
        line.starts_with(">") && line.contains("保存") && line.contains("^S"),
        "{s}"
    );
    golden("sr_14", &s);
    // [SR-17] 端末のカーソルは入力の位置。
    assert_eq!(view::cursor(&a), Some((6, 1)));
    // [SR-9] パレットも全行が幅以下。
    assert_fits(&mut a);
    // Esc で閉じる。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_14_palette_fuzzy_and_ctrl_p() {
    // [SR-14] Ctrl+P でも開く。表示名(日本語)でも探せる。飛び飛びの一致も拾う。
    let (_t, mut a) = make("sr14b", NOTES);
    ctrl(&mut a, 'p');
    assert_eq!(a.mode, Mode::Palette);
    typing(&mut a, "取り消し");
    assert_eq!(
        candidates(&a, "取り消し")[0].target,
        Target::Run(Action::Undo)
    );
    assert_eq!(
        candidates(&a, "opedtr")[0].target,
        Target::Run(Action::OpenEditor)
    );
    // Enter で実行する: 「取り消し」は取り消す変更が無いと知らせる。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.message.as_deref(), Some("取り消す変更が無い"));
    // 合わない文字では候補が無く、Enter で閉じずに知らせる。
    ch(&mut a, ':');
    typing(&mut a, "zzzz");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette);
    assert!(a.message.as_deref().unwrap().contains("合うコマンドが無い"));
    // Backspace で消し、↓ で次の候補。
    for _ in 0..4 {
        press(&mut a, KeyCode::Backspace);
    }
    press(&mut a, KeyCode::Down);
    assert_eq!(a.palette.as_ref().unwrap().sel, 1);
}

#[test]
fn test_nv_7_goto_line() {
    // [NV-7] `:120` → 120行目が選ばれる。
    let tmp = Tmp::new("nv7");
    for i in 1..=150 {
        tmp.write(&format!("n{i:03}.md"), &format!("---\nn: {i}\n---\n"));
    }
    let mut a = app_of(&tmp, ColorMode::None);
    assert_eq!(a.rows.len(), 150);
    ch(&mut a, ':');
    typing(&mut a, "120");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.row, 119);
    assert_eq!(a.src.label(&a.rows[a.row]), "n120.md");
    // 選んだ行が見える(SR-3)。
    assert!(a.top <= 119 && 119 < a.top + a.data_height());
    // 行が足りなければ末尾へ移って知らせる。
    ch(&mut a, ':');
    typing(&mut a, "999");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.row, 149);
    assert!(a.message.as_deref().unwrap().contains("999 行目は無い"));
}

#[test]
fn test_sr_16_palette_w_and_q() {
    // [SR-16] パレットの `w` は保存(差分の確認へ)、`q` は終了(ためた変更があれば終了の確認)。
    let (_t, mut a) = make("sr16", NOTES);
    ch(&mut a, ':');
    ch(&mut a, 'w');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.message.as_deref(), Some("保存する変更が無い"));
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    ch(&mut a, ':');
    ch(&mut a, 'w');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Esc);
    ch(&mut a, ':');
    ch(&mut a, 'q');
    assert_eq!(candidates(&a, "q")[0].target, Target::Run(Action::Quit));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    // 変更が無ければ `:q` で終わる。
    ch(&mut a, 'u');
    ch(&mut a, ':');
    ch(&mut a, 'q');
    press(&mut a, KeyCode::Enter);
    assert!(a.quit);
}

#[test]
fn test_sr_4_added_key_shows_everywhere() {
    // [SR-4] キーの表に1つ足す → 振り分け・下の帯・ヘルプ・パレットに同時に出る。
    let (_t, mut a) = make("sr4", NOTES);
    a.keys.insert(
        0,
        Binding {
            mode: Mode::Table,
            key: "x",
            action: Action::Save,
            label: "書き込む",
            section: "足した節",
            rank: 1,
            // 日本語では label・section を出す(英語ではこの文言)。
            msg: mdgrid::i18n::Msg::KeySave,
            section_msg: mdgrid::i18n::Msg::SecFile,
        },
    );
    // 下の帯
    let s = screen(&a);
    assert!(s.lines().nth(21).unwrap().contains("x 書き込む"), "{s}");
    // パレット: 保存の横に x と ^S
    let c = candidates(&a, "save");
    assert_eq!(c[0].keys, "x ^S");
    // ヘルプ: 先頭(今押せるキー)と、足した節
    ch(&mut a, '?');
    let s = screen(&a);
    assert!(s.lines().nth(2).unwrap().contains("x ^S"), "{s}");
    press(&mut a, KeyCode::Char('G'));
    let all = help::help_lines(&a, Mode::Table, 79);
    assert!(all.iter().any(|(l, h)| *h && l.contains("足した節")));
    press(&mut a, KeyCode::Esc);
    // 振り分け
    ch(&mut a, 'x');
    assert_eq!(a.message.as_deref(), Some("保存する変更が無い"));
}

#[test]
fn test_sr_5_help_stays_when_review_rebuilt() {
    // [SR-5] [WB-16] 保存の確認でヘルプを出している間に外の変更が見つかっても、ヘルプは消えない。
    // 閉じると作り直した保存の確認に戻る。
    let (t, mut a) = make(
        "sr5p",
        &[
            ("a.md", "---\ntitle: a\nstatus: todo\n---\n"),
            ("b.md", "---\ntitle: b\nstatus: doing\n---\n"),
        ],
    );
    edit_cell(&mut a, 0, "status", "done");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    ch(&mut a, '?');
    edit_outside(&t, "a.md", "---\ntitle: a2\nstatus: todo\n---\n");
    a.poll();
    assert_eq!(a.mode, Mode::Help);
    assert!(a.help.is_some());
    assert!(a.review.as_ref().unwrap().items[0].external);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Confirm);
}
