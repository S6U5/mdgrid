//! Markdown の本文の行の読み方: 字下げの幅と、コードのフェンスの開きと閉じ。本文のタグ(source::markdown)と
//! リンク(links)の両方が使う(どちらかに置くと、互いに頼り合う形になるので分けた)。

/// 行頭の字下げの幅(タブは4つ分)と、字下げの後ろ。
pub(crate) fn indent_of(line: &str) -> (usize, &str) {
    let mut w = 0;
    for (i, c) in line.char_indices() {
        match c {
            ' ' => w += 1,
            '\t' => w += 4 - w % 4,
            _ => return (w, &line[i..]),
        }
    }
    (w, "")
}

/// コードのフェンスの開き(字下げ3つまで、同じ記号 ``` か ~~~ が3つ以上)なら (記号, 長さ)。
/// バッククォートのフェンスの情報の文字列にバッククォートがあれば、フェンスではない(1行の ```a``` はインラインのコード)。
pub(crate) fn fence_open(line: &str) -> Option<(char, usize)> {
    let (w, rest) = indent_of(line);
    if w > 3 {
        return None;
    }
    let c = rest.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let n = rest.chars().take_while(|&x| x == c).count();
    (n >= 3 && !(c == '`' && rest[n..].contains('`'))).then_some((c, n))
}

/// フェンスの閉じ: 同じ記号が開きの長さ以上で、後ろは空白だけ。
pub(crate) fn fence_close(line: &str, (c, n): (char, usize)) -> bool {
    let (w, rest) = indent_of(line);
    let m = rest.chars().take_while(|&x| x == c).count();
    w <= 3 && m >= n && rest[m * c.len_utf8()..].trim().is_empty()
}
