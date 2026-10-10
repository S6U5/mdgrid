//! CSV・TSV のファイルの読み書きの芯(SC-15・SC-16)。画面に依存しない。
//! 読むときは、行ごと・値ごとのバイトの位置(引用符の有無を含む)を覚える。書くときは、変える値のバイトの範囲だけを
//! 置き換え、ほかの行・値・引用符の付け方・改行の形(LF・CRLF)・最後の改行の有無は1バイトも変えない。
//! 値に区切り・引用符・改行が入るときだけ引用符で囲む(`"` は `""` にする。1列の表の空の値は、空の行として読み飛ばされ
//! ないよう `""`)。改行は LF と CRLF を受け、CR だけの改行のファイルは理由を出して読まない。文字は UTF-8 だけを受け、先頭の BOM は
//! 読み飛ばす(書くときも残す)。空の行は行にしない(位置はそのまま)。

/// 1つの値。`start..end` は引用符を含めた元のバイトの範囲。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub start: usize,
    pub end: usize,
    pub quoted: bool,
    pub value: String,
}

/// 1行。`start..end` は改行を含まない範囲。`line` はファイルの何行目か(1から。引用符の中の改行も数える)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<Field>,
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

/// 読んだファイル。`records` はデータの行(1行目の列の名前を除く)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvFile {
    pub delim: u8,
    pub header: Vec<String>,
    pub records: Vec<Record>,
    /// 改行の形(ファイルの最初の改行に合わせる。改行が無ければ LF)。
    pub newline: &'static str,
    /// ファイルが改行で終わるか。
    pub ends_with_newline: bool,
}

/// 区切りを拡張子から(`.tsv` はタブ、ほかは `,`)。
pub fn delim_of(path: &std::path::Path) -> u8 {
    match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("tsv") => b'\t',
        _ => b',',
    }
}

/// バイトを読む。UTF-8 でない・列の名前の行が無いときは理由。
pub fn parse(bytes: &[u8], delim: u8) -> Result<CsvFile, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| crate::i18n::Msg::CsvNotUtf8.text().to_string())?;
    let start = if text.starts_with('\u{feff}') { 3 } else { 0 };
    let b = bytes;
    let mut records: Vec<Record> = Vec::new();
    let mut i = start;
    let mut line = 1;
    let mut newline: Option<&'static str> = None;
    let n = b.len();
    while i < n {
        // 空の行は行にしない。
        if b[i] == b'\n' || (b[i] == b'\r' && b.get(i + 1) == Some(&b'\n')) {
            let w = if b[i] == b'\r' { 2 } else { 1 };
            newline.get_or_insert(if w == 2 { "\r\n" } else { "\n" });
            i += w;
            line += 1;
            continue;
        }
        let rec_start = i;
        let rec_line = line;
        let mut fields = Vec::new();
        loop {
            let f_start = i;
            if i < n && b[i] == b'"' {
                // 引用符の値。`""` は `"`。閉じの引用符のあとは区切りか改行か終わり。
                let mut v = Vec::new();
                i += 1;
                loop {
                    if i >= n {
                        return Err(crate::i18n::Msg::CsvUnclosedQuote.fill(&[&rec_line]));
                    }
                    if b[i] == b'"' {
                        if b.get(i + 1) == Some(&b'"') {
                            v.push(b'"');
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    v.push(b[i]);
                    i += 1;
                }
                fields.push(Field {
                    start: f_start,
                    end: i,
                    quoted: true,
                    value: String::from_utf8_lossy(&v).into_owned(),
                });
            } else {
                while i < n
                    && b[i] != delim
                    && b[i] != b'\n'
                    && !(b[i] == b'\r' && b.get(i + 1) == Some(&b'\n'))
                {
                    // CR だけの改行は、LF・CRLF と違う改行の形で、書き戻しで守れないので読まない。
                    if b[i] == b'\r' {
                        return Err(crate::i18n::Msg::CsvCrOnly.fill(&[&line]));
                    }
                    i += 1;
                }
                fields.push(Field {
                    start: f_start,
                    end: i,
                    quoted: false,
                    value: String::from_utf8_lossy(&b[f_start..i]).into_owned(),
                });
            }
            if i < n && b[i] == delim {
                i += 1;
                continue;
            }
            break;
        }
        let rec_end = i;
        // 改行(あれば)を越える。引用符の閉じのあとに余計な文字があれば、改行まで読み飛ばして、その行の値に含めない。
        while i < n && b[i] != b'\n' && !(b[i] == b'\r' && b.get(i + 1) == Some(&b'\n')) {
            i += 1;
        }
        if i < n {
            let w = if b[i] == b'\r' { 2 } else { 1 };
            newline.get_or_insert(if w == 2 { "\r\n" } else { "\n" });
            i += w;
            line += 1;
        }
        records.push(Record {
            fields,
            start: rec_start,
            end: rec_end,
            line: rec_line,
        });
    }
    if records.is_empty() {
        return Err(crate::i18n::Msg::CsvNoHeader.text().to_string());
    }
    let head = records.remove(0);
    let header = head.fields.into_iter().map(|f| f.value).collect();
    Ok(CsvFile {
        delim,
        header,
        records,
        newline: newline.unwrap_or("\n"),
        ends_with_newline: bytes.ends_with(b"\n"),
    })
}

/// 値を書く形(区切り・引用符・改行が入るときだけ引用符で囲む)。
pub fn encode(value: &str, delim: u8) -> String {
    let needs = value
        .bytes()
        .any(|c| c == delim || c == b'"' || c == b'\n' || c == b'\r');
    if needs {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// `row` 行目(データの行の添字)・`col` 列目の値を `value` にしたバイト。その値の範囲だけを置き換える。
/// 行・列が無ければ None(列の数が足りない行は書かない)。
pub fn set(bytes: &[u8], file: &CsvFile, row: usize, col: usize, value: &str) -> Option<Vec<u8>> {
    set_many(bytes, file, &[(row, col, value.to_string())])
}

/// 複数の値を (行, 列, 値) で置き換えたバイト。位置はどれも `file`(`bytes` を読んだもの)の位置で、読み直さない
/// (前の置き換えで後ろの位置がずれない)。どれかの行・列が無ければ None。同じ値を2度渡したら後のもの。
pub fn set_many(bytes: &[u8], file: &CsvFile, edits: &[(usize, usize, String)]) -> Option<Vec<u8>> {
    let mut spans: Vec<(usize, usize, String)> = Vec::with_capacity(edits.len());
    for (row, col, value) in edits {
        let rec = file.records.get(*row)?;
        let f = rec.fields.get(*col)?;
        // 値が1つの行を空にすると空の行になり、読み飛ばされて行が消えるので `""` と書く。
        let text = if value.is_empty() && rec.fields.len() == 1 {
            "\"\"".to_string()
        } else {
            encode(value, file.delim)
        };
        spans.retain(|s| s.0 != f.start);
        spans.push((f.start, f.end, text));
    }
    spans.sort_by_key(|s| std::cmp::Reverse(s.0));
    let mut out = bytes.to_vec();
    for (start, end, text) in spans {
        out.splice(start..end, text.into_bytes());
    }
    Some(out)
}

/// 末尾に1行足したバイト(値は列の順。足りない列は空)。ファイルが改行で終わっていなければ、先に改行を足す。
/// 足した行は改行で終える(ファイルの改行の形で)。
pub fn append(bytes: &[u8], file: &CsvFile, values: &[String]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    if !out.is_empty() && !out.ends_with(b"\n") {
        out.extend_from_slice(file.newline.as_bytes());
    }
    let cols = file.header.len().max(values.len());
    let line: Vec<String> = (0..cols)
        .map(|i| encode(values.get(i).map(String::as_str).unwrap_or(""), file.delim))
        .collect();
    let mut line = line.join(&(file.delim as char).to_string());
    // 空の行は行にしない(読み飛ばす)ので、値が全部空の1列の行は `""` と書く。
    if line.is_empty() {
        line.push_str("\"\"");
    }
    out.extend_from_slice(line.as_bytes());
    out.extend_from_slice(file.newline.as_bytes());
    out
}
