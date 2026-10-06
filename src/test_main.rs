use super::*;

fn args(a: &[&str]) -> Vec<OsString> {
    a.iter().map(OsString::from).collect()
}

#[test]
fn test_cli_1_paths_or_current_folder() {
    // [CLI-1] 引数が無ければ今のフォルダ、あればフォルダの並び。
    assert_eq!(parse_args(&[]), Ok(Command::Run(vec![PathBuf::from(".")])));
    assert_eq!(
        parse_args(&args(&["a", "b"])),
        Ok(Command::Run(vec![PathBuf::from("a"), PathBuf::from("b")]))
    );
    assert_eq!(parse_args(&args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse_args(&args(&["--version"])), Ok(Command::Version));
    assert_eq!(
        parse_args(&args(&["--", "-x"])),
        Ok(Command::Run(vec![PathBuf::from("-x")]))
    );
}

#[test]
fn test_cli_1_no_args_opens_current_folder_default_table() {
    // [CLI-1] 引数なし → 今のフォルダの既定の表。parse_args(&[]) から check_paths → open_target → App まで
    // main と同じ道を通す。今のフォルダ(`.`)は試験の間で共有されるので、`.` の解決だけを一時フォルダに差し替える。
    assert_eq!(parse_args(&[]), Ok(Command::Run(vec![PathBuf::from(".")])));
    let t = TempDir::new("cli1");
    std::fs::write(t.0.join("a.md"), "---\ntitle: 会議\n---\n").unwrap();
    std::fs::write(t.0.join("b.md"), "---\nstatus: 完了\n---\n").unwrap();
    let Ok(Command::Run(paths)) = parse_args_in(&[], t.0.clone()) else {
        panic!("Run になる");
    };
    assert_eq!(paths, vec![t.0.clone()]);
    check_paths(&paths).unwrap();
    let target = open_target(&paths, None).unwrap();
    assert!(target.base.is_none(), ".base でなく既定の表");
    let mut app = App::new(Box::new(target.src), ColorMode::None);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: state_target(&paths),
        base: target.base,
    });
    while !app.loaded() {
        app.load_step(LOAD_BUDGET);
    }
    let mut labels: Vec<String> = app.rows.iter().map(|r| app.src.label(r)).collect();
    labels.sort();
    assert_eq!(labels, ["a.md", "b.md"]);
    let mut cols = app.cols.clone();
    cols.sort();
    assert_eq!(cols, ["status", "title"], "フロントマターのキーが列");
    assert!(app.view_names().is_empty(), "ビューのタブは無い(BV-1)");
}

#[test]
fn test_cli_4_missing_path_one_line_reason() {
    // [CLI-4] 無いパス・知らないオプションは理由1行(終了コード 2 は fail が返す)。
    let e = check_paths(&[PathBuf::from("/no/such/mdgrid/path")]).unwrap_err();
    assert!(e.contains("/no/such/mdgrid/path"));
    assert!(!e.contains('\n'));
    let e = parse_args(&args(&["--nope"])).unwrap_err();
    assert!(!e.contains('\n'));
    assert_eq!(fail("x"), ExitCode::from(2));
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(check_paths(std::slice::from_ref(&here)).is_ok());
    assert!(check_paths(&[here.join("Cargo.toml")]).is_err());
}

#[test]
fn test_sr_10_not_a_terminal() {
    // [SR-10] 標準の入出力が端末でなければ、画面を出さず理由1行。
    assert!(check_tty(true, true).is_ok());
    for (i, o) in [(false, true), (true, false), (false, false)] {
        let e = check_tty(i, o).unwrap_err();
        assert!(!e.contains('\n'));
    }
}

/// 試験ごとの一時フォルダ(終わりに消す)。
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d =
            std::env::temp_dir().join(format!("mdgrid-main-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        TempDir(d)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_cli_2_options() {
    // [CLI-2] `--readonly`・`--no-color`・`--config <パス>`・`--view <名前>` を受け、残りはパス。
    // `--` の後ろはパスのまま。値の無い `--config` は理由1行。
    let mut a = args(&[
        "notes",
        "--readonly",
        "--config",
        "/x/c.toml",
        "--no-color",
        "--view=全部",
        "--",
        "--readonly",
    ]);
    let o = take_options(&mut a).unwrap();
    assert_eq!(
        o,
        Options {
            view: Some("全部".into()),
            readonly: true,
            no_color: true,
            config: Some(PathBuf::from("/x/c.toml")),
        }
    );
    assert_eq!(
        parse_args(&a),
        Ok(Command::Run(vec![
            PathBuf::from("notes"),
            PathBuf::from("--readonly")
        ]))
    );
    let mut a = args(&["notes"]);
    assert_eq!(take_options(&mut a).unwrap(), Options::default());
    let e = take_options(&mut args(&["--config"])).unwrap_err();
    assert!(e.contains("--config") && !e.contains('\n'), "{e}");
}

#[test]
fn test_cli_11_print_config_arg() {
    // [CLI-11] [CLI-2] `--print-config` は既定の設定を出す命令。`--` の後ろならパス。
    assert_eq!(
        parse_args(&args(&["--print-config"])),
        Ok(Command::PrintConfig)
    );
    assert_eq!(
        parse_args(&args(&["--", "--print-config"])),
        Ok(Command::Run(vec![PathBuf::from("--print-config")]))
    );
    assert!(USAGE.contains("--print-config"));
}

#[test]
fn test_cli_2_config_path() {
    // [CLI-2] `--config <パス>` の設定を読む(既定の置き場より先)。無いパスは理由1行。
    let t = TempDir::new("cfg");
    let p = t.0.join("my.toml");
    std::fs::write(&p, "candidates = 7\n").unwrap();
    let other = t.0.join("default.toml");
    std::fs::write(&other, "candidates = 9\n").unwrap();
    let (c, w) = load_config(Some(&p), Some(other.clone())).unwrap();
    assert_eq!(c.candidates, 7);
    assert!(w.is_empty());
    let (c, _) = load_config(None, Some(other)).unwrap();
    assert_eq!(c.candidates, 9);
    let e = load_config(Some(&t.0.join("none.toml")), None).unwrap_err();
    assert!(e.contains("none.toml") && !e.contains('\n'), "{e}");
}

#[test]
fn test_cli_3_config_missing_unknown_broken() {
    // [CLI-3] 設定の無い環境 → 既定。知らない項目 → 警告して読む。壊れた TOML → 理由1行と終了コード 2。
    let t = TempDir::new("cli3");
    let (c, w) = load_config(None, Some(t.0.join("mdgrid/config.toml"))).unwrap();
    assert_eq!(c, Config::default());
    assert!(w.is_empty());
    assert_eq!(load_config(None, None).unwrap().0, Config::default());

    let p = t.0.join("config.toml");
    std::fs::write(&p, "nope = true\npoll_ms = 300\n").unwrap();
    let (c, w) = load_config(None, Some(p.clone())).unwrap();
    assert_eq!(c.poll_ms, 300);
    assert_eq!(w.len(), 1);
    assert!(w[0].contains("nope"));

    std::fs::write(&p, "poll_ms = 300\n[keys.table\nj = \"down\"\n").unwrap();
    let e = load_config(None, Some(p.clone())).unwrap_err();
    assert!(!e.contains('\n'), "{e}");
    assert!(e.contains("config.toml") && e.contains("行目"), "{e}");
    assert_eq!(fail(&e), ExitCode::from(2));
}

#[test]
fn test_sr_12_state_target_per_open_target() {
    // [SR-12] 状態の対象: `.base` のパス、フォルダ1つならそのパス、複数ならその組み合わせ。
    let b = PathBuf::from("v/tasks.base");
    assert_eq!(state_target(std::slice::from_ref(&b)), b);
    assert_eq!(state_target(&[PathBuf::from("a")]), PathBuf::from("a"));
    let two = state_target(&[PathBuf::from("/no/a"), PathBuf::from("/no/b")]);
    assert_eq!(two, PathBuf::from("/no/a\n/no/b"));
    assert_ne!(
        two,
        state_target(&[PathBuf::from("/no/b"), PathBuf::from("/no/a")])
    );
}
