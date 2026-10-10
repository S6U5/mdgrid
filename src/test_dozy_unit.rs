//! 組 dozy-pink(SR-36)。

use super::*;

#[test]
fn test_sr_36_dozy_pink_preset() {
    // [SR-36] preset = "dozy-pink" → 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ。上書きもできる。
    let (c, w) = crate::config::parse("[look]\npreset = \"dozy-pink\"\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    let s = c.resolved().style;
    assert_eq!(s.preset, Preset::DozyPink);
    assert_eq!((s.status, s.tags), (Status::Pill, Tags::Pill));
    assert_eq!(
        (s.select, s.rules, s.tabs),
        (Select::Bar, Rules::None_, Tabs::Pill)
    );
    let (c, _) =
        crate::config::parse("[look]\npreset = \"dozy-pink\"\n[look.style]\ntags = \"dots\"\n")
            .unwrap();
    let s = c.resolved().style;
    assert_eq!((s.status, s.tags), (Status::Pill, Tags::Dots));
}
