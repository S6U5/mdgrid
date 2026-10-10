//! WBS の番号と進み具合(NV-28)の核。

#[test]
fn test_nv_28_numbers_and_progress() {
    // [NV-28] 設計(1)の下に画面(1.1。その下に部品 1.1.1・ボタン 1.1.2)と API(1.2)、メモ(2)。
    // 進み具合: 部品 100・ボタン 0・API 50 → 画面 50%、設計 50%((100+0+50)/3)。
    let rows = ["設計", "画面", "部品", "ボタン", "API", "メモ"];
    let pairs = [
        ("画面", "設計"),
        ("部品", "画面"),
        ("ボタン", "画面"),
        ("API", "設計"),
    ];
    let p = |r: &&'static str| pairs.iter().find(|(c, _)| c == r).map(|(_, p)| *p);
    let nodes = crate::tree::order(&rows, p);
    let num = crate::tree::numbers(&nodes);
    assert_eq!(num["設計"], "1");
    assert_eq!(num["画面"], "1.1");
    assert_eq!(num["部品"], "1.1.1");
    assert_eq!(num["ボタン"], "1.1.2");
    assert_eq!(num["API"], "1.2");
    assert_eq!(num["メモ"], "2");
    let pct = |r: &&'static str| match *r {
        "部品" => 100,
        "API" => 50,
        "メモ" => 255,
        _ => 0,
    };
    let pr = crate::tree::progress(&nodes, pct);
    assert_eq!(pr[&"画面"], 50);
    assert_eq!(pr[&"設計"], 50);
    // 子の無い行は進み具合を持たない。100 を超える値は 100 とみなす(メモは子が無いので数えない)。
    assert!(!pr.contains_key(&"部品") && !pr.contains_key(&"メモ"));
    // 切り捨て: 100・0・0 → 33。
    let pct2 = |r: &&'static str| if *r == "部品" { 100 } else { 0 };
    assert_eq!(crate::tree::progress(&nodes, pct2)[&"設計"], 33);
}
