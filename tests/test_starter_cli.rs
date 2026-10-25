#![cfg(unix)]
//! 台帳のスターター(ST-2・ST-3・ST-4)を本物の実行ファイルで確かめる: `--starters` の一覧(ST-3)、
//! `[<フォルダ>] --init-workspace --starter <名前> [--with-samples]` で作る(ST-4)、組み込みから作った台帳が
//! 警告なしに読め `--print` が見本の行を出し `--check` が 0(ST-2)。実装を見ずに書いた。
//! 仕様: specs/starter/spec.md、記録: specs/_changes/2026-10-24-starter.md。
//! 画面を出さない起動なので、標準出力はパイプで受ける(疑似端末は使わない)。
//!
//! 材料(作った値): ユーザーのスターター `kanban` を設定の置き場 `config/mdgrid/starters/kanban/` に書く。
//!
//! | ファイル                          | 中身                                                     |
//! |-----------------------------------|----------------------------------------------------------|
//! | starter.toml                      | name en=Kanban ja=カンバン、description en/ja             |
//! | ledger/.mdgrid/workspace.toml     | open = "Board"、[[table]] Board → board                  |
//! | ledger/.mdgrid/templates/card.md  | `{name}`・`{date}`(雛形の変数。写すときは文字のまま)    |
//! | samples/board/First card.md       | `{date+3}`・`{date}`・`{time}`・`{date-10}`(見本の変数) |
//!
//! 仮定(仕様に書かれていない細部):
//! - `--starters` の各行は名前で始まり(名前のあとは空白)、説明はその行のどこかにある。行の並びは名前の順。
//! - 標準エラーの警告は `mdgrid: ` の頭が付いてもよい(付いていれば外して CLI-3 の形を見る)。
//! - 作ったファイルの行は、そのファイルの相対パス(`.mdgrid/workspace.toml` など)を含む。1ファイル1行。
//! - 引数の誤りの理由は `mdgrid: ` で始まる1行で、関わる旗の名前のどれかを含む。

mod pty;

use pty::TempDir;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 組み込みの4つ(名前の順)。
const BUILTIN: [&str; 4] = ["bookmarks", "people", "reading", "tasks"];

/// 旗を知らないときの理由の頭(日本語)。旗の組み合わせの誤りは、これでない理由で止まる(実装の無い版と見分ける)。
const UNKNOWN_OPTION: &str = "知らないオプション";

const MANIFEST: &str = "[name]\nen = \"Kanban\"\nja = \"カンバン\"\n\n[description]\nen = \"Cards on a board\"\nja = \"板の上の札\"\n";
const MARKER: &str = "open = \"Board\"\n\n[[table]]\nname = \"Board\"\npath = \"board\"\n";
const TEMPLATE: &str = "# {name}\n\nMade on {date}\n";
const SAMPLE: &str =
    "---\ntitle: First card\ndue: \"{date+3}\"\n---\nWritten {date} at {time}, ten days after {date-10}\n";

/// `home` を HOME と設定の置き場に、`cwd` を作業のフォルダに、言語を `lang` にして起動する(標準出力はパイプ)。
/// `env` は足す環境変数(`MDGRID_TODAY` など)。
fn run_lang(home: &Path, cwd: &Path, lang: &str, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_TODAY")
        .env_remove("MDGRID_CONFIG")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"));
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().unwrap()
}

/// 日本語で起動する(.cargo/config.toml の [env] と同じ)。
fn run(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    run_lang(home, cwd, "ja_JP.UTF-8", &[], args)
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 標準エラーの空でない行。
fn err_lines(o: &Output) -> Vec<String> {
    err(o)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

/// 標準出力の空でない行。
fn out_lines(o: &Output) -> Vec<String> {
    out(o)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

/// 警告の行から `mdgrid: ` の頭を外す(付いていなければそのまま)。
fn bare(line: &str) -> &str {
    line.strip_prefix("mdgrid: ").unwrap_or(line)
}

/// 成功を確かめて標準出力を返す。
fn ok(o: Output, what: &str) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{what}: stderr={} stdout={}",
        err(&o),
        out(&o)
    );
    out(&o)
}

/// 終了コード 2・理由1行(`mdgrid: ` で始まる)・標準出力は空、を確かめて理由を返す。
fn fails_one_line(o: &Output, what: &str) -> String {
    assert_eq!(
        o.status.code(),
        Some(2),
        "{what}: stdout={} stderr={}",
        out(o),
        err(o)
    );
    let lines = err_lines(o);
    assert_eq!(lines.len(), 1, "{what}: 理由は1行: {lines:?}");
    assert!(
        lines[0].starts_with("mdgrid: "),
        "{what}: 理由の頭: {}",
        lines[0]
    );
    assert!(out(o).is_empty(), "{what}: 標準出力は空: {}", out(o));
    lines[0].clone()
}

/// `--starters` の各行の名前(行の最初の語)。
fn starter_names(o: &Output) -> Vec<String> {
    out_lines(o)
        .iter()
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect()
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// HOME(設定の置き場 config/mdgrid と state つき)を作る。
fn home(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    std::fs::create_dir_all(t.path().join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(t.path().join("state")).unwrap();
    t
}

/// ユーザーのスターター `config/mdgrid/starters/<name>/` を書き、そのフォルダを返す。
fn user_starter(home: &Path, name: &str, manifest: Option<&str>) -> PathBuf {
    let dir = home.join("config/mdgrid/starters").join(name);
    if let Some(m) = manifest {
        put(&dir.join("starter.toml"), m);
    }
    put(&dir.join("ledger/.mdgrid/workspace.toml"), MARKER);
    put(&dir.join("ledger/.mdgrid/templates/card.md"), TEMPLATE);
    put(&dir.join("samples/board/First card.md"), SAMPLE);
    dir
}

/// `root` の下の全部のファイル(根からの相対)とその中身。無ければ空。
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                map.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    map
}

/// 台帳の見本のノート(`.mdgrid/` の外の `.md`)の相対パス。
fn sample_paths(files: &BTreeMap<String, Vec<u8>>) -> Vec<&String> {
    files
        .keys()
        .filter(|k| !k.starts_with(".mdgrid/") && k.ends_with(".md"))
        .collect()
}

/// 雛形(`.mdgrid/templates/` の `.md`)の相対パス。
fn template_paths(files: &BTreeMap<String, Vec<u8>>) -> Vec<&String> {
    files
        .keys()
        .filter(|k| k.starts_with(".mdgrid/templates/") && k.ends_with(".md"))
        .collect()
}

/// 作ったファイルを1行ずつ出していること: 空でない行の数がファイルの数と同じで、どのファイルの相対パスも行にある。
fn assert_lists_files(stdout: &str, files: &BTreeMap<String, Vec<u8>>, what: &str) {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        files.len(),
        "{what}: 1ファイル1行: {stdout}\nファイル: {:?}",
        files.keys()
    );
    for rel in files.keys() {
        assert!(
            lines.iter().any(|l| l.contains(rel.as_str())),
            "{what}: {rel} の行が無い: {stdout}"
        );
    }
}

// ---- ST-3: --starters ----

#[test]
fn test_st_3_starters_lists_builtin_without_config_dir() {
    // [ST-3] 設定の置き場が無くても --starters は組み込みの4つを名前の順に、名前と説明を1行ずつ標準出力に出し、
    // 終了コード 0。画面を開かず、ファイルは書かない。HOME も XDG_CONFIG_HOME も無いときも同じ。
    let t = TempDir::new("st-list-builtin");
    let h = t.path();
    let o = run(h, h, &["--starters"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(starter_names(&o), BUILTIN, "{}", out(&o));
    for line in out_lines(&o) {
        let name = line.split_whitespace().next().unwrap_or("");
        let desc = line[name.len()..].trim();
        assert!(!desc.is_empty(), "説明が無い: {line}");
    }
    assert!(err(&o).trim().is_empty(), "警告なし: {}", err(&o));
    assert!(
        snapshot(h).is_empty(),
        "ファイルは書かない: {:?}",
        snapshot(h).keys()
    );
    assert!(!h.join("config").exists(), "設定の置き場も作らない");
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg("--starters")
        .current_dir(h)
        .env_clear()
        .env("LANG", "ja_JP.UTF-8")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "環境なし: {}", err(&o));
    assert_eq!(starter_names(&o), BUILTIN, "環境なし: {}", out(&o));
}

#[test]
fn test_st_3_user_starters_first_in_name_order_and_hide_builtin_of_same_name() {
    // [ST-3] 設定の置き場の starters/<名前>/ のものは名前の順に先に出て、同じ名前(tasks)の組み込みは出ない。
    // 説明は今の言語(ja と en で変わる)。starter.toml の無いものはフォルダの名前で出る。ファイルは書かない。
    let t = home("st-list-user");
    let h = t.path();
    user_starter(h, "kanban", Some(MANIFEST));
    user_starter(
        h,
        "tasks",
        Some("[name]\nen = \"My tasks\"\nja = \"自分のタスク\"\n\n[description]\nen = \"Mine\"\nja = \"自分の台帳\"\n"),
    );
    user_starter(h, "alpha", None);
    let before = snapshot(h);
    let o = run(h, h, &["--starters"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(
        starter_names(&o),
        ["alpha", "kanban", "tasks", "bookmarks", "people", "reading"],
        "{}",
        out(&o)
    );
    let lines = out_lines(&o);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("kanban") && l.contains("板の上の札")),
        "kanban の日本語の説明: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("tasks") && l.contains("自分の台帳")),
        "tasks はユーザーのもの: {lines:?}"
    );
    assert!(err(&o).trim().is_empty(), "警告なし: {}", err(&o));
    assert_eq!(snapshot(h), before, "ファイルは書かない");
    let o = run_lang(h, h, "en_US.UTF-8", &[], &["--starters"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(
        starter_names(&o),
        ["alpha", "kanban", "tasks", "bookmarks", "people", "reading"]
    );
    let lines = out_lines(&o);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("kanban") && l.contains("Cards on a board")),
        "kanban の英語の説明: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("tasks") && l.contains("Mine")),
        "tasks の英語の説明: {lines:?}"
    );
}

#[test]
fn test_st_3_broken_manifest_warns_one_line_and_others_are_listed() {
    // [ST-3] 壊れた starter.toml は `starters/kanban/starter.toml:` で始まる警告1行(CLI-3 の形)で飛ばし、
    // ほかのユーザーのものと組み込みは出る。終了コード 0。
    let t = home("st-list-broken");
    let h = t.path();
    user_starter(h, "kanban", Some("[name\nen = \"Kanban\"\n"));
    user_starter(h, "okay", Some(MANIFEST));
    let o = run(h, h, &["--starters"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(
        starter_names(&o),
        ["okay", "bookmarks", "people", "reading", "tasks"],
        "{}",
        out(&o)
    );
    let w = err_lines(&o);
    assert_eq!(w.len(), 1, "警告は1行: {w:?}");
    assert!(
        bare(&w[0]).starts_with("starters/kanban/starter.toml:"),
        "警告の形: {}",
        w[0]
    );
}

#[test]
fn test_st_3_starter_without_marker_warns_and_is_skipped() {
    // [ST-3] ledger/.mdgrid/workspace.toml の無いスターターは読めないので、警告(名前が入る)して飛ばす。
    let t = home("st-list-nomarker");
    let h = t.path();
    let dir = h.join("config/mdgrid/starters/nomarker");
    put(&dir.join("starter.toml"), MANIFEST);
    put(&dir.join("ledger/.mdgrid/templates/card.md"), TEMPLATE);
    put(&dir.join("samples/board/First card.md"), SAMPLE);
    let o = run(h, h, &["--starters"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(starter_names(&o), BUILTIN, "{}", out(&o));
    let w = err_lines(&o);
    assert_eq!(w.len(), 1, "警告は1行: {w:?}");
    assert!(w[0].contains("nomarker"), "{}", w[0]);
}

#[test]
fn test_st_3_starters_does_not_combine_with_other_commands() {
    // [ST-3] --starters は一覧だけ: --init-workspace や --print と一緒には組めず、理由1行と終了コード 2。何も書かない。
    let t = home("st-list-conflict");
    let h = t.path();
    for args in [
        &["--starters", "--init-workspace"][..],
        &["--starters", "--print"][..],
    ] {
        let o = run(h, h, args);
        let reason = fails_one_line(&o, &format!("{args:?}"));
        assert!(
            !reason.contains(UNKNOWN_OPTION),
            "{args:?}: 知らないオプションでなく、組めない理由: {reason}"
        );
    }
    assert!(!h.join(".mdgrid").exists(), "印を書かない");
}

// ---- ST-4: --init-workspace --starter ----

#[test]
fn test_st_4_init_with_starter_writes_ledger_only_and_lists_files() {
    // [ST-4] `books --init-workspace --starter reading`: 無いフォルダ books を作り、印 .mdgrid/workspace.toml と
    // .mdgrid/templates/ の雛形を書き、見本は書かない。作ったファイルを1行ずつ標準出力に出し、終了コード 0。
    let t = home("st-init");
    let h = t.path();
    let books = h.join("books");
    assert!(!books.exists());
    let s = ok(
        run(h, h, &["books", "--init-workspace", "--starter", "reading"]),
        "init",
    );
    assert!(books.join(".mdgrid/workspace.toml").is_file(), "印");
    let files = snapshot(&books);
    assert!(
        !template_paths(&files).is_empty(),
        "雛形が無い: {:?}",
        files.keys()
    );
    assert!(
        sample_paths(&files).is_empty(),
        "見本は書かない: {:?}",
        sample_paths(&files)
    );
    assert_lists_files(&s, &files, "init");
}

#[test]
fn test_st_4_without_folder_uses_the_current_folder() {
    // [ST-4] フォルダを渡さなければ今のフォルダに作る(WS-7 と同じ)。
    let t = home("st-init-cwd");
    let h = t.path();
    let here = h.join("here");
    std::fs::create_dir_all(&here).unwrap();
    let s = ok(
        run(h, &here, &["--init-workspace", "--starter", "tasks"]),
        "init in cwd",
    );
    assert!(here.join(".mdgrid/workspace.toml").is_file(), "{s}");
    assert!(!h.join(".mdgrid").exists(), "HOME には書かない");
    assert_lists_files(&s, &snapshot(&here), "init in cwd");
}

#[test]
fn test_st_4_with_samples_fills_sample_vars_and_keeps_template_vars() {
    // [ST-4] --with-samples と MDGRID_TODAY=2026-10-06: 見本のノートが書かれ、見本の {date-10} は 2026-09-26 で、
    // 変数は残らない。雛形の {date} は文字のまま。作ったファイル(見本も)を1行ずつ出す。
    let t = home("st-samples");
    let h = t.path();
    let s = ok(
        run_lang(
            h,
            h,
            "ja_JP.UTF-8",
            &[("MDGRID_TODAY", "2026-10-06")],
            &[
                "books",
                "--init-workspace",
                "--starter",
                "reading",
                "--with-samples",
            ],
        ),
        "with samples",
    );
    let books = h.join("books");
    let files = snapshot(&books);
    let samples = sample_paths(&files);
    assert!(!samples.is_empty(), "見本が無い: {:?}", files.keys());
    let sample_text: String = samples
        .iter()
        .map(|k| String::from_utf8_lossy(&files[*k]).into_owned())
        .collect();
    assert!(
        sample_text.contains("2026-09-26"),
        "見本の {{date-10}} が 2026-09-26 になる: {sample_text}"
    );
    for var in ["{date", "{now}", "{time}"] {
        assert!(
            !sample_text.contains(var),
            "見本に変数 {var} が残る: {sample_text}"
        );
    }
    let template_text: String = template_paths(&files)
        .iter()
        .map(|k| String::from_utf8_lossy(&files[*k]).into_owned())
        .collect();
    assert!(
        template_text.contains("{date"),
        "雛形の {{date}} は文字のまま: {template_text}"
    );
    assert_lists_files(&s, &files, "with samples");
}

#[test]
fn test_st_4_user_starter_samples_get_exact_values() {
    // [ST-4] ユーザーのスターター(作った値)で、見本の {date}・{date+3}・{date-10}・{time} が CE-32 の変数として埋まり、
    // 印と雛形は1バイトも変えずに写る。MDGRID_TODAY=2026-10-06 なので今は 00:00。
    let t = home("st-samples-user");
    let h = t.path();
    user_starter(h, "kanban", Some(MANIFEST));
    let s = ok(
        run_lang(
            h,
            h,
            "ja_JP.UTF-8",
            &[("MDGRID_TODAY", "2026-10-06")],
            &[
                "board",
                "--init-workspace",
                "--starter",
                "kanban",
                "--with-samples",
            ],
        ),
        "user starter",
    );
    let root = h.join("board");
    assert_eq!(read(&root.join(".mdgrid/workspace.toml")), MARKER);
    assert_eq!(read(&root.join(".mdgrid/templates/card.md")), TEMPLATE);
    assert_eq!(
        read(&root.join("board/First card.md")),
        "---\ntitle: First card\ndue: \"2026-10-09\"\n---\nWritten 2026-10-06 at 00:00, ten days after 2026-09-26\n"
    );
    let files = snapshot(&root);
    assert_eq!(files.len(), 3, "{:?}", files.keys());
    assert_lists_files(&s, &files, "user starter");
}

#[test]
fn test_st_4_second_run_fails_with_2_and_changes_nothing() {
    // [ST-4] 書く先に同じパスのファイルが1つでもあれば、どれも書かずに理由1行と終了コード 2(WS-7 と同じく上書きしない)。
    // もう一度(見本あり・なし)・見本の1つだけが先にある・印だけが先にある、のどれも。
    let t = home("st-twice");
    let h = t.path();
    let env = [("MDGRID_TODAY", "2026-10-06")];
    let args = [
        "books",
        "--init-workspace",
        "--starter",
        "reading",
        "--with-samples",
    ];
    ok(run_lang(h, h, "ja_JP.UTF-8", &env, &args), "first");
    let books = snapshot(&h.join("books"));
    let before = snapshot(h);
    let o = run_lang(h, h, "ja_JP.UTF-8", &env, &args);
    fails_one_line(&o, "second run");
    assert_eq!(snapshot(h), before, "2回目は何も変えない");
    let o = run(h, h, &["books", "--init-workspace", "--starter", "reading"]);
    fails_one_line(&o, "second run without samples");
    assert_eq!(snapshot(h), before, "見本なしの2回目も何も変えない");
    // 見本の1つだけが先にある → 印も雛形も書かない。
    let sample = sample_paths(&books)
        .first()
        .map(|k| (*k).clone())
        .expect("reading の見本");
    let other = h.join("other");
    put(&other.join(&sample), "keep\n");
    let o = run_lang(
        h,
        h,
        "ja_JP.UTF-8",
        &env,
        &[
            "other",
            "--init-workspace",
            "--starter",
            "reading",
            "--with-samples",
        ],
    );
    fails_one_line(&o, "one sample exists");
    let got = snapshot(&other);
    assert_eq!(got.len(), 1, "何も書かない: {:?}", got.keys());
    assert_eq!(got.get(&sample).map(Vec::as_slice), Some(&b"keep\n"[..]));
    // 印だけが先にある(WS-7 と同じ)。
    let marked = h.join("marked");
    put(&marked.join(".mdgrid/workspace.toml"), "name = \"mine\"\n");
    let o = run(
        h,
        h,
        &["marked", "--init-workspace", "--starter", "reading"],
    );
    let reason = fails_one_line(&o, "marker exists");
    assert!(
        reason.contains("workspace.toml"),
        "理由に印のパス: {reason}"
    );
    let got = snapshot(&marked);
    assert_eq!(got.len(), 1, "雛形を書かない: {:?}", got.keys());
    assert_eq!(
        read(&marked.join(".mdgrid/workspace.toml")),
        "name = \"mine\"\n"
    );
}

#[test]
fn test_st_4_unknown_starter_one_line_names_starters_flag() {
    // [ST-4] --starter nope → 理由1行(nope と、--starters で一覧を見られること)と終了コード 2。フォルダも作らない。
    let t = home("st-unknown");
    let h = t.path();
    let o = run(h, h, &["books", "--init-workspace", "--starter", "nope"]);
    let reason = fails_one_line(&o, "unknown starter");
    assert!(reason.contains("nope"), "名前を名指す: {reason}");
    assert!(
        reason.contains("--starters"),
        "--starters を添える: {reason}"
    );
    assert!(!h.join("books").exists(), "フォルダを作らない");
    assert!(!h.join(".mdgrid").exists());
}

#[test]
fn test_st_4_flag_misuse_is_one_line_and_code_2() {
    // [ST-4] --init-workspace の無い --starter、--starter の無い --with-samples、値の無い --starter、
    // 2つ以上のフォルダは、CLI-4 と同じく理由1行と終了コード 2。何も作らない。
    let t = home("st-misuse");
    let h = t.path();
    let cases: [(&[&str], &[&str]); 6] = [
        (
            &["--starter", "reading"],
            &["--starter", "--init-workspace"],
        ),
        (
            &["books", "--starter", "reading", "--with-samples"],
            &["--starter", "--init-workspace", "--with-samples"],
        ),
        (
            &["--init-workspace", "--with-samples"],
            &["--with-samples", "--starter"],
        ),
        (
            &["books", "--with-samples"],
            &["--with-samples", "--starter", "--init-workspace"],
        ),
        (&["--init-workspace", "--starter"], &["--starter"]),
        (&["a", "b", "--init-workspace", "--starter", "reading"], &[]),
    ];
    for (args, mentions) in cases {
        let o = run(h, h, args);
        let reason = fails_one_line(&o, &format!("{args:?}"));
        assert!(
            !reason.contains(UNKNOWN_OPTION),
            "{args:?}: 知らないオプションでなく、旗の組み合わせの理由: {reason}"
        );
        if !mentions.is_empty() {
            assert!(
                mentions.iter().any(|m| reason.contains(m)),
                "{args:?}: 理由に旗の名前({mentions:?}): {reason}"
            );
        }
    }
    for made in ["books", "a", "b", ".mdgrid"] {
        assert!(!h.join(made).exists(), "{made} を作らない");
    }
}

// ---- ST-2: 組み込みから作った台帳 ----

#[test]
fn test_st_2_builtin_ledgers_read_without_warnings_print_samples_and_check_clean() {
    // [ST-2] 4つの組み込みから見本つきで作る(ja と en): --print-config --resolved の標準エラーに mdgrid: の行が無く、
    // --print --with-path が見本の行(パスが .md)を出し、--check の終了コードが 0。LANG=en_US.UTF-8 でも同じ。
    let t = home("st-builtin-cli");
    let h = t.path();
    for lang in ["ja_JP.UTF-8", "en_US.UTF-8"] {
        for name in BUILTIN {
            let tag = format!("{name}-{}", &lang[..2]);
            let root = h.join(&tag);
            ok(
                run_lang(
                    h,
                    h,
                    lang,
                    &[("MDGRID_TODAY", "2026-10-06")],
                    &[
                        &tag,
                        "--init-workspace",
                        "--starter",
                        name,
                        "--with-samples",
                    ],
                ),
                &format!("{tag}: create"),
            );
            assert!(root.join(".mdgrid/workspace.toml").is_file(), "{tag}");
            let o = run_lang(h, &root, lang, &[], &["--print-config", "--resolved"]);
            assert_eq!(o.status.code(), Some(0), "{tag}: {}", err(&o));
            assert!(
                !err(&o).lines().any(|l| l.contains("mdgrid:")),
                "{tag}: 印の警告がある:\n{}",
                err(&o)
            );
            let o = run_lang(h, &root, lang, &[], &["--print", "--with-path"]);
            assert_eq!(o.status.code(), Some(0), "{tag}: {}", err(&o));
            let rows: Vec<String> = out_lines(&o).into_iter().skip(1).collect();
            assert!(!rows.is_empty(), "{tag}: 見本の行が無い: {}", out(&o));
            assert!(
                rows.iter()
                    .any(|r| r.split(',').next().unwrap_or("").ends_with(".md")),
                "{tag}: 行の先頭が見本のパスでない: {rows:?}"
            );
            let o = run_lang(h, &root, lang, &[], &["--check"]);
            assert_eq!(
                o.status.code(),
                Some(0),
                "{tag}: --check: {}\n{}",
                out(&o),
                err(&o)
            );
        }
    }
}
