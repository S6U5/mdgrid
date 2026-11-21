//! 縦に並べる(specs/requirements/stack.md)の画面の試験。表は main と同じ道筋(`open_target` → `App::new` →
//! `start`)で開き、設定のフォルダは渡さない(範囲は印 `.mdgrid/workspace.toml` から決まる)。期待値は要求の
//! 受け入れの例から取った。

use super::keymap::{Action, Mode};
use super::stack::{heights, table_label, StackRequest, MIN_H};
use super::startup::Startup;
use super::test_screen::text;
use super::*;
use mdgrid::source::NewValue;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use std::path::PathBuf;

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-stack-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Tmp(std::fs::canonicalize(&dir).unwrap_or(dir))
    }

    fn ws(&self) -> PathBuf {
        self.0.join("ws")
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.ws().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TASKS_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"tasks\")\nviews:\n  - type: table\n    name: 今日\n    filters:\n      and:\n        - status != \"done\"\n    order:\n      - file.name\n      - title\n      - status\n  - type: table\n    name: 全部\n    order:\n      - file.name\n      - title\n      - status\n";

const PROJECTS_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"projects\")\nviews:\n  - type: table\n    name: 進行中\n    filters:\n      and:\n        - status == \"進行中\"\n    order:\n      - file.name\n      - title\n      - status\n  - type: table\n    name: 全部\n    order:\n      - file.name\n      - title\n      - status\n";

/// ワークスペース work(印): tasks(今日・全部)と projects(進行中・全部)の `.base`、同じフォルダを指す表
/// tasks2、フォルダの表 notes。`marker_extra` は印の末尾に足す文。
fn workspace(tag: &str, marker_extra: &str) -> Tmp {
    let t = Tmp::new(tag);
    t.write("tasks/t1.md", "---\ntitle: 会議の準備\nstatus: todo\n---\n");
    t.write("tasks/t2.md", "---\ntitle: 買い物\nstatus: todo\n---\n");
    t.write("tasks/t3.md", "---\ntitle: 読書\nstatus: done\n---\n");
    for (n, title, status) in [
        ("p1", "会議室の予約", "進行中"),
        ("p2", "新製品", "進行中"),
        ("p3", "採用", "進行中"),
        ("p4", "会議の改善", "進行中"),
        ("p5", "旧案", "完了"),
    ] {
        t.write(
            &format!("projects/{n}.md"),
            &format!("---\ntitle: {title}\nstatus: {status}\n---\n"),
        );
    }
    t.write("notes/n1.md", "---\ntitle: メモ\n---\n");
    t.write("tasks.base", TASKS_BASE);
    t.write("projects.base", PROJECTS_BASE);
    t.write(
        ".mdgrid/workspace.toml",
        &format!(
            "name = \"work\"\n\n[[table]]\nname = \"tasks\"\npath = \"tasks.base\"\n\n[[table]]\nname = \"projects\"\npath = \"projects.base\"\n\n[[table]]\nname = \"tasks2\"\npath = \"tasks\"\n\n[[table]]\nname = \"notes\"\npath = \"notes\"\n{marker_extra}"
        ),
    );
    t
}

/// main と同じ道筋で表を開いて読み終える(設定のフォルダなし)。
fn open(t: &Tmp, rel: &str) -> App {
    open_with(t, rel, "")
}

/// `open` と同じで、設定の文 `conf` を当てる。
fn open_with(t: &Tmp, rel: &str, conf: &str) -> App {
    let path = t.ws().join(rel);
    let target = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut app = App::new(target.src, ColorMode::None);
    app.resize(80, 24);
    let (config, _) = mdgrid::config::parse(conf).unwrap();
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: path,
        base: target.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.message = None;
    app
}

/// tasks の下に projects を並べる(stack.add と同じ道。projects を選ぶ)。
fn stacked(t: &Tmp) -> App {
    let mut app = open(t, "tasks.base");
    let mut next = open(t, "projects.base");
    next.place_name = Some("projects".into());
    app.stack_push(next);
    app.resize(80, 24);
    app
}

fn draw_at(app: &mut App, w: u16, h: u16) -> String {
    app.resize(w, h);
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw_screen(f, app)).unwrap();
    text(term.backend().buffer())
}

fn draw(app: &mut App) -> String {
    draw_at(app, 80, 24)
}

fn key(app: &mut App, code: KeyCode, m: KeyModifiers) {
    app.key(KeyEvent::new(code, m));
}

/// `Ctrl+w` のあとに `c`。
fn window(app: &mut App, c: char) {
    key(app, KeyCode::Char('w'), KeyModifiers::CONTROL);
    key(app, KeyCode::Char(c), KeyModifiers::NONE);
}

fn line(s: &str, y: usize) -> String {
    s.lines().nth(y).unwrap_or_default().to_string()
}

fn focus(app: &App) -> usize {
    app.stack.as_ref().map_or(0, |s| s.focus)
}

fn rows_of(app: &mut App, k: usize) -> usize {
    if k == focus(app) {
        app.rows.len()
    } else {
        app.with_table(k, |o| o.rows.len()).unwrap()
    }
}

#[test]
fn test_stack_add_two_halves() {
    // [stack.add] tasks の下に projects を並べる → 上半分に tasks、下半分に projects の行(本物の実行ファイルの
    // 試験は tests/test_stack_e2e.rs)。
    let t = workspace("add", "");
    let mut app = stacked(&t);
    let s = draw(&mut app);
    let hs = app.stack.as_ref().unwrap().heights();
    assert_eq!(hs, vec![11, 10]);
    assert!(line(&s, 0).starts_with("  tasks · 今日 · 2"), "{s}");
    assert!(line(&s, 11).starts_with("▶ projects · 進行中 · 4"), "{s}");
    let top: String = s.lines().take(11).collect::<Vec<_>>().join("\n");
    let bottom: String = s.lines().skip(11).take(10).collect::<Vec<_>>().join("\n");
    assert!(top.contains("会議の準備") && !top.contains("新製品"), "{s}");
    assert!(
        bottom.contains("新製品") && !bottom.contains("会議の準備"),
        "{s}"
    );
}

#[test]
fn test_stack_title_marks_the_focused_table() {
    // [utest -> req~stack.title~1] projects を選ぶ → projects の見出しの頭に選びの印、tasks の見出しには無い。
    // 見出しは「projects · 進行中 · 4」の形。
    let t = workspace("title", "");
    let mut app = stacked(&t);
    let s = draw(&mut app);
    assert_eq!(line(&s, 11), "▶ projects · 進行中 · 4", "{s}");
    assert_eq!(line(&s, 0), "  tasks · 今日 · 2", "{s}");
    window(&mut app, 'k');
    let s = draw(&mut app);
    assert_eq!(line(&s, 0), "▶ tasks · 今日 · 2", "{s}");
    assert_eq!(line(&s, 11), "  projects · 進行中 · 4", "{s}");
}

#[test]
fn test_stack_focus_keys_and_click() {
    // [utest -> req~stack.focus~1] 上の tasks を選んで Ctrl+w j → 下の projects が選ばれ、j が projects の中の行を
    // 動かす。いちばん下で Ctrl+w j → 動かない。下の表の行をクリック → その表が選ばれ、その行が選ばれる。
    let t = workspace("focus", "");
    let mut app = stacked(&t);
    window(&mut app, 'k');
    assert_eq!(table_label(&app), "tasks");
    window(&mut app, 'j');
    assert_eq!(table_label(&app), "projects");
    let before_top = app.with_table(0, |o| o.row).unwrap();
    let r = app.row;
    key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.row, r + 1);
    assert_eq!(app.with_table(0, |o| o.row).unwrap(), before_top);
    window(&mut app, 'j');
    assert_eq!(table_label(&app), "projects");
    assert_eq!(focus(&app), 1);
    // 上の tasks の2つ目の行(見出し・列の見出しの下)をクリック → tasks が選ばれ、その行。
    draw(&mut app);
    app.click(20, 3);
    assert_eq!(table_label(&app), "tasks");
    assert_eq!(app.row, 1);
    assert_eq!(app.mode, Mode::Table);
    // 見出しの行のクリックは、その表を選ぶだけ。
    app.click(5, 11);
    assert_eq!(table_label(&app), "projects");
    assert_eq!(app.mode, Mode::Table);
}

#[test]
fn test_stack_each_table_search_and_view() {
    // [utest -> req~stack.each-table~1] 下の projects を選んで `\` で「会議」と打つ → projects だけが絞られ、上の
    // tasks の行は変わらない。projects でビューのタブを移る → projects だけのビューが替わる。
    let t = workspace("each", "");
    let mut app = stacked(&t);
    let tasks_before = rows_of(&mut app, 0);
    key(&mut app, KeyCode::Char('\\'), KeyModifiers::NONE);
    for c in "会議".chars() {
        key(&mut app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.rows.len(), 2);
    assert_eq!(rows_of(&mut app, 0), tasks_before);
    let s = draw(&mut app);
    assert!(s.contains("会議室の予約") && !s.contains("新製品"), "{s}");
    let tasks_view = app.with_table(0, |o| o.view_index()).unwrap();
    key(&mut app, KeyCode::Char(']'), KeyModifiers::NONE);
    assert_eq!(app.view_index(), 1);
    assert_eq!(app.with_table(0, |o| o.view_index()).unwrap(), tasks_view);
}

#[test]
fn test_stack_close_and_last_table() {
    // [utest -> req~stack.close~1] tasks と projects を並べて projects を選び Ctrl+w c → tasks だけの画面。表が
    // 1つだけのときの Ctrl+w c → 何もせず、メッセージ行に閉じられない理由。
    let t = workspace("close", "");
    let mut app = stacked(&t);
    window(&mut app, 'c');
    assert!(app.stack.is_none());
    assert_eq!(table_label(&app), "tasks");
    let s = draw(&mut app);
    assert!(!s.contains("新製品"), "{s}");
    window(&mut app, 'c');
    assert!(app.stack.is_none());
    assert_eq!(
        app.message.as_deref(),
        Some(mdgrid::i18n::Msg::StackCloseOnly.text())
    );
    // パレットの「この表を閉じる」も同じ動作。
    let mut app = stacked(&t);
    app.apply(Action::StackClose);
    assert!(app.stack.is_none());
}

#[test]
fn test_stack_limit_four_and_height() {
    // [utest -> req~stack.limit~1] 表を4つ並べて「下に表を並べる」→ 足さずに理由。高さ 12 行の端末で2つ目を
    // 足す → 入るなら足す。入らなければ理由。端末を低くして入らなくなる → 下の表から順に畳んで見出しだけにする。
    let t = workspace("limit", "");
    let mut app = stacked(&t);
    for rel in ["tasks", "notes"] {
        let next = open(&t, rel);
        app.stack_push(next);
    }
    assert_eq!(app.stack_len(), 4);
    app.apply(Action::StackAdd);
    assert_eq!(
        app.message.as_deref(),
        Some(
            mdgrid::i18n::Msg::StackMax
                .fill(&[&super::stack::MAX_TABLES])
                .as_str()
        )
    );
    assert_ne!(app.mode, Mode::Palette);
    // 高さ 12 行: 表1つの下に2つ目は入る(見出しと3行が2つ)。
    let mut one = open(&t, "tasks.base");
    one.resize(80, 12);
    one.apply(Action::StackAdd);
    assert_eq!(one.mode, Mode::Palette, "{:?}", one.message);
    // 高さ 9 行: 入らないので理由。
    let mut low = open(&t, "tasks.base");
    low.resize(80, 9);
    low.apply(Action::StackAdd);
    assert_eq!(
        low.message.as_deref(),
        Some(mdgrid::i18n::Msg::StackNoRoom.text())
    );
    // 並べてから端末を低くする: 選んでいる上の表は残し、下の表を見出しだけにする。
    let mut two = stacked(&t);
    window(&mut two, 'k');
    let s = draw_at(&mut two, 80, 8);
    assert_eq!(two.stack.as_ref().unwrap().heights(), vec![4, 1]);
    assert!(line(&s, 0).starts_with("▶ tasks"), "{s}");
    assert!(line(&s, 4).starts_with("  projects · 進行中 · 4"), "{s}");
    assert_eq!(heights(&[1, 1, 1], 0, 6), vec![4, 1, 1]);
    assert_eq!(heights(&[1, 1, 1], 2, 6), vec![1, 1, 4]);
}

#[test]
fn test_stack_resize_keys() {
    // [utest -> req~stack.resize~1] 2つ並べて上を選び Ctrl+w + を2回 → 上が2行高く、下が2行低い。下が見出しと
    // 3行になったら、それ以上は上を高くしない。
    let t = workspace("resize", "");
    let mut app = stacked(&t);
    window(&mut app, 'k');
    let before = app.stack.as_ref().unwrap().heights();
    assert_eq!(before, vec![11, 10]);
    window(&mut app, '+');
    window(&mut app, '+');
    assert_eq!(app.stack.as_ref().unwrap().heights(), vec![13, 8]);
    for _ in 0..10 {
        window(&mut app, '+');
    }
    assert_eq!(
        app.stack.as_ref().unwrap().heights(),
        vec![21 - MIN_H, MIN_H]
    );
    assert_eq!(
        app.message.as_deref(),
        Some(mdgrid::i18n::Msg::StackGrowLimit.text())
    );
    window(&mut app, '-');
    assert_eq!(app.stack.as_ref().unwrap().heights(), vec![16, 5]);
    // 描いた画面でも、下の表の見出しはその高さの下。
    let s = draw(&mut app);
    assert!(line(&s, 16).starts_with("  projects"), "{s}");
}

#[test]
fn test_stack_shared_pending_same_note() {
    // [stack.shared-pending] 同じフォルダを指す表を2つ並べ、上の表で t1.md の status、下の表で title を直して保存
    // → どちらの変更も書く(本物の実行ファイルの試験は tests/test_stack_e2e.rs)。
    let t = workspace("shared", "");
    let mut app = open(&t, "tasks");
    let mut next = open(&t, "tasks");
    next.place_name = Some("tasks2".into());
    app.stack_push(next);
    app.resize(80, 24);
    let row = mdgrid::source::RowId(
        t.ws()
            .join("tasks/t1.md")
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    );
    window(&mut app, 'k');
    app.changes
        .set(
            app.src.as_ref(),
            &row,
            "status",
            NewValue::Str("done".into()),
        )
        .unwrap();
    window(&mut app, 'j');
    // 下の表にも、上の表でためた値がためた変更として見える。
    assert_eq!(
        app.changes.pending(&row, "status"),
        Some(&NewValue::Str("done".into()))
    );
    app.changes
        .set(app.src.as_ref(), &row, "title", NewValue::Str("新".into()))
        .unwrap();
    app.apply(Action::Save);
    assert_eq!(app.mode, Mode::Confirm);
    app.apply(Action::SaveAll);
    let body = std::fs::read_to_string(t.ws().join("tasks/t1.md")).unwrap();
    assert!(body.contains("status: done"), "{body}");
    assert!(body.contains("title: 新"), "{body}");
    assert_eq!(app.changes.count(), 0);
}

#[test]
fn test_stack_save_all_one_review_by_table() {
    // [utest -> req~stack.save-all~1] 上で1つ・下で2つ直して Ctrl+s → 差分の確認に tasks の1件と projects の2件が
    // 見出しつきで並び、確定で3件とも書く。
    let t = workspace("save-all", "");
    let mut app = stacked(&t);
    let id = |rel: &str| {
        mdgrid::source::RowId(
            t.ws()
                .join(rel)
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        )
    };
    window(&mut app, 'k');
    app.changes
        .set(
            app.src.as_ref(),
            &id("tasks/t2.md"),
            "title",
            NewValue::Str("買い出し".into()),
        )
        .unwrap();
    window(&mut app, 'j');
    for rel in ["projects/p2.md", "projects/p3.md"] {
        app.changes
            .set(
                app.src.as_ref(),
                &id(rel),
                "title",
                NewValue::Str("改め".into()),
            )
            .unwrap();
    }
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(app.mode, Mode::Confirm);
    let tables: Vec<Option<String>> = app
        .review
        .as_ref()
        .unwrap()
        .items
        .iter()
        .map(|i| i.table.clone())
        .collect();
    assert_eq!(
        tables,
        vec![
            Some("tasks".to_string()),
            Some("projects".to_string()),
            Some("projects".to_string())
        ]
    );
    let s = draw(&mut app);
    let tasks_at = s.find("── tasks ──").expect(&s);
    let projects_at = s.find("── projects ──").expect(&s);
    assert!(tasks_at < projects_at, "{s}");
    assert_eq!(s.matches("── projects ──").count(), 1, "{s}");
    app.apply(Action::SaveAll);
    assert_eq!(app.changes.count(), 0);
    let read = |rel: &str| std::fs::read_to_string(t.ws().join(rel)).unwrap();
    assert!(read("tasks/t2.md").contains("title: 買い出し"));
    assert!(read("projects/p2.md").contains("title: 改め"));
    assert!(read("projects/p3.md").contains("title: 改め"));
}

#[test]
fn test_stack_close_pending_asks() {
    // [utest -> req~stack.close-pending~1] 下の projects に1件ためて Ctrl+w c → 保存か取りやめの確認。取りやめ →
    // projects は並んだまま。保存 → その表の変更だけの差分を確かめて書き、閉じる。
    let t = workspace("close-pending", "");
    let mut app = stacked(&t);
    let p2 = mdgrid::source::RowId(
        t.ws()
            .join("projects/p2.md")
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    );
    app.changes
        .set(
            app.src.as_ref(),
            &p2,
            "status",
            NewValue::Str("完了".into()),
        )
        .unwrap();
    window(&mut app, 'c');
    assert_eq!(app.mode, Mode::Quit);
    assert_eq!(app.stack_len(), 2);
    let s = draw(&mut app);
    assert!(s.contains("projects に未保存の変更が 1 ある"), "{s}");
    assert!(s.contains("保存して閉じる"), "{s}");
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.mode, Mode::Table);
    assert_eq!(app.stack_len(), 2);
    assert_eq!(table_label(&app), "projects");
    assert_eq!(app.changes.count(), 1);
    // もう一度閉じて保存する → 差分の確認 → 確定で書いて閉じる。
    window(&mut app, 'c');
    key(&mut app, KeyCode::Char('s'), KeyModifiers::NONE);
    assert_eq!(app.mode, Mode::Confirm);
    app.apply(Action::SaveAll);
    assert!(app.stack.is_none());
    assert_eq!(table_label(&app), "tasks");
    let body = std::fs::read_to_string(t.ws().join("projects/p2.md")).unwrap();
    assert!(body.contains("status: 完了"), "{body}");
}

#[test]
fn test_stack_close_keeps_changes_shown_elsewhere() {
    // [utest -> req~stack.close-pending~1] 閉じる表の変更が、残る表にも出ているノートのものなら、確かめずに閉じ、
    // 変更は残る表のためた変更として残る(黙って捨てない)。
    let t = workspace("close-shared", "");
    let mut app = open(&t, "tasks");
    let mut next = open(&t, "tasks");
    next.place_name = Some("tasks2".into());
    app.stack_push(next);
    let row = mdgrid::source::RowId(
        t.ws()
            .join("tasks/t1.md")
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    );
    app.changes
        .set(
            app.src.as_ref(),
            &row,
            "status",
            NewValue::Str("done".into()),
        )
        .unwrap();
    window(&mut app, 'c');
    assert!(app.stack.is_none());
    assert_eq!(app.changes.count(), 1);
}

#[test]
fn test_stack_page_open_from_drawer() {
    // [utest -> req~stack.page-open~1] ページ「朝」のある work で引き出しを開く → 表の上に「朝」→ 選ぶ → その
    // ページを開くよう頼む(main がページの表を並べて開く)。
    let t = workspace(
        "page-open",
        "\n[[page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\" },\n  { table = \"projects\", view = \"進行中\" },\n]\n",
    );
    let mut app = open(&t, "tasks.base");
    assert_eq!(app.pages.len(), 1);
    app.apply(Action::WsDrawer);
    assert_eq!(app.mode, Mode::Drawer);
    let s = draw(&mut app);
    let page_at = s.find("朝 · ページ").expect(&s);
    let table_at = s.find("tasks2").expect(&s);
    assert!(page_at < table_at, "{s}");
    // 引き出しの頭(ページ)を選んで Enter。
    if let Some(d) = app.drawer.as_mut() {
        d.sel = 0;
    }
    app.apply(Action::Run);
    assert_eq!(app.stack_request, Some(StackRequest::Page("朝".into())));
    assert!(app.quit);
}

#[test]
fn test_stack_page_save_from_palette_to_marker() {
    // [utest -> req~stack.page-save-marker~1] [stack.page-save] tasks(今日)と projects(進行中)を並べ、名前
    // 「朝」で残す → 印に `[[page]]`・`name = "朝"`・2つの表とビューを足す(前の文はそのまま)。
    let t = workspace("page-save", "");
    let marker = t.ws().join(".mdgrid/workspace.toml");
    let before = std::fs::read_to_string(&marker).unwrap();
    let mut app = stacked(&t);
    app.apply(Action::PageSave);
    assert_eq!(app.mode, Mode::Palette, "{:?}", app.message);
    for c in "朝".chars() {
        key(&mut app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    let after = std::fs::read_to_string(&marker).unwrap();
    assert!(after.starts_with(&before), "{after}");
    assert!(
        after.ends_with("[[page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\", height = 1 },\n  { table = \"projects\", view = \"進行中\", height = 1 },\n]\n"),
        "{after}"
    );
    assert_eq!(app.pages.len(), 1);
    assert_eq!(app.stack_page().map(|p| p.0), Some("朝".to_string()));
}

#[test]
fn test_stack_page_save_needs_workspace() {
    // [stack.page-save] 範囲がワークスペースでないとき → 残さずに理由。
    let t = Tmp::new("page-no-ws");
    t.write("a/x.md", "---\ntitle: x\n---\n");
    t.write("b/y.md", "---\ntitle: y\n---\n");
    let mut app = open(&t, "a");
    let next = open(&t, "b");
    app.stack_push(next);
    app.apply(Action::PageSave);
    assert_ne!(app.mode, Mode::Palette);
    assert_eq!(
        app.message.as_deref(),
        Some(mdgrid::i18n::Msg::PageNotWorkspace.text())
    );
}

#[test]
fn test_stack_page_resume_command_and_title() {
    // [utest -> req~stack.page-resume~1, req~resume.herdr-title-form~1] ページ「朝」を開いている → 伝える命令は
    // `mdgrid -w work --page 朝`、窓の題は「朝」。その場で並べた形 → 選んでいる表とビュー。
    let mut r = mdgrid::resume::Reopen::new(
        &[PathBuf::from("/x/notes/tasks")],
        Some("work".into()),
        false,
        false,
        None,
    );
    r.page = Some("朝".into());
    assert_eq!(r.argv(), vec!["mdgrid", "-w", "work", "--page", "朝"]);
    let t = workspace(
        "resume",
        "\n[[page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\" },\n  { table = \"projects\" },\n]\n",
    );
    let mut app = stacked(&t);
    // その場で並べた形: 命令は選んでいる表(projects)とビュー。
    assert_eq!(app.stack_page(), None);
    let base =
        mdgrid::resume::Reopen::new(&super::stack::target_paths(&app), None, false, false, None);
    assert!(
        base.argv()[1].ends_with("projects.base"),
        "{:?}",
        base.argv()
    );
    // ページを開いた形: 印のワークスペースは表のパスと --page。
    app.stack.as_mut().unwrap().page = Some("朝".into());
    let (page, ws) = app.stack_page().unwrap();
    assert_eq!(page, "朝");
    assert_eq!(ws, None);
    let mut now = base.clone();
    now.page = Some(page);
    let argv = now.argv();
    assert_eq!(&argv[argv.len() - 2..], ["--page", "朝"]);
    assert_eq!(crate::reopen::title(&app, &now, 2), "朝");
}

#[test]
fn test_stack_page_cli_unknown_name() {
    // [utest -> req~stack.page-cli~1] 無い名前 → 「ページ「夜」はありません(ある名前: 朝)」の1行。
    let t = workspace(
        "page-cli",
        "\n[[page]]\nname = \"朝\"\ntables = [{ table = \"tasks\" }, { table = \"projects\" }]\n",
    );
    let app = open(&t, "tasks.base");
    let e = crate::open_page(&app, "夜", &crate::Options::default())
        .err()
        .unwrap();
    assert_eq!(e, "ページ「夜」はありません(ある名前: 朝)");
}

#[test]
fn test_stack_page_store_warns_at_start() {
    // [utest -> req~stack.page-store~1] 印のページに無い表の名前 → 起動して警告が出て、ページは残りの表。
    let t = workspace(
        "page-store",
        "\n[[page]]\nname = \"朝\"\ntables = [{ table = \"tasks\" }, { table = \"nothing\" }, { table = \"projects\" }]\n",
    );
    let path = t.ws().join("tasks.base");
    let target = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut app = App::new(target.src, ColorMode::None);
    let (config, _) = mdgrid::config::parse("").unwrap();
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: path,
        base: target.base,
    });
    let m = app.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: page[1].tables[2].table: "),
        "{m}"
    );
    let names: Vec<&str> = app.pages[0]
        .tables
        .iter()
        .map(|x| x.table.as_str())
        .collect();
    assert_eq!(names, ["tasks", "projects"]);
}

#[test]
fn test_stack_screen_fits_and_other_screens_full() {
    // [stack.title] どの行も幅以下。ガントなど表でない画面とヘルプは、選んでいる表だけを画面いっぱいに出す。
    let t = workspace("full", "");
    let mut app = stacked(&t);
    let s = draw(&mut app);
    for l in s.lines() {
        assert!(width::width(l) <= 80, "{l}");
    }
    app.apply(Action::Help);
    assert_eq!(app.mode, Mode::Help);
    assert_eq!(app.size, (80, 24));
    let s = draw(&mut app);
    assert!(!s.contains("  tasks · 今日"), "{s}");
    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.mode, Mode::Table);
    assert_eq!(app.size, (80, 11));
}

#[test]
fn test_stack_notice_band_stays_on_top() {
    // [stack.title] [notice.opt-in] 通知の欄([[notices]])を書いた人は、表を並べても画面のいちばん上に欄が出る
    // (選んでいる表の欄)。区画はその下から並べ、欄のクリックは欄へ移る。
    let t = workspace("notice", "");
    let conf = "[conditions.todo]\nexpr = 'status == \"todo\"'\n\n[[notices]]\ncond = \"todo\"\ntext = \"未着手 {count} 件\"\n";
    let mut app = open_with(&t, "tasks.base", conf);
    let next = open_with(&t, "projects.base", conf);
    app.stack_push(next);
    window(&mut app, 'k');
    app.notice_poll();
    let s = draw(&mut app);
    assert!(line(&s, 0).contains("未着手 2 件"), "{s}");
    assert!(line(&s, 1).starts_with("▶ tasks · 今日 · 2"), "{s}");
    assert_eq!(app.stack.as_ref().unwrap().heights(), vec![10, 10]);
    assert!(line(&s, 11).starts_with("  projects"), "{s}");
    app.click(3, 0);
    assert_eq!(app.mode, Mode::Notice);
    assert_eq!(app.stack_len(), 2);
    let s = draw(&mut app);
    assert!(line(&s, 1).starts_with("▶ tasks"), "{s}");
}
