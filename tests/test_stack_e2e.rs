//! 縦に並べる(specs/requirements/stack.md)を、本物の実行ファイルで確かめる。画面の無い起動の誤り(`--page`)は子の
//! 標準エラーと終了コードで、画面は疑似端末(80×24)で見る。ワークスペース work は設定のフォルダの
//! workspaces.toml に書く。期待値は要求の受け入れの例から取った。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mdgrid-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(std::fs::canonicalize(&dir).unwrap_or(dir))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TASKS_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"tasks\")\nviews:\n  - type: table\n    name: 今日\n    filters:\n      and:\n        - status != \"done\"\n    order:\n      - file.name\n      - title\n      - status\n  - type: table\n    name: 全部\n    order:\n      - file.name\n      - title\n      - status\n";

const PROJECTS_BASE: &str = "filters:\n  and:\n    - file.inFolder(\"projects\")\nviews:\n  - type: table\n    name: 進行中\n    filters:\n      and:\n        - status == \"進行中\"\n    order:\n      - file.name\n      - title\n      - status\n  - type: table\n    name: 全部\n    order:\n      - file.name\n      - title\n      - status\n";

/// ホーム: notes に tasks(今日・全部)と projects(進行中・全部)の `.base` とノート、設定のフォルダの
/// workspaces.toml にワークスペース work(tasks・projects と、同じフォルダ notes/tasks を指す a・b)。`pages` は
/// work の区画の末尾に足す文。
fn home(tag: &str, pages: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in ["config/mdgrid", "state", "notes/tasks", "notes/projects"] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    let w = |rel: &str, text: &str| std::fs::write(p.join(rel), text).unwrap();
    w(
        "notes/tasks/t1.md",
        "---\ntitle: 会議の準備\nstatus: todo\n---\n",
    );
    w(
        "notes/tasks/t2.md",
        "---\ntitle: 買い物\nstatus: todo\n---\n",
    );
    w("notes/tasks/t3.md", "---\ntitle: 読書\nstatus: done\n---\n");
    for (n, title, status) in [
        ("p1", "会議室の予約", "進行中"),
        ("p2", "新製品", "進行中"),
        ("p3", "採用", "進行中"),
        ("p4", "会議の改善", "進行中"),
        ("p5", "旧案", "完了"),
    ] {
        w(
            &format!("notes/projects/{n}.md"),
            &format!("---\ntitle: {title}\nstatus: {status}\n---\n"),
        );
    }
    w("notes/tasks.base", TASKS_BASE);
    w("notes/projects.base", PROJECTS_BASE);
    w("config/mdgrid/config.toml", "");
    let n = p.join("notes");
    w(
        "config/mdgrid/workspaces.toml",
        &format!(
            "[[workspace]]\nname = \"work\"\n\n[[workspace.table]]\nname = \"tasks\"\npath = \"{}\"\n\n[[workspace.table]]\nname = \"projects\"\npath = \"{}\"\n\n[[workspace.table]]\nname = \"a\"\npath = \"{}\"\n\n[[workspace.table]]\nname = \"b\"\npath = \"{}\"\n{pages}",
            n.join("tasks.base").display(),
            n.join("projects.base").display(),
            n.join("tasks").display(),
            n.join("tasks").display(),
        ),
    );
    h
}

const PAGES: &str = "\n[[workspace.page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\" },\n  { table = \"projects\", view = \"進行中\" },\n]\n\n[[workspace.page]]\nname = \"同じ\"\ntables = [{ table = \"a\" }, { table = \"b\" }]\n";

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

#[test]
fn test_stack_page_cli_unknown_name_exits_2() {
    // [stest -> req~stack.page-cli~1] 無い名前 → 「ページ「夜」はありません(ある名前: 朝, 同じ)」の1行と終了コード 2。
    let h = home("stack-page-cli", PAGES);
    let o = run(h.path(), &["-w", "work", "--page", "夜"]);
    assert_eq!(o.status.code(), Some(2));
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(
        err.trim_end(),
        "mdgrid: ページ「夜」はありません(ある名前: 朝, 同じ)"
    );
    assert!(o.stdout.is_empty());
}

#[test]
fn test_stack_page_conflicts_with_print() {
    // [stest -> req~stack.page-cli~1] `--page` は画面を出す起動だけ(`--print` とは一緒に受けない)。
    let h = home("stack-page-print", PAGES);
    let o = run(h.path(), &["-w", "work", "--page", "朝", "--print"]);
    assert_eq!(o.status.code(), Some(2));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("--page"), "{err}");
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, PAGES};
    use crate::pty::{wait_file, Tui};

    /// 起動の直後の1打は、端末の地の色の問い合わせ(テーマの auto)に吸われることがあるので、効くまで押す。
    fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done(&t.screen()) && std::time::Instant::now() < end {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    fn line(s: &str, y: usize) -> String {
        s.lines().nth(y).unwrap_or_default().to_string()
    }

    /// パレットを開いて動作の名前を打ち、Enter。
    fn palette(t: &mut Tui, name: &str) {
        press_until(t, ":", |s| s.lines().any(|l| l.trim_end() == ":"));
        t.send_settle(name);
        t.send("\r");
    }

    /// tasks を開き、パレットの「下に表を並べる」で projects を選ぶ。
    fn stack_projects(t: &mut Tui) {
        t.wait_for("会議の準備");
        palette(t, "stack_add");
        t.wait_for("下に並べる表");
        t.send_settle("projects");
        t.send("\r");
        t.wait_until("projects を並べる", |s| {
            line(s, 11).starts_with("▶ projects")
        });
    }

    #[test]
    fn test_stack_add_on_screen() {
        // [stest -> req~stack.add~1] ワークスペース work の tasks を開き、パレットの「下に表を並べる」で projects を
        // 選ぶ → 画面の上半分に tasks、下半分に projects の行が出る。
        let h = home("stack-add", "");
        let mut t = Tui::spawn(&["-w", "work"], h.path(), &[]);
        stack_projects(&mut t);
        let s = t.screen();
        assert!(line(&s, 0).starts_with("  tasks · 今日 · 2"), "{s}");
        assert!(line(&s, 11).starts_with("▶ projects · 進行中 · 4"), "{s}");
        let top: Vec<&str> = s.lines().take(11).collect();
        let bottom: Vec<&str> = s.lines().skip(11).take(10).collect();
        assert!(top.iter().any(|l| l.contains("会議の準備")), "{s}");
        assert!(bottom.iter().any(|l| l.contains("新製品")), "{s}");
        assert!(!top.iter().any(|l| l.contains("新製品")), "{s}");
        drop(t);
    }

    #[test]
    fn test_stack_page_save_on_screen() {
        // [stest -> req~stack.page-save~1] tasks(今日)と projects(進行中)を並べ、名前「朝」で残す → workspaces.toml
        // の work に `[[workspace.page]]`・`name = "朝"`・2つの表とビュー。
        let h = home("stack-page-save", "");
        let mut t = Tui::spawn(&["-w", "work"], h.path(), &[]);
        stack_projects(&mut t);
        palette(&mut t, "page_save");
        t.wait_for("ページの名前");
        t.send_settle("朝");
        t.send("\r");
        let file = h.path().join("config/mdgrid/workspaces.toml");
        let text = wait_file(&file, &t, |s| s.contains("[[workspace.page]]"));
        assert!(
            text.contains("[[workspace.page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\", height = 1 },\n  { table = \"projects\", view = \"進行中\", height = 1 },\n]\n"),
            "{text}"
        );
        t.wait_for("ページ「朝」を workspaces.toml に残した");
        drop(t);
    }

    #[test]
    fn test_stack_page_cli_on_screen() {
        // [stest -> req~stack.page-cli~1, req~stack.page-open~1] `mdgrid -w work --page 朝` → tasks と projects が並ぶ。
        let h = home("stack-page-open", PAGES);
        let t = Tui::spawn(&["-w", "work", "--page", "朝"], h.path(), &[]);
        t.wait_until("ページの表", |s| {
            line(s, 0).starts_with("▶ tasks · 今日 · 2")
                && line(s, 11).starts_with("  projects · 進行中 · 4")
        });
        let s = t.screen();
        assert!(line(&s, 0).starts_with("▶ tasks · 今日 · 2"), "{s}");
        assert!(line(&s, 11).starts_with("  projects · 進行中 · 4"), "{s}");
        drop(t);
    }

    #[test]
    fn test_stack_page_store_warns_on_screen() {
        // [stest -> req~stack.page-store~1] 手で書いたページに無い表の名前を1つ含む → 起動して警告(ファイル: 道筋:
        // 理由)が出て、ページは残りの表で開く。
        let pages = "\n[[workspace.page]]\nname = \"朝\"\ntables = [{ table = \"tasks\" }, { table = \"nothing\" }, { table = \"projects\" }]\n";
        let h = home("stack-page-store", pages);
        let t = Tui::spawn(&["-w", "work", "--page", "朝"], h.path(), &[]);
        t.wait_until("警告とページ", |s| {
            s.contains("workspaces.toml: workspace.page[1].tables[2].table:")
                && line(s, 11).starts_with("  projects")
        });
        let s = t.screen();
        assert!(line(&s, 0).starts_with("▶ tasks"), "{s}");
        assert!(s.contains("nothing"), "{s}");
        drop(t);
    }

    #[test]
    fn test_stack_shared_pending_on_screen() {
        // [stest -> req~stack.shared-pending~1] 同じフォルダを指す表を2つ並べ、上の表で t1.md の status を done に、
        // 下の表で title を「新」に直して保存 → t1.md の status が done で title が「新」(どちらの変更も消えない)。
        let h = home("stack-shared", PAGES);
        let mut t = Tui::spawn(&["-w", "work", "--page", "同じ"], h.path(), &[]);
        t.wait_until("並んだ", |s| line(s, 0).starts_with("▶ a · 3"));
        // 上の表(a): t1 の status。
        press_until(&mut t, "l", |s| s.contains("2/2列"));
        t.send("\r");
        t.wait_for("Enter 確定");
        t.send_settle("done");
        t.send_settle("\r");
        t.wait_for("未保存 1");
        // 下の表(b): t1 の title。
        t.send("\x17");
        t.send_settle("j");
        t.wait_until("下の表", |s| line(s, 11).starts_with("▶ b · 3"));
        t.send("\r");
        t.wait_for("Enter 確定");
        t.send_settle("新");
        t.send_settle("\r");
        t.wait_for("未保存 2");
        t.send("\x13");
        t.wait_for("+status: done");
        t.send("\r");
        let body = wait_file(&h.path().join("notes/tasks/t1.md"), &t, |s| {
            s.contains("title: 新")
        });
        assert_eq!(body, "---\ntitle: 新\nstatus: done\n---\n");
        drop(t);
    }
}
