//! 登録した表の追加の試験(照合の指摘): 読むだけでも登録でき、ノートは書かない(CLI-18)。名前の既定は今の
//! ビュー(CLI-18)。開き直した表で登録のビューを選ぶ(CLI-19)。保存してから移る(CLI-19・WB-11)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::{self, Place};
use mdgrid::settings::Settings;
use mdgrid::source::markdown::Markdown;
use mdgrid::views::{save_views, NativeView};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const A: &str = "---\nstatus: todo\n---\n";

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", A);
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn boot(tmp: &Tmp, readonly: bool) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

fn with_view(tmp: &Tmp, name: &str) {
    let view = NativeView {
        name: name.into(),
        settings: Settings::default(),
        ..Default::default()
    };
    save_views(&tmp.0.join("config"), &tmp.notes(), &[view]).unwrap();
}

#[test]
fn test_cli_18_readonly_registers_without_touching_notes() {
    // [CLI-18][WB-15] 読むだけで開いても登録でき、書くのは places.toml だけ(ノートはそのまま)。
    let tmp = vault("plm_ro");
    let mut a = boot(&tmp, true);
    palette(&mut a, "この表を登録");
    assert_eq!(a.mode, Mode::Palette);
    press(&mut a, KeyCode::Enter); // 名前は既定のまま
    press(&mut a, KeyCode::Enter); // 分類なし
    let got = places::load(&tmp.0.join("config")).0;
    assert_eq!(got.len(), 1, "{:?}", a.message);
    assert_eq!(got[0].name, "notes");
    assert_eq!(got[0].group, "");
    assert_eq!(
        std::fs::read_to_string(tmp.notes().join("a.md")).unwrap(),
        A
    );
}

#[test]
fn test_cli_18_default_name_is_current_view() {
    // [CLI-18] 名前の既定は今のビューの名前で、登録にもそのビューが入る。
    let tmp = vault("plm_view");
    with_view(&tmp, "やること");
    let mut a = boot(&tmp, false);
    let i = a.view_names().iter().position(|v| v == "やること").unwrap();
    a.select_view(i);
    palette(&mut a, "この表を登録");
    assert_eq!(a.palette.as_ref().unwrap().query, "やること");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    let got = places::load(&tmp.0.join("config")).0;
    assert_eq!(got[0].view.as_deref(), Some("やること"));
}

#[test]
fn test_cli_19_reopen_selects_registered_view() {
    // [CLI-19] 開き直した表で、登録のビューを名前で選ぶ(main が開き直したあとに呼ぶ)。無ければ理由を出して先頭。
    let tmp = vault("plm_sel");
    with_view(&tmp, "やること");
    let mut a = boot(&tmp, false);
    a.select_view_named("やること");
    assert_eq!(a.native_view().map(|v| v.name.as_str()), Some("やること"));
    let mut b = boot(&tmp, false);
    b.select_view_named("無いビュー");
    assert!(b.native_view().is_none());
    assert!(b.message.as_deref().unwrap_or("").contains("無いビュー"));
}

#[test]
fn test_cli_19_save_then_switch() {
    // [CLI-19][WB-11] ためた変更があるとき、s → 保存の確認で Enter → 書いてから終わり、選んだ表へ移る。
    let tmp = vault("plm_save");
    let other = tmp.0.join("Other");
    std::fs::create_dir_all(&other).unwrap();
    places::save(
        &tmp.0.join("config"),
        Place {
            name: "O".into(),
            group: String::new(),
            path: other.clone(),
            view: None,
        },
    )
    .unwrap();
    let mut a = boot(&tmp, false);
    let row = a.src.rows()[0].clone();
    a.changes
        .set(
            a.src.as_ref(),
            &row,
            "status",
            mdgrid::source::NewValue::Str("done".into()),
        )
        .unwrap();
    palette(&mut a, "登録した表を開く");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    ch(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "保存したら終わる");
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(PathBuf::from(&other))
    );
    assert!(std::fs::read_to_string(tmp.notes().join("a.md"))
        .unwrap()
        .contains("status: done"));
}
