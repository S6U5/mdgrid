//! 読み込みの途中の組み立て直し(BV-16): 読んだ数が倍になるごと。読み終えたら必ず組み直し、全部の行が出る。

use super::startup::Startup;
use super::test_screen::Tmp;
use super::*;
use mdgrid::source::markdown::Markdown;

#[test]
fn test_bv_16_rebuild_when_loaded_doubles() {
    // [BV-16] 1000 ノートを 100 ずつ読む → 表の行は最初の一束で出て、倍ごとに増え、最後に 1000。
    let tmp = Tmp::new("bv16_throttle");
    for i in 0..1000 {
        tmp.write(&format!("n{i:04}.md"), "---\nx: 1\n---\n");
    }
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: true,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    let mut seen = Vec::new();
    while !a.loaded() {
        a.load_step(100);
        seen.push(a.rows.len());
    }
    assert!(seen[0] > 0, "最初の画面は最初の一束で出す: {seen:?}");
    assert_eq!(seen.last(), Some(&1000), "読み終えたら全部: {seen:?}");
    let rebuilds = seen.windows(2).filter(|w| w[0] != w[1]).count() + 1;
    assert!(rebuilds <= 6, "組み直しは倍ごと: {seen:?}");
    assert!(seen.windows(2).all(|w| w[0] <= w[1]), "{seen:?}");
}
