#![cfg(unix)]
//! スターターから台帳を作る(ST-5)と、作った台帳を開いて閉じてもバイトが変わらないこと(ST-6)の、本物の実行ファイル
//! での確かめ(端末 80×24)。値は作ったもの。

use crate::pty;

use pty::{TempDir, Tui};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// `root` の下の全部のファイル(根からの相対)とその中身。
fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                map.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    map
}

/// 組み込みのスターター `name` から、`h` の下の `name` に見本つきで作る(画面なし。今日は 2026-10-06)。
fn make(h: &Path, name: &str) {
    make_in(h, name, "ja_JP.UTF-8");
}

/// `make` と同じで、言語を `lang` にする。
fn make_in(h: &Path, name: &str, lang: &str) {
    let root = h.join(name);
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args([
            root.to_str().unwrap(),
            "--init-workspace",
            "--starter",
            name,
            "--with-samples",
        ])
        .current_dir(h)
        .env("HOME", h)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", h.join("config"))
        .env("XDG_STATE_HOME", h.join("state"))
        .env("MDGRID_TODAY", "2026-10-06")
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// ノートのフォルダ notes/ とその下の tasks/(1件)。
fn home(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("notes/tasks/a.md", "---\nstatus: todo\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

#[test]
fn test_st_5_palette_creates_a_ledger_from_a_starter_and_opens_it_e2e() {
    // [stest -> req~starter.palette-create~2] `:` → 「スターターから台帳を作る」→ reading → 作る場所(既定 reading)→ y → 読書の表が開き、
    // タブに「読みたい」がある。q で終えると、作った台帳のファイルがある。
    let home = home("starter-e2e");
    let h = home.path();
    let mut tui = Tui::spawn(&["notes/tasks"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| {
        s.contains("a.md") || s.contains("status")
    });
    tui.send(":");
    tui.wait_until("パレット", |s| s.contains(": "));
    tui.send("スターターから台帳を作る");
    tui.wait_until("候補", |s| s.contains("スターターから台帳を作る"));
    tui.send("\r");
    tui.wait_until("一覧", |s| {
        s.contains("スターター: ") && s.contains("reading")
    });
    tui.send("reading");
    tui.send("\r");
    tui.wait_until("作る場所", |s| s.contains("作る場所: reading"));
    tui.send("\r");
    tui.wait_until("見本", |s| s.contains("見本のノートも入れる?"));
    tui.send("y");
    tui.send("\r");
    tui.wait_until("読書の表", |s| {
        s.contains("読みたい") && s.contains("reading")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
    let root = h.join("notes/tasks/reading");
    assert!(root.join(".mdgrid/workspace.toml").is_file());
    assert!(root.join(".mdgrid/templates/entry.md").is_file());
    assert!(root.join("Reading").is_dir(), "見本のフォルダ");
}

#[test]
fn test_st_5_tilde_folder_is_under_the_home_folder_e2e() {
    // [stest -> req~starter.palette-create~2] 作る場所の欄の既定(スターターの名前)を消して `~/books` と打つ → HOME の下の books に作り、
    // 開いたフォルダの下に `~` という名前のフォルダはできない。n で見本は入れない。
    let home = home("starter-e2e-tilde");
    let h = home.path();
    let mut tui = Tui::spawn(&["notes/tasks"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| s.contains("status"));
    tui.send(":");
    tui.wait_until("パレット", |s| s.contains(": "));
    tui.send("スターターから台帳を作る");
    tui.wait_until("候補", |s| s.contains("スターターから台帳を作る"));
    tui.send("\r");
    tui.wait_until("一覧", |s| {
        s.contains("スターター:") && s.contains("tasks")
    });
    tui.send("tasks");
    tui.send("\r");
    tui.wait_until("作る場所", |s| s.contains("作る場所: tasks"));
    for _ in 0.."tasks".len() {
        tui.send("\x7f");
    }
    tui.wait_until("欄が空", |s| {
        s.contains("作る場所:") && !s.contains("作る場所: tasks")
    });
    tui.send("~/books");
    tui.wait_until("打った場所", |s| s.contains("作る場所: ~/books"));
    tui.send("\r");
    tui.wait_until("見本", |s| s.contains("見本のノートも入れる?"));
    tui.send("n");
    tui.send("\r");
    tui.wait_until("タスクの表", |s| {
        s.contains("未完了") && s.contains("books")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
    assert!(
        h.join("books/.mdgrid/workspace.toml").is_file(),
        "HOME の下に作る"
    );
    assert!(!h.join("books/Tasks").exists(), "n なので見本は無い");
    assert!(
        !h.join("notes/tasks/~").exists(),
        "`~` という名前のフォルダは作らない"
    );
}

#[test]
fn test_st_6_opening_a_made_ledger_changes_no_bytes_e2e() {
    // [stest -> req~starter.same-as-manual-bytes~1] 組み込みの4つから見本つきで作った台帳を、開いて(表が描かれるまで待って)閉じても、印・雛形・見本の
    // バイトが1つも変わらない。
    let home = home("starter-e2e-bytes");
    let h = home.path();
    for name in ["tasks", "bookmarks", "reading", "people"] {
        make(h, name);
        let root = h.join(name);
        let before = files(&root);
        assert!(before.contains_key(".mdgrid/workspace.toml"), "{name}");
        let mut tui = Tui::spawn(&[name], h, &[("MDGRID_TODAY", "2026-10-06")]);
        tui.wait_until("表が出る", |s| s.contains("▦ 表 ▾"));
        tui.send("q");
        assert_eq!(tui.wait_exit(), 0, "{name}");
        assert_eq!(
            files(&root),
            before,
            "{name}: 開いて閉じたあとに変わったファイルがある"
        );
    }
}

#[test]
fn test_st_2_tasks_starter_prefixes_new_notes_and_hides_the_prefix_e2e() {
    // [stest -> req~starter.tasks-name-prefix~3] tasks から作った台帳は、新しいノートの名前を今日の8桁と `_` で始め、ノートの欄にも名前を打つ欄にもその頭を出さない。
    // 観点: 日本語と英語の変種の両方。頭の無い名前はそのまま出る。
    for (lang, word) in [
        ("ja_JP.UTF-8", "請求書を送る"),
        ("en_US.UTF-8", "Send the invoice"),
    ] {
        let home = TempDir::new("starter-prefix");
        let h = home.path();
        std::fs::create_dir_all(h.join("state")).unwrap();
        make_in(h, "tasks", lang);
        let note = "---\nstatus: todo\n---\n";
        std::fs::write(h.join(format!("tasks/Tasks/20261006_{word}.md")), note).unwrap();
        std::fs::write(h.join("tasks/Tasks/メモ.md"), note).unwrap();
        let mut tui = Tui::spawn(
            &["tasks"],
            h,
            &[("MDGRID_TODAY", "2026-10-06"), ("LANG", lang)],
        );
        tui.wait_until("表が出る", |s| s.contains(word) && s.contains("メモ"));
        assert!(
            !tui.screen().contains("20261006_"),
            "{lang}: ノートの欄に頭の日付が出ている\n{}",
            tui.screen()
        );
        tui.send("a");
        tui.wait_until("新しいノートの窓が開く", |s| s.contains("^S"));
        assert!(
            !tui.screen().contains("20261006_"),
            "{lang}: 名前の欄に頭の日付が出ている\n{}",
            tui.screen()
        );
        tui.send("memo2");
        tui.wait_until("打った名前が出る", |s| s.contains("memo2"));
        // Ctrl+S: 作る。
        tui.send("\x13");
        let made = h.join("tasks/Tasks/20261006_memo2.md");
        tui.wait_until("頭の付いた名前で作られる", |_| made.exists());
        tui.send("q");
        assert_eq!(tui.wait_exit(), 0, "{lang}");
    }
}

#[test]
fn test_st_2_tasks_starter_fills_the_done_date_e2e() {
    // [stest -> req~starter.tasks-done-at~1] tasks から作った台帳で状態を done にして保存すると、ノートの done_at に今日が書かれる。
    let home = TempDir::new("starter-done-at");
    let h = home.path();
    std::fs::create_dir_all(h.join("state")).unwrap();
    make_in(h, "tasks", "ja_JP.UTF-8");
    // 見本のノートは除き、札を1枚だけにする。
    std::fs::remove_dir_all(h.join("tasks/Tasks")).unwrap();
    std::fs::create_dir_all(h.join("tasks/Tasks")).unwrap();
    let note = h.join("tasks/Tasks/請求書を送る.md");
    std::fs::write(&note, "---\nstatus: todo\ndone_at:\n---\n").unwrap();
    let mut tui = Tui::spawn(&["tasks"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| s.contains("請求書を送る"));
    // カンバンで todo の札を、doing・waiting を通って done のレーンへ動かす。
    tui.send("B");
    tui.wait_until("カンバン", |s| s.contains("done"));
    for lane in ["doing", "waiting", "done"] {
        tui.send("L");
        tui.wait_until("札が動く", |s| s.contains(&format!("→ {lane}")));
    }
    tui.wait_until("未保存", |s| s.contains("未保存 2"));
    tui.send("\x13");
    tui.wait_for("+done_at: 2026-10-06");
    tui.send("\r");
    let body = pty::wait_file(&note, &tui, |s| s.contains("done_at: 2026"));
    assert_eq!(body, "---\nstatus: done\ndone_at: 2026-10-06\n---\n");
    tui.stop();
}

/// 新しいノートの窓で、欄の名前 `label` のある行(`│ > 名前` か `│   名前` の形)。
fn form_row(screen: &str, label: &str) -> Option<usize> {
    screen.lines().position(|l| {
        [
            format!("│ > {label} "),
            format!("│   {label} "),
            format!("│ > {label}│"),
            format!("│   {label}│"),
        ]
        .iter()
        .any(|p| l.contains(p.as_str()))
    })
}

#[test]
fn test_st_2_tasks_starters_ask_start_right_before_due_e2e() {
    // [stest -> req~starter.tasks-start-column~1] tasks と tasks-ai の新しいノートの窓は、名前の下に期限のすぐ前で開始日を聞く。
    for (starter, lang, labels) in [
        (
            "tasks",
            "ja_JP.UTF-8",
            &["名前", "開始", "期限", "優先度", "URL"][..],
        ),
        (
            "tasks",
            "en_US.UTF-8",
            &["name", "Start", "Due", "Priority", "URL"][..],
        ),
        (
            "tasks-ai",
            "ja_JP.UTF-8",
            &["名前", "誰が", "開始", "期限", "優先度"][..],
        ),
        (
            "tasks-ai",
            "en_US.UTF-8",
            &["name", "Actor", "Start", "Due", "Priority"][..],
        ),
    ] {
        let home = TempDir::new("starter-ask-start");
        let h = home.path();
        std::fs::create_dir_all(h.join("state")).unwrap();
        make_in(h, starter, lang);
        let mut tui = Tui::spawn(
            &[starter],
            h,
            &[("MDGRID_TODAY", "2026-10-06"), ("LANG", lang)],
        );
        tui.wait_until("表が出る", |s| s.contains("a +"));
        tui.send("a");
        tui.wait_until("新しいノートの窓が開く", |s| s.contains("^S"));
        let screen = tui.screen();
        let rows: Vec<Option<usize>> = labels.iter().map(|l| form_row(&screen, l)).collect();
        let first = rows[0].unwrap_or_else(|| panic!("{starter} {lang}: 名前の欄が無い\n{screen}"));
        let want: Vec<Option<usize>> = (0..labels.len()).map(|i| Some(first + i)).collect();
        assert_eq!(
            rows, want,
            "{starter} {lang}: 欄の並び {labels:?}\n{screen}"
        );
        tui.stop();
    }
}

#[test]
fn test_st_2_tasks_starters_ask_tags_e2e() {
    // [stest -> req~starter.tasks-ask-tags~1] tasks と tasks-ai の新しいタスクの窓はタグ(tasks-ai は URL も)を聞き、tasks-ai の新しいルーチンの窓もタグを聞く。
    for (starter, lang, table, labels) in [
        (
            "tasks",
            "ja_JP.UTF-8",
            None,
            &["名前", "開始", "期限", "優先度", "URL", "タグ"][..],
        ),
        (
            "tasks",
            "en_US.UTF-8",
            None,
            &["name", "Start", "Due", "Priority", "URL", "Tags"][..],
        ),
        (
            "tasks-ai",
            "ja_JP.UTF-8",
            None,
            &["名前", "誰が", "開始", "期限", "優先度", "タグ", "URL"][..],
        ),
        (
            "tasks-ai",
            "en_US.UTF-8",
            None,
            &["name", "Actor", "Start", "Due", "Priority", "Tags", "URL"][..],
        ),
        (
            "tasks-ai",
            "ja_JP.UTF-8",
            Some("ルーチン"),
            &["名前", "繰り返し", "誰が", "次にやる日", "タグ"][..],
        ),
        (
            "tasks-ai",
            "en_US.UTF-8",
            Some("Routines"),
            &["name", "Recurrence", "Actor", "Next", "Tags"][..],
        ),
    ] {
        let home = TempDir::new("starter-ask-tags");
        let h = home.path();
        std::fs::create_dir_all(h.join("state")).unwrap();
        make_in(h, starter, lang);
        let mut args = vec![starter];
        if let Some(t) = table {
            args.extend(["--table", t]);
        }
        let mut tui = Tui::spawn(&args, h, &[("MDGRID_TODAY", "2026-10-06"), ("LANG", lang)]);
        tui.wait_until("表が出る", |s| s.contains("a +"));
        tui.send("a");
        tui.wait_until("新しいノートの窓が開く", |s| s.contains("│ > "));
        let screen = tui.screen();
        let rows: Vec<Option<usize>> = labels.iter().map(|l| form_row(&screen, l)).collect();
        let first = rows[0]
            .unwrap_or_else(|| panic!("{starter} {lang} {table:?}: 名前の欄が無い\n{screen}"));
        let want: Vec<Option<usize>> = (0..labels.len()).map(|i| Some(first + i)).collect();
        assert_eq!(
            rows, want,
            "{starter} {lang} {table:?}: 欄の並び {labels:?}\n{screen}"
        );
        tui.stop();
    }
}

#[test]
fn test_st_2_tasks_starters_fill_the_start_date_on_doing_e2e() {
    // [stest -> req~starter.tasks-start-on-doing~1] tasks と tasks-ai で状態を doing にして保存すると、開始日が空のときだけ今日が入る。
    for (starter, before, after, pending) in [
        (
            "tasks",
            "---\nstatus: todo\n---\n",
            "---\nstatus: doing\nstart: 2026-10-06\n---\n",
            2,
        ),
        (
            "tasks-ai",
            "---\nstatus: todo\n---\n",
            "---\nstatus: doing\nstart: 2026-10-06\n---\n",
            2,
        ),
        (
            "tasks",
            "---\nstatus: todo\nstart: 2026-10-01\n---\n",
            "---\nstatus: doing\nstart: 2026-10-01\n---\n",
            1,
        ),
    ] {
        let home = TempDir::new("starter-start-doing");
        let h = home.path();
        std::fs::create_dir_all(h.join("state")).unwrap();
        make_in(h, starter, "ja_JP.UTF-8");
        // 見本のノートは除き、札を1枚だけにする。
        std::fs::remove_dir_all(h.join(format!("{starter}/Tasks"))).unwrap();
        std::fs::create_dir_all(h.join(format!("{starter}/Tasks"))).unwrap();
        let note = h.join(format!("{starter}/Tasks/請求書を送る.md"));
        std::fs::write(&note, before).unwrap();
        let mut tui = Tui::spawn(&[starter], h, &[("MDGRID_TODAY", "2026-10-06")]);
        tui.wait_until("表が出る", |s| s.contains("請求書を送る"));
        // カンバンで todo の札を doing のレーンへ動かす。
        tui.send("B");
        tui.wait_until("カンバン", |s| s.contains("doing"));
        tui.send("L");
        tui.wait_until("札が動く", |s| s.contains("→ doing"));
        tui.wait_until("未保存", |s| s.contains(&format!("未保存 {pending}")));
        tui.send("\x13");
        tui.wait_for("+status: doing");
        tui.send("\r");
        let body = pty::wait_file(&note, &tui, |s| s.contains("status: doing"));
        assert_eq!(body, after, "{starter}: {before:?}");
        tui.stop();
    }
}

#[test]
fn test_st_2_tasks_ai_starter_fills_the_done_date_e2e() {
    // [stest -> req~starter.tasks-ai-done-at~1] tasks-ai から作った台帳で状態を done にして保存すると、ノートの done_at に今日が書かれる。
    let home = TempDir::new("starter-ai-done-at");
    let h = home.path();
    std::fs::create_dir_all(h.join("state")).unwrap();
    make_in(h, "tasks-ai", "ja_JP.UTF-8");
    std::fs::remove_dir_all(h.join("tasks-ai/Tasks")).unwrap();
    std::fs::create_dir_all(h.join("tasks-ai/Tasks")).unwrap();
    let note = h.join("tasks-ai/Tasks/請求書を送る.md");
    std::fs::write(&note, "---\nstatus: todo\n---\n").unwrap();
    let mut tui = Tui::spawn(&["tasks-ai"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| s.contains("請求書を送る"));
    tui.send("B");
    tui.wait_until("カンバン", |s| s.contains("done"));
    for lane in ["doing", "waiting", "done"] {
        tui.send("L");
        tui.wait_until("札が動く", |s| s.contains(&format!("→ {lane}")));
    }
    tui.send("\x13");
    tui.wait_for("+done_at: 2026-10-06");
    tui.send("\r");
    let body = pty::wait_file(&note, &tui, |s| s.contains("status: done"));
    assert!(body.contains("done_at: 2026-10-06"), "{body}");
}
