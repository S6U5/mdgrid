//! 文字の幅(SR-9)。書記素のまとまりで幅を数え、幅に合わせて切り詰める。
//! 制御文字は見える文字に置き換える。ratatui には、ここで切り詰めた文字列だけを渡す。
//!
//! 幅の数え方は ratatui のバッファ(ratatui-core の `cell_width`)に合わせる:
//! 書記素ごとに unicode-width の幅、半角の濁点・半濁点(U+FF9E・U+FF9F)は1つにつき +1。
//! East Asian Ambiguous は既定で 1。設定(CV-6 の `ambiguous_wide`)で 2 にすると、ratatui の数え方と
//! ずれるので、描画は mod.rs の `write_lines` がセルに幅を直接書く。

use std::cell::Cell;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

thread_local! {
    /// East Asian Ambiguous を幅2と数えるか(CV-6)。描画の入口(view::render・view::layout)が App の設定から置く。
    static AMBIGUOUS_WIDE: Cell<bool> = const { Cell::new(false) };
}

/// East Asian Ambiguous の幅を2にするか(CV-6)。この糸の上の以後の幅の計算に効く。
pub fn set_ambiguous_wide(on: bool) {
    AMBIGUOUS_WIDE.with(|c| c.set(on));
}

/// 制御文字を見える文字に置き換える(SR-9)。C0 は U+2400 台の図記号(ESC → `␛`)、DEL は `␡`、C1 は `�`。
pub fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c as u32 {
            0x00..=0x1f => char::from_u32(0x2400 + c as u32).unwrap_or('�'),
            0x7f => '␡',
            0x80..=0x9f => '�',
            // 双方向の制御・行と段落の区切り・BOM(画面の並びを崩す見えない文字)。
            0x061c
            | 0x200e
            | 0x200f
            | 0x202a..=0x202e
            | 0x2066..=0x2069
            | 0x2028
            | 0x2029
            | 0xfeff => '�',
            _ => c,
        })
        .collect()
}

/// 1つの書記素のまとまりの幅(CV-6 の設定を当てたもの)。
pub fn grapheme_width(g: &str) -> usize {
    if AMBIGUOUS_WIDE.with(Cell::get) && g.len() > 1 {
        let marks = g
            .chars()
            .filter(|c| matches!(c, '\u{ff9e}' | '\u{ff9f}'))
            .count();
        return UnicodeWidthStr::width_cjk(g) + marks;
    }
    narrow_width(g)
}

/// 1つの書記素のまとまりの、ratatui のバッファが数える幅(East Asian Ambiguous は 1)。
pub fn narrow_width(g: &str) -> usize {
    let marks = g
        .chars()
        .filter(|c| matches!(c, '\u{ff9e}' | '\u{ff9f}'))
        .count();
    if g.len() == 1 {
        1
    } else {
        UnicodeWidthStr::width(g) + marks
    }
}

/// 文字列の幅(制御文字を置き換えたあとの文字列に使う)。
pub fn width(s: &str) -> usize {
    s.graphemes(true).map(grapheme_width).sum()
}

/// Extended_Pictographic(Unicode の emoji-data。U+00FF より上の範囲)と地域指示子(国旗)。
const PICTOGRAPHIC: &[(u32, u32)] = &[
    (0x203C, 0x203C),
    (0x2049, 0x2049),
    (0x2122, 0x2122),
    (0x2139, 0x2139),
    (0x2194, 0x2199),
    (0x21A9, 0x21AA),
    (0x231A, 0x231B),
    (0x2328, 0x2328),
    (0x2388, 0x2388),
    (0x23CF, 0x23CF),
    (0x23E9, 0x23F3),
    (0x23F8, 0x23FA),
    (0x24C2, 0x24C2),
    (0x25AA, 0x25AB),
    (0x25B6, 0x25B6),
    (0x25C0, 0x25C0),
    (0x25FB, 0x25FE),
    (0x2600, 0x2605),
    (0x2607, 0x2612),
    (0x2614, 0x2685),
    (0x2690, 0x2705),
    (0x2708, 0x2712),
    (0x2714, 0x2714),
    (0x2716, 0x2716),
    (0x271D, 0x271D),
    (0x2721, 0x2721),
    (0x2728, 0x2728),
    (0x2733, 0x2734),
    (0x2744, 0x2744),
    (0x2747, 0x2747),
    (0x274C, 0x274C),
    (0x274E, 0x274E),
    (0x2753, 0x2755),
    (0x2757, 0x2757),
    (0x2763, 0x2767),
    (0x2795, 0x2797),
    (0x27A1, 0x27A1),
    (0x27B0, 0x27B0),
    (0x27BF, 0x27BF),
    (0x2934, 0x2935),
    (0x2B05, 0x2B07),
    (0x2B1B, 0x2B1C),
    (0x2B50, 0x2B50),
    (0x2B55, 0x2B55),
    (0x3030, 0x3030),
    (0x303D, 0x303D),
    (0x3297, 0x3297),
    (0x3299, 0x3299),
    (0x1F000, 0x1F0FF),
    (0x1F10D, 0x1F10F),
    (0x1F12F, 0x1F12F),
    (0x1F16C, 0x1F171),
    (0x1F17E, 0x1F17F),
    (0x1F18E, 0x1F18E),
    (0x1F191, 0x1F19A),
    (0x1F1AD, 0x1F1FF),
    (0x1F201, 0x1F20F),
    (0x1F21A, 0x1F21A),
    (0x1F22F, 0x1F22F),
    (0x1F232, 0x1F23A),
    (0x1F23C, 0x1F23F),
    (0x1F249, 0x1F3FA),
    (0x1F400, 0x1F53D),
    (0x1F546, 0x1F64F),
    (0x1F680, 0x1F6FF),
    (0x1F774, 0x1F77F),
    (0x1F7D5, 0x1F7FF),
    (0x1F80C, 0x1F80F),
    (0x1F848, 0x1F84F),
    (0x1F85A, 0x1F85F),
    (0x1F888, 0x1F88F),
    (0x1F8AE, 0x1F8FF),
    (0x1F90C, 0x1F93A),
    (0x1F93C, 0x1F945),
    (0x1F947, 0x1FAFF),
    (0x1FC00, 0x1FFFD),
];

fn pictographic(c: char) -> bool {
    let u = c as u32;
    u >= 0x203C
        && PICTOGRAPHIC
            .binary_search_by(|&(lo, hi)| {
                if hi < u {
                    std::cmp::Ordering::Less
                } else if lo > u {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
}

/// 絵文字の書記素か(SR-10): Extended_Pictographic を含み、絵文字として描かれるもの(幅2か、異体字セレクタ
/// U+FE0F つき)。幅1の記号として描かれるもの(`™`・`↔` など)は文字として残す。
pub fn is_emoji(g: &str) -> bool {
    g.chars().any(pictographic) && (narrow_width(g) >= 2 || g.contains('\u{fe0f}'))
}

/// 絵文字を含むか。
pub fn has_emoji(s: &str) -> bool {
    s.chars().any(|c| (c as u32) >= 0x203C) && s.graphemes(true).any(is_emoji)
}

/// 絵文字の書記素を `?` と空白(元と同じ幅)に置き換える(`TERM=dumb`。SR-10)。
pub fn replace_emoji(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for g in s.graphemes(true) {
        if is_emoji(g) {
            let w = grapheme_width(g).max(1);
            out.push('?');
            out.push_str(&" ".repeat(w - 1));
        } else {
            out.push_str(g);
        }
    }
    out
}

/// 寄せ方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

/// 幅 `w` 以下になるまで先頭から取る(`…` は付けない)。
pub fn take(s: &str, w: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for g in s.graphemes(true) {
        let gw = grapheme_width(g);
        if used + gw > w {
            break;
        }
        out.push_str(g);
        used += gw;
    }
    out
}

/// 幅 `w` 以下に切り詰める。はみ出すときは末尾を `…` にする。
pub fn truncate(s: &str, w: usize) -> String {
    if width(s) <= w {
        return s.to_string();
    }
    // `…` は East Asian Ambiguous なので、CV-6 で幅2と数えるときは2桁を空ける(SR-9)。
    let ell = width("…");
    if w < ell {
        return take(s, w);
    }
    let mut out = String::new();
    let mut used = 0;
    for g in s.graphemes(true) {
        let gw = grapheme_width(g);
        if used + gw > w - ell {
            break;
        }
        out.push_str(g);
        used += gw;
    }
    out.push('…');
    out
}

/// ちょうど幅 `w` にする(切り詰めて、空白で埋める)。
pub fn fit(s: &str, w: usize, align: Align) -> String {
    let t = truncate(s, w);
    let pad = w - width(&t);
    match align {
        Align::Left => format!("{t}{}", " ".repeat(pad)),
        Align::Right => format!("{}{t}", " ".repeat(pad)),
        Align::Center => {
            let l = pad / 2;
            format!("{}{t}{}", " ".repeat(l), " ".repeat(pad - l))
        }
    }
}

#[cfg(test)]
#[path = "test_width.rs"]
mod tests;
