//! `.base` のビューの `tab_break`(NV-32)の受け入れ試験: 切れ目のタブの前でタブの行を改め、タブを全部、
//! 検索の欄の行の下に並べる。記録: specs/_changes/2026-10-19-tab-break.md(タスク 1・2)。
//!
//! 内部の状態のフィールドや新しい関数には頼らず、`.base` の文・ワークスペースの印の文・設定の文字・
//! 端末の大きさ・クリックとキー・描いた画面の文字・今のビューの名前で確かめる。置いた仮定:
//! - ビューの帯の真ん中の行は、検索の欄「\ 検索」のある行(NV-23)。足した行はその行より下にある。
//! - タブの字はビューの名前を含む(今のタブは「[名前]」、ほかは「 名前 」)。
//! - 表の1行目の高さは、ノートの title「会議のメモ」の字のある行で見る。
//! - 隠したタブ・タブの順は、設定の置き場(tmp/config)の views.toml に残した好み(NV-26)で組む。

use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use super::*;
use mdgrid::views::{self, TabPrefs};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;
use std::collections::BTreeSet;
use std::path::PathBuf;

// ---- 材料 ----

/// 4つのタブ A・B・C・D(要件の例の名前に、ほかの字と取り違えない頭を付けたもの)。
const A: &str = "タブA";
const B: &str = "タブB";
const C: &str = "タブC";
const D: &str = "タブD";
const ABCD: &[&str] = &[A, B, C, D];

/// ノート。どのビューも絞り込まないので、どのビューでも表に出る。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 会議のメモ\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: 買い物\nstatus: doing\n---\n"),
];

/// ビューの名前と、`tab_break: true` を書くかの並びから `.base` の文を作る。
fn base_text(views: &[(&str, bool)]) -> String {
    let mut s = String::from("views:\n");
    for (name, brk) in views {
        s.push_str(&format!(
            "  - type: table\n    name: {name}\n    order: [title, status]\n"
        ));
        if *brk {
            s.push_str("    tab_break: true\n");
        }
    }
    s
}

/// A・B・C・D で、`brk` に挙げた名前のビューに `tab_break: true` を書いた並び。
fn abcd(brk: &[&str]) -> Vec<(&'static str, bool)> {
    ABCD.iter().map(|v| (*v, brk.contains(v))).collect()
}

/// 開く前に置くもの。
#[derive(Default)]
struct Opts<'a> {
    /// 設定(config.toml)の文字。
    toml: &'a str,
    /// views.toml に残すタブの好み(隠す・順)。
    prefs: Option<TabPrefs>,
}

/// `.base`(tasks.base)の文 `base` を、幅 `w`・高さ 24 で開く。
fn boot(name: &str, base: &str, w: u16, o: Opts) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let p = tmp.notes().join("tasks.base");
    std::fs::write(&p, base).unwrap();
    let a = start(&tmp, p, w, o);
    (tmp, a)
}

/// ワークスペースの印(`.mdgrid/workspace.toml`)の `[table.base]` に書いたビューで、台帳を幅 `w` で開く(WS-9)。
fn boot_marker(name: &str, marker: &str, w: u16) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let p = tmp.notes().join("台帳.base");
    let a = start(&tmp, p, w, Opts::default());
    (tmp, a)
}

fn start(tmp: &Tmp, p: PathBuf, w: u16, o: Opts) -> App {
    let (config, warns) = mdgrid::config::parse(o.toml).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    if let Some(prefs) = &o.prefs {
        views::save_tab_prefs(&cdir, &p, prefs).unwrap();
    }
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(w, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: p.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

const NO_WRAP: &str = "[display]\ntab_wrap = false\n";

/// 今の端末の大きさ(`a.size`)で描いたバッファ。
fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

fn scr(a: &App) -> String {
    text(&buf(a))
}

/// その文字を含む最初の行の番号。
fn row_of(s: &str, t: &str) -> usize {
    s.lines()
        .position(|l| l.contains(t))
        .unwrap_or_else(|| panic!("「{t}」が無い:\n{s}"))
}

/// ビューの帯の真ん中の行(検索の欄のある行)。
fn search_row(s: &str) -> usize {
    row_of(s, "\\ 検索")
}

/// 検索の欄の行から表の1行目の前までの行の番号(帯の真ん中の行と足した行。ヘッダーのビューの名前
/// (SR-20)や表の字とタブを取り違えないように)。
fn bar_rows(s: &str) -> std::ops::Range<usize> {
    search_row(s)..row_of(s, "会議のメモ")
}

/// 帯の真ん中の行と足した行で、タブの名前 `name` の字のある行(タブの字の形で探す)。
fn tab_row(s: &str, name: &str) -> Option<usize> {
    let shapes = [format!("[{name}]"), format!(" {name} ")];
    let lines: Vec<&str> = s.lines().collect();
    bar_rows(s).find(|&r| {
        let l = format!(" {} ", lines[r]);
        shapes.iter().any(|t| l.contains(t.as_str()))
    })
}

/// 表の1行目より上で、`names` のどれかのタブのある行の番号(上から)。
fn tab_lines(s: &str, names: &[&str]) -> Vec<usize> {
    names
        .iter()
        .filter_map(|v| tab_row(s, v))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// 行 y で文字 `t` の始まる桁(左から最初のもの)。
fn col_in(a: &App, y: u16, t: &str) -> Option<u16> {
    let b = buf(a);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        cs.iter().all(|c| {
            let hit = xx < b.area.width && b[(xx, y)].symbol() == c;
            xx += width::width(c).max(1) as u16;
            hit
        })
    })
}

fn current(a: &App) -> String {
    a.view_names()[a.view_index()].clone()
}

/// A・B が下の1行目、C・D が下の2行目(真ん中の行のすぐ下から)にあることを確かめる。
fn assert_ab_cd(s: &str) {
    let mid = search_row(s);
    for v in ABCD {
        assert_ne!(tab_row(s, v), Some(mid), "検索の欄の行にタブ「{v}」:\n{s}");
    }
    assert_eq!(tab_row(s, A), Some(mid + 1), "A は下の1行目:\n{s}");
    assert_eq!(tab_row(s, B), Some(mid + 1), "B は下の1行目:\n{s}");
    assert_eq!(tab_row(s, C), Some(mid + 2), "C は下の2行目:\n{s}");
    assert_eq!(tab_row(s, D), Some(mid + 2), "D は下の2行目:\n{s}");
    assert!(
        s.lines()
            .nth(mid + 1)
            .unwrap()
            .starts_with(&format!("[{A}]")),
        "下の1行目は左の端から:\n{s}"
    );
    assert!(
        s.lines().nth(mid + 2).unwrap().trim_start().starts_with(C),
        "下の2行目は C から:\n{s}"
    );
}

// ---- 試験 ----

#[test]
fn test_nv_32_break_two_rows_wide_no_wrap() {
    // [NV-32] C に tab_break、幅 200・tab_wrap = false → 検索の欄の行にタブが無く、すぐ下の行に A と B、
    // その下の行に C と D。表は、切れ目の無いとき(タブが真ん中の行)よりタブの行の2行ぶん下がる。
    // 既定の設定(tab_wrap を書かない)でも同じ。
    for (name, toml) in [("nv32_two_rows", NO_WRAP), ("nv32_two_rows_default", "")] {
        let (_t, a) = boot(
            name,
            &base_text(&abcd(&[C])),
            200,
            Opts {
                toml,
                ..Default::default()
            },
        );
        let s = scr(&a);
        assert_ab_cd(&s);

        let (_t2, b) = boot(
            &format!("{name}_ref"),
            &base_text(&abcd(&[])),
            200,
            Opts {
                toml,
                ..Default::default()
            },
        );
        let s2 = scr(&b);
        let mid2 = search_row(&s2);
        for v in ABCD {
            assert_eq!(
                tab_row(&s2, v),
                Some(mid2),
                "切れ目の無いときは真ん中の行:\n{s2}"
            );
        }
        assert_eq!(
            search_row(&s),
            mid2,
            "検索の欄の行は動かない:\n{s}\n----\n{s2}"
        );
        assert_eq!(
            row_of(&s, "会議のメモ"),
            row_of(&s2, "会議のメモ") + 2,
            "表はタブの行の2行ぶん下がる:\n{s}\n----\n{s2}"
        );
    }
}

#[test]
fn test_nv_32_click_lower_row_selects_view() {
    // [NV-32] 下の行の D をクリック → D のビュー。上の行の B をクリック → B のビュー。
    let (_t, mut a) = boot(
        "nv32_click",
        &base_text(&abcd(&[C])),
        200,
        Opts {
            toml: NO_WRAP,
            ..Default::default()
        },
    );
    assert_eq!(current(&a), A);
    assert_ab_cd(&scr(&a));
    for target in [D, B] {
        let s = scr(&a);
        let y =
            tab_row(&s, target).unwrap_or_else(|| panic!("タブ「{target}」が無い:\n{s}")) as u16;
        let x =
            col_in(&a, y, target).unwrap_or_else(|| panic!("行 {y} に「{target}」が無い:\n{s}"));
        a.click(x, y);
        assert_eq!(current(&a), target, "{}", scr(&a));
    }
}

#[test]
fn test_nv_32_gap_only_before_tab_rows() {
    // [NV-32] tab_wrap_gap = true → 検索の欄の行とタブの行の間に空き行が1つで、2つのタブの行の間には
    // 空き行が無い。表は空き行の無いときよりちょうど1行下がる。
    let base = base_text(&abcd(&[C]));
    let (_t, a) = boot(
        "nv32_gap",
        &base,
        200,
        Opts {
            toml: "[display]\ntab_wrap_gap = true\n",
            ..Default::default()
        },
    );
    let (_t2, b) = boot("nv32_gap_ref", &base, 200, Opts::default());
    let s = scr(&a);
    let s2 = scr(&b);
    let mid = search_row(&s);
    assert_eq!(
        s.lines().nth(mid + 1).unwrap().trim(),
        "",
        "空き行が1つ:\n{s}"
    );
    assert_eq!(tab_row(&s, A), Some(mid + 2), "{s}");
    assert_eq!(tab_row(&s, B), Some(mid + 2), "{s}");
    assert_eq!(
        tab_row(&s, C),
        Some(mid + 3),
        "タブの行の間に空き行は無い:\n{s}"
    );
    assert_eq!(tab_row(&s, D), Some(mid + 3), "{s}");
    assert_eq!(
        row_of(&s, "会議のメモ"),
        row_of(&s2, "会議のメモ") + 1,
        "{s}\n----\n{s2}"
    );
}

#[test]
fn test_nv_32_narrow_wraps_within_row() {
    // [NV-32] 切れ目の前のまとまり(4つ)が幅 60 の1行に入らない → そのまとまりの中で折り返し、タブを
    // 省かない(tab_wrap = false でも)。後ろのまとまり(todo・doing)は、前のまとまりの行の下の行から始まり、
    // 混ざらない。全部のタブの名前が画面にある。
    let first = [
        "期限切れのタスク",
        "今日までのタスク",
        "7日以内のタスク",
        "今月までのタスク",
    ];
    let second = ["todo", "doing"];
    let mut vs: Vec<(&str, bool)> = first.iter().map(|v| (*v, false)).collect();
    vs.push((second[0], true));
    vs.push((second[1], false));
    let all: Vec<&str> = first.iter().chain(second.iter()).copied().collect();
    let (_t, a) = boot(
        "nv32_narrow",
        &base_text(&vs),
        60,
        Opts {
            toml: NO_WRAP,
            ..Default::default()
        },
    );
    let s = scr(&a);
    let mid = search_row(&s);
    for v in &all {
        assert!(
            tab_row(&s, v).is_some_and(|r| r > mid),
            "タブ「{v}」が検索の欄の行の下に無い:\n{s}"
        );
    }
    let r1 = tab_lines(&s, &first);
    let r2 = tab_lines(&s, &second);
    assert!(
        r1.len() >= 2,
        "前のまとまりが行の中で折り返していない:\n{s}"
    );
    assert_eq!(r1[0], mid + 1, "タブの行は真ん中の行のすぐ下から:\n{s}");
    assert_eq!(
        r2[0],
        r1.last().unwrap() + 1,
        "後ろのまとまりは前のまとまりのすぐ下の行から:\n{s}"
    );
    assert_eq!(r2.len(), 1, "後ろのまとまりは1行に入る:\n{s}");
    let rows: BTreeSet<usize> = r1.iter().chain(r2.iter()).copied().collect();
    assert_eq!(
        rows.len(),
        rows.iter().last().unwrap() - mid,
        "タブの行は間を空けない:\n{s}"
    );
}

#[test]
fn test_nv_32_first_tab_break_changes_nothing() {
    // [NV-32] 先頭の A にだけ tab_break → 今のまま(切れ目の無いときと同じ画面。幅 200 ではタブは真ん中の行、
    // 幅 60・tab_wrap = false では今のでないタブを省く NV-23 のまま)。
    for w in [200, 60] {
        let (_t, a) = boot(
            &format!("nv32_first_{w}"),
            &base_text(&abcd(&[A])),
            w,
            Opts {
                toml: NO_WRAP,
                ..Default::default()
            },
        );
        let (_t2, b) = boot(
            &format!("nv32_first_ref_{w}"),
            &base_text(&abcd(&[])),
            w,
            Opts {
                toml: NO_WRAP,
                ..Default::default()
            },
        );
        let s = scr(&a);
        assert_eq!(s, scr(&b), "幅 {w}");
        assert_eq!(tab_row(&s, A), Some(search_row(&s)), "幅 {w}:\n{s}");
    }
}

#[test]
fn test_nv_32_hidden_break_moves_to_next_shown() {
    // [NV-32] C に tab_break で C を隠す → 下の1行目に A と B、2行目に D(C の切れ目は次に見せる D へ)。
    let (_t, a) = boot(
        "nv32_hidden",
        &base_text(&abcd(&[C])),
        200,
        Opts {
            toml: NO_WRAP,
            prefs: Some(TabPrefs {
                hidden: vec![C.into()],
                ..Default::default()
            }),
        },
    );
    let s = scr(&a);
    let mid = search_row(&s);
    assert_eq!(tab_row(&s, C), None, "隠した C は出ない:\n{s}");
    assert_eq!(tab_row(&s, A), Some(mid + 1), "{s}");
    assert_eq!(tab_row(&s, B), Some(mid + 1), "{s}");
    assert_eq!(tab_row(&s, D), Some(mid + 2), "{s}");
    assert_eq!(tab_lines(&s, ABCD).len(), 2, "{s}");
}

#[test]
fn test_nv_32_break_moves_with_tab_order() {
    // [NV-32・NV-26] タブの順を D・C・A・B にする → 切れ目は C と一緒に動き、下の1行目に D、2行目に C・A・B。
    // `]` の順は変えない(A の次は B)。
    let (_t, mut a) = boot(
        "nv32_order",
        &base_text(&abcd(&[C])),
        200,
        Opts {
            toml: NO_WRAP,
            prefs: Some(TabPrefs {
                order: vec![D.into(), C.into(), A.into(), B.into()],
                ..Default::default()
            }),
        },
    );
    let s = scr(&a);
    let mid = search_row(&s);
    assert_eq!(tab_row(&s, D), Some(mid + 1), "{s}");
    for v in [C, A, B] {
        assert_eq!(
            tab_row(&s, v),
            Some(mid + 2),
            "タブ「{v}」は下の2行目:\n{s}"
        );
    }
    // `]` で今のビューから順に進む先は、切れ目が無いときと同じ。
    let (_t2, mut b) = boot(
        "nv32_order_ref",
        &base_text(&abcd(&[])),
        200,
        Opts {
            toml: NO_WRAP,
            prefs: Some(TabPrefs {
                order: vec![D.into(), C.into(), A.into(), B.into()],
                ..Default::default()
            }),
        },
    );
    assert_eq!(current(&a), current(&b));
    for _ in 0..ABCD.len() {
        ch(&mut a, ']');
        ch(&mut b, ']');
        assert_eq!(current(&a), current(&b), "`]` の順は変わらない");
    }
}

#[test]
fn test_nv_32_box_tabs_below_bottom_line() {
    // [NV-32] 囲む帯(box)・余白 1 → タブの行は囲みの下の線の行の下(NV-31 と同じ置き場)。
    let (_t, a) = boot(
        "nv32_box",
        &base_text(&abcd(&[C])),
        200,
        Opts {
            toml: "[display]\nbuttons = \"box\"\n",
            ..Default::default()
        },
    );
    let s = scr(&a);
    let mid = search_row(&s);
    assert_eq!(tab_lines(&s, ABCD).first(), Some(&(mid + 2)), "{s}");
    assert_eq!(tab_row(&s, A), Some(mid + 2), "{s}");
    assert_eq!(tab_row(&s, C), Some(mid + 3), "{s}");
}

#[test]
fn test_nv_32_tabs_never_has_no_tab_rows() {
    // [NV-32・SR-34] tabs = "never" → タブの行は無く、切れ目の無いときと同じ画面。
    let toml = "[display]\ntabs = \"never\"\n";
    let (_t, a) = boot(
        "nv32_never",
        &base_text(&abcd(&[C])),
        200,
        Opts {
            toml,
            ..Default::default()
        },
    );
    let (_t2, b) = boot(
        "nv32_never_ref",
        &base_text(&abcd(&[])),
        200,
        Opts {
            toml,
            ..Default::default()
        },
    );
    let s = scr(&a);
    assert_eq!(tab_lines(&s, ABCD), Vec::<usize>::new(), "{s}");
    assert_eq!(s, scr(&b));
}

#[test]
fn test_nv_32_marker_base_view_tab_break() {
    // [NV-32・WS-9] ワークスペースの印の [table.base] のビューに tab_break = true → `.base` のときと同じく、
    // 下の1行目に A と B、2行目に C と D。
    let mut marker = String::from("[[table]]\nname = \"台帳\"\n\n[table.base]\n");
    for v in ABCD {
        marker.push_str(&format!(
            "\n[[table.base.views]]\ntype = \"table\"\nname = \"{v}\"\norder = [\"title\", \"status\"]\n"
        ));
        if *v == C {
            marker.push_str("tab_break = true\n");
        }
    }
    let (_t, a) = boot_marker("nv32_marker", &marker, 200);
    assert_ab_cd(&scr(&a));
}
