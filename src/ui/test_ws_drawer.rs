//! ワークスペースの表の引き出し(WS-8): 左から重ねて出し、今の範囲の表・ほかのワークスペースの見出しを並べ、
//! 選んだ表へ移る。見本は test_relations.rs の notes/(tasks・projects・members)に books を足したもの。
//! 「仕事」は Work(Tasks・Projects)、「趣味」は Hobby(Books)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, press, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use mdgrid::workspace::{self, WsTable};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

// ---- 材料 ----

/// 引き出しが置かれる左の帯の幅の上限(設計: 24〜40 桁)。
const LEFT: u16 = 40;

fn dir(tmp: &Tmp, d: &str) -> PathBuf {
    tmp.notes().join(d).canonicalize().unwrap()
}

fn table(tmp: &Tmp, name: &str, d: &str) -> WsTable {
    WsTable {
        name: name.into(),
        path: dir(tmp, d),
        view: None,
        profile: Default::default(),
    }
}

/// notes/ に books を足し、Work に Tasks・Projects、Hobby に Books を書く。
fn write_work_hobby(tmp: &Tmp) {
    let p = tmp.notes().join("books/Dune.md");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "---\nauthor: Herbert\n---\n").unwrap();
    let cfg = tmp.0.join("config");
    workspace::add(&cfg, "Work", table(tmp, "Tasks", "tasks")).unwrap();
    workspace::add(&cfg, "Work", table(tmp, "Projects", "projects")).unwrap();
    workspace::add(&cfg, "Hobby", table(tmp, "Books", "books")).unwrap();
}

/// 「仕事」の tasks で開いた画面(範囲は Work)。
fn boot_work(name: &str) -> (Tmp, App) {
    let tmp = workspace(name);
    write_work_hobby(&tmp);
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    assert_eq!(a.scope.as_ref().map(|s| s.name.as_str()), Some("Work"));
    (tmp, a)
}

/// 登録した表もワークスペースも無いフォルダで開く。
fn boot_bare(tmp: &Tmp) -> App {
    let d = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&d)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: d,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

/// 今の端末の大きさ(`a.size`)で描いたバッファ。
fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| super::draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

/// 今の端末の大きさで描いた画面の文字(行末の空白は除く。幅2の字の後ろの空きは飛ばす)。
fn scr(a: &App) -> String {
    super::test_screen::text(&buf(a))
}

/// 2行目から下で、文字 `t` の始まる桁と行(左から、上から最初のもの)。
fn find(a: &App, t: &str) -> Option<(u16, u16)> {
    let b = buf(a);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    for y in 1..b.area.height {
        for x in 0..b.area.width {
            let mut xx = x;
            let ok = cs.iter().all(|c| {
                let hit = xx < b.area.width && b[(xx, y)].symbol() == c;
                xx += super::width::width(c).max(1) as u16;
                hit
            });
            if ok {
                return Some((x, y));
            }
        }
    }
    None
}

/// 引き出しの帯(左の端)に `t` があるか。
fn in_drawer(a: &App, t: &str) -> bool {
    matches!(find(a, t), Some((x, _)) if x < LEFT)
}

/// 1行目で文字 `t` の始まる桁。
fn head_col(a: &App, t: &str) -> Option<u16> {
    let b = buf(a);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        cs.iter().all(|c| {
            let hit = xx < b.area.width && b[(xx, super::bands::head_y(a))].symbol() == c;
            xx += super::width::width(c).max(1) as u16;
            hit
        })
    })
}

fn is_frame(s: &str) -> bool {
    s.chars()
        .all(|c| c == ' ' || c == '|' || ('\u{2500}'..='\u{257F}').contains(&c))
}

/// 引き出しの中の、名前 `t` の行の飾り(名前・枠・空白を除いた字)。行の右の枠までを見る。
fn deco(a: &App, t: &str) -> String {
    let b = buf(a);
    let (x0, y) = find(a, t).unwrap_or_else(|| panic!("{t} が無い\n{}", scr(a)));
    let end = x0 + super::width::width(t) as u16;
    let edge = (end..b.area.width)
        .find(|&x| {
            let s = b[(x, y)].symbol();
            s != " " && s.chars().all(|c| ('\u{2500}'..='\u{257F}').contains(&c))
        })
        .unwrap_or(LEFT.min(b.area.width));
    (0..edge)
        .filter(|&x| x < x0 || x >= end)
        .map(|x| b[(x, y)].symbol().to_string())
        .filter(|s| !is_frame(s))
        .collect()
}

fn click_on(a: &mut App, t: &str) {
    let (x, y) = find(a, t).unwrap_or_else(|| panic!("{t} が無い\n{}", scr(a)));
    a.click(x + 1, y);
}

fn pend(a: &mut App) {
    let row = a.src.rows()[0].clone();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("x".into()))
        .unwrap();
}

// ---- 試験 ----

#[test]
fn test_ws_8_no_drawer_by_default() {
    // [WS-8] 普段の画面に引き出しの帯は無い(ほかのワークスペースの名前も、範囲のほかの表の名前も出ない)。
    let (_tmp, a) = boot_work("ws8_default");
    assert_eq!(a.mode, Mode::Table);
    let s = scr(&a);
    let body: String = s.lines().skip(1).collect::<Vec<_>>().join("\n");
    assert!(!body.contains("Hobby"), "{s}");
    assert!(!body.contains("Projects"), "{s}");
    assert!(!in_drawer(&a, "Tasks"), "{s}");
}

#[test]
fn test_ws_8_w_opens_scope_tables_with_mark() {
    // [WS-8] `w` で開く → 左の端に今の範囲の表(Tasks・Projects)が並び、その下にほかのワークスペース Hobby の
    // 見出しだけ(Books は畳まれて出ない)。今の表 Tasks には、選びの位置と関係なく印がある。
    let (_tmp, mut a) = boot_work("ws8_open");
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer, "{}", scr(&a));
    let s = scr(&a);
    assert!(in_drawer(&a, "Tasks"), "{s}");
    assert!(in_drawer(&a, "Projects"), "{s}");
    assert!(in_drawer(&a, "Hobby"), "{s}");
    assert!(!s.contains("Books"), "畳んだ見出しの表は出ない\n{s}");
    let (_, yt) = find(&a, "Tasks").unwrap();
    let (_, yp) = find(&a, "Projects").unwrap();
    let (_, yh) = find(&a, "Hobby").unwrap();
    assert!(
        yt < yp && yp < yh,
        "範囲の表が先、ほかのワークスペースは下\n{s}"
    );
    // 選びを最後(Hobby の見出し)へ動かし、Tasks と Projects の違いが印だけになるようにする。
    ch(&mut a, 'G');
    assert_eq!(a.mode, Mode::Drawer);
    let (t, p) = (deco(&a, "Tasks"), deco(&a, "Projects"));
    assert!(
        !t.is_empty() && t != p,
        "今の表に印: Tasks={t:?} Projects={p:?}\n{}",
        scr(&a)
    );
}

#[test]
fn test_ws_8_palette_opens() {
    // [WS-8] パレットの動作 workspace_drawer でも開く。
    let (_tmp, mut a) = boot_work("ws8_palette");
    ch(&mut a, ':');
    typing(&mut a, "workspace_drawer");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Drawer, "{:?}\n{}", a.message, scr(&a));
    assert!(in_drawer(&a, "Projects"), "{}", scr(&a));
}

#[test]
fn test_ws_8_enter_switches_and_closes() {
    // [WS-8] Projects を Enter → switch_to が Projects、範囲は Work のまま。引き出しは閉じ、変更が無いのですぐ移る。
    let (tmp, mut a) = boot_work("ws8_enter");
    ch(&mut a, 'w');
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::Drawer, "引き出しは閉じる");
    let to = a.switch_to.as_ref().expect("移る表");
    assert_eq!(to.path, dir(&tmp, "projects"));
    assert_eq!(a.switch_workspace.as_deref(), Some("Work"));
    assert!(a.quit, "ためた変更が無ければすぐ移る");
}

#[test]
fn test_ws_8_current_table_just_closes() {
    // [WS-8] 今の表 Tasks を選ぶ → 開き直さず閉じるだけ。
    let (_tmp, mut a) = boot_work("ws8_current");
    ch(&mut a, 'w');
    click_on(&mut a, "Tasks");
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.switch_to.is_none() && !a.quit);
}

#[test]
fn test_ws_8_other_workspace_heading_then_click() {
    // [WS-8] 見出し Hobby を開くと Books が並ぶ → Books をクリック → Books の表へ、範囲は Hobby。
    let (tmp, mut a) = boot_work("ws8_other");
    ch(&mut a, 'w');
    ch(&mut a, 'G');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Drawer, "見出しの Enter は開くだけ");
    assert!(a.switch_to.is_none());
    assert!(in_drawer(&a, "Books"), "{}", scr(&a));
    let (_, yh) = find(&a, "Hobby").unwrap();
    let (_, yb) = find(&a, "Books").unwrap();
    assert!(yh < yb, "{}", scr(&a));
    click_on(&mut a, "Books");
    assert_ne!(a.mode, Mode::Drawer);
    let to = a.switch_to.as_ref().expect("移る表");
    assert_eq!(to.path, dir(&tmp, "books"));
    assert_eq!(a.switch_workspace.as_deref(), Some("Hobby"));
    assert!(a.quit);
}

#[test]
fn test_ws_8_heading_fold_keys() {
    // [WS-8] 見出しは l で開き h で畳む。畳むと Books は消える。
    let (_tmp, mut a) = boot_work("ws8_fold");
    ch(&mut a, 'w');
    ch(&mut a, 'G');
    ch(&mut a, 'l');
    assert!(in_drawer(&a, "Books"), "{}", scr(&a));
    ch(&mut a, 'h');
    assert_eq!(a.mode, Mode::Drawer);
    assert!(!scr(&a).contains("Books"), "{}", scr(&a));
}

#[test]
fn test_ws_8_esc_and_w_close_without_change() {
    // [WS-8] 開いて Esc → 何も変わらずに閉じる(画面も選びも前と同じ)。もう一度 `w` で開き、`w` でも閉じる。
    let (_tmp, mut a) = boot_work("ws8_esc");
    let before = scr(&a);
    let (row, col) = (a.row, a.col);
    ch(&mut a, 'w');
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none() && a.switch_workspace.is_none() && !a.quit);
    assert_eq!((a.row, a.col), (row, col));
    assert_eq!(scr(&a), before);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none() && !a.quit);
    assert_eq!(scr(&a), before);
}

#[test]
fn test_ws_8_click_outside_closes() {
    // [WS-8] 引き出しの外(表の右の側)をクリック → 閉じるだけで、移らない。
    let (_tmp, mut a) = boot_work("ws8_outside");
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer);
    a.click(100, 10);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.switch_to.is_none() && !a.quit);
    assert!(!in_drawer(&a, "Projects"), "{}", scr(&a));
}

#[test]
fn test_ws_8_click_row_switches() {
    // [WS-8] 表の行 Projects をクリック → その表へ移る。
    let (tmp, mut a) = boot_work("ws8_click");
    ch(&mut a, 'w');
    click_on(&mut a, "Projects");
    assert_ne!(a.mode, Mode::Drawer);
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(dir(&tmp, "projects"))
    );
    assert!(a.quit);
}

#[test]
fn test_ws_8_pending_changes_ask_then_back() {
    // [WS-8][CLI-19] ためた変更があるときに Projects を選ぶ → 保存か取りやめの確かめ。Esc で戻ると移らない。
    let (_tmp, mut a) = boot_work("ws8_pending");
    pend(&mut a);
    ch(&mut a, 'w');
    click_on(&mut a, "Projects");
    assert_eq!(a.mode, Mode::Quit, "{}", scr(&a));
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none(), "戻ったら移らない");
    a.changes = Default::default();
    ch(&mut a, 'q');
    assert!(a.quit && a.switch_to.is_none(), "前に選んだ表へは移らない");
}

#[test]
fn test_ws_8_pending_changes_save_then_switch() {
    // [WS-8][CLI-19] ためた変更があるときに Projects を選ぶ → s → 保存の確認で Enter → 書いてから移る。
    let (tmp, mut a) = boot_work("ws8_save");
    pend(&mut a);
    let row = a.src.rows()[0].clone();
    ch(&mut a, 'w');
    click_on(&mut a, "Projects");
    assert_eq!(a.mode, Mode::Quit);
    ch(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "保存したら終わる");
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(dir(&tmp, "projects"))
    );
    assert_eq!(a.switch_workspace.as_deref(), Some("Work"));
    let written = std::fs::read_to_string(&row.0).unwrap();
    assert!(written.contains("status: x"), "{written}");
}

#[test]
fn test_ws_8_registered_tables_when_no_workspace() {
    // [WS-8] ワークスペースが無く登録した表だけ → 登録した表(Projects・Members・Tasks)が並ぶ。見出しは無い。
    // 選ぶと、その表へ移る(範囲の選び方は今のまま。ワークスペースの名前は付けない)。
    let tmp = workspace("ws8_registered");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    assert!(a.scope.is_none());
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer, "{:?}\n{}", a.message, scr(&a));
    for n in ["Projects", "Members", "Tasks"] {
        assert!(in_drawer(&a, n), "{n}\n{}", scr(&a));
    }
    click_on(&mut a, "Members");
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(dir(&tmp, "members"))
    );
    assert!(a.switch_workspace.is_none());
    assert!(a.quit);
}

#[test]
fn test_ws_8_nothing_to_list_says_why() {
    // [WS-8] 並べる表もほかのワークスペースも無い → 引き出しを開かず、理由を出す。
    let tmp = workspace("ws8_bare");
    let mut a = boot_bare(&tmp);
    ch(&mut a, 'w');
    assert_ne!(a.mode, Mode::Drawer, "{}", scr(&a));
    assert!(
        !a.message.as_deref().unwrap_or("").is_empty(),
        "理由を出す: {:?}",
        a.message
    );
}

#[test]
fn test_ws_8_from_relmap_header_button() {
    // [WS-8][SR-42] 関係マップでヘッダーのパンくずの「Work」(前の配置のボタン「▾ Work」)を押す → 表の画面に
    // 戻って引き出しが開く。もう一度同じ所を押すと閉じる。
    let (_tmp, mut a) = boot_work("ws8_relmap");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    let x = head_col(&a, "Work").expect("パンくずのワークスペースの名前");
    a.click(x + 2, super::bands::head_y(&a));
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert_eq!(a.mode, Mode::Drawer, "{}", scr(&a));
    assert!(in_drawer(&a, "Projects"), "{}", scr(&a));
    let x = head_col(&a, "Work").expect("引き出しを開いていてもパンくずは出る");
    a.click(x + 2, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.switch_to.is_none() && !a.quit);
}
