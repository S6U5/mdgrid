//! GitHub Pages のサイト(説明書・見た目のカタログ・画面の一覧)の材料がそろっているか(SR-38)。
//! 説明書の画面の画像はリポに入れず、Pages の workflow が本物の mdgrid で撮る(scripts/tui_shot.py と
//! docs/manual-scenarios.toml)。説明書が指す画像は、どれも撮る場面にある名前でなければならない。

use std::collections::HashSet;
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn scene_ids() -> Vec<String> {
    let text = std::fs::read_to_string(root().join("docs/manual-scenarios.toml")).unwrap();
    let table: toml::Table = text.parse().unwrap();
    table["shot"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_string())
        .collect()
}

/// 説明書のページのリンクの行き先(コードの囲みの中は除く)。
fn links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            let after = &rest[i + 2..];
            let Some(j) = after.find(')') else { break };
            out.push(after[..j].to_string());
            rest = &after[j..];
        }
    }
    out
}

#[test]
fn test_sr_38_pages_manual_images_are_scenes() {
    // [SR-38] 説明書(日英)が指す images/X.svg は、どれも撮る場面の id。場面の id は重ならない。
    let ids = scene_ids();
    let set: HashSet<&String> = ids.iter().collect();
    assert_eq!(set.len(), ids.len(), "場面の id が重なっている");
    let mut seen = 0;
    for l in ["ja", "en"] {
        for e in std::fs::read_dir(root().join("docs/manual").join(l)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            for t in links(&text) {
                if let Some(name) = t.strip_prefix("images/").and_then(|n| n.strip_suffix(".svg")) {
                    seen += 1;
                    assert!(
                        set.contains(&name.to_string()),
                        "{} の {t} を撮る場面が無い",
                        p.display()
                    );
                }
            }
        }
    }
    assert!(seen > 20, "説明書の画像が少なすぎる: {seen}");
}

#[test]
fn test_sr_38_pages_manual_links_resolve() {
    // [SR-38] 説明書のページから docs/ の中を指すリンク(画像と見本の保管庫の外)は、在るファイルを指す。
    for l in ["ja", "en"] {
        let dir = root().join("docs/manual").join(l);
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            for t in links(&text) {
                if t.contains(':') || t.starts_with('#') || t.starts_with("images/") {
                    continue;
                }
                let path = t.split('#').next().unwrap();
                let target = dir.join(path);
                let inside = target
                    .canonicalize()
                    .map(|c| c.starts_with(root().join("docs").canonicalize().unwrap()))
                    .unwrap_or(true);
                if inside {
                    assert!(target.exists(), "{} の {t} が無い", p.display());
                }
            }
        }
    }
}

#[test]
fn test_sr_38_pages_workflow_shoots_and_builds() {
    // [SR-38] workflow は本物の mdgrid を作って画面を撮り、scripts/pages.py で site/ を作って上げる。
    // 画面の画像はリポに入れない(.gitignore)。
    let wf = std::fs::read_to_string(root().join(".github/workflows/pages.yml")).unwrap();
    for want in [
        "cargo build --release",
        "scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang ja",
        "scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang en",
        "python3 scripts/pages.py site --shots target/shots",
        "path: site",
    ] {
        assert!(wf.contains(want), "{want}: {wf}");
    }
    let ignore = std::fs::read_to_string(root().join(".gitignore")).unwrap();
    assert!(ignore.contains("/docs/manual/*/images/"), "{ignore}");
    let cat = std::fs::read_to_string(root().join("docs/catalog/index.html")).unwrap();
    assert!(cat.contains("get(\"lang\")"), "カタログが ?lang= を受ける");
}
