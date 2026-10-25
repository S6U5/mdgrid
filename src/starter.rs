//! 台帳のスターター(ST-1〜ST-6): 台帳の見せ方(印 `.mdgrid/workspace.toml`・新しいノートの本文の雛形・見本のノート)の
//! ひとまとまりを、組み込み(リポの `starters/<名前>/`。build.rs が `OUT_DIR/starters.rs` に埋め込む)と、設定の置き場の
//! `starters/<名前>/` から読み、そこから台帳を作る。
//!
//! 形: `starter.toml`(`[name]`・`[description]` に言語の符号の鍵)、`ledger/`(常に写す。台帳の根からの相対パスのまま)、
//! `ledger.<言語>/`(その言語のときに丸ごと代わりに使う。重ねない)、`samples/`・`samples.<言語>/`(見本。頼まれたときだけ。
//! CE-32 の変数を埋めて写す)。組み込みもユーザーのものも同じ `Starter` に読む(組み込みだけの特例を作らない。ST-2)。
//! ノートの形の決まり(スキーマ・lint)は持たない(ST-1)。

use crate::i18n::{Lang, Msg};
use crate::newnote;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/starters.rs"));

/// 設定の置き場の中の、ユーザーのスターターのフォルダ(ST-3)。
pub const DIR: &str = "starters";
/// 名前と説明のマニフェスト(ST-1)。
pub const MANIFEST: &str = "starter.toml";
/// 印のパス(台帳の根からの相対。WS-7)。
const MARKER: &str = ".mdgrid/workspace.toml";

/// スターターの部分: 常に写す `ledger/` と、頼まれたときだけ写す `samples/`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Ledger,
    Sample,
}

impl Part {
    /// フォルダの名前(`ledger`・`ledger.ja`・`samples`・`samples.ja`)から部分と言語。ほかの名前は None。
    fn parse(folder: &str) -> Option<(Part, Option<String>)> {
        let (kind, lang) = match folder.split_once('.') {
            Some((k, l)) if !l.is_empty() => (k, Some(l.to_string())),
            Some(_) => return None,
            None => (folder, None),
        };
        let part = match kind {
            "ledger" => Part::Ledger,
            "samples" => Part::Sample,
            _ => return None,
        };
        Some((part, lang))
    }
}

/// どこから来たスターターか。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// 実行ファイルの中(ST-2)。
    Builtin,
    /// 設定の置き場の `starters/<名前>/`(ST-3)。
    User(PathBuf),
}

/// スターターの1つのファイル。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarterFile {
    pub part: Part,
    /// 言語の変種(`ledger.ja/` なら `ja`)。接尾の無いものは None。
    pub lang: Option<String>,
    /// 台帳の根からの相対パス。
    pub rel: PathBuf,
    pub text: String,
}

/// 台帳のスターター(ST-1)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Starter {
    /// フォルダの名前(`--starter <名前>` で指す)。
    pub name: String,
    /// 言語の符号 → 見せる名前(`starter.toml` の `[name]`)。
    pub title: BTreeMap<String, String>,
    /// 言語の符号 → 1行の説明(`[description]`)。
    pub description: BTreeMap<String, String>,
    pub files: Vec<StarterFile>,
    pub origin: Origin,
}

impl Starter {
    /// 印の無い `ledger` の変種(接尾の無いものは None)。接尾の無い `ledger/` と、ある言語の `ledger.<言語>/` の
    /// どれにも印が要る(変種は丸ごと差し替えるので、印の無い変種からは印の無い台帳ができるため。ST-1・ST-3)。
    fn missing_marker(&self) -> Option<Option<String>> {
        let mut langs: Vec<Option<String>> = vec![None];
        for f in &self.files {
            if f.part == Part::Ledger && !langs.contains(&f.lang) {
                langs.push(f.lang.clone());
            }
        }
        langs.into_iter().find(|l| {
            !self
                .files
                .iter()
                .any(|f| f.part == Part::Ledger && f.lang == *l && f.rel == Path::new(MARKER))
        })
    }
}

/// 言語の符号 → 文(マニフェストの `[name]`・`[description]`)。
type Words = BTreeMap<String, String>;

/// マニフェストの `[name]`・`[description]` を読む(文字の値だけ。ほかは無視)。読めなければ TOML の誤り。
fn read_manifest(text: &str) -> Result<(Words, Words), toml::de::Error> {
    let t: toml::Table = text.parse()?;
    let words = |key: &str| -> Words {
        t.get(key)
            .and_then(|v| v.as_table())
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.trim().to_string())))
                    .filter(|(_, s)| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok((words("name"), words("description")))
}

/// 組み込みのスターター(ST-2。名前の順)。build.rs が形を確かめた物なので、ここで落ちることは無い。
pub fn builtin() -> Vec<Starter> {
    BUILTIN
        .iter()
        .map(|b| {
            let (title, description) = read_manifest(b.manifest).unwrap_or_default();
            let files = b
                .files
                .iter()
                .filter_map(|f| {
                    let (part, lang) = Part::parse(f.part)?;
                    Some(StarterFile {
                        part,
                        lang,
                        rel: f.rel.split('/').collect(),
                        text: f.text.to_string(),
                    })
                })
                .collect();
            Starter {
                name: b.name.to_string(),
                title,
                description,
                files,
                origin: Origin::Builtin,
            }
        })
        .collect()
}

/// CLI-3 の形の警告「ファイル: 項目の道筋: 理由」。
fn warn(file: &str, path: &str, reason: String) -> String {
    Msg::SettingWarn.fill(&[&file, &path, &reason])
}

/// ユーザーのスターターを1つ読む(ST-3)。`dir` は `starters/<名前>/`。読めなければ警告の文(CLI-3 の形)。
/// シンボリックリンクはたどらない(設定の置き場から外のファイルを読ませない)。
pub fn read_user(dir: &Path) -> Result<Starter, String> {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let label = format!("{DIR}/{name}");
    let mut title = BTreeMap::new();
    let mut description = BTreeMap::new();
    let manifest = dir.join(MANIFEST);
    let mlabel = format!("{label}/{MANIFEST}");
    if manifest.is_file() {
        // 読めないファイルは「starters/<名前>: <中の相対パス>: 理由」(印の無いときと同じ形)。
        let text = std::fs::read_to_string(&manifest)
            .map_err(|_| warn(&label, MANIFEST, Msg::StarterNotText.text().to_string()))?;
        match read_manifest(&text) {
            Ok((t, d)) => {
                title = t;
                description = d;
            }
            Err(e) => {
                let (line, msg) = crate::config::toml_error(&text, &e);
                let at = line.map(|l| format!("line {l}")).unwrap_or_default();
                return Err(warn(&mlabel, &at, Msg::StarterBadToml.fill(&[&msg])));
            }
        }
    }
    let mut files = Vec::new();
    let mut parts: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| warn(&label, "", e.to_string()))?
        .filter_map(|e| Some(e.ok()?.path()))
        .collect();
    parts.sort();
    for p in parts {
        let Some((part, lang)) = p.file_name().and_then(|s| s.to_str()).and_then(Part::parse)
        else {
            continue;
        };
        if !p.is_dir() || is_link(&p) {
            continue;
        }
        let mut found = Vec::new();
        walk(&p, &p, &mut found);
        found.sort();
        for rel in found {
            if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
                continue;
            }
            let full = p.join(&rel);
            let text = std::fs::read_to_string(&full).map_err(|_| {
                warn(
                    &label,
                    &rel_str(&p.strip_prefix(dir).unwrap_or(&p).join(&rel)),
                    Msg::StarterNotText.text().to_string(),
                )
            })?;
            files.push(StarterFile {
                part,
                lang: lang.clone(),
                rel,
                text,
            });
        }
    }
    let s = Starter {
        name,
        title,
        description,
        files,
        origin: Origin::User(dir.to_path_buf()),
    };
    if let Some(lang) = s.missing_marker() {
        let part = match lang {
            None => "ledger".to_string(),
            Some(l) => format!("ledger.{l}"),
        };
        return Err(warn(
            &label,
            &format!("{part}/{MARKER}"),
            Msg::StarterNoLedger.text().to_string(),
        ));
    }
    Ok(s)
}

fn is_link(p: &Path) -> bool {
    std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

/// `dir` の下のふつうのファイルを `base` からの相対パスで集める(リンクはたどらず、フォルダは下へ。FIFO などの
/// ファイルでないものは読まない)。
fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if is_link(&p) {
            continue;
        }
        if p.is_dir() {
            walk(base, &p, out);
        } else if p.is_file() {
            if let Ok(rel) = p.strip_prefix(base) {
                out.push(rel.to_path_buf());
            }
        }
    }
}

/// パスを `/` 区切りの文字にする(警告に出す)。
fn rel_str(p: &Path) -> String {
    p.iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// 一覧(ST-3): 設定の置き場 `config_dir` の `starters/` のユーザーのスターターを名前の順に先に、組み込みを後に。
/// 同じ名前はユーザーのものだけを出す。読めないものは警告にして飛ばす。置き場が無くても組み込みを出す。
pub fn list(config_dir: Option<&Path>) -> (Vec<Starter>, Vec<String>) {
    let mut warns = Vec::new();
    let mut out = Vec::new();
    if let Some(dir) = config_dir.map(|d| d.join(DIR)) {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            // スターターのフォルダそのものはリンクでもよい(設定の置き場は自分のもので、dotfiles がリンクで置く形がある)。
            // `.git` のような隠しフォルダは見ない。
            let mut dirs: Vec<PathBuf> = rd
                .filter_map(|e| Some(e.ok()?.path()))
                .filter(|p| {
                    p.is_dir()
                        && !p
                            .file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.starts_with('.'))
                })
                .collect();
            dirs.sort();
            for d in dirs {
                match read_user(&d) {
                    Ok(s) => out.push(s),
                    Err(w) => warns.push(w),
                }
            }
        }
    }
    for b in builtin() {
        if !out.iter().any(|s| s.name == b.name) {
            out.push(b);
        }
    }
    (out, warns)
}

/// 名前で引く。
pub fn find<'a>(list: &'a [Starter], name: &str) -> Option<&'a Starter> {
    list.iter().find(|s| s.name == name)
}

/// 今の言語の(名前, 説明)。その言語が無ければ英語、それも無ければ(フォルダの名前, 空)。
pub fn describe(s: &Starter, lang: Lang) -> (String, String) {
    let pick = |m: &BTreeMap<String, String>| {
        m.get(lang.code())
            .or_else(|| m.get(Lang::En.code()))
            .cloned()
    };
    (
        pick(&s.title).unwrap_or_else(|| s.name.clone()),
        pick(&s.description).unwrap_or_default(),
    )
}

/// `lang` で写すファイル(ST-1): 部分ごとに `<部分>.<言語>/` があればそれだけ、無ければ接尾の無いもの。
/// 見本は `with_samples` のときだけ。
fn picked(s: &Starter, lang: Lang, with_samples: bool) -> Vec<&StarterFile> {
    let parts: &[Part] = if with_samples {
        &[Part::Ledger, Part::Sample]
    } else {
        &[Part::Ledger]
    };
    let code = lang.code();
    let mut out = Vec::new();
    for part in parts {
        let has_lang = s
            .files
            .iter()
            .any(|f| f.part == *part && f.lang.as_deref() == Some(code));
        out.extend(s.files.iter().filter(|f| {
            f.part == *part
                && if has_lang {
                    f.lang.as_deref() == Some(code)
                } else {
                    f.lang.is_none()
                }
        }));
    }
    out
}

/// スターターから台帳を作る(ST-4)。`root` が無ければ作る。書く先に同じパスのファイルが1つでもあれば、どれも書かずに
/// 理由(WS-7 と同じ `WsInitExists`)。`ledger/` は素のまま、`samples/` は CE-32 の変数(`today_now` は
/// `clock::today_now` の(今日の日数, 地域の秒))を埋めて書く。作ったファイルのパスを返す。
pub fn create(
    root: &Path,
    s: &Starter,
    lang: Lang,
    with_samples: bool,
    today_now: (i64, i64),
) -> Result<Vec<PathBuf>, String> {
    if root.exists() && !root.is_dir() {
        return Err(Msg::StarterNotFolder.fill(&[&root.display()]));
    }
    let mut plan = Vec::new();
    for f in picked(s, lang, with_samples) {
        let p = root.join(&f.rel);
        if p.exists() || is_link(&p) {
            return Err(Msg::WsInitExists.fill(&[&p.display()]));
        }
        // 途中のフォルダがリンクなら書かない(既にある根の中の `Tasks -> どこか` を create_dir_all がたどり、台帳の外に
        // 書いてしまうため)。
        let mut acc = root.to_path_buf();
        for c in f.rel.parent().into_iter().flat_map(Path::components) {
            acc.push(c);
            if is_link(&acc) {
                return Err(Msg::StarterLink.fill(&[&acc.display()]));
            }
        }
        plan.push((p, f));
    }
    std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let (today, now) = today_now;
    let minutes = now.rem_euclid(86_400) / 60;
    let mut made = Vec::new();
    for (p, f) in plan {
        let text = match f.part {
            Part::Ledger => f.text.clone(),
            Part::Sample => {
                let name = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let folder = f.rel.parent().map(rel_str).unwrap_or_default();
                newnote::expand(
                    &f.text,
                    &newnote::Vars {
                        today,
                        minutes,
                        name,
                        folder,
                    },
                )
            }
        };
        newnote::create(&p, text.as_bytes()).map_err(|e| format!("{}: {e}", p.display()))?;
        made.push(p);
    }
    Ok(made)
}

#[cfg(test)]
#[path = "test_starter_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_starter_more_unit.rs"]
mod tests_more;
