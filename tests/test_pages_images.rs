//! 説明書(日英)の画像は、Pages の workflow が撮る画面(images/<場面>.svg)だけを指す(SR-38)。
//! 本の外のファイルを指す画像は、Pages では GitHub のファイルへのリンクになって出ない。

use std::path::Path;

#[test]
fn test_sr_38_pages_manual_images_only_scenes() {
    // [SR-38] 説明書のページの `![…](…)` は、どれも `images/` の下を指す。
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for l in ["ja", "en"] {
        for e in std::fs::read_dir(root.join("docs/manual").join(l)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            let mut rest = text.as_str();
            while let Some(i) = rest.find("![") {
                let after = &rest[i..];
                let Some(j) = after.find("](") else { break };
                let tail = &after[j + 2..];
                let Some(k) = tail.find(')') else { break };
                let target = &tail[..k];
                assert!(
                    target.starts_with("images/"),
                    "{} の画像 {target} は撮る場面でない",
                    p.display()
                );
                rest = &tail[k..];
            }
        }
    }
}
