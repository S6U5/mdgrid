//! ファイルを読む口(write-back.refuse-not-text-any-file・cli.read-text-only)。mdgrid が読むどのファイルも、ここで
//! 文字のファイルかを同じ決まり(`frontmatter::text_kind` と UTF-8)で見る。名前だけ `.md`・`.csv`・`.toml` の
//! ほかのファイル(偽装した拡張子)を、中身として使ったり上書きしたりしないため。
//!
//! `std::fs::read`・`std::fs::read_to_string` は、clippy の disallowed-methods(clippy.toml)でここの外では使えない。
//! 読む口を足すときは、ここの関数を通す(docs/design.md「ファイルを読む口」)。

use crate::frontmatter::{text_kind, TextKind};
use crate::i18n::Msg;
use std::io;
use std::path::Path;

/// 文字のファイルでない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotText {
    /// 先頭の `frontmatter::SNIFF_LEN` バイトに NUL がある(write-back.refuse-not-text)。
    Nul,
    /// UTF-16 の BOM で始まる(write-back.refuse-utf16)。
    Utf16,
    /// UTF-8 として読めない。
    NotUtf8,
}

impl NotText {
    /// 今の言語の理由の文。
    pub fn reason(self) -> &'static str {
        match self {
            NotText::Nul => Msg::LockNotText.text(),
            NotText::Utf16 => Msg::LockUtf16.text(),
            NotText::NotUtf8 => Msg::LockNotUtf8.text(),
        }
    }

    fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self.reason())
    }
}

/// `bytes` が文字のファイルなら、その文字。
pub fn check(bytes: &[u8]) -> Result<&str, NotText> {
    match text_kind(bytes) {
        TextKind::NotText => Err(NotText::Nul),
        TextKind::Utf16 => Err(NotText::Utf16),
        TextKind::Text => std::str::from_utf8(bytes).map_err(|_| NotText::NotUtf8),
    }
}

/// `bytes` を文字にする(文字のファイルでなければ理由)。
pub fn into_text(bytes: Vec<u8>) -> Result<String, NotText> {
    check(&bytes)?;
    String::from_utf8(bytes).map_err(|_| NotText::NotUtf8)
}

/// バイトのまま読む。中身を使う前に `check`(ノートは `frontmatter::parse`)で文字のファイルかを見ること。
/// ノートと CSV は、書き戻すときに元のバイトが要るので、これで読む。
#[allow(clippy::disallowed_methods)]
pub fn read_bytes(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// 文字として読む(`std::fs::read_to_string` の代わり)。文字のファイルでなければ `InvalidData` の誤りで、
/// 文は理由(「テキストでないファイル(NUL を含む)」など)。
pub fn read_text(path: impl AsRef<Path>) -> io::Result<String> {
    into_text(read_bytes(path)?).map_err(NotText::into_io)
}

/// 書き換えるために読む。ファイルが無ければ空。読めない・文字のファイルでないときは誤りにする
/// (読めない中身を空とみなして上書きしない。write-back.refuse-not-text-any-file)。
pub fn read_text_for_edit(path: impl AsRef<Path>) -> io::Result<String> {
    let path = path.as_ref();
    match read_text(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            Err(io::Error::new(
                e.kind(),
                Msg::FileNotRewritten.fill(&[&name, &e]),
            ))
        }
        ok => ok,
    }
}

/// 標準入力を文字として読む(`--apply -`)。
pub fn read_stdin_text() -> io::Result<String> {
    let mut bytes = Vec::new();
    io::Read::read_to_end(&mut io::stdin(), &mut bytes)?;
    into_text(bytes).map_err(NotText::into_io)
}
