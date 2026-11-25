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
fn test_favorite_current_marker_workspace_from_palette() {
    // [utest -> req~workspace.favorite-mark~3] 印のワークスペース lab を開いてパレットの「今のワークスペースをお気に入りにする」→ 切り替えの一覧に lab。
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
    // [utest -> req~workspace.favorite-mark~3] ワークスペースでない表では、パレットの「今のワークスペースをお気に入りにする」でも入れずに理由。
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
    // [utest -> req~workspace.switch-button~2] お気に入りが work・印の無いフォルダ・幅 100 → ヘッダーの右の端に「W ⇄」。
    // 押すと切り替えの一覧。
    let tmp = fixture("sw_button", &["work"]);
    let mut a = boot(&tmp, "scratch", None);
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
    // [utest -> req~workspace.switch-button~2] お気に入りが無く、起動したときの所にいる → ボタンは無い。
    let tmp = fixture("sw_button_none", &[]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    assert!(!sized(&a).contains('⇄'), "{}", sized(&a));
    assert_eq!(switch_button_x(&a), None);
}

#[test]
fn test_switch_button_dropped_before_settings_when_narrow() {
    // [utest -> req~workspace.switch-button~2] 設定のボタンを出せる下限の幅ちょうど(端末 80)→ 設定のボタンはあり、「W ⇄」は無い。
    let tmp = fixture("sw_button_narrow", &["work"]);
    let mut a = boot(&tmp, "scratch", None);
    a.mark_origin(None);
    a.resize(80, 30);
    let s = sized(&a);
    assert!(s.contains("設定"), "{s}");
    assert!(!s.contains('⇄'), "{s}");
}

#[test]
fn test_switch_button_hidden_when_the_crumb_shows_the_scope() {
    // [utest -> req~workspace.switch-button~2] 印のワークスペース lab の表・お気に入りが work・幅 100 → パンくずに
    // 範囲の名前のボタン「W lab」があり、右の端の「W ⇄」は無い。
    let tmp = fixture("sw_button_scope", &["work"]);
    let mut a = boot(&tmp, "lab/x", None);
    a.mark_origin(None);
    let s = sized(&a);
    let head = s
        .lines()
        .nth(usize::from(super::bands::head_y(&a)))
        .unwrap_or("")
        .to_string();
    assert!(head.contains("W lab"), "{head}");
    assert!(!head.contains('⇄'), "{head}");
    assert_eq!(switch_button_x(&a), None);
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

// ---- workspace.open-favorites・workspace.drawer-scope-only ----

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

#[test]
fn test_palette_open_workspace_lists_favorites_and_origin() {
    // [utest -> req~workspace.open-favorites~1] 印の mytestgrid で起動し、お気に入りの tasks へ移った → パレットの「ワークスペースを開く」→「ワークスペースはまだ無い」と言わずに、
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

// ---- workspace.drawer-scope-only ----

#[test]
fn test_drawer_lists_only_the_tables_of_the_scope() {
    // [utest -> req~workspace.drawer-scope-only~1] 「work」に tasks・projects、「趣味」に books があり、「趣味」をお気に入りに入れている → tasks で `w` →
    // 引き出しには tasks・projects だけ。「趣味」も books も無い。
    let tmp = fixture("dr_scope_only", &["趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer, "{:?}", a.message);
    let rows = window_rows(&a, "work");
    assert_eq!(rows, vec!["tasks", "projects"], "{}", sized(&a));
    let s = sized(&a);
    assert!(!s.contains("趣味") && !s.contains("books"), "{s}");
}

#[test]
fn test_drawer_has_no_origin_or_favorite_rows() {
    // [utest -> req~workspace.drawer-scope-only~1] 印の mytestgrid で起動し、お気に入りの tasks へ移った → 引き出しを開く → tasks の表(todo)だけで、
    // 「最初に開いた所」の行も、お気に入りの行も無い。
    let (_t, mut a) = moved_to_tasks("dr_scope_marker");
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer, "{:?}", a.message);
    let rows = window_rows(&a, "tasks");
    assert_eq!(rows, vec!["todo"], "{}", sized(&a));
}

// ---- workspace.favorite-mark・workspace.switch-add-here・workspace.switch-favorite-hints・workspace.favorite-reorder ----

/// 左の端に重ねた窓(上の縁に題 `title`)の中の行(下の縁までの、枠の縦線の間の文字。選びの印と今の印は除く)。
fn window_rows(a: &App, title: &str) -> Vec<String> {
    sized(a)
        .lines()
        .skip_while(|l| !(l.starts_with('╭') && l.contains(title)))
        .skip(1)
        .take_while(|l| !l.starts_with('╰'))
        .filter_map(|l| l.split('│').nth(1))
        .map(|r| {
            r.trim_start_matches(['>', ' ', '●', '*'])
                .trim_end()
                .to_string()
        })
        .filter(|r| !r.is_empty())
        .collect()
}

/// 切り替えの一覧の窓の中の行。
fn switch_rows(a: &App) -> Vec<String> {
    window_rows(a, "ワークスペース")
}

#[test]
fn test_switch_add_key_puts_the_current_workspace_in() {
    // [utest -> req~workspace.favorite-mark~3] 「趣味」の books にいて、お気に入りが work だけ → 切り替えの一覧で今いる所を入れるキー(`a`)→
    // 一覧に「2 趣味」が今いる所の印つきで入り、favorites は ["work", "趣味"]。一覧は開いたまま。
    let tmp = fixture("sw_add_key", &["work"]);
    let mut a = boot(&tmp, "books", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    ch(&mut a, 'a');
    assert_eq!(a.mode, Mode::Switch, "{:?}", a.message);
    assert!(sized(&a).contains("● 2 趣味"), "{}", sized(&a));
    assert_eq!(
        favorites::entries(&tmp.0.join("config")),
        vec!["work", "趣味"]
    );
}

#[test]
fn test_switch_remove_key_takes_the_selected_favorite_out() {
    // [utest -> req~workspace.favorite-mark~3] お気に入りが work・趣味 の一覧で「1 work」を選んでいる → 外すキー(`d`)→ 一覧から work が消えて
    // 「1 趣味」になり、favorites は ["趣味"]。
    let tmp = fixture("sw_remove_key", &["work", "趣味"]);
    let mut a = boot(&tmp, "books", None);
    ch(&mut a, 'W');
    ch(&mut a, 'd');
    assert_eq!(a.mode, Mode::Switch, "{:?}", a.message);
    let rows = switch_rows(&a);
    assert!(rows.iter().any(|r| r == "1 趣味"), "{rows:#?}");
    assert!(!rows.iter().any(|r| r.ends_with("work")), "{rows:#?}");
    assert_eq!(favorites::entries(&tmp.0.join("config")), vec!["趣味"]);
}

#[test]
fn test_switch_add_key_outside_a_workspace_says_why() {
    // [utest -> req~workspace.favorite-mark~3] ワークスペースでない表 scratch で、切り替えの一覧の今いる所を入れるキー → favorites は変わらず、理由が出る。
    let tmp = fixture("sw_add_nows", &["work"]);
    let mut a = boot(&tmp, "scratch", None);
    ch(&mut a, 'W');
    ch(&mut a, 'a');
    assert_eq!(favorites::entries(&tmp.0.join("config")), vec!["work"]);
    let msg = a.message.clone().unwrap_or_default();
    assert!(msg.contains("ワークスペース"), "{msg:?}");
}

#[test]
fn test_switch_add_here_row_below_favorites() {
    // [utest -> req~workspace.switch-add-here~1] 「趣味」の books にいて、お気に入りが work だけ → 一覧の「1 work」の下に「+ 趣味 をお気に入りに入れる」。
    let tmp = fixture("sw_add_row", &["work"]);
    let mut a = boot(&tmp, "books", None);
    ch(&mut a, 'W');
    let rows = switch_rows(&a);
    let pos = |t: &str| {
        rows.iter()
            .position(|r| r == t)
            .unwrap_or_else(|| panic!("{t} が無い: {rows:#?}"))
    };
    assert!(
        pos("1 work") < pos("+ 趣味 をお気に入りに入れる"),
        "{rows:#?}"
    );
}

#[test]
fn test_switch_add_here_row_click_adds_and_stays_open() {
    // [utest -> req~workspace.switch-add-here~1] 同じ一覧で「+ 趣味 をお気に入りに入れる」の行をクリック → 一覧は開いたままで、その行が「2 趣味」(今いる所の印つき)に替わる。
    let tmp = fixture("sw_add_click", &["work"]);
    let mut a = boot(&tmp, "books", None);
    ch(&mut a, 'W');
    let s = sized(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("+ 趣味 をお気に入りに入れる"))
        .unwrap_or_else(|| panic!("入れる行が無い:\n{s}"));
    let at = line.find("+ 趣味").unwrap();
    a.click(width::width(&line[..at]) as u16 + 1, y as u16);
    assert_eq!(a.mode, Mode::Switch, "{:?}\n{}", a.message, sized(&a));
    let s = sized(&a);
    assert!(s.contains("● 2 趣味"), "{s}");
    assert!(!s.contains("をお気に入りに入れる"), "{s}");
    assert_eq!(
        favorites::entries(&tmp.0.join("config")),
        vec!["work", "趣味"]
    );
}

#[test]
fn test_switch_add_here_row_absent_when_already_a_favorite_or_not_a_workspace() {
    // [utest -> req~workspace.switch-add-here~1] work の tasks でお気に入りが work → 入れる行は無い。ワークスペースでない scratch でも無い。
    let tmp = fixture("sw_add_none", &["work"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    assert!(!sized(&a).contains("をお気に入りに入れる"), "{}", sized(&a));
    let mut s = boot(&tmp, "scratch", None);
    ch(&mut s, 'W');
    assert_eq!(s.mode, Mode::Switch);
    assert!(!sized(&s).contains("をお気に入りに入れる"), "{}", sized(&s));
}

#[test]
fn test_switch_edge_and_band_tell_the_add_and_remove_keys() {
    // [utest -> req~workspace.switch-favorite-hints~1] お気に入りが work・趣味、幅 120 → 下の縁と下の帯の両方に、今いる所を入れるキーと外すキー(幅 100 では下の帯の後ろのキーは省かれる)。
    let tmp = fixture("sw_fav_hints", &["work", "趣味"]);
    let mut a = boot(&tmp, "books", None);
    a.resize(120, 30);
    ch(&mut a, 'W');
    let s = sized(&a);
    let edge = s
        .lines()
        .find(|l| l.contains('╰') && l.contains("Esc 閉じる"))
        .unwrap_or_else(|| panic!("縁の案内が無い:\n{s}"));
    let b = band(&a);
    for t in ["a 今いる所を入れる", "d 外す"] {
        assert!(edge.contains(t), "縁に {t}: {edge:?}");
        assert!(b.contains(t), "下の帯に {t}: {b:?}");
    }
}

#[test]
fn test_switch_edge_remove_key_click() {
    // [utest -> req~workspace.switch-favorite-hints~1] 同じ一覧で「2 趣味」を選び、下の縁の外すキーをクリック → 趣味がお気に入りから外れ、一覧は「1 work」だけ。
    let tmp = fixture("sw_fav_hint_click", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    ch(&mut a, 'j');
    let s = sized(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains('╰') && l.contains("d 外す"))
        .unwrap_or_else(|| panic!("縁の外すキーが無い:\n{s}"));
    let at = line.find("d 外す").unwrap();
    a.click(width::width(&line[..at]) as u16 + 1, y as u16);
    assert_eq!(a.mode, Mode::Switch, "{:?}", a.message);
    assert_eq!(favorites::entries(&tmp.0.join("config")), vec!["work"]);
    assert_eq!(switch_rows(&a), vec!["1 work"], "{}", sized(&a));
}

#[test]
fn test_switch_move_favorite_up() {
    // [utest -> req~workspace.favorite-reorder~1] お気に入りが work・趣味 の一覧で「2 趣味」を選び、上へ動かすキー(`K`)→ 「1 趣味」「2 work」、
    // 選びは趣味のまま、favorites は ["趣味", "work"]。もう一度(先頭で)上へ → 変わらない。
    let tmp = fixture("sw_move_up", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    ch(&mut a, 'j');
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Switch, "{:?}", a.message);
    assert_eq!(switch_rows(&a), vec!["1 趣味", "2 work"], "{}", sized(&a));
    assert!(
        sized(&a).lines().any(|l| l.contains(">   1 趣味")),
        "選びは趣味のまま:\n{}",
        sized(&a)
    );
    let cfg = tmp.0.join("config");
    assert_eq!(favorites::entries(&cfg), vec!["趣味", "work"]);
    ch(&mut a, 'K');
    assert_eq!(switch_rows(&a), vec!["1 趣味", "2 work"], "{}", sized(&a));
    assert_eq!(favorites::entries(&cfg), vec!["趣味", "work"]);
}

#[test]
fn test_switch_move_does_nothing_on_the_origin_row() {
    // [utest -> req~workspace.favorite-reorder~1] 起動したときの所から離れた一覧で「0 最初に開いた所」を選んで下へ動かすキー(`J`)→ 一覧も favorites も変わらない。
    let (tmp, mut w) = moved_from_lab("sw_move_origin");
    favorites::set(&tmp.0.join("config"), "趣味", true).unwrap();
    ch(&mut w, 'W');
    let before = switch_rows(&w);
    assert!(before[0].starts_with("0 最初に開いた所"), "{before:#?}");
    ch(&mut w, 'J');
    assert_eq!(switch_rows(&w), before);
    assert_eq!(
        favorites::entries(&tmp.0.join("config")),
        vec!["work", "趣味"]
    );
}

// ---- screen.breadcrumb-switch・screen.breadcrumb-drawer ----

/// ヘッダーの行で、文字 `t` の始まる桁。
fn head_x(a: &App, t: &str) -> u16 {
    let s = sized(a);
    let line = s
        .lines()
        .nth(usize::from(super::bands::head_y(a)))
        .unwrap_or("");
    let at = line
        .find(t)
        .unwrap_or_else(|| panic!("ヘッダーに {t} が無い: {line:?}"));
    width::width(&line[..at]) as u16
}

/// ヘッダーのパンくずの範囲の名前 `name` を押す。
fn click_scope(a: &mut App, name: &str) {
    let x = head_x(a, &format!("{name} ›"));
    a.click(x + 1, super::bands::head_y(a));
}

/// ヘッダーのパンくずの表の名前を押す(区切り「› 」の2桁あとから、キーの字と空白を飛ばした所)。
fn click_table(a: &mut App) {
    let x = head_x(a, "› ") + 2;
    a.click(x + 2, super::bands::head_y(a));
}

#[test]
fn test_crumb_scope_toggles_the_switch_list() {
    // [utest -> req~screen.breadcrumb-switch~1] work の tasks、お気に入りが work・趣味 → パンくずの「work」をクリック → 切り替えの一覧が開き、もう一度で閉じる。
    let tmp = fixture("crumb_sw_toggle", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    click_scope(&mut a, "work");
    assert_eq!(a.mode, Mode::Switch, "{}", sized(&a));
    assert!(sized(&a).contains("2 趣味"), "{}", sized(&a));
    click_scope(&mut a, "work");
    assert_eq!(a.mode, Mode::Table, "{}", sized(&a));
    assert!(a.switch_to.is_none() && !a.quit);
}

#[test]
fn test_crumb_scope_opens_the_switch_list_over_the_kanban() {
    // [utest -> req~screen.breadcrumb-switch~1] 同じ表のカンバンの画面で、パンくずの「work」をクリック → カンバンの画面のまま、その上に切り替えの一覧。
    let tmp = fixture("crumb_sw_kanban", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    click_scope(&mut a, "work");
    assert_eq!(a.mode, Mode::Switch, "{}", sized(&a));
    assert!(a.kanban.is_some(), "カンバンは閉じない");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Kanban);
}

#[test]
fn test_crumb_scope_from_the_drawer() {
    // [utest -> req~screen.breadcrumb-switch~1] 引き出しを開いているときに、パンくずの「work」をクリック → 引き出しが閉じ、切り替えの一覧が開く。
    let tmp = fixture("crumb_sw_drawer", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer);
    click_scope(&mut a, "work");
    assert_eq!(a.mode, Mode::Switch, "{}", sized(&a));
    assert!(a.drawer.is_none(), "引き出しは閉じる");
}

#[test]
fn test_crumb_table_from_the_switch_list() {
    // [utest -> req~screen.breadcrumb-drawer~2] 切り替えの一覧を開いているときに、パンくずの「tasks ▾」をクリック → 切り替えの一覧が閉じ、引き出しが開く。
    let tmp = fixture("crumb_dr_switch", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    click_table(&mut a);
    assert_eq!(a.mode, Mode::Drawer, "{}", sized(&a));
    assert!(a.switcher.is_none(), "切り替えの一覧は閉じる");
}

#[test]
fn test_crumb_detected_scope_on_the_relation_map() {
    // [utest -> req~screen.breadcrumb-switch~1] ワークスペースでなく検知した範囲 vault の表で、関係マップの画面(範囲の名前が出る)、お気に入りが無い →
    // パンくずの「vault」をクリック → 関係マップの上に、お気に入りの入れ方の案内の1行だけの切り替えの一覧。
    let tmp = fixture("crumb_sw_detected", &[]);
    let n = tmp.notes();
    std::fs::create_dir_all(n.join("vault/.obsidian")).unwrap();
    std::fs::create_dir_all(n.join("vault/memo")).unwrap();
    std::fs::write(n.join("vault/memo/m.md"), "---\nnote: x\n---\n").unwrap();
    let mut a = boot(&tmp, "vault/memo", None);
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    click_scope(&mut a, "vault");
    assert_eq!(a.mode, Mode::Switch, "{}", sized(&a));
    assert!(a.relmap.is_some(), "関係マップの画面のまま");
    let rows = switch_rows(&a);
    assert_eq!(rows.len(), 1, "{rows:#?}");
    assert!(rows[0].contains("お気に入りが無い"), "{rows:#?}");
}

#[test]
fn test_crumb_without_the_drawer_key() {
    // [utest -> req~screen.breadcrumb-button-look~3, req~screen.keys-vim-and-arrows-remap~1] 引き出しのキーを外す → 「W work › tasks ▾」。
    let tmp = fixture("crumb_no_key", &[]);
    let mut a = boot(&tmp, "tasks", None);
    let w = super::keymap::rebind(&mut a.keys, &[("table".into(), "w".into(), "none".into())]);
    assert!(w.is_empty(), "{w:?}");
    let s = sized(&a);
    let head = s
        .lines()
        .nth(usize::from(super::bands::head_y(&a)))
        .unwrap_or("");
    assert!(
        head.trim_start().starts_with("W work › tasks ▾"),
        "{head:?}"
    );
}
