//! [cell-edit.template-vars] 雛形の {time}・{now} は、App の now(地域の時計の秒)の時刻。
//! specs/_history/decidespec/_changes/2026-10-07-template-time-offset.md。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;

// [utest -> req~cell-edit.template-vars-list~1, req~cell-edit.template-vars-body~1]
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

/// 設定 `toml` で開いた、ノート1件の表。
fn app_with(tag: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    let (cfg, _) = mdgrid::config::parse(toml).unwrap();
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
    (tmp, a)
}

/// `dir` の `.md` のうち `a.md` でないものの(名前, 中身)。
fn made(dir: &std::path::Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .filter(|p| p.file_name().is_some_and(|n| n != "a.md"))
        .map(|p| {
            (
                p.file_stem().unwrap().to_string_lossy().into_owned(),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

/// フロントマターの `key: 値` の値。
fn value_of(text: &str, key: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .unwrap_or_else(|| panic!("{key} が無い: {text}"))
        .trim_matches('"')
        .to_string()
}

// [utest -> req~cell-edit.template-vars-uuid~1]
#[test]
fn test_ce_32_uuid_variable_fills_a_new_uuid_per_note() {
    // 窓に出さない欄の `{uuid}` は、ノートごとに違う版4の UUID になる。
    let (tmp, mut a) = app_with(
        "ce32uuid",
        "[new_note]\nask = [\"status\"]\nhidden = [\"id\"]\n\n[new_note.set]\nid = \"{uuid}\"\n",
    );
    for name in ["t", "u"] {
        ch(&mut a, 'a');
        typing(&mut a, name);
        ctrl(&mut a, 's');
        assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    }
    let notes = made(&tmp.notes());
    let ids: Vec<String> = notes.iter().map(|(_, t)| value_of(t, "id")).collect();
    let shape =
        regex::Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
            .unwrap();
    assert!(shape.is_match(&ids[0]), "{}", ids[0]);
    assert!(shape.is_match(&ids[1]), "{}", ids[1]);
    assert_ne!(ids[0], ids[1]);
}

// [utest -> req~cell-edit.template-vars-uid~1, req~cell-edit.template-vars-id-same~1]
#[test]
fn test_ce_32_uid_variable_is_the_same_in_the_name_and_the_frontmatter() {
    // 名前の雛形の `{uid}` と、前もって入れる値の `{uid}` は、1つのノートの中で同じ8字になる。
    let (tmp, mut a) = app_with(
        "ce32uid",
        "[new_note]\nname = \"{uid}_\"\nask = [\"status\"]\nhidden = [\"id\"]\n\n[new_note.set]\nid = \"{uid}\"\n",
    );
    for name in ["t", "u"] {
        ch(&mut a, 'a');
        typing(&mut a, name);
        ctrl(&mut a, 's');
        assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    }
    let notes = made(&tmp.notes());
    assert_eq!(notes.len(), 2);
    let shape = regex::Regex::new("^[a-z][0-9a-z]{7}$").unwrap();
    let mut seen = Vec::new();
    for (name, text) in &notes {
        let (uid, _) = name.split_once('_').unwrap_or_else(|| panic!("{name}"));
        assert!(shape.is_match(uid), "{uid}");
        assert_eq!(value_of(text, "id"), uid);
        seen.push(uid.to_string());
    }
    assert_ne!(seen[0], seen[1]);
}
