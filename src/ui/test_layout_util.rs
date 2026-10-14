//! 画面の中の字の位置を求める試験の手助け(SR-1 のビューの帯で配置が変わっても壊れにくいように、位置を数で
//! 決め打ちせずに求める)。錠のある試験のファイルの共通部分を変えないよう、別のファイルに置く。

use super::test_screen::{buffer, screen};
use super::*;

/// 画面(80×24)で文字 `t` の始まる (桁, 行)。上の行から、左から探して最初のもの。
/// 帯やボタンの位置を数で決め打ちせずに求めるため(SR-1 の配置が変わっても壊れにくいように)。
pub(super) fn find_at(app: &App, t: &str) -> Option<(u16, u16)> {
    let b = buffer(app);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    (0..b.area.height).find_map(|y| {
        (0..b.area.width)
            .find(|&x| {
                let mut xx = x;
                cs.iter().all(|c| {
                    let hit = xx < b.area.width && b[(xx, y)].symbol() == c;
                    xx += width::width(c).max(1) as u16;
                    hit
                })
            })
            .map(|x| (x, y))
    })
}

/// ビューの帯(SR-1)の検索の欄の頭「検索」の (桁, 行)。欄が無ければ失敗する。
pub(super) fn search_at(app: &App) -> (u16, u16) {
    let (x, y) = find_at(app, "検索").unwrap_or_else(|| panic!("検索の欄が無い:\n{}", screen(app)));
    assert!(
        (y as usize) < view::data_y(app),
        "検索の欄は表の上:\n{}",
        screen(app)
    );
    (x, y)
}
