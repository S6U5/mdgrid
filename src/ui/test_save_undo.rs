//! 保存の取り消し(WB-21)の画面の単体試験。仕様: specs/write-back/spec.md の WB-21。
//! 記録: specs/_changes/2026-11-13-save-undo.md(動作の名前 `undo_save`・コマンドの表示名・見出し・理由の文言)。
//! 値はどれも作ったもの。画面は行の番号でなく中身(見出し・差分の行・探す語)で探す。

use super::help::{candidates, Target};
use super::keymap::{self, Action, Mode};
use super::test_screen::*;
use super::*;
use mdgrid::i18n::{scoped, Lang};
use ratatui::crossterm::event::KeyCode;

const TWO: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\nstatus:   todo\n---\n本文 a\n"),
    ("b.md", "---\ntitle: b\nstatus: doing\n---\n本文 b\n"),
];
const A_DONE: &str = "---\ntitle: a\nstatus:   done\n---\n本文 a\n";
const B_DONE: &str = "---\ntitle: b\nstatus: done\n---\n本文 b\n";

const CMD_JA: &str = "保存を取り消す";
const HEAD_JA: &str = "保存の取り消し: ";
const NONE_JA: &str = "取り消す保存は無い";
const OUTSIDE_JA: &str = "保存のあとに外で変わったので戻さない";

/// a.md の status を done に直して保存する(差分を見て確定)。
fn saved_a(name: &str) -> (Tmp, App) {
    let (t, mut a) = make(name, TWO);
    edit_cell(&mut a, 0, "status", "done");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(read(&t, "a.md"), A_DONE);
    assert_eq!(a.changes.count(), 0);
    (t, a)
}

/// a.md と b.md の status を done に直して、1回で保存する。
fn saved_both(name: &str) -> (Tmp, App) {
    let (t, mut a) = make(name, TWO);
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 1, "status", "done");
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), A_DONE);
    assert_eq!(read(&t, "b.md"), B_DONE);
    (t, a)
}

fn names(t: &Tmp) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(t.notes())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn has_undo_save(a: &App, query: &str) -> bool {
    candidates(a, query)
        .iter()
        .any(|c| c.target == Target::Run(Action::UndoSave))
}

#[test]
fn test_wb_21_action_opens_review_with_heading_and_diff() {
    // [WB-21] 保存のあと、動作 `undo_save` → 確認の画面。見出し「保存の取り消し: 1 ファイル」と、
    // 今の中身から保存の前への差分(-done・+todo)。確定までは書かない。
    let (t, mut a) = saved_a("wb21open");
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("保存の取り消し: 1 ファイル"), "{s}");
    assert!(
        s.contains("-status:   done") && s.contains("+status:   todo"),
        "{s}"
    );
    assert!(s.contains("a.md"), "{s}");
    assert!(!s.contains(OUTSIDE_JA), "{s}");
    assert_eq!(read(&t, "a.md"), A_DONE);
    // [SR-9] 確認の画面も全行が幅以下。
    assert_fits(&mut a);
    assert_eq!(read(&t, "a.md"), A_DONE);
}

#[test]
fn test_wb_21_confirm_restores_bytes_and_pending_value() {
    // [WB-21] 確定 → ファイルは保存の前と1バイトも違わず、status のセルは done のまま「未保存 1」。
    // `u` → 「未保存 0」。ノートのフォルダに控えを作らない。
    let (t, mut a) = saved_a("wb21confirm");
    let before = names(&t);
    a.apply(Action::UndoSave);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(
        std::fs::read(t.notes().join("a.md")).unwrap(),
        TWO[0].1.as_bytes()
    );
    assert_eq!(read(&t, "b.md"), TWO[1].1);
    assert_eq!(a.changes.count(), 1);
    let s = screen(&a);
    assert!(s.contains("未保存 1") && s.contains("*done"), "{s}");
    assert_eq!(names(&t), before);
    // [WB-10] 1手で消える。ファイルは戻したまま。
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    let s = screen(&a);
    assert!(s.contains("未保存 0") && !s.contains("*done"), "{s}");
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    // 戻した保存はもう覚えていない。
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains(NONE_JA),
        "{:?}",
        a.message
    );
}

#[test]
fn test_wb_21_restored_value_can_be_saved_again() {
    // [WB-21] 戻したあと、ためる変更に戻った値をそのままもう一度保存できる。
    let (t, mut a) = saved_a("wb21again");
    a.apply(Action::UndoSave);
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains(HEAD_JA), "保存の確認が取り消しの見出し: {s}");
    assert!(s.contains("+status:   done"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), A_DONE);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_wb_21_palette_command_without_default_key() {
    // [WB-21] パレットで「保存を取り消す」(表示名)でも `undo_save`(名前)でも探せ、既定のキーは無い。
    // Enter で実行すると確認の画面が出る。
    let (_t, mut a) = saved_a("wb21palette");
    assert_eq!(Action::UndoSave.name(), "undo_save");
    let c = candidates(&a, CMD_JA);
    assert!(!c.is_empty(), "候補が無い");
    assert_eq!(c[0].target, Target::Run(Action::UndoSave));
    assert_eq!(c[0].label, CMD_JA);
    assert_eq!(c[0].keys, "", "既定のキーは割り当てない");
    assert_eq!(
        candidates(&a, "undo_save")[0].target,
        Target::Run(Action::UndoSave)
    );
    for mode in [Mode::Table, Mode::Confirm] {
        assert!(
            keymap::keys_of(&a.keys, mode, Action::UndoSave).is_empty(),
            "{mode:?} に既定のキーがある"
        );
    }

    ch(&mut a, ':');
    assert_eq!(a.mode, Mode::Palette);
    typing(&mut a, CMD_JA);
    let s = screen(&a);
    assert!(
        s.lines().any(|l| l.starts_with('>') && l.contains(CMD_JA)),
        "{s}"
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    assert!(screen(&a).contains("保存の取り消し: 1 ファイル"));
}

#[test]
fn test_wb_21_esc_cancels_without_writing() {
    // [WB-21] 確認の画面で Esc(と q)→ 書かずに表へ戻る。保存は覚えたままで、あとで取り消せる。
    let (t, mut a) = saved_a("wb21esc");
    a.apply(Action::UndoSave);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(read(&t, "a.md"), A_DONE);
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));

    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Confirm, "やめたあとに覚えが消えた");
    ch(&mut a, 'q');
    assert_eq!(a.mode, Mode::Table);
    assert!(!a.quit);
    assert_eq!(read(&t, "a.md"), A_DONE);

    a.apply(Action::UndoSave);
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
}

#[test]
fn test_wb_21_no_save_yet_only_tells() {
    // [WB-21] まだ保存していない → 確認の画面を出さず、何も書かず、取り消す保存が無いと知らせる。
    // ためた変更があっても同じ(ためた変更は残る)。
    let (t, mut a) = make("wb21none", TWO);
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains(NONE_JA),
        "{:?}",
        a.message
    );
    assert!(screen(&a).contains(NONE_JA));

    edit_cell(&mut a, 0, "status", "done");
    a.message = None;
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains(NONE_JA),
        "{:?}",
        a.message
    );
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    assert_eq!(read(&t, "b.md"), TWO[1].1);
}

#[test]
fn test_wb_21_readonly_has_no_command() {
    // [WB-21] [WB-15] 読むだけの起動ではコマンドを出さない。動作を呼んでも確認を開かず、書かない。
    let (t, mut a) = saved_a("wb21ro");
    assert!(keymap::commands(false).any(|c| c.action == Action::UndoSave));
    assert!(!keymap::commands(true).any(|c| c.action == Action::UndoSave));
    assert!(has_undo_save(&a, CMD_JA));

    a.readonly = true;
    assert!(!has_undo_save(&a, CMD_JA));
    assert!(!has_undo_save(&a, "undo_save"));
    assert!(!has_undo_save(&a, ""));
    ch(&mut a, ':');
    typing(&mut a, CMD_JA);
    assert!(!screen(&a)
        .lines()
        .any(|l| l.starts_with('>') && l.contains(CMD_JA)));
    press(&mut a, KeyCode::Esc);

    a.apply(Action::UndoSave);
    assert_ne!(a.mode, Mode::Confirm);
    assert!(a.review.is_none());
    assert_eq!(read(&t, "a.md"), A_DONE);
}

#[test]
fn test_wb_21_file_changed_outside_shows_reason_and_is_skipped() {
    // [WB-21] 保存のあとに外で title を直してから「保存を取り消す」→ 差分の代わりに外で変わった理由が出て、
    // 確定してもそのファイルは書かれない。変化の検出(poll)の前でも後でも同じ。
    for polled in [false, true] {
        let (t, mut a) = saved_a(if polled { "wb21outp" } else { "wb21out" });
        let outside = "---\ntitle: a outside\nstatus:   done\n---\n本文 a\n";
        edit_outside(&t, "a.md", outside);
        if polled {
            a.poll();
        }
        a.apply(Action::UndoSave);
        assert_eq!(a.mode, Mode::Confirm, "polled={polled} {:?}", a.message);
        let s = screen(&a);
        assert!(
            s.contains("保存の取り消し: 1 ファイル"),
            "polled={polled}\n{s}"
        );
        assert!(s.contains(OUTSIDE_JA), "polled={polled}\n{s}");
        assert!(!s.contains("+status:   todo"), "polled={polled}\n{s}");
        assert_fits(&mut a);

        press(&mut a, KeyCode::Enter);
        assert_eq!(
            read(&t, "a.md"),
            outside,
            "polled={polled}: 外の変更を消した"
        );
        assert_eq!(a.changes.count(), 0, "polled={polled}");
    }
}

#[test]
fn test_wb_21_two_files_one_changed_outside() {
    // [WB-21] 2ファイルを保存し、a.md だけ外で変わった → 見出しは 2 ファイル。確定 → b.md だけが戻り、
    // b の値だけがためる変更に戻る(未保存 1)。a.md は外の中身のまま。
    let (t, mut a) = saved_both("wb21two");
    let outside = "---\ntitle: a outside\nstatus:   done\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    assert!(screen(&a).contains("保存の取り消し: 2 ファイル"));
    // 2つのファイルを順に見る: 片方は理由、片方は差分。
    let mut seen = screen(&a);
    ch(&mut a, 'j');
    seen.push_str(&screen(&a));
    assert!(seen.contains(OUTSIDE_JA), "{seen}");
    assert!(
        seen.contains("-status: done") && seen.contains("+status: doing"),
        "{seen}"
    );

    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), outside);
    assert_eq!(
        std::fs::read(t.notes().join("b.md")).unwrap(),
        TWO[1].1.as_bytes()
    );
    assert_eq!(a.changes.count(), 1);
    let b = a
        .rows
        .iter()
        .find(|r| a.src.label(r) == "b.md")
        .unwrap()
        .clone();
    assert!(a.changes.pending(&b, "status").is_some());
}

#[test]
fn test_wb_21_only_last_save_is_undone() {
    // [WB-21] 2回保存したあと → 戻るのは2回目の保存だけ(見出しは 1 ファイル、1回目のファイルはそのまま)。
    let (t, mut a) = saved_a("wb21last");
    edit_cell(&mut a, 1, "status", "done");
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "b.md"), B_DONE);

    a.apply(Action::UndoSave);
    let s = screen(&a);
    assert!(s.contains("保存の取り消し: 1 ファイル"), "{s}");
    assert!(s.contains("+status: doing"), "{s}");
    assert!(!s.contains("+status:   todo"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "b.md"), TWO[1].1);
    assert_eq!(read(&t, "a.md"), A_DONE, "1回目の保存まで戻った");
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_wb_21_overwrite_and_discard_keys_do_nothing_in_undo_review() {
    // [WB-21] 設計: 取り消しの確認では、保存の確認の `o`(外の変更の上に書く)と `d`(ためた変更を捨てる)は
    // 効かない。外で変わったファイルの上に `o` で書けない。
    let (t, mut a) = saved_a("wb21keys");
    let outside = "---\ntitle: a outside\nstatus:   done\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    a.apply(Action::UndoSave);
    assert_eq!(a.mode, Mode::Confirm);
    ch(&mut a, 'o');
    assert_eq!(read(&t, "a.md"), outside, "o で外の変更の上に書いた");
    ch(&mut a, 'd');
    assert_eq!(read(&t, "a.md"), outside);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_wb_21_english_texts() {
    // [WB-21] 英語の文言: コマンド・見出し・外で変わった理由・取り消す保存が無い。
    let _g = scoped(Lang::En);
    let (t, mut a) = saved_a("wb21en");
    let c = candidates(&a, "Undo the last save");
    assert_eq!(c[0].target, Target::Run(Action::UndoSave));
    assert_eq!(c[0].label, "Undo the last save");
    a.apply(Action::UndoSave);
    let s = screen(&a);
    assert!(s.contains("Undo the last save: 1 file"), "{s}");
    press(&mut a, KeyCode::Esc);

    edit_outside(
        &t,
        "a.md",
        "---\ntitle: a outside\nstatus:   done\n---\n本文 a\n",
    );
    a.apply(Action::UndoSave);
    let s = screen(&a);
    assert!(
        s.contains("changed outside since the save, so it is not restored"),
        "{s}"
    );
    press(&mut a, KeyCode::Esc);

    let (_t2, mut b) = make("wb21en-none", TWO);
    b.apply(Action::UndoSave);
    assert!(
        b.message
            .as_deref()
            .unwrap_or("")
            .contains("there is no save to undo"),
        "{:?}",
        b.message
    );
}
