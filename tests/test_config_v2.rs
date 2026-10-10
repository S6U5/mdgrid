//! config-v2(specs/_changes/2026-10-10-config-v2.md)の受け入れ: 旧い書き方の読み取りと --migrate-config(CLI-20)、
//! 警告の形(CLI-3)、--print-config --resolved(CLI-21)、範囲ごとの上書き(SR-44)。本物の実行ファイルで確かめる。
//! 環境変数は子の環境だけで変える。

use mdgrid::config::parse;
use mdgrid::display::TabsMode;
use mdgrid::profile::ThemeSpec;
use mdgrid::theme::Theme;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let d =
            std::env::temp_dir().join(format!("mdgrid-v2-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(d.join("config/mdgrid")).unwrap();
        Tmp(d)
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mdgrid"))
            .args(args)
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env_remove("NO_COLOR")
            .output()
            .unwrap()
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stdout(o: &Output) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout.clone()).unwrap()
}

const OLD: &str = "theme = \"auto\"\ntheme_light = \"paper\"\nborders = \"ascii\"\nview_tabs = \"auto\"\n\
                   search_bar = false\ncolor = true\n\n[style]\npreset = \"grid\"\nstatus = \"chip\"\n\n\
                   [cells]\ncheckbox = false\n\n[colors]\naccent = \"orange\"\n";

#[test]
fn test_cli_20_old_names_are_read_with_warnings() {
    // [CLI-20] 0.3.0 の書き方は新しい道筋として読み、旧い名前ごとに移った先を警告する。
    let (c, w) = parse(OLD).unwrap();
    let r = c.resolved();
    assert_eq!(
        r.theme.pick(Some(true)),
        Theme::Paper,
        "明るい地で theme_light"
    );
    assert_eq!(r.theme.pick(Some(false)), Theme::Sumi);
    assert_eq!(r.style.frames.name(), "ascii");
    assert_eq!(r.display.tabs, TabsMode::Auto);
    assert!(!r.display.search_bar);
    assert_eq!(r.preset.name(), "grid");
    assert_eq!(r.style.status.name(), "chip");
    assert_eq!(
        r.style.check.name(),
        "text",
        "[cells] checkbox = false は check = text"
    );
    assert!(r.colors.role("accent").is_some());
    for (old, new) in [
        ("theme_light", "look.theme"),
        ("borders", "look.style.frames"),
        ("view_tabs", "display.tabs"),
        ("search_bar", "display.search_bar"),
        ("color", "terminal.color"),
        ("cells.checkbox", "look.style.check"),
        ("colors", "look.colors"),
    ] {
        assert!(
            w.iter()
                .any(|m| m.starts_with(&format!("config.toml: {old}: ")) && m.contains(new)),
            "{old} → {new} の警告が無い: {w:?}"
        );
    }
    // 新しい道筋にも書いてあれば新しいほう。
    let (c, _) = parse("view_tabs = \"auto\"\n[display]\ntabs = \"always\"\n").unwrap();
    assert_eq!(c.resolved().display.tabs, TabsMode::Always);
}

#[test]
fn test_cli_20_migrate_config_prints_the_new_layout() {
    // [CLI-20] --migrate-config は新しい形の文を出し、ファイルは書かない。出した文は警告なしで同じ振る舞い。
    let t = Tmp::new("migrate");
    let cfg = t.write("config/mdgrid/config.toml", OLD);
    let out = stdout(&t.run(&["--migrate-config"]));
    assert!(
        out.starts_with("# "),
        "先頭に注釈が移らないことのコメント: {out}"
    );
    assert!(
        out.contains("[look]\ntheme = { dark = \"sumi\", light = \"paper\" }"),
        "{out}"
    );
    assert_eq!(
        std::fs::read_to_string(&cfg).unwrap(),
        OLD,
        "ファイルは書かない"
    );
    let (new, w) = parse(&out).unwrap();
    assert!(w.is_empty(), "{w:?}\n{out}");
    let (old, _) = parse(OLD).unwrap();
    let (mut a, mut b) = (new.resolved(), old.resolved());
    a.origins.clear();
    b.origins.clear();
    assert_eq!(a, b);
    // --config のファイルも写せる。
    let other = t.write("other.toml", "date_format = \"YYYY/MM/DD\"\n");
    let out = stdout(&t.run(&["--config", other.to_str().unwrap(), "--migrate-config"]));
    assert!(out.contains("[dates]\nformat = \"YYYY/MM/DD\""), "{out}");
}

#[test]
fn test_cli_3_warning_shape_in_every_file() {
    // [CLI-3] 警告はどのファイルでも「ファイルの名前: 項目の道筋: 理由」。
    let (_, w) = parse("[look.style]\nstatus = \"neon\"\n").unwrap();
    assert!(
        w[0].starts_with("config.toml: look.style.status: "),
        "{w:?}"
    );
    let t = Tmp::new("shape");
    t.write("notes/a.md", "---\nx: 1\n---\n");
    t.write("config/mdgrid/ui.toml", "[look]\ntheme = \"neon\"\n");
    let out = t.run(&[
        t.0.join("notes").to_str().unwrap(),
        "--print-config",
        "--resolved",
    ]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("ui.toml: look.theme: "), "{err}");
}

#[test]
fn test_cli_21_print_resolved_names_the_origin() {
    // [CLI-21][SR-44] 印・views.toml・config.toml を重ねた値を、出どころのコメント付きで出す。読み直せる。
    let t = Tmp::new("resolved");
    t.write("config/mdgrid/config.toml", "[look]\ntheme = \"sumi\"\n");
    t.write("notes/tasks/a.md", "---\nx: 1\n---\n");
    t.write("notes/projects/b.md", "---\nx: 1\n---\n");
    t.write(
        "notes/.mdgrid/workspace.toml",
        "name = \"N\"\n[look]\ntheme = \"nord\"\npreset = \"saas\"\n",
    );
    let tasks = t.0.join("notes/tasks");
    let real = std::fs::canonicalize(&tasks).unwrap();
    t.write(
        "config/mdgrid/views.toml",
        &format!(
            "[[table]]\npath = {}\n[table.look]\ntheme = \"dracula\"\n",
            toml::Value::String(real.to_string_lossy().into_owned())
        ),
    );
    let out = stdout(&t.run(&[tasks.to_str().unwrap(), "--print-config", "--resolved"]));
    assert!(
        out.contains("# views.toml (tasks)\ntheme = \"dracula\""),
        "{out}"
    );
    assert!(
        out.contains("# N (.mdgrid/workspace.toml)\npreset = \"saas\""),
        "{out}"
    );
    assert!(out.contains("# default\ncells = \"rich\""), "{out}");
    let (c, w) = parse(&out).unwrap();
    assert!(w.is_empty(), "{w:?}\n{out}");
    assert_eq!(c.resolved().theme, ThemeSpec::Named(Theme::Dracula));
    // ほかの表はワークスペースの nord。
    let out = stdout(&t.run(&[
        t.0.join("notes/projects").to_str().unwrap(),
        "--print-config",
        "--resolved",
    ]));
    assert!(
        out.contains("# N (.mdgrid/workspace.toml)\ntheme = \"nord\""),
        "{out}"
    );
}

#[test]
fn test_cli_11_print_config_names_scopes() {
    // [CLI-11] --print-config は区画ごとに出し、各項目に書ける範囲を添える。旧い名前は出さない。
    let t = Tmp::new("print");
    let out = stdout(&t.run(&["--print-config"]));
    for sec in [
        "[terminal]",
        "[workspace]",
        "[look]",
        "[display]",
        "[dates]",
        "[edit]",
    ] {
        assert!(out.lines().any(|l| l == sec), "{sec} が無い\n{out}");
    }
    assert!(
        out.contains("# Scope: global only.")
            && out.contains("# Scope: global, workspace, table, view.")
    );
    for old in ["theme_light", "borders =", "view_tabs", "workspace_detect"] {
        assert!(!out.contains(old), "旧い名前 {old} が出ている\n{out}");
    }
}
