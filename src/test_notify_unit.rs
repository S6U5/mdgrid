//! 新しく当たった行を知らせる(specs/requirements/notify.md)の核の試験: 設定の読み・記録と比べて鳴らす行・
//! まとめ・届かないとき・記録の錠と壊れ・端末の合図・OS の通知・自分のコマンド・制御文字。
//! 期待値は要求の受け入れの例から取った。表を開いて行を選ぶのは画面の側(src/ui/test_notify.rs)。

use crate::config;
use crate::notice::Scope;
use crate::notify::{self, Check, Hit, Item, Note, Os, Report, Term, To};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// 試験ごとの一時のフォルダ(記録の置き場)。
struct Dir(PathBuf);

impl Dir {
    fn new(name: &str) -> Dir {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "mdgrid-notify-{name}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Dir(p)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn item(to: &[To]) -> Item {
    Item {
        cond: "remind".into(),
        scope: Scope::Workspace,
        title: None,
        text: None,
        to: to.to_vec(),
        group_over: notify::GROUP_OVER,
        group_text: None,
        command: Vec::new(),
    }
}

fn hit(id: &str) -> Hit {
    Hit {
        id: id.into(),
        path: PathBuf::from(id),
        title: "リマインダー".into(),
        text: id.into(),
    }
}

fn check(item: &Item, tables: &[(&str, Result<&[&str], &str>)]) -> Check {
    Check {
        item: item.clone(),
        label: "リマインダー".into(),
        tables: tables
            .iter()
            .map(|(t, r)| {
                (
                    (*t).to_string(),
                    r.map(|ids| ids.iter().map(|i| hit(i)).collect())
                        .map_err(str::to_string),
                )
            })
            .collect(),
    }
}

/// 出す先ごとに受けた知らせを覚え、`fail` の出す先は失敗にする。
#[derive(Default)]
struct Sink {
    got: Vec<(To, Note)>,
    fail: Vec<To>,
}

impl Sink {
    fn run(&mut self, dir: &Path, checks: &[Check]) -> Report {
        let fail = self.fail.clone();
        let got = &mut self.got;
        notify::run(dir, checks, &mut |to, _item, note| {
            got.push((to, note.clone()));
            if fail.contains(&to) {
                Err("落ちた".into())
            } else {
                Ok(())
            }
        })
    }

    fn texts(&self) -> Vec<String> {
        self.got.iter().map(|(_, n)| n.text.clone()).collect()
    }
}

fn parse(text: &str) -> (Vec<Item>, Vec<String>) {
    let (c, w) = config::parse(text).unwrap();
    (c.notify, w)
}

const REMIND: &str = "[conditions.remind]\nexpr = 'remind <= now()'\n";

#[test]
fn test_notify_item_defaults() {
    // [utest -> req~notify.item~1] scope を書かなければ範囲の表の全部。[utest -> req~notify.to~1] to を書かなければ OS の通知。
    let (items, w) = parse(&format!("{REMIND}[[notify]]\ncond = \"remind\"\n"));
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(items, [item(&[To::Desktop])]);
}

#[test]
fn test_notify_item_unknown_condition_is_warned() {
    // [utest -> req~notify.item~1] [conditions] に無い名前は警告して使わない。
    let (items, w) = parse(&format!("{REMIND}[[notify]]\ncond = \"nothing\"\n"));
    assert!(items.is_empty());
    assert!(w.iter().any(|x| x.contains("notify[1].cond")), "{w:?}");
}

#[test]
fn test_notify_item_same_cond_twice_uses_first() {
    // [utest -> req~notify.item~1] 同じ cond の2つ目は警告して使わず、1つ目を使う。
    let conf = format!(
        "{REMIND}[[notify]]\ncond = \"remind\"\n[[notify]]\ncond = \"remind\"\nto = [\"terminal\"]\n"
    );
    let (items, w) = parse(&conf);
    assert_eq!(items, [item(&[To::Desktop])]);
    assert!(w.iter().any(|x| x.contains("notify[2].cond")), "{w:?}");
}

#[test]
fn test_notify_to_unknown_value_is_warned() {
    // [utest -> req~notify.to~1] 知らない値は警告し、その値だけを使わない。
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\nto = [\"desktop\", \"phone\"]\n");
    let (items, w) = parse(&conf);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].to, [To::Desktop]);
    assert!(
        w.iter()
            .any(|x| x.contains("notify[1].to") && x.contains("phone")),
        "{w:?}"
    );
}

#[test]
fn test_notify_to_only_the_chosen_destinations() {
    // [utest -> req~notify.to~1] to = ["terminal", "command"] は、端末とコマンドに1回ずつ出し、OS の通知には出さない。
    let d = Dir::new("to");
    let it = item(&[To::Terminal, To::Command]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    let tos: Vec<To> = s.got.iter().map(|(t, _)| *t).collect();
    assert_eq!(tos, [To::Terminal, To::Command]);
}

#[test]
fn test_notify_command_without_command_is_warned() {
    // [utest -> req~notify.command~1] to に command があり command が無ければ、警告してその出す先を使わない。
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\nto = [\"desktop\", \"command\"]\n");
    let (items, w) = parse(&conf);
    assert_eq!(items[0].to, [To::Desktop]);
    assert!(w.iter().any(|x| x.contains("notify[1].command")), "{w:?}");
}

#[test]
fn test_notify_new_match_rings_once() {
    // [utest -> req~notify.new-match~1] 当たり始めの1回だけ鳴り、当たり続けても鳴らない。
    let d = Dir::new("once");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    for _ in 0..4 {
        s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    }
    assert_eq!(s.texts(), ["a"]);
}

#[test]
fn test_notify_new_match_again_after_leaving() {
    // [utest -> req~notify.new-match~1] 外れたあとにまた当たれば、もう1回鳴る。
    let d = Dir::new("again");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert_eq!(s.texts(), ["a", "a"]);
}

#[test]
fn test_notify_same_row_with_new_text_is_not_new() {
    // [utest -> req~notify.new-match~1] 同じファイル(同じ見分け)なら、題や文が変わっても鳴らさない。
    let d = Dir::new("same");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    let mut c = check(&it, &[("t", Ok(&["a"]))]);
    if let Ok(hits) = &mut c.tables[0].1 {
        hits[0].text = "直した".into();
    }
    s.run(&d.0, &[c]);
    assert_eq!(s.texts(), ["a"]);
}

#[test]
fn test_notify_first_check_is_silent() {
    // [utest -> req~notify.first-silent~1] 初めての確かめは5つ当たっていても鳴らさず、6つ目だけを鳴らす。
    let d = Dir::new("first");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(
        &d.0,
        &[check(&it, &[("t", Ok(&["a", "b", "c", "d", "e"]))])],
    );
    assert!(s.got.is_empty());
    s.run(
        &d.0,
        &[check(&it, &[("t", Ok(&["a", "b", "c", "d", "e", "f"]))])],
    );
    assert_eq!(s.texts(), ["f"]);
}

#[test]
fn test_notify_added_table_is_silent() {
    // [utest -> req~notify.first-silent~1] 前から確かめている範囲に足した表の、当たっている2行は鳴らない。
    let d = Dir::new("added");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    s.run(
        &d.0,
        &[check(&it, &[("t", Ok(&["a"])), ("u", Ok(&["x", "y"]))])],
    );
    assert!(s.got.is_empty());
}

#[test]
fn test_notify_group_three_rings_each() {
    // [utest -> req~notify.group~1] 新しく3行なら1行ずつ3件。
    let d = Dir::new("g3");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a", "b", "c"]))])]);
    assert_eq!(s.texts(), ["a", "b", "c"]);
}

#[test]
fn test_notify_group_four_rings_once() {
    // [utest -> req~notify.group~1] 新しく4行なら、題が条件の見せる名前、文が「新しく 4 件」の1件で、4行とも覚える。
    let d = Dir::new("g4");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[])), ("u", Ok(&[]))])]);
    s.run(
        &d.0,
        &[check(&it, &[("t", Ok(&["a", "b"])), ("u", Ok(&["c"]))])],
    );
    assert_eq!(s.got.len(), 3);
    s.got.clear();
    s.run(
        &d.0,
        &[check(
            &it,
            &[
                ("t", Ok(&["a", "b", "d", "e"])),
                ("u", Ok(&["c", "f", "g"])),
            ],
        )],
    );
    assert_eq!(s.got.len(), 1);
    assert_eq!(s.got[0].1.title, "リマインダー");
    assert_eq!(s.got[0].1.text, "新しく 4 件");
    s.run(
        &d.0,
        &[check(
            &it,
            &[
                ("t", Ok(&["a", "b", "d", "e"])),
                ("u", Ok(&["c", "f", "g"])),
            ],
        )],
    );
    assert_eq!(s.got.len(), 1);
}

#[test]
fn test_notify_group_over_zero_always_groups() {
    // [utest -> req~notify.group~1] group_over = 0 は1行でも「新しく 1 件」にまとめる。group_text の {count} も埋める。
    let d = Dir::new("g0");
    let mut it = item(&[To::Desktop]);
    it.group_over = 0;
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert_eq!(s.texts(), ["新しく 1 件"]);
    it.group_text = Some("{count} 件の知らせ".into());
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a", "b"]))])]);
    assert_eq!(s.texts(), ["新しく 1 件", "1 件の知らせ"]);
}

#[test]
fn test_notify_fail_is_not_remembered() {
    // [utest -> req~notify.fail~1] どの出す先にも届かなければ理由を出し、覚えずに次の確かめでまた鳴らす。
    let d = Dir::new("fail");
    let it = item(&[To::Command]);
    let mut s = Sink {
        fail: vec![To::Command],
        ..Sink::default()
    };
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    let r = s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert_eq!(r.failures.len(), 1, "{r:?}");
    assert!(r.failures[0].contains("command"), "{r:?}");
    assert!(r.rung.is_empty());
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert_eq!(s.texts(), ["a", "a"]);
}

#[test]
fn test_notify_fail_on_one_destination_still_remembers() {
    // [utest -> req~notify.fail~1] OS の通知に届いてコマンドだけが落ちたら、理由は出すが、その行は覚える。
    let d = Dir::new("partial");
    let it = item(&[To::Desktop, To::Command]);
    let mut s = Sink {
        fail: vec![To::Command],
        ..Sink::default()
    };
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    let r = s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert_eq!(r.failures.len(), 1);
    assert_eq!(r.rung.len(), 1);
    s.got.clear();
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"]))])]);
    assert!(s.got.is_empty());
}

#[test]
fn test_notify_unreadable_table_keeps_its_record() {
    // [utest -> req~notify.fail~1] 読めない表は理由を出し、ほかの表は確かめて鳴らし、読めない表の記録は消さない。
    let d = Dir::new("gone");
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&["a"])), ("u", Ok(&["b"]))])]);
    let r = s.run(
        &d.0,
        &[check(
            &it,
            &[("t", Err("t がありません")), ("u", Ok(&["b", "c"]))],
        )],
    );
    assert_eq!(s.texts(), ["c"]);
    assert!(
        r.failures.iter().any(|f| f.contains("t がありません")),
        "{r:?}"
    );
    s.run(
        &d.0,
        &[check(&it, &[("t", Ok(&["a"])), ("u", Ok(&["b", "c"]))])],
    );
    assert_eq!(s.texts(), ["c"]);
}

#[cfg(unix)]
#[test]
fn test_notify_spawn_times_out() {
    // [utest -> req~notify.fail~1] 打ち切りの時間までに終わらないプログラムは止めて、理由を返す。
    // unix だけ: 待つプログラムに sleep を使う(Windows の CI のジョブでは飛ぶ。macOS・Ubuntu のジョブで走る)。
    let sp = notify::Spawn {
        program: "sleep".into(),
        args: vec!["5".into()],
        ..notify::Spawn::default()
    };
    let start = Instant::now();
    let r = notify::run_spawn(&sp, Duration::from_millis(300));
    assert!(r.is_err(), "{r:?}");
    assert!(start.elapsed() < Duration::from_secs(3));
}

#[test]
fn test_notify_missing_program_fails() {
    // [utest -> req~notify.desktop~1] 通知の仕組みのプログラムが無ければ、届けられなかったとして理由を返す。
    let sp = notify::Spawn {
        program: "mdgrid-no-such-program".into(),
        ..notify::Spawn::default()
    };
    let r = notify::run_spawn(&sp, Duration::from_secs(2));
    assert!(r.is_err(), "{r:?}");
}

#[test]
fn test_notify_state_lock_rings_once() {
    // [utest -> req~notify.state~1] 2つの確かめを同時に走らせても、新しく当たった行は合わせて1回だけ鳴る。
    let d = Dir::new("lock");
    let it = item(&[To::Desktop]);
    notify::run(
        &d.0,
        &[check(&it, &[("t", Ok(&[]))])],
        &mut |_, _, _| Ok(()),
    );
    let rung = std::sync::Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let (dir, it, rung) = (d.0.clone(), it.clone(), rung.clone());
            std::thread::spawn(move || {
                notify::run(&dir, &[check(&it, &[("t", Ok(&["a"]))])], &mut |_, _, _| {
                    std::thread::sleep(Duration::from_millis(50));
                    rung.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                });
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(rung.load(Ordering::SeqCst), 1);
}

#[test]
fn test_notify_no_items_writes_nothing() {
    // [utest -> req~notify.opt-in~1] 項目が無ければ、記録のファイルを作らない。
    let d = Dir::new("none");
    let r = notify::run(&d.0, &[], &mut |_, _, _| Ok(()));
    assert_eq!(r, Report::default());
    assert!(!d.0.join(notify::STATE_FILE).exists());
}

#[test]
fn test_notify_broken_state_is_rebuilt_silently() {
    // [utest -> req~notify.state-broken~1] 壊れた記録は、当たっている3行を鳴らさずに覚え直して警告を1回出し、
    // 次は4つ目だけを鳴らす。
    let d = Dir::new("broken");
    std::fs::write(d.0.join(notify::STATE_FILE), "{ 壊れた").unwrap();
    let it = item(&[To::Desktop]);
    let mut s = Sink::default();
    let r = s.run(&d.0, &[check(&it, &[("t", Ok(&["a", "b", "c"]))])]);
    assert!(s.got.is_empty());
    assert_eq!(r.warnings.len(), 1, "{r:?}");
    let r = s.run(&d.0, &[check(&it, &[("t", Ok(&["a", "b", "c", "d"]))])]);
    assert!(r.warnings.is_empty());
    assert_eq!(s.texts(), ["d"]);
}

fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |k| {
        pairs
            .iter()
            .find(|(n, _)| *n == k)
            .map(|(_, v)| (*v).to_string())
    }
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

#[test]
fn test_notify_terminal_kitty_uses_osc99_only() {
    // [utest -> req~notify.terminal~1] kitty の中では OSC 99 だけを送り、OSC 9 と OSC 777 は送らない。
    let (t, tmux) = notify::detect_term(&env(&[("TERM", "xterm-kitty")]));
    assert_eq!((t, tmux), (Term::Osc99, false));
    let b = notify::terminal_bytes(t, tmux, "思い出す", "請求書を送る");
    assert!(count(&b, b"\x1b]99;") >= 1, "{b:?}");
    assert_eq!(count(&b, b"\x1b]9;"), 0);
    assert_eq!(count(&b, b"\x1b]777;"), 0);
}

#[test]
fn test_notify_terminal_tmux_passthrough() {
    // [utest -> req~notify.terminal~1] tmux の中で外が Ghostty なら、OSC 777 を tmux の通り抜けの DCS で包む。
    let e = env(&[
        ("TMUX", "/tmp/tmux-501/default,1,0"),
        ("TERM_PROGRAM", "tmux"),
        (
            "GHOSTTY_RESOURCES_DIR",
            "/Applications/Ghostty.app/Contents/Resources/ghostty",
        ),
    ]);
    let (t, tmux) = notify::detect_term(&e);
    assert_eq!((t, tmux), (Term::Osc777, true));
    let b = notify::terminal_bytes(t, tmux, "思い出す", "請求書を送る");
    assert!(b.starts_with(b"\x1bPtmux;"), "{b:?}");
    assert!(b.ends_with(b"\x1b\\"), "{b:?}");
    assert_eq!(count(&b, b"\x1b\x1b]777;notify;"), 1, "{b:?}");
}

#[test]
fn test_notify_terminal_unknown_sends_bel() {
    // [utest -> req~notify.terminal~1] 見分けられない端末には BEL を1つだけ送り、OSC は送らない。
    let (t, tmux) = notify::detect_term(&env(&[("TERM", "xterm-256color")]));
    assert_eq!((t, tmux), (Term::Bel, false));
    assert_eq!(
        notify::terminal_bytes(t, tmux, "思い出す", "請求書"),
        b"\x07"
    );
}

#[test]
fn test_notify_desktop_macos() {
    // [utest -> req~notify.desktop~1] macOS では macOS の通知の仕組みに、題と文が1回渡る。
    let os = notify::detect_os("macos", &env(&[]), None);
    assert_eq!(os, Os::Mac);
    let sp = notify::desktop_spawn(os, "思い出す", "請求書を送る", None);
    assert_eq!(sp.program, "osascript");
    assert_eq!(
        sp.args.iter().filter(|a| *a == "思い出す").count(),
        1,
        "{sp:?}"
    );
    assert_eq!(sp.args.iter().filter(|a| *a == "請求書を送る").count(), 1);
}

#[test]
fn test_notify_desktop_wsl_goes_to_windows() {
    // [utest -> req~notify.desktop~1] WSL の中(WSL_DISTRO_NAME か、カーネルの版の名前の microsoft)では、Linux の
    // 仕組みを使わず、Windows の通知の仕組みに題と文を渡す。
    assert_eq!(
        notify::detect_os("linux", &env(&[("WSL_DISTRO_NAME", "Ubuntu")]), None),
        Os::Wsl
    );
    let os = notify::detect_os(
        "linux",
        &env(&[]),
        Some("5.15.167.4-microsoft-standard-WSL2"),
    );
    assert_eq!(os, Os::Wsl);
    let sp = notify::desktop_spawn(os, "思い出す", "請求書を送る", Some("WT_SESSION"));
    assert_eq!(sp.program, "powershell.exe");
    let get = |k: &str| sp.env.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    assert_eq!(get("MDGRID_NOTIFY_TITLE"), Some("思い出す"));
    assert_eq!(get("MDGRID_NOTIFY_TEXT"), Some("請求書を送る"));
    let wslenv = get("WSLENV").unwrap_or_default();
    assert!(wslenv.starts_with("WT_SESSION:"), "{wslenv}");
    assert!(wslenv.contains("MDGRID_NOTIFY_TITLE"), "{wslenv}");
    assert!(wslenv.contains("MDGRID_NOTIFY_TEXT"), "{wslenv}");
}

#[test]
fn test_notify_desktop_windows_and_linux() {
    // [utest -> req~notify.desktop~1] Windows は Windows の通知の仕組み、WSL でない Linux は Linux の仕組み。
    let os = notify::detect_os("windows", &env(&[]), None);
    assert_eq!(os, Os::Windows);
    let sp = notify::desktop_spawn(os, "思い出す", "請求書を送る", None);
    assert_eq!(sp.program, "powershell.exe");
    assert!(sp
        .env
        .iter()
        .any(|(k, v)| k == "MDGRID_NOTIFY_TEXT" && v == "請求書を送る"));
    let os = notify::detect_os("linux", &env(&[]), Some("6.8.0-45-generic"));
    assert_eq!(os, Os::Linux);
    let sp = notify::desktop_spawn(os, "思い出す", "請求書を送る", None);
    assert_eq!(sp.program, "notify-send");
    assert!(sp
        .args
        .ends_with(&["思い出す".to_string(), "請求書を送る".to_string()]));
}

#[test]
fn test_notify_command_placeholders() {
    // [utest -> req~notify.command~1] 引数の {title}・{text}・{path} を、題・文・行のファイルのパスに置き換える。
    let note = Note {
        title: "リマインダー".into(),
        text: "請求書を送る".into(),
        path: PathBuf::from("notes/請求書.md"),
    };
    let argv: Vec<String> = ["notify-me", "{title}", "{text}", "--file={path}", "{other}"]
        .map(String::from)
        .to_vec();
    let sp = notify::command_spawn(&argv, &note).unwrap();
    assert_eq!(sp.program, "notify-me");
    assert_eq!(
        sp.args,
        [
            "リマインダー",
            "請求書を送る",
            "--file=notes/請求書.md",
            "{other}"
        ]
    );
    assert!(notify::command_spawn(&[], &note).is_none());
}

#[cfg(unix)]
#[test]
fn test_notify_command_runs_without_shell() {
    // [utest -> req~notify.command~1] 値の `$(touch made)` は文字のまま渡り、made のファイルはできない。
    // unix だけ: 受けた引数をファイルに書くプログラムに sh を使う(Windows のジョブでは飛ぶ。macOS・Ubuntu で走る)。
    let d = Dir::new("cmd");
    let out = d.0.join("out.txt");
    let made = d.0.join("made");
    let note = Note {
        title: "リマインダー".into(),
        text: format!("$(touch {})", made.display()),
        path: PathBuf::new(),
    };
    let argv: Vec<String> = vec![
        "sh".into(),
        "-c".into(),
        format!("printf '%s\\n' \"$1\" \"$2\" > '{}'", out.display()),
        "sh".into(),
        "{title}".into(),
        "{text}".into(),
    ];
    let sp = notify::command_spawn(&argv, &note).unwrap();
    notify::run_spawn(&sp, Duration::from_secs(5)).unwrap();
    let got = crate::textfile::read_text(&out).unwrap();
    assert_eq!(got, format!("リマインダー\n$(touch {})\n", made.display()));
    assert!(!made.exists());
}

#[test]
fn test_notify_sanitize_terminal_escape() {
    // [utest -> req~notify.sanitize~1] 題の ESC・`]52;c;aGk=`・BEL は合図に混ざらず、ESC は合図の枠の分だけ。
    let d = Dir::new("esc");
    let it = item(&[To::Terminal]);
    let mut c = check(&it, &[("t", Ok(&["a"]))]);
    if let Ok(hits) = &mut c.tables[0].1 {
        hits[0].title = "\x1b]52;c;aGk=\x07".into();
    }
    notify::run(
        &d.0,
        &[check(&it, &[("t", Ok(&[]))])],
        &mut |_, _, _| Ok(()),
    );
    let mut bytes = Vec::new();
    notify::run(&d.0, &[c], &mut |_, _, n| {
        bytes = notify::terminal_bytes(Term::Osc777, false, &n.title, &n.text);
        Ok(())
    });
    assert_eq!(count(&bytes, b"\x1b"), 1, "{bytes:?}");
    assert_eq!(count(&bytes, b"\x07"), 1, "{bytes:?}");
    assert_eq!(count(&bytes, b"\x1b]52;"), 0, "{bytes:?}");
}

#[test]
fn test_notify_sanitize_newline() {
    // [utest -> req~notify.sanitize~1] 文の改行は空白にする。
    let d = Dir::new("nl");
    let it = item(&[To::Desktop]);
    let mut c = check(&it, &[("t", Ok(&["a"]))]);
    if let Ok(hits) = &mut c.tables[0].1 {
        hits[0].text = "請求書\nを送る".into();
    }
    let mut s = Sink::default();
    s.run(&d.0, &[check(&it, &[("t", Ok(&[]))])]);
    s.run(&d.0, &[c]);
    assert_eq!(s.texts(), ["請求書 を送る"]);
}

#[test]
fn test_notify_fill_placeholders() {
    // [utest -> req~notify.text~1] `{名前}` を値に置き換え、値の無い名前と閉じない `{` はそのまま。
    let v = |k: &str| match k {
        "title" => Some("請求書を送る".to_string()),
        "owner" => Some(String::new()),
        _ => None,
    };
    assert_eq!(notify::fill("{owner}: {title}", &v), ": 請求書を送る");
    assert_eq!(notify::fill("{nope} {title", &v), "{nope} {title");
}
