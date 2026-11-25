//! お気に入りのワークスペースと切り替え(specs/requirements/workspace.md の「お気に入りのワークスペースと切り替え」)。
//! 期待値は要求の受け入れの例から取った。見本: notes/ に tasks・projects(projects.base でも開く)・books・scratch。
//! workspaces.toml に work(tasks・projects.base)と 趣味(books)。印のワークスペース lab(x、木の設定つき)。
//! 一覧を開く本物の実行ファイルの画面は tests/test_switch_e2e.rs で見る。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::favorites;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use mdgrid::workspace::{self, WsTable};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const BASE: &str = "views:\n  - type: table\n    name: 全部\n  - type: table\n    name: 進行中\n";

fn real(p: PathBuf) -> PathBuf {
    p.canonicalize().unwrap()
}

/// 見本を作る。`favs` はお気に入りに入れるもの(名前か印の根)。
fn fixture(name: &str, favs: &[&str]) -> Tmp {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        "tasks/a.md",
        "---\nstatus: todo\nstart: 2026-11-01\ndue: 2026-11-10\n---\n",
    );
    w(
        "tasks/b.md",
        "---\nstatus: done\nstart: 2026-11-02\ndue: 2026-11-12\n---\n",
    );
    w("projects/p.md", "---\nstatus: active\n---\n");
    w("projects.base", BASE);
    w("books/dune.md", "---\nauthor: Herbert\n---\n");
    w("scratch/s.md", "---\nnote: x\n---\n");
    w("lab/x/one.md", "---\nfolder: a > b\n---\n");
    w(
        "lab/.mdgrid/workspace.toml",
        "[[table]]\nname = \"x\"\npath = \"x\"\ntree = { path = \"folder\" }\n",
    );
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    let table = |name: &str, rel: &str| WsTable {
        name: name.into(),
        path: real(n.join(rel)),
        ..WsTable::default()
    };
    workspace::add(&cfg, "work", table("tasks", "tasks")).unwrap();
    workspace::add(&cfg, "work", table("projects", "projects.base")).unwrap();
    workspace::add(&cfg, "趣味", table("books", "books")).unwrap();
    for f in favs {
        favorites::set(&cfg, f, true).unwrap();
    }
    tmp
}

/// `rel`(notes/ からのフォルダか .base)を開いた画面。`ws` は起動の `-w`。
fn boot(tmp: &Tmp, rel: &str, ws: Option<&str>) -> App {
    let target = real(tmp.notes().join(rel));
    let (src, base) = if target.extension().is_some_and(|x| x == "base") {
        let b = mdgrid::base::parse(&std::fs::read_to_string(&target).unwrap()).unwrap();
        let src = Markdown::open_vault(&target).unwrap();
        (src, Some((b, rel.to_string(), None)))
    } else {
        (Markdown::open(std::slice::from_ref(&target)).unwrap(), None)
    };
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.workspace_choice = ws.map(str::to_string);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

/// 今の画面を離れて、次の画面に見た表を渡す(main の run_switching と同じ)。
fn move_on(mut from: App, next: &mut App) {
    from.leave_visit();
    next.visits = std::mem::take(&mut from.visits);
}

#[test]
fn test_switch_lists_favorites_in_order_with_numbers() {
    // [workspace.switch-open] お気に入りが work・趣味 → 切り替えのキーで「1 work」「2 趣味」の一覧(今の work に印)。
    let tmp = fixture("sw_list", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let s = screen(&a);
    let pos = |t: &str| s.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{s}"));
    assert!(pos("1 work") < pos("2 趣味"), "{s}");
    assert!(s.contains("● 1 work"), "今いるワークスペースに印:\n{s}");
}

#[test]
fn test_switch_opens_from_every_screen_and_esc_keeps_it() {
    // [utest -> req~workspace.switch-keys~2] 表・カンバン・ガント・グラフ・関係マップ・木のどれからでも開き、Esc で元の画面のまま。
    let tmp = fixture("sw_every", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    for (open, mode) in [
        (None, Mode::Table),
        (Some('B'), Mode::Kanban),
        (Some('C'), Mode::Gantt),
        (Some('P'), Mode::Chart),
        (Some('R'), Mode::Relations),
    ] {
        a.close_screen();
        if let Some(k) = open {
            ch(&mut a, k);
        }
        assert_eq!(a.mode, mode, "{:?}", a.message);
        ch(&mut a, 'W');
        assert_eq!(a.mode, Mode::Switch, "{mode:?} から開く");
        assert!(screen(&a).contains("2 趣味"), "{mode:?}:\n{}", screen(&a));
        press(&mut a, KeyCode::Esc);
        assert_eq!(a.mode, mode, "Esc で {mode:?} のまま");
        assert!(a.switch_to.is_none() && !a.quit);
    }
    let mut t = boot(&tmp, "lab/x", None);
    ch(&mut t, 'T');
    assert_eq!(t.mode, Mode::Tree, "{:?}", t.message);
    ch(&mut t, 'W');
    assert_eq!(t.mode, Mode::Switch);
    press(&mut t, KeyCode::Esc);
    assert_eq!(t.mode, Mode::Tree);
}

#[test]
fn test_switch_from_palette_keeps_the_screen() {
    // [workspace.switch-open] パレットの「ワークスペースを切り替える」でも、カンバンの上に開く。
    let tmp = fixture("sw_palette", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'B');
    ch(&mut a, ':');
    typing(&mut a, "ワークスペースを切り替える");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Switch);
    assert!(a.kanban.is_some(), "カンバンは閉じない");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Kanban);
}

#[test]
fn test_switch_without_favorites_shows_how_to_add() {
    // [workspace.switch-open] お気に入りが無い → 一覧に、お気に入りの入れ方の案内の1行。
    let tmp = fixture("sw_none", &[]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let s = screen(&a);
    assert!(s.contains("お気に入りが無い"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert!(a.switch_to.is_none() && !a.quit, "案内の行では移らない");
}

#[test]
fn test_switch_digit_moves_to_that_workspace() {
    // [utest -> req~workspace.switch-keys~2] 一覧で `2` → 趣味へ移る(範囲は趣味)。
    let tmp = fixture("sw_digit", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    ch(&mut a, '2');
    assert!(a.quit, "ためた変更が無ければすぐ移る");
    let to = a.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("books")));
    assert_eq!(a.switch_workspace.as_deref(), Some("趣味"));
    // 番号の無いキーは何もしない。
    let mut b = boot(&tmp, "tasks", None);
    ch(&mut b, 'W');
    ch(&mut b, '7');
    assert_eq!(b.mode, Mode::Switch);
    assert!(b.switch_to.is_none());
}

#[test]
fn test_switch_target_last_seen_table_and_view() {
    // [utest -> req~workspace.switch-target~2] work の projects(進行中)を見てから趣味へ移り、また work を選ぶ → projects の進行中。
    let tmp = fixture("sw_target", &["work", "趣味"]);
    let mut p = boot(&tmp, "projects.base", None);
    p.select_view_named("進行中");
    assert_eq!(p.current_view_name().as_deref(), Some("進行中"));
    let mut b = boot(&tmp, "books", Some("趣味"));
    move_on(p, &mut b);
    ch(&mut b, 'W');
    ch(&mut b, '1');
    let to = b.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("projects.base")));
    assert_eq!(to.view.as_deref(), Some("進行中"));
    assert_eq!(b.switch_workspace.as_deref(), Some("work"));
}

#[test]
fn test_switch_target_first_table_when_not_seen() {
    // [utest -> req~workspace.switch-target~2] この起動でまだ開いていない趣味を選ぶ → 趣味の最初の表。印のワークスペースは印の表。
    let tmp = fixture("sw_first", &["work", "趣味"]);
    let lab = real(tmp.notes().join("lab"));
    favorites::set(&tmp.0.join("config"), &favorites::marker_entry(&lab), true).unwrap();
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert!(screen(&a).contains("3 lab"), "{}", screen(&a));
    ch(&mut a, '2');
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(real(tmp.notes().join("books")))
    );
    let mut b = boot(&tmp, "tasks", None);
    ch(&mut b, 'W');
    ch(&mut b, '3');
    assert_eq!(
        b.switch_to.as_ref().map(|p| p.path.clone()),
        Some(lab.join("x"))
    );
    assert!(
        b.switch_leave_workspace,
        "印のワークスペースは -w を外して開く"
    );
}

#[test]
fn test_switch_origin_row_and_back() {
    // [workspace.switch-origin] ワークスペースでない scratch で起動し、work へ移る → 先頭に「最初に開いた所: scratch」→ 選ぶと scratch。
    // scratch にいるあいだは出さない。
    let tmp = fixture("sw_origin", &["work", "趣味"]);
    let mut s = boot(&tmp, "scratch", None);
    s.mark_origin(None);
    ch(&mut s, 'W');
    assert!(!screen(&s).contains("最初に開いた所"), "{}", screen(&s));
    press(&mut s, KeyCode::Esc);
    let mut w = boot(&tmp, "tasks", Some("work"));
    move_on(s, &mut w);
    ch(&mut w, 'W');
    let sc = screen(&w);
    let pos = |t: &str| sc.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{sc}"));
    assert!(pos("最初に開いた所: scratch") < pos("1 work"), "{sc}");
    press(&mut w, KeyCode::Enter);
    let to = w.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("scratch")));
    assert!(w.switch_leave_workspace, "-w を外して戻る");
}

#[test]
fn test_switch_origin_returns_to_the_last_view_there() {
    // [workspace.switch-target] 「最初に開いた所」を選ぶ → 起動したときの表で、そこで最後に見ていたビュー。
    let tmp = fixture("sw_origin_view", &["趣味"]);
    let mut p = boot(&tmp, "projects.base", None);
    p.mark_origin(None);
    p.select_view_named("進行中");
    let mut b = boot(&tmp, "books", Some("趣味"));
    move_on(p, &mut b);
    ch(&mut b, 'W');
    press(&mut b, KeyCode::Enter);
    let to = b.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("projects.base")));
    assert_eq!(to.view.as_deref(), Some("進行中"));
}

#[test]
fn test_switch_asks_before_leaving_unsaved_changes() {
    // [utest -> req~workspace.switch-unsaved~2] 未保存の変更があるときに一覧で趣味を選ぶ → 保存の確かめ。取りやめ → 元の画面(カンバン)のまま。
    let tmp = fixture("sw_unsaved", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    let row = a.src.rows()[0].clone();
    a.changes
        .set(
            a.src.as_ref(),
            &row,
            "status",
            NewValue::Str("doing".into()),
        )
        .unwrap();
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban);
    ch(&mut a, 'W');
    ch(&mut a, '2');
    assert_eq!(a.mode, Mode::Quit, "保存の確かめ");
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Kanban, "取りやめたらカンバンのまま");
    assert!(a.switch_to.is_none());
    assert_eq!(a.changes.count(), 1, "変更は捨てない");
}

#[test]
fn test_favorite_mark_in_drawer_heading() {
    // [utest -> req~workspace.favorite-mark~2] 引き出しの「趣味」の見出しでお気に入りのキー → ★。もう一度 → ★ が消える。
    let tmp = fixture("sw_mark", &[]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer);
    // 今の範囲 work の表(tasks・projects)の下に 趣味 の見出し。
    ch(&mut a, 'G');
    ch(&mut a, 'f');
    assert!(screen(&a).contains("趣味 ★"), "{}", screen(&a));
    assert_eq!(favorites::entries(&tmp.0.join("config")), vec!["趣味"]);
    ch(&mut a, 'f');
    assert!(!screen(&a).contains("★"), "{}", screen(&a));
    assert!(favorites::entries(&tmp.0.join("config")).is_empty());
    // 表の行では入れず、入れ方を言う。
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    ch(&mut a, 'f');
    assert!(a.message.as_deref().unwrap_or("").contains("見出し"));
}

#[test]
fn test_favorite_current_marker_workspace_from_palette() {
    // [utest -> req~workspace.favorite-mark~2] 印のワークスペース lab を開いてパレットの「今のワークスペースをお気に入りにする」→ 切り替えの一覧に lab。
    let tmp = fixture("sw_marker", &[]);
    let mut a = boot(&tmp, "lab/x", None);
    ch(&mut a, ':');
    typing(&mut a, "今のワークスペースをお気に入りにする");
    press(&mut a, KeyCode::Enter);
    assert!(
        a.message.as_deref().unwrap_or("").contains("lab"),
        "{:?}",
        a.message
    );
    let lab = real(tmp.notes().join("lab"));
    assert_eq!(
        favorites::entries(&tmp.0.join("config")),
        vec![favorites::marker_entry(&lab)]
    );
    ch(&mut a, 'W');
    assert!(screen(&a).contains("1 lab"), "{}", screen(&a));
}

#[test]
fn test_favorite_current_needs_a_workspace() {
    // [utest -> req~workspace.favorite-mark~2] ワークスペースでない表では入れずに理由。
    let tmp = fixture("sw_nows", &[]);
    let mut a = boot(&tmp, "scratch", None);
    a.apply(super::keymap::Action::WsFavorite);
    assert!(a
        .message
        .as_deref()
        .unwrap_or("")
        .contains("ワークスペース"));
    assert!(favorites::entries(&tmp.0.join("config")).is_empty());
}

/// 印のワークスペース lab で起動し(最初に開いた所)、切り替えで work へ移った画面。
fn moved_from_lab(tag: &str) -> (Tmp, App) {
    let tmp = fixture(tag, &["work"]);
    let mut l = boot(&tmp, "lab/x", None);
    l.mark_origin(None);
    let mut w = boot(&tmp, "tasks", Some("work"));
    move_on(l, &mut w);
    (tmp, w)
}

#[test]
fn test_switch_origin_row_has_number_zero() {
    // [utest -> req~workspace.switch-origin-key~1] 印の lab から work へ移ったあとの一覧: 先頭は「0 最初に開いた所: lab」、次が「1 work」(今いる所の印つき)。
    let (_t, mut w) = moved_from_lab("sw_zero_row");
    ch(&mut w, 'W');
    let sc = screen(&w);
    let pos = |t: &str| sc.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{sc}"));
    assert!(pos("0 最初に開いた所: lab") < pos("● 1 work"), "{sc}");
}

#[test]
fn test_switch_zero_goes_back_to_the_origin() {
    // [utest -> req~workspace.switch-origin-key~1] 同じ一覧で `0` → lab の表に戻る。
    let (tmp, mut w) = moved_from_lab("sw_zero_back");
    ch(&mut w, 'W');
    ch(&mut w, '0');
    assert!(w.quit, "ためた変更が無ければすぐ移る");
    let to = w.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("lab/x")));
}

#[test]
fn test_switch_zero_without_origin_row_does_nothing() {
    // [utest -> req~workspace.switch-origin-key~1] 起動したときの所にいるあいだ(「最初に開いた所」の行が無い)の `0` は何もしない。
    let tmp = fixture("sw_zero_none", &["work"]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    ch(&mut a, 'W');
    ch(&mut a, '0');
    assert_eq!(a.mode, Mode::Switch);
    assert!(a.switch_to.is_none() && !a.quit);
}

/// App の大きさ(`resize` した幅と高さ)で描いた画面の文字(共通の screen は 80×24 で描く)。
fn sized(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
    term.draw(|f| super::draw(f, a)).unwrap();
    super::test_screen::text(term.backend().buffer())
}

/// ヘッダーの行で、切り替えのボタンの桁(無ければ None)。
fn switch_button_x(a: &App) -> Option<u16> {
    let y = super::bands::head_y(a);
    (0..a.size.0).find(|x| {
        super::bands::header_button_at(a, *x, y) == Some(super::bands::HeaderButton::Switch)
    })
}

#[test]
fn test_switch_button_in_the_header_opens_the_list() {
    // [utest -> req~workspace.switch-button~1] お気に入りが work・幅 100 → ヘッダーの右の端に「W ⇄」。押すと切り替えの一覧。
    let tmp = fixture("sw_button", &["work"]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    let head = sized(&a)
        .lines()
        .nth(usize::from(super::bands::head_y(&a)))
        .unwrap_or("")
        .to_string();
    assert!(head.contains("W ⇄"), "{head}");
    let x = switch_button_x(&a).expect("切り替えのボタン");
    a.click(x, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Switch);
    assert!(screen(&a).contains("1 work"), "{}", screen(&a));
}

#[test]
fn test_switch_button_hidden_without_favorites_at_the_origin() {
    // [utest -> req~workspace.switch-button~1] お気に入りが無く、起動したときの所にいる → ボタンは無い。
    let tmp = fixture("sw_button_none", &[]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    assert!(!sized(&a).contains('⇄'), "{}", sized(&a));
    assert_eq!(switch_button_x(&a), None);
}

#[test]
fn test_switch_button_dropped_before_settings_when_narrow() {
    // [utest -> req~workspace.switch-button~1] 設定のボタンを出せる下限の幅ちょうど(端末 80)→ 設定のボタンはあり、「W ⇄」は無い。
    let tmp = fixture("sw_button_narrow", &["work"]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    a.resize(80, 30);
    let s = sized(&a);
    assert!(s.contains("設定"), "{s}");
    assert!(!s.contains('⇄'), "{s}");
}

// ---- workspace.switch-hint ----

/// 下の帯の行(「未保存」で始まる行)。
fn band(a: &App) -> String {
    sized(a)
        .lines()
        .find(|l| l.trim_start().starts_with("未保存"))
        .unwrap_or("")
        .to_string()
}

#[test]
fn test_switch_band_tells_the_digit_keys() {
    // [utest -> req~workspace.switch-hint~1] 切り替えの一覧を開いている → 下の帯に、数字のキー(0〜9)で番号の所へ移れることが出る。
    let tmp = fixture("sw_hint_band", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let b = band(&a);
    assert!(b.contains("0〜9 番号の所へ移る"), "{b}\n{}", sized(&a));
}

#[test]
fn test_switch_number_of_the_current_place_says_so() {
    // [utest -> req~workspace.switch-hint~1] work(tasks の表)にいて、一覧の「1 work」が今いる所 → `1` → 一覧は閉じ、下の帯に
    // 「今いるワークスペース」である旨の1行が出る(移らない)。
    let tmp = fixture("sw_hint_here", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert!(sized(&a).contains("● 1 work"), "{}", sized(&a));
    ch(&mut a, '1');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none() && !a.quit);
    let msg = a.message.clone().unwrap_or_default();
    assert!(msg.contains("今いるワークスペース"), "{msg}");
    assert!(sized(&a).contains(&msg), "{}", sized(&a));
}

// ---- workspace.drawer-favorites ----

/// 印のワークスペース mytestgrid(表 memo)と tasks(表 todo)を作り、tasks をお気に入りに入れる。
fn marker_pair(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    let n = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("mytestgrid/memo/m.md", "---\nnote: x\n---\n");
    w(
        "mytestgrid/.mdgrid/workspace.toml",
        "name = \"mytestgrid\"\n\n[[table]]\nname = \"memo\"\npath = \"memo\"\n",
    );
    w("tasks/todo/t.md", "---\nstatus: todo\n---\n");
    w(
        "tasks/.mdgrid/workspace.toml",
        "name = \"tasks\"\n\n[[table]]\nname = \"todo\"\npath = \"todo\"\n",
    );
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    let root = real(n.join("tasks"));
    favorites::set(&cfg, &favorites::marker_entry(&root), true).unwrap();
    tmp
}

/// 印のワークスペース mytestgrid で起動し(最初に開いた所)、お気に入りの tasks へ移った画面。
fn moved_to_tasks(tag: &str) -> (Tmp, App) {
    let tmp = marker_pair(tag);
    let mut m = boot(&tmp, "mytestgrid/memo", None);
    m.mark_origin(None);
    let mut t = boot(&tmp, "tasks/todo", None);
    move_on(m, &mut t);
    (tmp, t)
}

/// 引き出しの枠の中の行(上から。枠の縦線の間の文字)。
fn drawer_rows(a: &App) -> Vec<String> {
    sized(a)
        .lines()
        .filter_map(|l| l.split('│').nth(1).map(|x| x.trim_end().to_string()))
        .filter(|x| !x.trim().is_empty())
        .collect()
}

#[test]
fn test_drawer_lists_origin_and_favorites_below_tables() {
    // [utest -> req~workspace.drawer-favorites~1] 印の mytestgrid で起動し、お気に入りの tasks へ移った → 引き出しを開く → 今の範囲の表
    // (todo)の下に「最初に開いた所: mytestgrid」と「tasks」(今いる印つき)が並ぶ。
    let (_t, mut a) = moved_to_tasks("dr_fav_list");
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer, "{:?}", a.message);
    let rows = drawer_rows(&a);
    let pos = |t: &str| {
        rows.iter()
            .position(|r| r.contains(t))
            .unwrap_or_else(|| panic!("{t} が無い: {rows:#?}"))
    };
    let origin = pos("最初に開いた所: mytestgrid");
    assert!(pos("todo") < origin, "{rows:#?}");
    let fav = rows
        .iter()
        .position(|r| r.contains("tasks") && !r.contains("最初に開いた所"))
        .unwrap_or_else(|| panic!("tasks が無い: {rows:#?}"));
    assert!(origin < fav, "{rows:#?}");
    assert!(rows[fav].contains("● tasks"), "{rows:#?}");
    assert!(!rows[origin].contains('●'), "{rows:#?}");
}

#[test]
fn test_drawer_origin_row_goes_back() {
    // [utest -> req~workspace.drawer-favorites~1] 同じ引き出しで「最初に開いた所: mytestgrid」を選ぶ → mytestgrid の表に戻る
    // (切り替えの一覧と同じく、印のワークスペースは -w を外して開く)。
    let (tmp, mut a) = moved_to_tasks("dr_fav_origin");
    ch(&mut a, 'w');
    let at = drawer_rows(&a)
        .iter()
        .position(|r| r.contains("最初に開いた所: mytestgrid"))
        .expect("最初に開いた所");
    if let Some(d) = a.drawer.as_mut() {
        d.sel = at;
    }
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "ためた変更が無ければすぐ移る: {:?}", a.message);
    let to = a.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("mytestgrid/memo")));
    assert!(a.switch_leave_workspace);
}

#[test]
fn test_drawer_favorite_row_moves_like_the_switch_list() {
    // [utest -> req~workspace.drawer-favorites~1] mytestgrid にいるあいだの引き出し(起動したときの所なので「最初に開いた所」は無い)で、
    // お気に入りの「tasks」を選ぶ → 切り替えの一覧と同じく tasks の表(印の最初の表)へ移る。
    let tmp = marker_pair("dr_fav_move");
    let mut a = boot(&tmp, "mytestgrid/memo", None);
    a.mark_origin(None);
    ch(&mut a, 'w');
    let rows = drawer_rows(&a);
    assert!(
        !rows.iter().any(|r| r.contains("最初に開いた所")),
        "{rows:#?}"
    );
    let at = rows
        .iter()
        .position(|r| r.contains("tasks"))
        .unwrap_or_else(|| panic!("tasks が無い: {rows:#?}"));
    assert!(!rows[at].contains('●'), "{rows:#?}");
    if let Some(d) = a.drawer.as_mut() {
        d.sel = at;
    }
    press(&mut a, KeyCode::Enter);
    let to = a.switch_to.clone().expect("移る表");
    assert_eq!(to.path, real(tmp.notes().join("tasks/todo")));
    assert!(
        a.switch_leave_workspace,
        "印のワークスペースは -w を外して開く"
    );
}

#[test]
fn test_palette_open_workspace_lists_favorites_and_origin() {
    // [utest -> req~workspace.drawer-favorites~1] 同じ状態で、パレットの「ワークスペースを開く」→「ワークスペースはまだ無い」と言わずに、
    // お気に入りの tasks と最初に開いた所を並べる。最初に開いた所を選ぶと mytestgrid の表に戻る。
    let (tmp, mut a) = moved_to_tasks("dr_fav_palette");
    ch(&mut a, ':');
    typing(&mut a, "ワークスペースを開く");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    let s = sized(&a);
    assert!(!s.contains("まだ無い"), "{s}");
    assert!(s.contains("最初に開いた所: mytestgrid"), "{s}");
    assert!(
        s.lines()
            .skip(1)
            .any(|l| l.contains("tasks") && !l.contains("最初に開いた所")),
        "{s}"
    );
    typing(&mut a, "最初に開いた所");
    press(&mut a, KeyCode::Enter);
    let to = a.switch_to.clone().expect("移る表");
    assert_eq!(to.path, real(tmp.notes().join("mytestgrid/memo")));
}

#[test]
fn test_switch_bottom_edge_key_hints() {
    // [utest -> req~screen.window-key-hints~1] 切り替えの一覧の下の縁に、その窓で効く主なキー(Enter と Esc)を「キー 動作」で並べ、縁の「Esc 閉じる」のクリックで閉じる。
    let tmp = fixture("sw_hints", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let s = sized(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains('╰') && l.contains("Esc 閉じる"))
        .unwrap_or_else(|| panic!("縁の案内が無い:\n{s}"));
    assert!(line.contains("Enter "), "{line:?}");
    let at = line.find("Esc 閉じる").unwrap();
    let x = width::width(&line[..at]) as u16;
    a.click(x + 1, y as u16);
    assert_eq!(a.mode, Mode::Table, "{}", sized(&a));
}
