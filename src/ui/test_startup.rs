//! 起動の試験(タスク 8): キーの割り当て直し(SR-13・SR-16)、設定(CLI-3)、`--readonly`(WB-15)・
//! `--no-color`(CLI-2・SR-10)、見た目の状態(SR-11・SR-12・NV-4)。
//! 状態の置き場は一時フォルダ(`<一時>/state`)を `Startup` で渡す(XDG_STATE_HOME には触らない)。

use super::keymap::{self, Mode};
use super::startup::Startup;
use super::test_grid::{base_path, labels, vault, TASKS};
use super::test_screen::{buffer, ch, col_named, press, screen, Tmp};
use super::*;
use mdgrid::config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOTES: &[(&str, &str)] = &[
    (
        "a.md",
        "---\ntitle: 会議のメモ\nstatus: 進行中\ndue: 2026-10-05\n---\n",
    ),
    (
        "b.md",
        "---\ntitle: 買い物\nstatus: 完了\ndue: 2026-09-01\n---\n",
    ),
    ("c.md", "---\ntitle: 読書\nstatus: 進行中\n---\n"),
];

fn notes(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    tmp
}

fn state_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("state")
}

struct Opts<'a> {
    toml: &'a str,
    readonly: bool,
    no_color: bool,
    color: ColorMode,
}

const PLAIN: Opts<'static> = Opts {
    toml: "",
    readonly: false,
    no_color: false,
    color: ColorMode::None,
};

/// main と同じ道筋(`App::new` → `start` → 読み込み)でフォルダを開く。
fn boot_with(tmp: &Tmp, o: Opts) -> App {
    let (config, warnings) = config::parse(o.toml).expect("読める設定");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), o.color);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings,
        readonly: o.readonly,
        no_color: o.no_color,
        state_dir: Some(state_dir(tmp)),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn boot(tmp: &Tmp) -> App {
    boot_with(tmp, PLAIN)
}

fn boot_toml(tmp: &Tmp, toml: &str) -> App {
    boot_with(tmp, Opts { toml, ..PLAIN })
}

/// main と同じ道筋で `.base` を開く(`--view` は `view`)。
fn boot_base(tmp: &Tmp, view: Option<&str>) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), view).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(state_dir(tmp)),
        config_dir: None,
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn ctrl(app: &mut App, c: char) {
    app.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
}

/// フォルダの中のファイル(名前 → 中身)。下のフォルダも含む。
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn state_files(tmp: &Tmp) -> usize {
    std::fs::read_dir(state_dir(tmp))
        .map(|d| d.flatten().count())
        .unwrap_or(0)
}

// ---- キーの割り当て直し(SR-13・SR-16) ----

#[test]
fn test_sr_13_unbind_j_leaves_down_arrow() {
    // [SR-13] 設定で `j` を外すと ↓ だけが効く。`"ctrl+n" = "down"` で足したキーも効く。
    let tmp = notes("sr13");
    let mut a = boot_toml(&tmp, "[keys.table]\nj = \"none\"\n\"ctrl+n\" = \"down\"\n");
    assert_eq!(a.message, None, "警告は無い");
    assert_eq!(a.row, 0);
    ch(&mut a, 'j');
    assert_eq!(a.row, 0, "外した j は効かない");
    press(&mut a, KeyCode::Down);
    assert_eq!(a.row, 1, "↓ は効く");
    ctrl(&mut a, 'n');
    assert_eq!(a.row, 2, "Ctrl+N で下へ");
    // 下の帯・ヘルプ・パレットも同じ写しを引く(SR-4): 下は ↓ と ^N、j は無い。
    let keys = keymap::keys_of(&a.keys, Mode::Table, keymap::Action::Down);
    assert_eq!(keys, ["↓", "^N"]);
    // 表の k(上)は既定のまま。
    ch(&mut a, 'k');
    assert_eq!(a.row, 1);
}

#[test]
fn test_sr_13_rebind_replaces_and_mode_names() {
    // [SR-13] 同じキーに別の動作を書けば置き換える。モードは英語の名前でも表示名(SR-16 の名前)でも書ける。
    let tmp = notes("sr13b");
    let mut a = boot_toml(
        &tmp,
        "[keys.table]\nj = \"up\"\n[keys.\"表\"]\nx = \"down\"\n[keys.edit]\n\"ctrl+u\" = \"revert\"\n",
    );
    assert_eq!(a.message, None);
    ch(&mut a, 'x');
    ch(&mut a, 'x');
    assert_eq!(a.row, 2);
    ch(&mut a, 'j');
    assert_eq!(a.row, 1, "j は上");
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Edit, "Ctrl+u"),
        Some(keymap::Action::Revert)
    );
}

#[test]
fn test_sr_13_unknown_mode_key_action_warned() {
    // [SR-13] 知らないモード・キー・動作、そのモードで使えない動作はメッセージ行に警告して、起動は止めない。
    let tmp = notes("sr13w");
    let mut a = boot_toml(
        &tmp,
        "[keys.nope]\nj = \"down\"\n[keys.table]\nx = \"fly\"\n\"hyper+q\" = \"down\"\n\"ctrl+q\" = \"commit\"\n",
    );
    let m = a.message.clone().expect("警告");
    assert!(
        m.contains("keys.nope") && m.contains("知らないモード"),
        "{m}"
    );
    assert!(m.contains("fly"), "{m}");
    assert!(m.contains("hyper"), "{m}");
    assert!(m.contains("commit") && m.contains("使えない"), "{m}");
    assert!(!m.contains('\n'));
    // 既定のキーはそのまま効く。
    ch(&mut a, 'j');
    assert_eq!(a.row, 1);
    // 警告の行は画面のメッセージ行に出る。
    let mut b = boot_toml(&tmp, "[keys.nope]\nj = \"down\"\n");
    b.resize(80, 24);
    assert!(screen(&b).contains("keys.nope"), "{}", screen(&b));
}

#[test]
fn test_sr_16_conflicting_binding_warned_not_applied() {
    // [SR-13][SR-16] 同じモードの同じキーに2つの動作(表記の違う同じキー)、前置きと重なるキーは警告して当てない。
    // 割り当て直したあとも、モードごとにキーの重複が無い。
    let tmp = notes("sr16");
    let a = boot_toml(
        &tmp,
        "[keys.table]\n\"ctrl+n\" = \"down\"\n\"control+n\" = \"up\"\ng = \"top\"\n",
    );
    let m = a.message.clone().expect("警告");
    assert!(m.contains("control+n") && m.contains("ctrl+n"), "{m}");
    assert!(m.contains("前置き"), "{m}");
    assert_eq!(keymap::lookup(&a.keys, Mode::Table, "Ctrl+n"), None);
    assert_eq!(keymap::lookup(&a.keys, Mode::Table, "g"), None);
    let mut seen = std::collections::HashSet::new();
    for b in &a.keys {
        assert!(seen.insert((b.mode, b.key)), "重複: {:?} {}", b.mode, b.key);
        assert!(
            !keymap::is_prefix(&a.keys, b.mode, b.key),
            "前置き: {}",
            b.key
        );
    }
    // 先に "none" で外せば、`g` を当てられる。
    let a = boot_toml(&tmp, "[keys.table]\ng = \"top\"\n\"g g\" = \"none\"\n");
    assert_eq!(a.message, None);
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "g"),
        Some(keymap::Action::Top)
    );
}

#[test]
fn test_sr_13_parse_key_notation() {
    // [SR-13] キーの表記を表のキーの名前に読む。
    let ok = |s: &str| keymap::parse_key(s).unwrap();
    assert_eq!(ok("j"), "j");
    assert_eq!(ok("G"), "G");
    assert_eq!(ok("shift+g"), "G");
    assert_eq!(ok("ctrl+s"), "Ctrl+s");
    assert_eq!(ok("Ctrl+S"), "Ctrl+Shift+s");
    assert_eq!(ok("ctrl+shift+z"), "Ctrl+Shift+z");
    assert_eq!(ok("shift+tab"), "Shift+Tab");
    assert_eq!(ok("shift+up"), "Shift+Up");
    assert_eq!(ok("shift+left"), "Shift+Left");
    assert_eq!(ok("enter"), "Enter");
    assert_eq!(ok("ESC"), "Esc");
    assert_eq!(ok("pagedown"), "PageDown");
    assert_eq!(ok("space"), "Space");
    assert_eq!(ok("bs"), "Backspace");
    assert_eq!(ok("+"), "+");
    assert_eq!(ok("ctrl++"), "Ctrl++");
    assert_eq!(ok("g g"), "g g");
    assert_eq!(ok("ｊ"), "j");
    for bad in ["", "hyper+j", "f13", "shift+home", "shift+1", " "] {
        assert!(keymap::parse_key(bad).is_err(), "{bad}");
    }
}

// ---- 設定(CLI-3・BV-9・CE-3・CV-6・SR-10) ----

#[test]
fn test_cli_3_no_config_defaults_and_unknown_item_warned() {
    // [CLI-3] 設定が無ければ既定で動く。知らない項目は警告をメッセージ行に出して起動する。
    let tmp = notes("cli3");
    let a = boot(&tmp);
    assert_eq!(a.message, None);
    assert_eq!(a.candidates, 20);
    assert_eq!(a.poll_interval(), std::time::Duration::from_millis(1000));
    assert_eq!(a.keys.len(), keymap::BINDINGS.len());
    let mut a = boot_toml(&tmp, "foo = 1\ncandidates = 3\n");
    let m = a.message.clone().expect("警告");
    assert!(m.contains("foo"), "{m}");
    assert_eq!(a.candidates, 3, "知っている項目は当てる");
    ch(&mut a, 'j');
    assert_eq!(a.row, 1);
}

#[test]
fn test_cli_3_applies_items() {
    // [CLI-3][BV-9] poll_ms は読み直しの間隔、candidates・ambiguous_wide・color を当てる。
    let tmp = notes("cli3b");
    let a = boot_with(
        &tmp,
        Opts {
            toml: "poll_ms = 250\ncandidates = 5\nambiguous_wide = true\ncolor = false\n",
            color: ColorMode::Indexed,
            ..PLAIN
        },
    );
    assert_eq!(a.poll_interval(), std::time::Duration::from_millis(250));
    assert_eq!(a.candidates, 5);
    assert!(a.ambiguous_wide);
    assert_eq!(a.color, ColorMode::None);
}

#[test]
fn test_cli_2_no_color() {
    // [CLI-2][SR-10] `--no-color` で、色の出せる端末でも色を出さない(ためる変更のセルも)。
    let tmp = notes("nocolor");
    let mut a = boot_with(
        &tmp,
        Opts {
            no_color: true,
            color: ColorMode::Rgb,
            ..PLAIN
        },
    );
    assert_eq!(a.color, ColorMode::None);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    for c in buffer(&a).content() {
        assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset), "{c:?}");
    }
    // 付けなければ色を出す(比べるため)。
    let mut b = boot_with(
        &tmp,
        Opts {
            color: ColorMode::Rgb,
            ..PLAIN
        },
    );
    col_named(&mut b, "status");
    press(&mut b, KeyCode::Backspace);
    assert!(buffer(&b)
        .content()
        .iter()
        .any(|c| c.fg != Color::Reset || c.bg != Color::Reset));
}

// ---- 読むだけ(WB-15) ----

#[test]
fn test_wb_15_readonly_writes_nothing() {
    // [WB-15] `--readonly` では Ctrl+S で「読むだけ」と出し、何も書かない。Backspace・Enter の編集も効かない。
    // 保存・空にするのキーは下の帯・ヘルプ・パレットに出さない。見た目の状態も書かない。
    let tmp = notes("wb15");
    let before = snapshot(&tmp.notes());
    let mut a = boot_with(
        &tmp,
        Opts {
            readonly: true,
            ..PLAIN
        },
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 0, "Backspace は効かない");
    assert!(a.message.as_deref().unwrap().contains("読むだけ"));
    press(&mut a, KeyCode::Enter);
    assert!(a.input.is_none(), "編集を開かない");
    assert_eq!(a.mode, Mode::Table);
    ctrl(&mut a, 's');
    assert!(
        a.message.as_deref().unwrap().contains("読むだけ"),
        "{:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Table, "保存の確認を開かない");
    // 動作を出さない: 下の帯・キーの表・パレットに保存・空にする・編集が無い。
    let s = screen(&a);
    let band = s.lines().nth(21).unwrap();
    assert!(!band.contains("^S") && !band.contains("編集"), "{band}");
    for act in [keymap::Action::Save, keymap::Action::Clear] {
        assert!(keymap::keys_of(&a.keys, Mode::Table, act).is_empty());
    }
    assert!(super::help::candidates(&a, "w")
        .iter()
        .all(|c| c.target != super::help::Target::Run(keymap::Action::Save)));
    // 詳細の表示の Enter も編集しない。
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    press(&mut a, KeyCode::Enter);
    assert!(a.input.is_none());
    press(&mut a, KeyCode::Esc);
    // 列の幅を変えて終わっても、どのファイルも書かない。
    ch(&mut a, '>');
    ch(&mut a, 'q');
    assert!(a.quit);
    assert_eq!(snapshot(&tmp.notes()), before);
    assert_eq!(state_files(&tmp), 0);
}

// ---- 見た目の状態(SR-11・SR-12・NV-4) ----

#[test]
fn test_nv_4_hidden_column_survives_restart() {
    // [NV-4][SR-12] 列を隠して終了 → 同じ対象で開き直すと隠したまま。`+` で元の位置に戻る。
    let tmp = notes("nv4");
    let mut a = boot(&tmp);
    let cols = a.cols.clone();
    col_named(&mut a, "status");
    ch(&mut a, '-');
    assert!(!a.cols.contains(&"status".to_string()));
    ch(&mut a, 'q');
    assert!(a.quit);

    let mut b = boot(&tmp);
    assert!(!b.cols.contains(&"status".to_string()), "{:?}", b.cols);
    assert!(!screen(&b).lines().nth(3).unwrap().contains("status"));
    ch(&mut b, '+');
    assert_eq!(b.cols, cols, "元の位置に戻る");
}

#[test]
fn test_sr_11_state_not_in_note_folder() {
    // [SR-11] 列の幅を変えて終了 → ノートのフォルダに新しいファイルが無い(状態は状態の置き場に)。
    let tmp = notes("sr11");
    let before = snapshot(&tmp.notes());
    let mut a = boot(&tmp);
    col_named(&mut a, "title");
    let w0 = view::layout(&a).widths[a.col];
    ch(&mut a, '>');
    ch(&mut a, '>');
    ch(&mut a, 'q');
    assert_eq!(snapshot(&tmp.notes()), before);
    assert_eq!(state_files(&tmp), 1);
    // [SR-12] 幅は開き直しても残る。
    let mut b = boot(&tmp);
    col_named(&mut b, "title");
    assert_eq!(view::layout(&b).widths[b.col], w0 + 2);
}

#[test]
fn test_sr_12_broken_state_starts_with_defaults() {
    // [SR-12] 状態のファイルを壊して起動 → 既定で起動し、エラーで止まらない。
    let tmp = notes("sr12broken");
    let default_cols = boot(&tmp).cols;
    let mut a = boot(&tmp);
    col_named(&mut a, "status");
    ch(&mut a, '-');
    ch(&mut a, 'q');
    assert_eq!(state_files(&tmp), 1);
    for e in std::fs::read_dir(state_dir(&tmp)).unwrap().flatten() {
        std::fs::write(e.path(), b"order = [\x00\xff not toml").unwrap();
    }
    let mut b = boot(&tmp);
    assert_eq!(b.cols, default_cols);
    assert!(b.hidden.is_empty());
    assert_eq!(b.message, None);
    ch(&mut b, 'j');
    assert_eq!(b.row, 1);
}

#[test]
fn test_sr_12_temporary_sort_not_kept() {
    // [SR-12][NV-3] 見出しからの並べ替えは設定の並べ替えとして残す。列の並び(H・L)も残す。
    let tmp = notes("sr12sort");
    let mut a = boot(&tmp);
    col_named(&mut a, "status");
    ch(&mut a, 's');
    assert_eq!(a.settings.sorts.len(), 1);
    let sorted = labels(&a);
    ch(&mut a, 'H');
    let cols = a.cols.clone();
    ch(&mut a, 'q');
    let b = boot(&tmp);
    assert_eq!(b.settings.sorts, a.settings.sorts);
    assert_eq!(b.cols, cols);
    assert_eq!(labels(&b), sorted);
}

#[test]
fn test_sr_12_new_base_column_added_right_and_view_kept() {
    // [SR-12] 状態の並びは今の `.base` の列にだけ当て、`.base` に増えた列は右に足す。
    // 状態は `.base` のパスとビューの名前ごと。[BV-13] 開き直すと先頭のビュー(`--view` があればそちら)で、
    // 前に選んだビューは起動に使わない。
    let tmp = vault("sr12base");
    base_path(&tmp, TASKS);
    let mut a = boot_base(&tmp, None);
    assert_eq!(a.view_index(), 0);
    assert_eq!(a.cols, ["status", "due", "title"]);
    col_named(&mut a, "title");
    ch(&mut a, 'H');
    ch(&mut a, 'H');
    assert_eq!(a.cols, ["title", "status", "due"]);
    // ビューを切り替えると前のビューの状態を書き、次のビューは既定。
    ch(&mut a, ']');
    assert_eq!(a.view_index(), 1);
    ch(&mut a, '[');
    assert_eq!(a.cols, ["title", "status", "due"], "切り替えて戻っても残る");
    ch(&mut a, ']');
    ch(&mut a, ']');
    assert_eq!(a.view_index(), 2);
    ch(&mut a, 'q');

    let b = boot_base(&tmp, None);
    assert_eq!(b.view_index(), 0, "起動は先頭のビュー(BV-13)");
    assert_eq!(
        b.cols,
        ["title", "status", "due"],
        "先頭のビューの状態を当てる"
    );
    let b = boot_base(&tmp, Some("全部"));
    assert_eq!(b.view_index(), 2);
    let b = boot_base(&tmp, Some("進行中"));
    assert_eq!(b.view_index(), 0);
    assert_eq!(b.cols, ["title", "status", "due"]);

    // `.base` の order に列を足し、消す: 状態の並びは残った列に当て、増えた列は右に足す。
    let text = TASKS.replace(
        "order: [status, due, title]",
        "order: [file.name, status, title]",
    );
    base_path(&tmp, &text);
    let b = boot_base(&tmp, Some("進行中"));
    assert_eq!(b.cols, ["title", "status", "file.name"]);
}

#[test]
fn test_sr_13_cannot_unbind_every_exit() {
    // [SR-13][SR-16] 抜ける動作のキーを全部外す・置き換える設定は当てず、既定のキーを残して警告する
    // (終了・パレット・ヘルプを閉じる・入力の取り消しの手段が無くならない)。
    let tmp = notes("sr13exit");
    let mut a = boot_toml(
        &tmp,
        "[keys.table]\nq = \"none\"\n\":\" = \"none\"\n\"ctrl+p\" = \"none\"\n\
         [keys.help]\nesc = \"none\"\nq = \"none\"\n\"?\" = \"down\"\n\
         [keys.edit]\nesc = \"none\"\n",
    );
    let m = a.message.clone().expect("警告");
    assert!(m.contains("quit") && m.contains("palette"), "{m}");
    assert!(m.contains("ヘルプ") && m.contains("close"), "{m}");
    assert!(m.contains("cancel"), "{m}");
    for (mode, act) in [
        (Mode::Table, keymap::Action::Quit),
        (Mode::Table, keymap::Action::Palette),
        (Mode::Help, keymap::Action::Close),
        (Mode::Edit, keymap::Action::Cancel),
    ] {
        assert!(
            !keymap::keys_of(&a.keys, mode, act).is_empty(),
            "{mode:?} {act:?}"
        );
    }
    // ヘルプを開いて閉じ、q で終われる。
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    ch(&mut a, 'q');
    assert!(a.quit);
    // 1本でも残せば、ほかは外せる(警告なし)。
    let a = boot_toml(&tmp, "[keys.table]\nq = \"none\"\n\":\" = \"none\"\n");
    assert_eq!(a.message, None, "Ctrl+P のパレットから終われる");
    assert_eq!(keymap::lookup(&a.keys, Mode::Table, "q"), None);
}

#[test]
fn test_sr_12_state_hiding_every_column_keeps_one() {
    // [SR-12][NV-4] 状態で全部の列を隠していても、列の無い表では起動しない(最後の列は隠せない)。
    let tmp = notes("sr12all");
    let cols = boot(&tmp).cols;
    let mut a = boot(&tmp);
    while a.cols.len() > 1 {
        ch(&mut a, '-');
    }
    ch(&mut a, 'q');
    let st = config::ViewState {
        order: cols.clone(),
        hidden: cols.clone(),
        ..Default::default()
    };
    config::save_state(&state_dir(&tmp), &tmp.notes(), "", &st).unwrap();
    let b = boot(&tmp);
    assert_eq!(b.cols.len(), 1, "{:?}", b.cols);
    assert_eq!(b.hidden.len(), cols.len() - 1);
}

#[test]
fn test_sr_12_view_named_empty_not_overwritten() {
    // [SR-12] 空の名前のビューの状態も、そのビューの名前ごとに残る。[BV-13] 起動は先頭のビュー。
    let tmp = vault("sr12empty");
    let text = TASKS.replace("name: 完了", "name: \"\"");
    base_path(&tmp, &text);
    let mut a = boot_base(&tmp, None);
    ch(&mut a, ']');
    assert_eq!(a.view_index(), 1);
    col_named(&mut a, "title");
    ch(&mut a, '>');
    let w = view::layout(&a).widths[a.col];
    ch(&mut a, 'q');
    let mut b = boot_base(&tmp, None);
    assert_eq!(b.view_index(), 0, "起動は先頭のビュー(BV-13)");
    ch(&mut b, ']');
    assert_eq!(b.view_index(), 1);
    col_named(&mut b, "title");
    assert_eq!(
        view::layout(&b).widths[b.col],
        w,
        "空の名前のビューの状態が残る"
    );
}

#[test]
fn test_bv_13_startup_opens_first_view() {
    // [BV-13] 起動は先頭のビュー。別のビューを選んで終えても、次の起動は先頭のビューで、
    // `--view` があればそのビュー。見た目の状態(SR-12)はビューごとの列の順に使い続ける。
    let tmp = vault("bv13start");
    base_path(&tmp, TASKS);
    let mut a = boot_base(&tmp, None);
    assert_eq!(a.view_index(), 0);
    ch(&mut a, ']');
    ch(&mut a, ']');
    assert_eq!(a.view_index(), 2);
    col_named(&mut a, "status");
    ch(&mut a, 'H');
    let cols = a.cols.clone();
    ch(&mut a, 'q');
    let b = boot_base(&tmp, None);
    assert_eq!(b.view_index(), 0, "前に選んだビューは起動に使わない");
    assert!(screen(&b).lines().nth(1).unwrap().contains("[進行中]"));
    let c = boot_base(&tmp, Some("全部"));
    assert_eq!(c.view_index(), 2);
    assert_eq!(c.cols, cols, "ビューごとの列の順は残る");
}
