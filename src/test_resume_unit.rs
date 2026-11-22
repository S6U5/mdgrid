//! 開き直し(resume.rs)の単体の試験。期待値は specs/requirements/resume.md の受け入れの例から。

use super::*;

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-resume-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Tmp(dir.canonicalize().unwrap())
    }

    fn dir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(std::string::ToString::to_string).collect()
}

#[test]
fn test_reopen_args_folder_with_workspace_and_readonly() {
    // [utest -> req~resume.reopen-args~1] `-w tasks --readonly` でビューの無いフォルダ notes を開く → mdgrid <notes の絶対パス> --raw -w tasks --readonly。
    let t = Tmp::new("args-folder");
    let notes = t.dir("notes");
    let r = Reopen::new(
        std::slice::from_ref(&notes),
        Some("tasks".into()),
        true,
        false,
        None,
    );
    let want = vec![
        "mdgrid".to_string(),
        notes.to_string_lossy().into_owned(),
        "--raw".into(),
        "-w".into(),
        "tasks".into(),
        "--readonly".into(),
    ];
    assert_eq!(r.argv(), want);
}

#[test]
fn test_reopen_args_base_view_and_relative_path() {
    // [utest -> req~resume.reopen-args~1] `.base` のビュー「進行中」を開いている → mdgrid <.base の絶対パス> --view 進行中(相対で渡しても絶対パス。`--raw` は付けない)。
    let t = Tmp::new("args-base");
    let base = t.0.join("tasks.base");
    std::fs::write(&base, "views: []\n").unwrap();
    let rel = PathBuf::from("tasks.base");
    let mut r = Reopen::new(std::slice::from_ref(&rel), None, false, false, None);
    // 今のフォルダからの相対は、今のフォルダの絶対パスにつなぐ。
    let here = std::env::current_dir().unwrap();
    assert_eq!(r.paths, vec![here.join("tasks.base")]);
    let mut r2 = Reopen::new(std::slice::from_ref(&base), None, false, false, None);
    r2.view = Some("進行中".into());
    assert_eq!(
        r2.argv(),
        vec![
            "mdgrid".to_string(),
            base.to_string_lossy().into_owned(),
            "--view".into(),
            "進行中".into(),
        ]
    );
    r.view = Some("進行中".into());
    assert!(!r.argv().contains(&"--raw".to_string()));
}

#[test]
fn test_reopen_args_order_of_all_flags() {
    // [utest -> req~resume.reopen-args~1] 命令は、パス・`--raw`・`--view`・`-w`・`--readonly`・`--no-color`・`--config`(絶対パス)の順。
    let t = Tmp::new("args-all");
    let notes = t.dir("notes");
    let mut r = Reopen::new(
        std::slice::from_ref(&notes),
        Some("ws".into()),
        true,
        true,
        Some(Path::new("conf/my.toml")),
    );
    r.view = Some("全部".into());
    let here = std::env::current_dir().unwrap();
    assert_eq!(
        r.argv(),
        vec![
            "mdgrid".to_string(),
            notes.to_string_lossy().into_owned(),
            "--raw".into(),
            "--view".into(),
            "全部".into(),
            "-w".into(),
            "ws".into(),
            "--readonly".into(),
            "--no-color".into(),
            "--config".into(),
            here.join("conf")
                .join("my.toml")
                .to_string_lossy()
                .into_owned(),
        ]
    );
    // `--continue` で読み替える引数は、ビューを除いた同じ並び(ビューは開いてから選ぶ)。
    assert_eq!(
        r.args(false),
        r.argv()[1..3]
            .iter()
            .chain(r.argv()[5..].iter())
            .cloned()
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_seq_grows_within_a_run_and_across_restarts() {
    // [utest -> req~resume.herdr-seq~1] 起動してビューを切り替えて終える → 3つの数がこの順に大きい。起動し直す → 最初の数が前の回の最後より大きい。
    // 観点: 同じ時刻に続けて伝えても(時計が進まなくても)増える。
    let now = 1_700_000_000_000_000;
    let mut s = Seq::default();
    let a = s.next(now);
    let b = s.next(now);
    let c = s.next(now);
    assert!(a < b && b < c, "{a} {b} {c}");
    // 起動し直しは 1 ミリ秒より長くかかる。
    let mut again = Seq::default();
    let d = again.next(now + 1_000);
    assert!(d > c, "{d} > {c}");
    // 時計が戻らなければ、時刻の数より小さくならない。
    assert!(now_us() > now);
}

#[test]
fn test_herdr_accepts_boundaries() {
    // [utest -> req~resume.herdr-unsendable~1] アポストロフィ・制御文字・65 個・8 KiB を超える命令は受け付けない。ちょうど 64 個・8 KiB は受け付ける。
    assert!(herdr_accepts(&strings(&["mdgrid", "/n/tasks.base"])));
    assert!(!herdr_accepts(&strings(&["mdgrid", "/n/it's"])));
    assert!(!herdr_accepts(&strings(&["mdgrid", "/n/a\nb"])));
    assert!(!herdr_accepts(&strings(&["mdgrid", "/n/a\tb"])));
    assert!(!herdr_accepts(&[]));
    let n = |k: usize| -> Vec<String> { (0..k).map(|i| format!("a{i}")).collect() };
    assert!(herdr_accepts(&n(63)));
    assert!(herdr_accepts(&n(64)));
    assert!(!herdr_accepts(&n(65)));
    // 引数の字と区切り1つずつを合わせて数える: 2 個で 8192 バイトちょうど → 受け付ける、1 バイト多い → 受け付けない。
    let long = |k: usize| vec!["mdgrid".to_string(), "x".repeat(k)];
    assert!(herdr_accepts(&long(8192 - 7 - 1)));
    assert!(!herdr_accepts(&long(8192 - 7)));
}

#[test]
fn test_herdr_from_env_needs_all_three() {
    // [utest -> req~resume.herdr-report~1] HERDR_ENV=1 と窓の ID と道具のパスがそろった窓だけ。HERDR_ENV の無い窓 → 呼ばない。
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert_eq!(
        Herdr::from_env(env(&[
            ("HERDR_ENV", "1"),
            ("HERDR_PANE_ID", "w1:p1"),
            ("HERDR_BIN_PATH", "/bin/herdr"),
        ])),
        Some(Herdr {
            bin: PathBuf::from("/bin/herdr"),
            pane: "w1:p1".into()
        })
    );
    assert_eq!(
        Herdr::from_env(env(&[
            ("HERDR_PANE_ID", "w1:p1"),
            ("HERDR_BIN_PATH", "/bin/herdr")
        ])),
        None
    );
    assert_eq!(
        Herdr::from_env(env(&[
            ("HERDR_ENV", "0"),
            ("HERDR_PANE_ID", "w1:p1"),
            ("HERDR_BIN_PATH", "/bin/herdr"),
        ])),
        None
    );
    assert_eq!(
        Herdr::from_env(env(&[("HERDR_ENV", "1"), ("HERDR_BIN_PATH", "/bin/herdr")])),
        None
    );
    assert_eq!(
        Herdr::from_env(env(&[
            ("HERDR_ENV", "1"),
            ("HERDR_PANE_ID", "w1:p1"),
            ("HERDR_BIN_PATH", ""),
        ])),
        None
    );
}

#[test]
fn test_herdr_report_and_release_args() {
    // [utest -> req~resume.herdr-report~1] pane report-agent w1:p1 --source mdgrid --agent mdgrid --state idle --seq <数> -- mdgrid <パス> --view 進行中。
    // [utest -> req~resume.herdr-release~1] pane release-agent w1:p1 --source mdgrid --agent mdgrid --seq <数>。
    let h = Herdr {
        bin: PathBuf::from("/bin/herdr"),
        pane: "w1:p1".into(),
    };
    let argv = strings(&["mdgrid", "/n/tasks.base", "--view", "進行中"]);
    assert_eq!(
        h.report_args(5, &argv),
        strings(&[
            "pane",
            "report-agent",
            "w1:p1",
            "--source",
            "mdgrid",
            "--agent",
            "mdgrid",
            "--state",
            "idle",
            "--seq",
            "5",
            "--",
            "mdgrid",
            "/n/tasks.base",
            "--view",
            "進行中",
        ])
    );
    assert_eq!(
        h.release_args(6),
        strings(&[
            "pane",
            "release-agent",
            "w1:p1",
            "--source",
            "mdgrid",
            "--agent",
            "mdgrid",
            "--seq",
            "6",
        ])
    );
}

#[test]
fn test_last_screen_round_trip_per_folder() {
    // [utest -> req~resume.last-screen-save~1] 起動したフォルダの最後の画面として、表とビューを書いて読める。別のフォルダの最後の画面は読まない。
    let t = Tmp::new("last");
    let state = t.dir("state");
    let notes = t.dir("notes");
    let other = t.dir("other");
    let mut r = Reopen::new(
        &[notes.join("tasks.base")],
        Some("ws".into()),
        true,
        false,
        None,
    );
    r.view = Some("進行中".into());
    save_last(&state, &notes, &r).unwrap();
    assert_eq!(load_last(&state, &notes), Some(r.clone()));
    assert_eq!(load_last(&state, &other), None);
    // 書き直すと、あとの中身。
    r.view = Some("全部".into());
    save_last(&state, &notes, &r).unwrap();
    assert_eq!(
        load_last(&state, &notes).unwrap().view.as_deref(),
        Some("全部")
    );
    // ノートのフォルダには書かない。
    assert_eq!(std::fs::read_dir(&notes).unwrap().count(), 0);
}

#[test]
fn test_last_screen_broken_file_is_none() {
    // [utest -> req~resume.continue-fallback~1] 最後の画面のファイルが壊れている・無い → 読まない(引数なしの起動にする)。
    let t = Tmp::new("broken");
    let state = t.dir("state");
    let notes = t.dir("notes");
    assert_eq!(load_last(&state, &notes), None);
    let r = Reopen::new(&[notes.join("tasks.base")], None, false, false, None);
    save_last(&state, &notes, &r).unwrap();
    let dir = state.join(LAST_DIR);
    let file = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "toml"))
        .unwrap();
    std::fs::write(&file, "paths = [[[\n").unwrap();
    assert_eq!(load_last(&state, &notes), None);
}

fn pane() -> Herdr {
    Herdr {
        bin: PathBuf::from("/bin/herdr"),
        pane: "w1:p1".into(),
    }
}

/// 呼び出しの `--seq` の数。
fn seq_in(call: &[String]) -> u64 {
    let at = call.iter().position(|a| a == "--seq").unwrap();
    call[at + 1].parse().unwrap()
}

/// 呼び出しの `--seq` の数を `N` に置き換える(数の大きさは別に確かめる)。
fn without_seq(call: &[String]) -> Vec<String> {
    let at = call.iter().position(|a| a == "--seq").unwrap();
    let mut c = call.to_vec();
    c[at + 1] = "N".into();
    c
}

#[test]
fn test_herdr_title_and_clear_args() {
    // [utest -> req~resume.herdr-title-clear~1] 題を消すのは pane report-metadata w1:p1 --source mdgrid --agent mdgrid --clear-title --seq <数>。
    // 観点(resume.herdr-title。確かめは stest なので印は付けない): 題を出すのは同じ呼び出しの --title "tasks · 進行中"。
    let h = pane();
    assert_eq!(
        h.title_args(5, Some("tasks · 進行中")),
        strings(&[
            "pane",
            "report-metadata",
            "w1:p1",
            "--source",
            "mdgrid",
            "--agent",
            "mdgrid",
            "--title",
            "tasks · 進行中",
            "--seq",
            "5",
        ])
    );
    assert_eq!(
        h.title_args(6, None),
        strings(&[
            "pane",
            "report-metadata",
            "w1:p1",
            "--source",
            "mdgrid",
            "--agent",
            "mdgrid",
            "--clear-title",
            "--seq",
            "6",
        ])
    );
}

#[test]
fn test_herdr_title_sent_with_command_and_cleared_before_release() {
    // [utest -> req~resume.herdr-title-clear~1] 終える → --clear-title の呼び出しのあとに release-agent。
    // 観点(resume.herdr-title): 「進行中」で開く → 命令と一緒に題「tasks · 進行中」、「全部」へ移る → 題が「tasks · 全部」に替わる。
    let h = pane();
    let cmd = |v: &str| strings(&["mdgrid", "/n/tasks.base", "--view", v]);
    let show = |v: &str| Want::Show {
        argv: cmd(v),
        title: format!("tasks · {v}"),
    };
    let now = 1_700_000_000_000_000;
    let mut sent = Sent::default();
    let first = sent.plan(&h, &show("進行中"), now);
    let moved = sent.plan(&h, &show("全部"), now);
    let end = sent.plan(&h, &Want::Release, now);
    // 窓を初めて持つときは report-agent が先、持っている間は題が先(移った命令が届いたときには題も替わっている)。
    let plain = |calls: &[Vec<String>]| calls.iter().map(|c| without_seq(c)).collect::<Vec<_>>();
    assert_eq!(
        plain(&first),
        vec![
            without_seq(&h.report_args(0, &cmd("進行中"))),
            without_seq(&h.title_args(0, Some("tasks · 進行中"))),
        ]
    );
    assert_eq!(
        plain(&moved),
        vec![
            without_seq(&h.title_args(0, Some("tasks · 全部"))),
            without_seq(&h.report_args(0, &cmd("全部"))),
        ]
    );
    assert_eq!(
        plain(&end),
        vec![
            without_seq(&h.title_args(0, None)),
            without_seq(&h.release_args(0)),
        ]
    );
    // resume.herdr-seq: 題の呼び出しも含めて、伝えた順に大きくなる。
    let seqs: Vec<u64> = first
        .iter()
        .chain(&moved)
        .chain(&end)
        .map(|c| seq_in(c))
        .collect();
    assert!(seqs.windows(2).all(|w| w[0] < w[1]), "{seqs:?}");
}

#[test]
fn test_herdr_title_unsendable_is_not_sent() {
    // 観点(resume.herdr-unsendable の決まりを題にも当てる): 題に `'` か制御文字があれば題を伝えず、前の題があれば
    // 消す(古い表の名前を残さない)。命令を受け付けられないときは、題を消してから窓を返し、新しい題は伝えない。
    // 題を一度も伝えていなければ、終えるときも消さない。
    let h = pane();
    let now = 1_700_000_000_000_000;
    let kinds = |calls: &[Vec<String>]| -> Vec<String> {
        calls
            .iter()
            .map(|c| {
                if c.contains(&"--clear-title".to_string()) {
                    "clear".to_string()
                } else if let Some(at) = c.iter().position(|a| a == "--title") {
                    format!("title {}", c[at + 1])
                } else {
                    c[1].clone()
                }
            })
            .collect()
    };
    let show = |path: &str, title: &str| Want::Show {
        argv: strings(&["mdgrid", path, "--raw"]),
        title: title.to_string(),
    };
    let mut sent = Sent::default();
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/notes", "notes"), now)),
        ["report-agent", "title notes"]
    );
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/bob", "Bob's"), now)),
        ["clear", "report-agent"]
    );
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/tab", "a\tb"), now)),
        ["report-agent"]
    );
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/notes", "notes"), now)),
        ["title notes", "report-agent"]
    );
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/it's", "it's"), now)),
        ["clear", "release-agent"]
    );
    // 窓を返したあとは、また report-agent から。
    assert_eq!(
        kinds(&sent.plan(&h, &show("/n/notes", "notes"), now)),
        ["report-agent", "title notes"]
    );
    let mut quiet = Sent::default();
    assert_eq!(
        kinds(&quiet.plan(&h, &show("/n/it's", "it's"), now)),
        ["release-agent"]
    );
    assert_eq!(
        kinds(&quiet.plan(&h, &Want::Release, now)),
        ["release-agent"]
    );
}

#[test]
fn test_title_form_workspace_base_and_folder() {
    // [utest -> req~resume.herdr-title-form~1] ワークスペースの表 `name = "タスク"` の notes/tasks → 「タスク · 進行中」。ビューの無いフォルダ notes → 「notes」。ワークスペースに無い `.base` は拡張子を除いたファイル名で、ビューのタブを出さない表(ビューが1つ)は表の名前だけ。
    let t = Tmp::new("title");
    let notes = t.dir("notes");
    let base = notes.join("tasks.base");
    std::fs::write(&base, "views: []\n").unwrap();
    let ws = |name: &str, path: &Path, view: Option<&str>| WsTable {
        name: name.into(),
        path: path.to_path_buf(),
        view: view.map(str::to_string),
        ..WsTable::default()
    };
    let tables = vec![ws("タスク", &base, None)];
    let name = table_name(std::slice::from_ref(&base), &tables, Some("進行中"));
    assert_eq!(title(&name, Some("進行中")), "タスク · 進行中");
    let name = table_name(std::slice::from_ref(&notes), &[], None);
    assert_eq!(title(&name, None), "notes");
    let name = table_name(std::slice::from_ref(&base), &[], Some("進行中"));
    assert_eq!(title(&name, Some("進行中")), "tasks · 進行中");
    assert_eq!(title(&name, None), "tasks");
    // 観点: 同じ表がワークスペースに2つ(ビュー違い)あれば、今のビューを持つ方の名前。無ければ先の方。
    let tables = vec![
        ws("タスク", &base, Some("進行中")),
        ws("全部のタスク", &base, Some("全部")),
    ];
    assert_eq!(
        table_name(std::slice::from_ref(&base), &tables, Some("全部")),
        "全部のタスク"
    );
    assert_eq!(
        table_name(std::slice::from_ref(&base), &tables, Some("ほか")),
        "タスク"
    );
}
