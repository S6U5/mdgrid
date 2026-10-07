//! [CE-32] 雛形の {time}・{now} は、App の now(地域の時計の秒)の時刻。
//! specs/_changes/2026-10-07-template-time-offset.md。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;

#[test]
fn test_ce_32_time_variable_uses_local_now() {
    let tmp = Tmp::new("ce32time");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    let (cfg, _) = mdgrid::config::parse(
        "[new_note]\nask = [\"status\"]\nhidden = [\"at\"]\n\n[new_note.set]\nat = \"{now}\"\n",
    )
    .unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    let day = types::parse_date("2026-10-02").unwrap();
    a.today = day;
    // 地域の時計の秒(時差は足し済み): 2026-10-02 13:45。
    a.now = day * 86_400 + 13 * 3600 + 45 * 60;
    ch(&mut a, 'a');
    typing(&mut a, "t");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let text = std::fs::read_to_string(tmp.notes().join("t.md")).unwrap();
    assert!(text.contains("at: 2026-10-02T13:45"), "{text}");
}
