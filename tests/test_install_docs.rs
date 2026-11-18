//! 入れ方の説明に書いた Nix のタグ(`github:S6U5/mdgrid/v<版>`)が、Cargo.toml の版と同じかを見る試験。
//! 関係する要件: なし(配布。docs/design.md の「配布」)。タグを付けない `github:S6U5/mdgrid` は main の先頭を
//! 取るので、文書はリリースした版のタグを書く。版を上げて文書を直し忘れたら、ここで落ちる。

use std::path::PathBuf;

const FLAKE_REF: &str = "github:S6U5/mdgrid/v";

/// 入れ方を書いた文書。
const DOCS: [&str; 4] = [
    "README.md",
    "README.ja.md",
    "docs/manual/en/getting-started.md",
    "docs/manual/ja/getting-started.md",
];

/// `github:S6U5/mdgrid/v` の後ろに続く版(数字と点)を全部取る。
fn tagged_versions(text: &str) -> Vec<String> {
    text.match_indices(FLAKE_REF)
        .map(|(at, _)| {
            text[at + FLAKE_REF.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect()
        })
        .collect()
}

#[test]
fn test_nix_tag_in_install_docs_matches_the_crate_version() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    for doc in DOCS {
        let path = root.join(doc);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()));
        let found = tagged_versions(&text);
        assert!(
            !found.is_empty(),
            "{doc} に Nix のタグ付きの入れ方({FLAKE_REF}{version})が無い"
        );
        for v in found {
            assert_eq!(
                v, version,
                "{doc} の Nix のタグ v{v} が Cargo.toml の版 {version} と違う"
            );
        }
    }
}
