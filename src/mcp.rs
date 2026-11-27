//! MCP の口(ai.mcp-server・ai.mcp-tools・ai.mcp-update): `mdgrid <台帳> --mcp` は、標準入力と標準出力で MCP
//! (JSON-RPC 2.0、1行に1つのメッセージ)を話す。手元だけで動き、ネットにはつながない。
//!
//! 道具は表の一覧・表の設定・行を読む・数える・点検・直すの6つ。読み書きは、mdgrid 自身の `--print`・`--count`・
//! `--check`・`--apply … --propose` を、決まった引数で(シェルを通さずに)走らせて同じ道を通す(検査と書き戻しを
//! 二重に持たない)。起動で渡した台帳のパスをいつも先頭に渡すので、その台帳の外は読み書きしない。

use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// 答えられる MCP の版(新しい順)。相手の版が無ければ先頭の版で答える。
const VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

/// `query` の既定の行の数の上限と、上げられる上限。
const QUERY_LIMIT: usize = 200;
const QUERY_MAX: usize = 2000;

/// 道具の定義(`tools/list`)。
fn tools() -> serde_json::Value {
    let table = serde_json::json!({"type": "string", "description": "Table name in the ledger (see the tables tool). Omit for the table mdgrid was started with."});
    let view = serde_json::json!({"type": "string", "description": "View name of the table. Omit for the table's default view."});
    let filter = serde_json::json!({"type": ["string", "array"], "items": {"type": "string"}, "description": "Obsidian Bases filter expression(s), e.g. status == \"todo\" or due <= today() + \"7d\". Several are all required."});
    serde_json::json!([
        {
            "name": "tables",
            "description": "List the tables of this ledger (name, path and default view).",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "describe",
            "description": "Show the settings of a table as TOML: column types ([types]), allowed values and rules ([rules]), and ai.write (the columns you may change directly; changes to other columns become proposals for a person to review).",
            "inputSchema": {"type": "object", "properties": {"table": table, "view": view}},
        },
        {
            "name": "query",
            "description": "Read rows as a JSON array of objects: path, the view's columns, and _rev. Keep path and _rev when you send changes to update.",
            "inputSchema": {"type": "object", "properties": {
                "table": table, "view": view, "filter": filter,
                "sort": {"type": ["string", "array"], "items": {"type": "string"}, "description": "Column to sort by, column:desc for descending. Several break ties."},
                "limit": {"type": "integer", "minimum": 1, "maximum": QUERY_MAX, "description": "Most rows to return (default 200)."},
            }},
        },
        {
            "name": "count",
            "description": "Count the rows of a table or view, optionally filtered.",
            "inputSchema": {"type": "object", "properties": {"table": table, "view": view, "filter": filter}},
        },
        {
            "name": "check",
            "description": "Check the rows against the table's rules (types, required columns, allowed values). One problem per line: path:line: column: reason.",
            "inputSchema": {"type": "object", "properties": {"table": table, "view": view}},
        },
        {
            "name": "update",
            "description": "Change values. Each change is an object with path and _rev from query plus the columns to set. Columns listed in the ledger's ai.write are written; every other change is left as a proposal that a person reviews in mdgrid. A row whose note changed after your query is refused: query again and redo the change.",
            "inputSchema": {"type": "object", "required": ["reason", "changes"], "properties": {
                "table": table, "view": view,
                "reason": {"type": "string", "description": "One line on why, shown to the person who reviews the proposal."},
                "changes": {"type": "array", "items": {"type": "object", "required": ["path"], "properties": {"path": {"type": "string"}, "_rev": {"type": "string"}}, "additionalProperties": true}},
            }},
        },
    ])
}

/// 標準入力の要求に答え続け、閉じたら終わる。
pub(crate) fn serve(paths: &[PathBuf], table: Option<&str>, view: Option<&str>) -> ExitCode {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = answer(&line, paths, table, view) {
            if writeln!(out, "{reply}").and_then(|()| out.flush()).is_err() {
                break;
            }
        }
    }
    ExitCode::SUCCESS
}

/// 1つのメッセージに答える(通知には答えない)。
fn answer(
    line: &str,
    paths: &[PathBuf],
    table: Option<&str>,
    view: Option<&str>,
) -> Option<serde_json::Value> {
    let msg: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(error(
                serde_json::Value::Null,
                -32700,
                &format!("parse error: {e}"),
            ))
        }
    };
    let id = msg.get("id").cloned();
    let method = msg
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let params = msg
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let id = id?; // 通知(id の無いもの)には答えない
    let result = match method {
        "initialize" => {
            let asked = params
                .get("protocolVersion")
                .and_then(serde_json::Value::as_str);
            let version = asked
                .filter(|v| VERSIONS.contains(v))
                .unwrap_or(VERSIONS[0]);
            serde_json::json!({
                "protocolVersion": version,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "mdgrid", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Read the ledger with query, and change values with update: only columns in ai.write are written, the rest become proposals for a person to review.",
            })
        }
        "ping" => serde_json::json!({}),
        "tools/list" => serde_json::json!({"tools": tools()}),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            match call(name, &args, paths, table, view) {
                Some((text, is_error)) => serde_json::json!({
                    "content": [{"type": "text", "text": text}],
                    "isError": is_error,
                }),
                None => return Some(error(id, -32602, &format!("unknown tool: {name}"))),
            }
        }
        _ => return Some(error(id, -32601, &format!("method not found: {method}"))),
    };
    Some(serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn error(id: serde_json::Value, code: i64, message: &str) -> serde_json::Value {
    serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// 文字か文字の並びの引数を、並びにする。
fn strings(v: Option<&serde_json::Value>) -> Vec<String> {
    match v {
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        Some(serde_json::Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

/// 道具を走らせる: (答えの文, 誤りか)。知らない道具は None。
fn call(
    name: &str,
    args: &serde_json::Value,
    paths: &[PathBuf],
    table: Option<&str>,
    view: Option<&str>,
) -> Option<(String, bool)> {
    let get = |k: &str| args.get(k).and_then(serde_json::Value::as_str);
    let mut base: Vec<OsString> = paths.iter().map(|p| p.clone().into_os_string()).collect();
    if let Some(t) = get("table").or(table) {
        base.push("--table".into());
        base.push(t.into());
    }
    if let Some(v) = get("view").or(view) {
        base.push("--view".into());
        base.push(v.into());
    }
    let with = |extra: &[&str], more: Vec<OsString>| -> Vec<OsString> {
        let mut a = base.clone();
        a.extend(extra.iter().map(OsString::from));
        a.extend(more);
        a
    };
    let filters: Vec<OsString> = strings(args.get("filter"))
        .into_iter()
        .flat_map(|f| [OsString::from("--filter"), OsString::from(f)])
        .collect();
    Some(match name {
        "tables" => (tables_json(paths), false),
        "describe" => run(
            &with(&["--print-config", "--resolved"], Vec::new()),
            None,
            false,
        ),
        "count" => run(&with(&["--count"], filters), None, false),
        "check" => run(&with(&["--check"], Vec::new()), None, true),
        "query" => {
            let sorts: Vec<OsString> = strings(args.get("sort"))
                .into_iter()
                .flat_map(|s| [OsString::from("--sort"), OsString::from(s)])
                .collect();
            let mut more = filters;
            more.extend(sorts);
            let (text, err) = run(
                &with(&["--print", "--with-path", "--format", "json"], more),
                None,
                false,
            );
            if err {
                return Some((text, true));
            }
            let limit = args
                .get("limit")
                .and_then(serde_json::Value::as_u64)
                .map_or(QUERY_LIMIT, |n| (n as usize).clamp(1, QUERY_MAX));
            match serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                Ok(rows) if rows.len() > limit => {
                    let shown = serde_json::Value::Array(rows[..limit].to_vec());
                    (format!("{shown}\n(showing {limit} of {} rows; narrow it with filter or raise limit)", rows.len()), false)
                }
                Ok(rows) => (serde_json::Value::Array(rows).to_string(), false),
                Err(_) => (text, false),
            }
        }
        "update" => {
            let Some(reason) = get("reason").filter(|r| !r.trim().is_empty()) else {
                return Some(("update needs a non-empty reason".into(), true));
            };
            let Some(changes) = args.get("changes").filter(|c| c.is_array()) else {
                return Some(("update needs changes: an array of objects with path, _rev and the columns to set".into(), true));
            };
            run(
                &with(&["--apply", "-", "--propose"], vec![reason.into()]),
                Some(&changes.to_string()),
                false,
            )
        }
        _ => return None,
    })
}

/// mdgrid 自身を `args` で走らせる(シェルを通さない)。`input` は標準入力に渡す。終了コード 0(`ok1` なら 1 も)のほかは誤り。
fn run(args: &[OsString], input: Option<&str>, ok1: bool) -> (String, bool) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return (format!("cannot find mdgrid: {e}"), true),
    };
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (format!("cannot run mdgrid: {e}"), true),
    };
    if let (Some(text), Some(mut w)) = (input, child.stdin.take()) {
        let _ = w.write_all(text.as_bytes());
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return (format!("mdgrid failed: {e}"), true),
    };
    let stdout = String::from_utf8_lossy(&out.stdout).trim_end().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim_end().to_string();
    let code = out.status.code().unwrap_or(-1);
    let text = match (stdout.is_empty(), stderr.is_empty()) {
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
        (false, false) => format!("{stdout}\n{stderr}"),
    };
    (text, !(code == 0 || ok1 && code == 1))
}

/// 台帳の表(印の表。無ければ起動で渡したフォルダ1つ)を JSON の並びにする。
fn tables_json(paths: &[PathBuf]) -> String {
    let start = paths
        .first()
        .map_or_else(|| PathBuf::from("."), Clone::clone);
    let dir = if start.is_file() {
        start
            .parent()
            .map_or_else(|| start.clone(), Path::to_path_buf)
    } else {
        start.clone()
    };
    let tables = mdgrid::workspace::find_marker(&dir)
        .map(|root| {
            let (w, _) = mdgrid::workspace::read_marker(&root);
            w.tables
                .into_iter()
                .map(|t| {
                    let rel = t.path.strip_prefix(&root).map_or_else(|_| t.path.clone(), Path::to_path_buf);
                    serde_json::json!({"name": t.name, "path": rel.to_string_lossy(), "view": t.view})
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if tables.is_empty() {
        let name = mdgrid::workspace::stem_of(&start);
        return serde_json::json!([{"name": name, "path": start.to_string_lossy(), "view": null}])
            .to_string();
    }
    serde_json::Value::Array(tables).to_string()
}
