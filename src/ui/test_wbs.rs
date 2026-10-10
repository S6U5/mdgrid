//! WBS の番号と進み具合と値の対応表(NV-28)。

use super::keymap::Mode;
use super::settings::Sec;
use super::test_screen::{ch, make, press, screen, typing, Tmp};
use super::*;
use mdgrid::settings::{Wbs, WbsValue};
use ratatui::crossterm::event::KeyCode;

/// 設計の下に画面(その下に部品 done・ボタン todo)と API(doing)。
fn notes(name: &str) -> (Tmp, App) {
    let (t, mut a) = make(
        name,
        &[
            ("設計.md", "---\nstatus: doing\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nn: 1\n---\n"),
            (
                "部品.md",
                "---\nparent: \"[[画面]]\"\nstatus: done\nn: 1\n---\n",
            ),
            (
                "ボタン.md",
                "---\nparent: \"[[画面]]\"\nstatus: todo\nn: 2\n---\n",
            ),
            (
                "API.md",
                "---\nparent: \"[[設計]]\"\nstatus: doing\nn: 2\n---\n",
            ),
        ],
    );
    a.resize(100, 24);
    (t, a)
}

fn map() -> Wbs {
    let v = |value: &str, percent: u8, label: &str| WbsValue {
        value: value.into(),
        percent,
        label: label.into(),
    };
    Wbs {
        key: "status".into(),
        map: vec![
            v("done", 100, "完了"),
            v("doing", 50, "作業中"),
            v("todo", 0, "未着手"),
        ],
    }
}

fn on(a: &mut App, wbs: Option<Wbs>) {
    a.settings.tree = Some("parent".into());
    a.settings.wbs = wbs;
    a.settings.sorts = vec![("n".into(), mdgrid::settings::Dir::Asc)];
    a.regrid = true;
    a.refresh_if_needed();
}

#[test]
fn test_nv_28_numbers_progress_and_labels() {
    // [NV-28] 1 設計 · 50%、1.1 画面 · 50%、1.1.1 部品 · 完了、1.1.2 ボタン · 未着手、1.2 API · 作業中。
    let (_t, mut a) = notes("nv28a");
    on(&mut a, Some(map()));
    let s = screen(&a);
    for want in [
        "1 設計 · 50%",
        "1.1 画面 · 50%",
        "1.1.1 部品 · 完了",
        "1.1.2 ボタン · 未着手",
        "1.2 API · 作業中",
    ] {
        assert!(s.contains(want), "{want}: {s}");
    }
}

#[test]
fn test_nv_28_unmapped_counts_zero_and_off_hides() {
    // [NV-28] API を対応表に無い blocked に → 0% と数えて設計は (100+0+0)/3 = 33%、ラベルなし。
    // 親子をオフ → 番号と進み具合が消える。WBS だけオフでも消える。
    let (t, mut a) = notes("nv28b");
    t.write(
        "API.md",
        "---\nparent: \"[[設計]]\"\nstatus: blocked\nn: 2\n---\n",
    );
    let mut b = super::test_screen::app_of(&t, ColorMode::None);
    b.resize(100, 24);
    on(&mut b, Some(map()));
    let s = screen(&b);
    assert!(s.contains("1 設計 · 33%"), "{s}");
    assert!(s.contains("1.2 API") && !s.contains("1.2 API ·"), "{s}");
    on(&mut a, None);
    assert!(!screen(&a).contains("1.1 画面"), "{}", screen(&a));
    a.settings.tree = None;
    a.settings.wbs = Some(map());
    a.regrid = true;
    a.refresh_if_needed();
    assert!(!screen(&a).contains("1 設計"), "{}", screen(&a));
}

#[test]
fn test_nv_28_settings_builds_the_table() {
    // [NV-28] 設定の画面の親子の区画: 親子と WBS をオン → 値の行(件数つき)。done に「100 完了」を打つ →
    // 「→ 100% 完了」。反映 → 表に番号と「部品 · 完了」。割合が数でない → 理由を出して入力のまま。
    let (_t, mut a) = notes("nv28c");
    ch(&mut a, 'o');
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Tree {
            break;
        }
        press(&mut a, KeyCode::Tab);
    }
    let set = |a: &mut App, i: usize| a.draft.as_mut().unwrap().sel[Sec::Tree as usize] = i;
    set(&mut a, 0);
    press(&mut a, KeyCode::Enter);
    set(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(
        s.contains("WBS(番号と進み具合)") && s.contains("● オン"),
        "{s}"
    );
    assert!(s.contains("doing(2件)") && s.contains("done(1件)"), "{s}");
    let k = a
        .draft
        .as_ref()
        .unwrap()
        .wbs_vals
        .iter()
        .position(|(v, _)| v == "done")
        .unwrap();
    set(&mut a, 4 + k);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "x 完了");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText, "数でない割合は入力のまま");
    assert!(
        a.message.as_deref().unwrap_or("").contains("0〜100"),
        "{:?}",
        a.message
    );
    for _ in 0..4 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "100 完了");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert!(screen(&a).contains("→ 100% 完了"), "{}", screen(&a));
    let d = a.draft.as_mut().unwrap();
    d.sec = Sec::Buttons;
    d.sel[Sec::Buttons as usize] = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let w = a.settings.wbs.clone().unwrap();
    assert_eq!(w.key, "status");
    assert_eq!(
        w.of("done").map(|m| (m.percent, m.label.as_str())),
        Some((100, "完了"))
    );
    assert!(screen(&a).contains("部品 · 完了"), "{}", screen(&a));
    assert!(screen(&a).contains("1 設計 · "), "{}", screen(&a));
}

#[test]
fn test_nv_28_saved_with_native_view() {
    // [NV-28] WBS と対応表は mdgrid のビューの設定として views.toml に残り、読み直すと同じ。
    let mut s = mdgrid::settings::Settings {
        tree: Some("parent".into()),
        wbs: Some(map()),
        ..Default::default()
    };
    s.sorts.clear();
    let v = mdgrid::views::NativeView {
        name: "WBS".into(),
        settings: s.clone(),
        ..Default::default()
    };
    let dir = std::env::temp_dir().join(format!("mdgrid-nv28-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("notes");
    mdgrid::views::save_views(&dir, &target, std::slice::from_ref(&v)).unwrap();
    let (got, warns) = mdgrid::views::load_views(&dir, &target);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got[0].settings, s);
    let _ = std::fs::remove_dir_all(&dir);
}
