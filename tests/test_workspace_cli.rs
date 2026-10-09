#![cfg(unix)]
//! ワークスペースの旗(WS-3・WS-7)を本物の実行ファイルで確かめる: パイプの上で --workspaces・--add-to・
//! --remove-from・--remove-workspace・--init-workspace、`-w` で最初の表を開く、無い名前は理由1行。
//! 旗なので、`workspace` という名前のフォルダはそのまま開ける。

mod pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn notes(home: &Path) {
    for (dir, note) in [("tasks", "a"), ("projects", "p")] {
        std::fs::create_dir_all(home.join(dir)).unwrap();
        std::fs::write(
            home.join(dir).join(format!("{note}.md")),
            "---\nstatus: todo\n---\n",
        )
        .unwrap();
    }
    std::fs::create_dir_all(home.join("config/mdgrid")).unwrap();
}

#[test]
fn test_ws_3_add_list_remove_on_pipe() {
    // [WS-3] --add-to で作り、--workspaces に出る(標準出力はパイプ)。--remove-from で表を外し、
    // --remove-workspace でワークスペースごと消す。
    let home = TempDir::new("ws-cli");
    let h = home.path();
    notes(h);
    let o = run(h, &["--workspaces"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("ワークスペースは無い"), "{}", out(&o));
    let o = run(h, &["tasks", "--add-to", "Product"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = run(h, &["projects", "--add-to", "Product", "--as", "Projects"]);
    assert!(o.status.success(), "{}", err(&o));
    let s = out(&run(h, &["--workspaces"]));
    assert!(s.starts_with("Product\n"), "{s}");
    assert!(s.contains("  tasks  ") && s.contains("  Projects  "), "{s}");
    let toml = std::fs::read_to_string(h.join("config/mdgrid/workspaces.toml")).unwrap();
    assert!(toml.contains("name = \"Product\""), "{toml}");
    let o = run(h, &["tasks", "--remove-from", "Product"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(!out(&run(h, &["--workspaces"])).contains("  tasks  "));
    let o = run(h, &["tasks", "--remove-from", "Product"]);
    assert_eq!(o.status.code(), Some(2), "もう無い表");
    let o = run(h, &["--remove-workspace", "Product"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&run(h, &["--workspaces"])).contains("ワークスペースは無い"));
}

#[test]
fn test_ws_3_bad_input_one_line_reason() {
    // [WS-3] 無いパス・無い名前・組めない旗は、理由1行と 0 以外の終了コード。
    let home = TempDir::new("ws-cli-bad");
    let h = home.path();
    notes(h);
    for args in [
        &["nowhere", "--add-to", "P"][..],
        &["--remove-workspace", "Nope"][..],
        &["tasks", "--remove-from", "Nope"][..],
        &["tasks", "--add-to", "P", "--print"][..],
        &["tasks", "projects", "--add-to", "P", "--as", "X"][..],
        &["-w", "Nope"][..],
    ] {
        let o = run(h, args);
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        assert!(err(&o).starts_with("mdgrid: "), "{args:?}: {}", err(&o));
        assert_eq!(err(&o).lines().count(), 1, "{args:?}: {}", err(&o));
        assert!(out(&o).is_empty(), "{args:?}");
    }
    assert!(err(&run(h, &["-w", "Nope"])).contains("ワークスペース Nope は無い"));
    assert!(
        !h.join("config/mdgrid/workspaces.toml").exists(),
        "誤りでは何も書かない"
    );
}

#[test]
fn test_ws_3_folder_named_workspace_opens() {
    // [WS-3] 操作は旗なので、`workspace` という名前のフォルダは `mdgrid workspace` でそのまま開ける。
    let home = TempDir::new("ws-cli-folder");
    let h = home.path();
    notes(h);
    std::fs::create_dir_all(h.join("workspace")).unwrap();
    std::fs::write(h.join("workspace/plan.md"), "---\nstatus: doing\n---\n").unwrap();
    let o = run(h, &["workspace", "--print"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("doing"), "{}", out(&o));
}

#[test]
fn test_ws_7_init_marker() {
    // [WS-7] --init-workspace で、そのフォルダ(無ければ今のフォルダ)に印を作る。もうあれば書かずに理由。
    let home = TempDir::new("ws-cli-init");
    let h = home.path();
    notes(h);
    let o = run(h, &["--init-workspace"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(h.join(".mdgrid/workspace.toml").is_file());
    let o = run(h, &[".", "--init-workspace"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(err(&o).contains("もうある"), "{}", err(&o));
    let o = run(h, &["tasks", "--init-workspace"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(h.join("tasks/.mdgrid/workspace.toml").is_file());
}

#[test]
fn test_ws_3_w_opens_first_table() {
    // [WS-3] `-w <名前>` だけなら、そのワークスペースの最初の表を開き、ヘッダーに名前(WS-6)。
    let home = TempDir::new("ws-cli-w");
    let h = home.path();
    notes(h);
    assert!(run(h, &["projects", "tasks", "--add-to", "Product"])
        .status
        .success());
    let mut tui = Tui::spawn(&["-w", "Product"], h, &[]);
    tui.wait_until("最初の表とワークスペースの名前", |s| {
        s.contains("projects") && s.contains("ワークスペース Product")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
