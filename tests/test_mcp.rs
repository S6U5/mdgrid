//! [ai.mcp-server][ai.mcp-tools][ai.mcp-update] `mdgrid <台帳> --mcp`: 標準入出力の MCP(JSON-RPC 2.0、1行1メッセージ)。
//! 本物の実行ファイルに要求を書き、答えの行を読む。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const A: &str = "---\nstatus: todo\ndue: 2026-12-01\n---\nbody a\n";

/// 台帳の根 `notes/`(印に表 `t` と `table_extra`)と、表のノート `notes/t/a.md`。
fn ledger(name: &str, table_extra: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-mcp-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("notes/.mdgrid")).unwrap();
    std::fs::create_dir_all(root.join("notes/t")).unwrap();
    std::fs::write(
        root.join("notes/.mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"t\"\n{table_extra}"),
    )
    .unwrap();
    std::fs::write(root.join("notes/t/a.md"), A).unwrap();
    root
}

struct Server {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    next: u64,
}

impl Server {
    fn start(root: &Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
            .current_dir(root)
            .args(["notes/t", "--mcp"])
            .env("XDG_CONFIG_HOME", root.join("cfg"))
            .env("XDG_STATE_HOME", root.join("state"))
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .env_remove("MDGRID_CONFIG")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        Server {
            child,
            input,
            output,
            next: 1,
        }
    }

    /// 要求を1つ送り、同じ id の答えを返す(届くまで読む。読めなければ Null)。
    fn call(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next;
        self.next += 1;
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let Some(input) = self.input.as_mut() else {
            return serde_json::Value::Null;
        };
        if writeln!(input, "{msg}")
            .and_then(|()| input.flush())
            .is_err()
        {
            return serde_json::Value::Null;
        }
        let mut line = String::new();
        loop {
            line.clear();
            if self.output.read_line(&mut line).unwrap_or(0) == 0 {
                return serde_json::Value::Null;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
                continue;
            };
            if v["id"] == id {
                return v;
            }
        }
    }

    fn tool(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        self.call(
            "tools/call",
            serde_json::json!({"name": name, "arguments": args}),
        )
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn init(s: &mut Server) -> serde_json::Value {
    s.call(
        "initialize",
        serde_json::json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}),
    )
}

/// 道具の答えの文(content の最初の text)。
fn text(v: &serde_json::Value) -> String {
    v["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn test_mcp_initialize_and_unknown_method() {
    // [stest -> req~ai.mcp-server~1] initialize → serverInfo.name が mdgrid で capabilities.tools がある。知らないメソッド → -32601 を
    // 返し、続けて次の要求(ping)に答える。
    let root = ledger("init", "");
    let mut s = Server::start(&root);
    let r = init(&mut s);
    assert_eq!(r["result"]["serverInfo"]["name"], "mdgrid", "{r}");
    assert!(r["result"]["capabilities"]["tools"].is_object(), "{r}");
    let r = s.call("no/such", serde_json::json!({}));
    assert_eq!(r["error"]["code"], -32601, "{r}");
    let r = s.call("ping", serde_json::json!({}));
    assert!(r["result"].is_object(), "{r}");
}

#[test]
fn test_mcp_exits_when_input_closes() {
    // [stest -> req~ai.mcp-server~1] 標準入力を閉じる → 終了コード 0 で終わる。
    let root = ledger("eof", "");
    let mut s = Server::start(&root);
    let r = init(&mut s);
    assert_eq!(r["result"]["serverInfo"]["name"], "mdgrid", "{r}");
    s.input.take();
    let mut code = None;
    for _ in 0..100 {
        if let Ok(Some(st)) = s.child.try_wait() {
            code = st.code();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert_eq!(code, Some(0));
}

#[test]
fn test_mcp_lists_six_tools_with_schemas() {
    // [stest -> req~ai.mcp-tools~1] tools/list → tables・describe・query・count・check・update の6つが、入力の JSON Schema つきで並ぶ。
    let root = ledger("list", "");
    let mut s = Server::start(&root);
    init(&mut s);
    let r = s.call("tools/list", serde_json::json!({}));
    let tools = r["result"]["tools"].as_array().cloned().unwrap_or_default();
    let mut names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap_or_default().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        ["check", "count", "describe", "query", "tables", "update"],
        "{r}"
    );
    assert!(
        tools.iter().all(|t| t["inputSchema"]["type"] == "object"),
        "{r}"
    );
}

#[test]
fn test_mcp_query_returns_rows_with_rev() {
    // [stest -> req~ai.mcp-tools~1] query を filter つきで呼ぶ → 答えの文に、そのノートの path・status・_rev を持つ JSON の行。
    let root = ledger("query", "");
    let mut s = Server::start(&root);
    init(&mut s);
    let r = s.tool("query", serde_json::json!({"filter": "status == \"todo\""}));
    let rows: serde_json::Value = serde_json::from_str(&text(&r)).unwrap_or_default();
    let row = &rows[0];
    assert_eq!(row["status"], "todo", "{r}");
    assert!(
        row["path"].as_str().is_some_and(|p| p.ends_with("a.md")),
        "{r}"
    );
    assert!(row["_rev"].as_str().is_some_and(|x| !x.is_empty()), "{r}");
}

#[test]
fn test_mcp_unknown_table_is_an_error() {
    // [stest -> req~ai.mcp-tools~1] query の table に台帳に無い表の名前 → isError が真で、ある表の名前(T)を並べた理由。
    let root = ledger("notable", "");
    let mut s = Server::start(&root);
    init(&mut s);
    let r = s.tool("query", serde_json::json!({"table": "無い表"}));
    assert_eq!(r["result"]["isError"], true, "{r}");
    assert!(text(&r).contains('T'), "{r}");
}

#[test]
fn test_mcp_update_writes_allowed_and_proposes_the_rest() {
    // [stest -> req~ai.mcp-update~1] `[table.ai] write = ["status"]` → update で status は書き、due は案。答えに数。
    let root = ledger("update", "[table.ai]\nwrite = [\"status\"]\n");
    let mut s = Server::start(&root);
    init(&mut s);
    let r = s.tool("query", serde_json::json!({}));
    let rows: serde_json::Value = serde_json::from_str(&text(&r)).unwrap_or_default();
    let (path, rev) = (rows[0]["path"].clone(), rows[0]["_rev"].clone());
    let r = s.tool(
        "update",
        serde_json::json!({"reason": "見直す", "changes": [{"path": path, "_rev": rev, "status": "done", "due": "2026-12-20"}]}),
    );
    assert_ne!(r["result"]["isError"], true, "{r}");
    assert_eq!(
        std::fs::read_to_string(root.join("notes/t/a.md")).unwrap(),
        A.replace("todo", "done")
    );
    let n = std::fs::read_dir(root.join("notes/.mdgrid/proposals")).map_or(0, Iterator::count);
    assert_eq!(n, 1, "{r}");
    assert!(text(&r).contains('1'), "{r}");
}

#[test]
fn test_mcp_update_refuses_stale_rows() {
    // [stest -> req~ai.mcp-update~1] query のあとに外で due を直した → update は isError が真で、何も書かず案も置かない。
    let root = ledger("stale", "[table.ai]\nwrite = [\"status\"]\n");
    let mut s = Server::start(&root);
    init(&mut s);
    let r = s.tool("query", serde_json::json!({}));
    let rows: serde_json::Value = serde_json::from_str(&text(&r)).unwrap_or_default();
    let (path, rev) = (rows[0]["path"].clone(), rows[0]["_rev"].clone());
    let outside = A.replace("2026-12-01", "2026-12-05");
    std::fs::write(root.join("notes/t/a.md"), &outside).unwrap();
    let r = s.tool(
        "update",
        serde_json::json!({"reason": "x", "changes": [{"path": path, "_rev": rev, "status": "done"}]}),
    );
    assert_eq!(r["result"]["isError"], true, "{r}");
    assert_eq!(
        std::fs::read_to_string(root.join("notes/t/a.md")).unwrap(),
        outside
    );
    assert!(!root.join("notes/.mdgrid/proposals").exists());
}
