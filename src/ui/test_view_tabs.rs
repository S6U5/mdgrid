//! ビューの設定の画面の「ビュー」の区画(NV-26): タブの順・出す隠す・既定・名前・削除・切り替えの案内。
//! 材料は `.base`(進行中・完了・全部)と mdgrid のビュー(未完了・メモ)。設定の置き場は tmp/config。

use super::keymap::Mode;
use super::settings::Sec;
use super::startup::Startup;
use super::test_grid::{base_path, vault, TASKS};
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::views::{self, NativeView};
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp, readonly: bool) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p.clone(),
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// `.base` と mdgrid のビュー2つの対象。
fn setup(name: &str) -> Tmp {
    let tmp = vault(name);
    let p = base_path(&tmp, TASKS);
    let dir = tmp.0.join("config");
    std::fs::create_dir_all(&dir).unwrap();
    let v = |n: &str| NativeView {
        name: n.into(),
        ..Default::default()
    };
    views::save_views(&dir, &p, &[v("未完了"), v("メモ")]).unwrap();
    tmp
}

fn toml(tmp: &Tmp) -> String {
    std::fs::read_to_string(tmp.0.join("config/views.toml")).unwrap_or_default()
}

/// 表の上のタブの行(画面の2行目)。
fn tab_line(a: &App) -> String {
    screen(a).lines().nth(1).unwrap_or_default().to_string()
}

fn current(a: &App) -> String {
    a.view_names()[a.view_index()].clone()
}

/// 設定の画面を開いて、ビューの区画の `row` 行目を選ぶ。
fn views_section(a: &mut App, row: usize) {
    if a.mode != Mode::Settings {
        ch(a, 'o');
    }
    assert_eq!(a.mode, Mode::Settings);
    for _ in 0..10 {
        if a.draft.as_ref().unwrap().sec == Sec::Views {
            break;
        }
        press(a, KeyCode::Tab);
    }
    let d = a.draft.as_mut().unwrap();
    assert_eq!(d.sec, Sec::Views);
    d.sel[Sec::Views as usize] = row;
}

/// 区画の行の並び(タブの名前)。
fn rows(a: &App) -> Vec<String> {
    a.section_tabs().into_iter().map(|t| t.0).collect()
}

fn row_of(a: &App, name: &str) -> usize {
    rows(a).iter().position(|n| n == name).unwrap()
}

#[test]
fn test_nv_26_hide_order_default_persist() {
    // [NV-26] ビューの区画で「全部」を隠す → すぐタブから消え、`]` で移らない。「未完了」を K で上へ。
    // 「完了」を既定にする → views.toml に default_view。開き直すと同じ順・同じ隠し方で「完了」から。
    let tmp = setup("nv26a");
    let mut a = boot(&tmp, false);
    views_section(&mut a, 0);
    let s = screen(&a);
    assert!(s.contains("ビュー       5/5"), "{s}");
    assert!(s.contains("変えるとすぐ保存する"), "{s}");
    let i = row_of(&a, "全部");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    ch(&mut a, ' ');
    assert_eq!(a.message.as_deref(), Some("タブ「全部」を隠した"));
    assert!(screen(&a).contains("ビュー       4/5"));
    assert!(
        toml(&tmp).contains("hidden_tabs = [\"全部\"]"),
        "{}",
        toml(&tmp)
    );
    // 順: 未完了を上へ(全部の上 → 完了の上)。
    let i = row_of(&a, "未完了");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    ch(&mut a, 'K');
    ch(&mut a, 'K');
    assert_eq!(rows(&a), ["進行中", "未完了", "完了", "全部", "メモ"]);
    // 既定: 完了 → Enter で選び手、Enter で既定にする。
    let i = row_of(&a, "完了");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("タブ「完了」"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert!(
        toml(&tmp).contains("default_view = \"完了\""),
        "{}",
        toml(&tmp)
    );
    assert!(screen(&a).contains("★ 既定"), "{}", screen(&a));
    // 取り消しで閉じても、区画の変更は残る(反映を待たない)。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    let t = tab_line(&a);
    assert!(!t.contains("全部"), "{t}");
    assert!(t.find("未完了").unwrap() < t.find("完了 ").unwrap(), "{t}");
    for _ in 0..6 {
        ch(&mut a, ']');
        assert_ne!(current(&a), "全部");
    }
    let b = boot(&tmp, false);
    assert_eq!(current(&b), "完了");
    let t = tab_line(&b);
    assert!(!t.contains("全部"), "{t}");
    assert!(t.find("未完了").unwrap() < t.find("[完了]").unwrap(), "{t}");
}

#[test]
fn test_nv_26_current_and_base_refused() {
    // [NV-26] 開いているビューは隠さない。`.base` のビューは名前の変更も削除もできない(理由が出る)。
    // 既定のタブでもう一度選ぶと既定をやめる。
    let tmp = setup("nv26b");
    let mut a = boot(&tmp, false);
    assert_eq!(current(&a), "進行中");
    views_section(&mut a, 0);
    ch(&mut a, ' ');
    assert_eq!(
        a.message.as_deref(),
        Some("開いているビューは隠せない(先に別のタブへ)")
    );
    assert!(!toml(&tmp).contains("hidden_tabs"));
    let i = row_of(&a, "完了");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains("名前の変更と削除はできない"),
        "{:?}",
        a.message
    );
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert!(a.message.as_deref().unwrap_or("").contains("できない"));
    assert!(a.view_names().contains(&"完了".to_string()));
    // 既定にして、もう一度でやめる。
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert!(toml(&tmp).contains("default_view = \"完了\""));
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("既定をやめる"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert!(!toml(&tmp).contains("default_view"), "{}", toml(&tmp));
}

#[test]
fn test_nv_26_rename_delete_follow_prefs() {
    // [NV-26] 開いていない mdgrid のビューの名前を変える・消す。設定の画面は開いたまま、タブの順・隠す・
    // 既定の名前も合わせる。
    let tmp = setup("nv26c");
    let mut a = boot(&tmp, false);
    views_section(&mut a, 0);
    // メモを隠し、未完了を既定にしてから、未完了を「作業中」に変える。
    let i = row_of(&a, "メモ");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    ch(&mut a, ' ');
    let i = row_of(&a, "未完了");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    ch(&mut a, 'J');
    let i = row_of(&a, "未完了");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    for _ in 0.."未完了".chars().count() {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "作業中");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    let t = toml(&tmp);
    assert!(t.contains("name = \"作業中\""), "{t}");
    assert!(!t.contains("未完了"), "{t}");
    assert!(t.contains("default_view = \"作業中\""), "{t}");
    assert!(rows(&a).contains(&"作業中".to_string()));
    // メモを消す(開いていないので設定の画面は開いたまま)。
    let i = row_of(&a, "メモ");
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    let t = toml(&tmp);
    assert!(!t.contains("メモ"), "{t}");
    assert_eq!(rows(&a).len(), 4);
    assert!(screen(&a).contains("ビュー       4/4"), "{}", screen(&a));
}

#[test]
fn test_nv_26_switch_hint() {
    // [NV-26] 末尾の行で切り替えの案内をオフ → タブの行に「[ ] で切り替え」が無い。開き直しても同じ。
    let tmp = setup("nv26d");
    let mut a = boot(&tmp, false);
    assert!(tab_line(&a).contains("[ ] で切り替え"), "{}", tab_line(&a));
    let n = rows(&a).len();
    views_section(&mut a, n);
    assert!(screen(&a).contains("● オン"), "{}", screen(&a));
    ch(&mut a, ' ');
    assert!(toml(&tmp).contains("tab_hint = false"), "{}", toml(&tmp));
    press(&mut a, KeyCode::Esc);
    assert!(!tab_line(&a).contains("で切り替え"), "{}", tab_line(&a));
    let b = boot(&tmp, false);
    assert!(!tab_line(&b).contains("で切り替え"));
}

#[test]
fn test_nv_26_readonly_shows_but_does_not_write() {
    // [NV-26][WB-15] 読むだけでは区画を見せるが、変える操作は理由を出して書かない。
    let tmp = setup("nv26e");
    let before = toml(&tmp);
    let mut a = boot(&tmp, true);
    let i = row_of(&a, "完了");
    views_section(&mut a, i);
    assert!(screen(&a).contains("完了"), "{}", screen(&a));
    ch(&mut a, ' ');
    assert!(
        a.message.as_deref().unwrap_or("").contains("読むだけ"),
        "{:?}",
        a.message
    );
    ch(&mut a, 'K');
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(toml(&tmp), before);
}
