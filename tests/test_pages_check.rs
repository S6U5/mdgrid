//! Pages の workflow は、作ったサイトを公開の前にブラウザで確かめる(scripts/check_site.py)。落ちたら上げない(SR-38)。

use std::path::Path;

#[test]
fn test_sr_38_pages_workflow_checks_site() {
    // [SR-38] サイトを作る → ブラウザで確かめる → 上げる、の順。確かめる道具が要ることを確かめる。
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wf = std::fs::read_to_string(root.join(".github/workflows/pages.yml")).unwrap();
    let build = wf
        .find("python3 scripts/pages.py site")
        .expect("サイトを作る");
    let check = wf
        .find("python3 scripts/check_site.py site")
        .expect("ブラウザで確かめる");
    let upload = wf.find("actions/upload-pages-artifact").expect("上げる");
    assert!(
        build < check && check < upload,
        "作る → 確かめる → 上げる の順"
    );
    let py = std::fs::read_to_string(root.join("scripts/check_site.py")).unwrap();
    for want in [
        "console",
        "390",
        "lang-switch",
        "hreflang",
        "#term .scr",
        "text-size-adjust",
    ] {
        assert!(py.contains(want), "check_site.py が {want} を確かめる");
    }
}
