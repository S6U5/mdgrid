//! 詳細の表示の本文の先頭(NV-6・SR-9・SR-23)の受け入れ試験。
//! 記録: specs/_changes/2026-10-06-detail-body.md のタスク #1。
use super::keymap::Mode;
use super::test_screen::*;
use super::{draw, view, width};
use mdgrid::i18n::{scoped, Lang};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const WITH_BODY: &str =
    "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n\n# 議題\n\n予算の確認をする。\n";

/// `K` を押して詳細の表示の画面を返す。
fn detail_screen(notes: &[(&str, &str)], name: &str) -> (Tmp, super::App, String) {
    let (t, mut a) = make(name, notes);
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    let s = screen(&a);
    (t, a, s)
}

/// 見出しの行(前後の空白を除いてちょうど `head`)の添字。
fn head_at(s: &str, head: &str) -> Option<usize> {
    s.lines().position(|l| l.trim() == head)
}

#[test]
fn test_nv_6_body_heading_and_first_lines() {
    // [NV-6] 本文のあるノートで `K` → プロパティの下に「本文」の見出しと本文の先頭の行。
    let (_t, _a, s) = detail_screen(&[("a.md", WITH_BODY)], "nv6body");
    let h = head_at(&s, "本文").unwrap_or_else(|| panic!("本文の見出しが無い: {s}"));
    let status = s.lines().position(|l| l.contains("status")).unwrap();
    assert!(status < h, "見出しはプロパティの下: {s}");
    let below: Vec<&str> = s.lines().skip(h + 1).collect();
    assert_eq!(
        below[0].trim(),
        "# 議題",
        "先頭の空行は除き、本文の先頭の行: {s}"
    );
    assert!(
        below.iter().any(|l| l.contains("予算の確認をする。")),
        "{s}"
    );
}

#[test]
fn test_nv_6_body_none_without_body() {
    // [NV-6] 本文の無いノート(フロントマターだけ)では見出しを出さない。空行だけの本文も無いのと同じ。
    for (i, n) in ["---\ntitle: a\n---\n", "---\ntitle: a\n---\n\n  \n\n"]
        .iter()
        .enumerate()
    {
        let (_t, _a, s) = detail_screen(&[("a.md", n)], &format!("nv6nobody{i}"));
        assert!(head_at(&s, "本文").is_none(), "見出しを出さない: {s}");
    }
}

#[test]
fn test_nv_6_body_whole_note_without_frontmatter() {
    // [NV-6] フロントマターの無いノートは全体が本文。
    let (_t, _a, s) = detail_screen(
        &[
            ("a.md", "一行目の本文\n二行目の本文\n"),
            ("b.md", "---\ntitle: b\n---\n"),
        ],
        "nv6nofm",
    );
    let h = head_at(&s, "本文").unwrap_or_else(|| panic!("本文の見出しが無い: {s}"));
    let below: Vec<&str> = s.lines().skip(h + 1).collect();
    assert_eq!(below[0].trim(), "一行目の本文", "{s}");
    assert_eq!(below[1].trim(), "二行目の本文", "{s}");
}

#[test]
fn test_nv_6_body_capped_at_20_lines() {
    // [NV-6] 25 行の本文は先頭の 20 行と「…」。
    let mut n = String::from("---\nt: x\n---\n");
    for i in 1..=25 {
        n.push_str(&format!("本文の行{i:02}\n"));
    }
    let (_t, mut a) = make("nv6cap", &[("a.md", n.as_str())]);
    a.resize(80, 40);
    ch(&mut a, 'K');
    let w = 79;
    let lines: Vec<String> = view::render(&a, w, 39)
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect();
    let all = lines.join("\n");
    let h = lines
        .iter()
        .position(|l| l.trim() == "本文")
        .unwrap_or_else(|| panic!("本文の見出しが無い: {all}"));
    for i in 1..=20 {
        assert_eq!(lines[h + i].trim(), format!("本文の行{i:02}"), "{all}");
    }
    assert_eq!(lines[h + 21].trim(), "…", "21 行目は「…」: {all}");
    assert!(!all.contains("本文の行21"), "21 行目からは出さない: {all}");
}

#[test]
fn test_nv_6_body_no_control_chars_and_fits() {
    // [NV-6] [SR-9] 本文の制御文字(エスケープ)は画面に出さず、80 桁と 40 桁で全行が幅以下。
    let long = "とても長い本文の行が続いて画面の幅を超えていく。".repeat(5);
    let n = format!("---\ntitle: x\n---\n赤い\u{1b}[31m文字\u{7}\n{long}\n");
    let (_t, mut a) = make("nv6ctl", &[("a.md", n.as_str())]);
    ch(&mut a, 'K');
    let s = screen(&a);
    assert!(head_at(&s, "本文").is_some(), "{s}");
    assert!(s.contains("赤い␛[31m文字␇"), "制御文字は見える文字に: {s}");
    for w in [80u16, 40] {
        a.resize(w, 24);
        for l in view::render(&a, (w - 1) as usize, 23) {
            let lw: usize = l.spans.iter().map(|s| width::width(&s.content)).sum();
            assert!(lw <= (w - 1) as usize, "幅 {w} で {lw}: {l:?}");
        }
        let mut term = Terminal::new(TestBackend::new(w, 24)).unwrap();
        term.draw(|f| draw(f, &a)).unwrap();
        let buf = term.backend().buffer();
        for y in 0..24 {
            for x in 0..w {
                let s = buf[(x, y)].symbol();
                assert!(!s.contains(char::is_control), "制御文字 {s:?}");
            }
        }
    }
    assert_fits(&mut a);
}

#[test]
fn test_nv_6_body_arrows_stay_on_properties() {
    // [NV-6] ↑↓ はプロパティの間だけを動き、本文の行に止まらない。
    let (_t, mut a, _) = detail_screen(&[("a.md", WITH_BODY)], "nv6arrows");
    let n = {
        let d = a.detail.as_ref().unwrap();
        a.detail_props(&d.row).len()
    };
    for _ in 0..(n + 10) {
        ch(&mut a, 'j');
    }
    assert_eq!(
        a.detail.as_ref().unwrap().sel,
        n - 1,
        "最後のプロパティで止まる"
    );
    let s = screen(&a);
    let marked: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(marked.len(), 1, "印はプロパティの1行だけ: {s}");
    assert!(!marked[0].contains("予算"), "{s}");
    for _ in 0..(n + 10) {
        ch(&mut a, 'k');
    }
    assert_eq!(a.detail.as_ref().unwrap().sel, 0);
}

#[test]
fn test_nv_6_body_heading_english() {
    // [NV-6] [SR-23] 英語の画面では見出しが "Body"。
    let _g = scoped(Lang::En);
    let (_t, _a, s) = detail_screen(
        &[("a.md", "---\ntitle: meeting\n---\nfirst line of the body\n")],
        "nv6en",
    );
    let h = head_at(&s, "Body").unwrap_or_else(|| panic!("Body の見出しが無い: {s}"));
    assert_eq!(
        s.lines().nth(h + 1).unwrap().trim(),
        "first line of the body",
        "{s}"
    );
    assert!(head_at(&s, "本文").is_none());
}
