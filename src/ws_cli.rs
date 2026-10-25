//! ワークスペースの旗(WS-3・WS-7): `--workspaces`・`--add-to`・`--remove-from`・`--remove-workspace`・
//! `--init-workspace`。開くパス(位置の引数)と重ならない。画面を出さないので、標準出力がパイプでも動く。
//! `workspaces.toml` は設定のフォルダ(views.toml と同じ)に置く。

use mdgrid::i18n::{self, Msg};
use mdgrid::workspace::{self, WsTable};
use mdgrid::{clock, starter};
use std::path::{Path, PathBuf};

/// ワークスペースの操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Op {
    /// 一覧。
    List,
    /// 開くパスの表を足す(ワークスペースの名前、パス、`--as` の表の名前)。
    Add(String, Vec<PathBuf>, Option<String>),
    /// 開くパスの表を外す。
    Remove(String, Vec<PathBuf>),
    /// ワークスペースごと消す。
    Delete(String),
    /// 印を作る(フォルダ)。スターターがあれば、その名前と見本を入れるか(ST-4)。
    Init(Vec<PathBuf>, Option<(String, bool)>),
    /// スターターの一覧(ST-3)。
    Starters,
}

/// 動かす。出す文(標準出力)か、理由(誤り)を返す。
pub(crate) fn run(op: Op, dir: Option<&Path>) -> Result<String, String> {
    let need_dir = || dir.ok_or_else(|| Msg::WsNoConfigDir.text().to_string());
    // 名前の前後の空白は落とし、空の名前は断る(画面の「ワークスペースを作る」と同じ)。
    let named = |n: String| -> Result<String, String> {
        let n = n.trim().to_string();
        if n.is_empty() {
            Err(Msg::PlaceEmptyName.text().to_string())
        } else {
            Ok(n)
        }
    };
    let op = match op {
        Op::Add(n, p, a) => Op::Add(named(n)?, p, a),
        Op::Remove(n, p) => Op::Remove(named(n)?, p),
        Op::Delete(n) => Op::Delete(named(n)?),
        o => o,
    };
    match op {
        Op::List => Ok(list(dir)),
        Op::Add(name, paths, as_name) => {
            if as_name.is_some() && paths.len() > 1 {
                return Err(Msg::WsCliAsOne.text().to_string());
            }
            let d = need_dir()?;
            // 先に全部を確かめる(途中まで足して止まらないように)。
            let reals = paths
                .iter()
                .map(|p| real(p))
                .collect::<Result<Vec<_>, _>>()?;
            let mut out = String::new();
            for path in reals {
                let tname = as_name.clone().unwrap_or_else(|| workspace::stem_of(&path));
                let table = WsTable {
                    name: tname.clone(),
                    path,
                    ..WsTable::default()
                };
                workspace::add(d, &name, table).map_err(|e| Msg::WsSaveError.fill(&[&e]))?;
                out.push_str(&Msg::WsAdded.fill(&[&tname, &name]));
                out.push('\n');
            }
            Ok(out)
        }
        Op::Remove(name, paths) => {
            let d = need_dir()?;
            known(d, &name)?;
            let mut out = String::new();
            for p in paths {
                // 消えたフォルダも、今のフォルダからの相対で外せる。
                let p = std::fs::canonicalize(&p)
                    .unwrap_or_else(|_| std::env::current_dir().map(|c| c.join(&p)).unwrap_or(p));
                let done = workspace::remove(d, &name, Some(&p))
                    .map_err(|e| Msg::WsSaveError.fill(&[&e]))?;
                if !done {
                    return Err(Msg::WsCliNotIn.fill(&[&name, &p.display()]));
                }
                out.push_str(&Msg::WsRemoved.fill(&[&p.display(), &name]));
                out.push('\n');
            }
            Ok(out)
        }
        Op::Delete(name) => {
            let d = need_dir()?;
            known(d, &name)?;
            workspace::remove(d, &name, None).map_err(|e| Msg::WsSaveError.fill(&[&e]))?;
            Ok(Msg::WsCliDeleted.fill(&[&name]) + "\n")
        }
        Op::Init(paths, None) => {
            let mut out = String::new();
            for p in paths {
                let made = workspace::init(&real(&p)?)?;
                out.push_str(&Msg::WsCliInit.fill(&[&made.display()]));
                out.push('\n');
            }
            Ok(out)
        }
        // ST-4: スターターから台帳を作る(根が無ければ作るので、パスの実体は取らない)。
        Op::Init(paths, Some((name, with_samples))) => {
            if paths.len() > 1 {
                return Err(Msg::StarterOne.text().to_string());
            }
            let (list, warns) = starter::list(dir);
            for w in &warns {
                eprintln!("mdgrid: {w}");
            }
            let Some(s) = starter::find(&list, &name) else {
                return Err(Msg::StarterUnknown.fill(&[&name]));
            };
            let root = paths
                .into_iter()
                .next()
                .unwrap_or_else(|| PathBuf::from("."));
            let today_now = clock::today_now(|k| std::env::var(k).ok());
            let made = starter::create(&root, s, i18n::current(), with_samples, today_now)?;
            let mut out = String::new();
            for p in made {
                out.push_str(&Msg::WsCliInit.fill(&[&p.display()]));
                out.push('\n');
            }
            Ok(out)
        }
        // ST-3: 名前と今の言語の名前・説明を1行ずつ。ユーザーのものには印を添える。
        Op::Starters => {
            let (list, warns) = starter::list(dir);
            for w in &warns {
                eprintln!("mdgrid: {w}");
            }
            let lang = i18n::current();
            let mut out = String::new();
            for s in &list {
                let (title, desc) = starter::describe(s, lang);
                out.push_str(&format!("{}  {title}", s.name));
                if matches!(s.origin, starter::Origin::User(_)) {
                    out.push(' ');
                    out.push_str(Msg::StarterUser.text());
                }
                if !desc.is_empty() {
                    out.push_str(" · ");
                    out.push_str(&desc);
                }
                out.push('\n');
            }
            Ok(out)
        }
    }
}

/// その名前のワークスペースがあるか。無ければ理由。
fn known(dir: &Path, name: &str) -> Result<(), String> {
    if workspace::load(dir).0.iter().any(|w| w.name == name) {
        Ok(())
    } else {
        Err(Msg::WsUnknown.fill(&[&name]))
    }
}

/// 一覧: ワークスペースの名前と、その下に表(名前・パス・ビュー)。
fn list(dir: Option<&Path>) -> String {
    let list = dir.map(|d| workspace::load(d).0).unwrap_or_default();
    if list.is_empty() {
        return Msg::WsCliNone.text().to_string() + "\n";
    }
    let mut out = String::new();
    for w in &list {
        out.push_str(&w.name);
        out.push('\n');
        for t in &w.tables {
            out.push_str(&format!("  {}  {}", t.name, t.path.display()));
            if let Some(v) = &t.view {
                out.push_str(&format!(" · {v}"));
            }
            out.push('\n');
        }
    }
    out
}

/// 在るパスの実体。無ければ理由。
fn real(p: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(p).map_err(|_| Msg::WsCliNoPath.fill(&[&p.display()]))
}
