//! 信用できない YAML(.base とノートのフロントマター)を、再帰する読み手に渡す前の見張り(BV-8・WB-5)。
//! .base(base)とフロントマター(frontmatter)の両方が使うので、どちらにも頼らない所に置く。

use crate::i18n::Msg;
use std::collections::HashMap;

/// 別名を展開して増えるノードの数の上限(saphyr は別名ごとにノードを複製するので、重ねた別名で膨らむ)。
const MAX_ALIAS_NODES: u64 = 10_000;
/// シーケンス・マップの入れ子の深さの上限(読み手の再帰がスタックを使い切らないように)。
const MAX_DEPTH: usize = 64;

/// 読む前に saphyr_parser のイベントを数え、別名を展開して増えるノードの数か、入れ子の深さが上限を
/// 超えたら Err(信用できない .base・ノートの中身を、再帰する読み手に渡さない)。
/// YAML の誤りはここでは見ない(load_from_str が理由を出す)。
pub(crate) fn check_aliases(text: &str) -> Result<(), String> {
    use saphyr_parser::{Event, Parser};
    // アンカーの id → そのノードを展開したノードの数。
    let mut sizes: HashMap<usize, u64> = HashMap::new();
    // 開いているシーケンス・マップ(アンカーの id, 中のノードの数)。
    let mut open: Vec<(usize, u64)> = Vec::new();
    let mut expanded: u64 = 0;
    for ev in Parser::new_from_str(text) {
        let Ok((ev, _)) = ev else {
            return Ok(());
        };
        let (n, anchor) = match ev {
            Event::Scalar(_, _, a, _) => (1, a),
            Event::Alias(id) => {
                let n = sizes.get(&id).copied().unwrap_or(1);
                expanded = expanded.saturating_add(n);
                if expanded > MAX_ALIAS_NODES {
                    return Err(Msg::BaseAliasTooBig.fill(&[&MAX_ALIAS_NODES]));
                }
                (n, 0)
            }
            Event::SequenceStart(a, _) | Event::MappingStart(a, _) => {
                open.push((a, 1));
                if open.len() > MAX_DEPTH {
                    return Err(Msg::BaseTooDeep.fill(&[&MAX_DEPTH]));
                }
                continue;
            }
            Event::SequenceEnd | Event::MappingEnd => match open.pop() {
                Some(x) => (x.1, x.0),
                None => continue,
            },
            _ => continue,
        };
        if anchor != 0 {
            sizes.insert(anchor, n);
        }
        if let Some(top) = open.last_mut() {
            top.1 = top.1.saturating_add(n);
        }
    }
    Ok(())
}
