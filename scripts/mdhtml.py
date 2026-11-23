"""ブログの記事の Markdown を HTML にする(scripts/pages.py が使う。site.blog-markdown)。標準ライブラリだけを使う。

扱う形: 見出し(# 〜 ######)・段落・箇条書き(-・*・+ と 1.、字下げで入れ子)・表(| と区切りの行。:--- で寄せ)・
コードのブロック(``` か ~~~、言語の名前つき)・引用(>)・横線(---)・リンク・画像・<https://…> のリンク・
強調(** と *)・取り消し(~~)・コードの字(`)・行末の2つの空白か \\ での改行・\\ で字をそのまま出すこと。

生の HTML は書けない(< と & は文字として出す)。リンクと画像の行き先は、http・https・mailto と、方式の無い
相対の道筋・# だけを受け、ほか(javascript: など)はリンクにせず文字のまま出す。
"""

import html
import re

FENCE = re.compile(r"^( {0,3})(`{3,}|~{3,})\s*([^`\s]*)\s*$")
HEADING = re.compile(r"^ {0,3}(#{1,6})(?:\s+(.*?))?(?:\s+#+)?\s*$")
RULE = re.compile(r"^ {0,3}([-*_])(?:\s*\1){2,}\s*$")
ITEM = re.compile(r"^( *)([-*+]|\d{1,9}[.)])(?:\s+(.*)|$)")
QUOTE = re.compile(r"^ {0,3}>\s?(.*)$")
TABLE_RULE = re.compile(r"^\s*\|?\s*:?-+:?\s*(?:\|\s*:?-+:?\s*)*\|?\s*$")
SCHEME = re.compile(r"^([A-Za-z][A-Za-z0-9+.-]*):")
SAFE_SCHEMES = {"http", "https", "mailto"}

# 字の中に置く印(書いた文に出ない私用の字)。コードの字・リンク・画像を先に組んで印に替え、
# 強調の置き換えが中に入らないようにする。
HOLD, HOLD_END, BREAK = "", "", ""


def convert(text: str) -> str:
    """Markdown の本文を HTML にする。"""
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    return "\n".join(_blocks(lines))


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _table_start(lines: list, i: int) -> bool:
    return (
        "|" in lines[i]
        and i + 1 < len(lines)
        and "-" in lines[i + 1]
        and bool(TABLE_RULE.match(lines[i + 1]))
    )


def _starts_block(lines: list, i: int) -> bool:
    """段落を切る行か(見出し・コードのブロック・横線・引用・箇条書き・表)。"""
    line = lines[i]
    item = ITEM.match(line)
    return bool(
        FENCE.match(line)
        or HEADING.match(line)
        or RULE.match(line)
        or QUOTE.match(line)
        or (item and _indent(line) < 4 and item.group(3))
        or _table_start(lines, i)
    )


def _blocks(lines: list) -> list:
    out = []
    i, n = 0, len(lines)
    while i < n:
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        m = FENCE.match(line)
        if m:
            indent, fence, lang = m.groups()
            body = []
            i += 1
            while i < n:
                close = lines[i].strip()
                if (
                    close
                    and set(close) == {fence[0]}
                    and len(close) >= len(fence)
                    and _indent(lines[i]) < 4
                ):
                    break
                body.append(lines[i][min(len(indent), _indent(lines[i])) :])
                i += 1
            i += 1
            cls = f' class="language-{html.escape(lang)}"' if lang else ""
            out.append(
                f"<pre><code{cls}>{html.escape(chr(10).join(body))}</code></pre>"
            )
            continue
        m = HEADING.match(line)
        if m:
            level = len(m.group(1))
            out.append(f"<h{level}>{_inline(m.group(2) or '')}</h{level}>")
            i += 1
            continue
        if RULE.match(line):
            out.append("<hr>")
            i += 1
            continue
        if QUOTE.match(line):
            inner = []
            while i < n and lines[i].strip():
                q = QUOTE.match(lines[i])
                inner.append(q.group(1) if q else lines[i])
                i += 1
            out.append("<blockquote>\n" + "\n".join(_blocks(inner)) + "\n</blockquote>")
            continue
        if _table_start(lines, i):
            html_, i = _table(lines, i)
            out.append(html_)
            continue
        m = ITEM.match(line)
        if m and _indent(line) < 4:
            html_, i = _list(lines, i)
            out.append(html_)
            continue
        para = [line]
        i += 1
        while i < n and lines[i].strip() and not _starts_block(lines, i):
            para.append(lines[i])
            i += 1
        out.append(f"<p>{_paragraph(para)}</p>")
    return out


def _cells(row: str) -> list:
    row = row.strip()
    row = row.removeprefix("|")
    if row.endswith("|") and not row.endswith("\\|"):
        row = row[:-1]
    return [c.strip() for c in re.split(r"(?<!\\)\|", row)]


def _table(lines: list, i: int):
    head = _cells(lines[i])
    aligns = []
    for c in _cells(lines[i + 1]):
        left, right = c.startswith(":"), c.endswith(":")
        aligns.append(
            "center" if left and right else "right" if right else "left" if left else ""
        )

    def row(cells, tag):
        tds = []
        for k in range(len(head)):
            text = cells[k] if k < len(cells) else ""
            a = aligns[k] if k < len(aligns) else ""
            style = f' style="text-align:{a}"' if a else ""
            tds.append(f"<{tag}{style}>{_inline(text)}</{tag}>")
        return "<tr>" + "".join(tds) + "</tr>"

    body = []
    i += 2
    while i < len(lines) and lines[i].strip() and "|" in lines[i]:
        body.append(row(_cells(lines[i]), "td"))
        i += 1
    tbody = f"\n<tbody>\n{chr(10).join(body)}\n</tbody>" if body else ""
    return (
        f'<div class="table"><table>\n<thead>\n{row(head, "th")}\n</thead>{tbody}\n</table></div>',
        i,
    )


def _list(lines: list, i: int):
    """箇条書き1つ(同じ字下げで同じ種類の項目が続くところまで)。項目の中の字下げした行は、項目の中身として読み直す。"""
    first = ITEM.match(lines[i])
    base = _indent(lines[i])
    ordered = first.group(2)[0].isdigit()
    start = int(first.group(2)[:-1]) if ordered else 1
    items = []  # (中身の行, 空の行をはさんだか)
    width = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        m = ITEM.match(line)
        if m and _indent(line) == base and m.group(2)[0].isdigit() == ordered:
            text = m.group(3) or ""
            width = base + len(m.group(2)) + 1
            items.append([[text], False])
            i += 1
            continue
        if not line.strip():
            j = i + 1
            while j < n and not lines[j].strip():
                j += 1
            nxt = ITEM.match(lines[j]) if j < n else None
            sibling = (
                nxt
                and _indent(lines[j]) == base
                and nxt.group(2)[0].isdigit() == ordered
            )
            if j < n and (_indent(lines[j]) > base or sibling):
                items[-1][1] = True
                items[-1][0].append("")
                i += 1
                continue
            break
        if _indent(line) > base:
            items[-1][0].append(line[min(_indent(line), width) :])
            i += 1
            continue
        if not _starts_block(lines, i):
            # 字下げの無い続きの行は、項目の段落の続き。
            items[-1][0].append(line)
            i += 1
            continue
        break
    tag = "ol" if ordered else "ul"
    attr = f' start="{start}"' if ordered and start != 1 else ""
    lis = []
    for content, loose in items:
        blocks = _blocks(content)
        if not loose:
            blocks = [
                b[3:-4] if b.startswith("<p>") and b.endswith("</p>") else b
                for b in blocks
            ]
        lis.append("<li>" + "\n".join(blocks) + "</li>")
    return f"<{tag}{attr}>\n" + "\n".join(lis) + f"\n</{tag}>", i


def _paragraph(lines: list) -> str:
    parts = []
    for k, line in enumerate(lines):
        last = k == len(lines) - 1
        hard = not last and (line.endswith("  ") or line.rstrip().endswith("\\"))
        line = line.strip()
        if hard and line.endswith("\\"):
            line = line[:-1]
        parts.append(line + (BREAK if hard else ""))
    return _inline("\n".join(parts)).replace(BREAK, "<br>")


def _safe(url: str) -> bool:
    m = SCHEME.match(url)
    return not m or m.group(1).lower() in SAFE_SCHEMES


def _emphasis(text: str) -> str:
    text = re.sub(r"\*\*(?=\S)(.+?)(?<=\S)\*\*", r"<strong>\1</strong>", text)
    text = re.sub(
        r"(?<![*\w])\*(?=[^\s*])(.+?)(?<=[^\s*])\*(?![*\w])", r"<em>\1</em>", text
    )
    return re.sub(r"~~(?=\S)(.+?)(?<=\S)~~", r"<del>\1</del>", text)


IMAGE = re.compile(r"!\[([^\]]*)\]\(\s*([^\s)]+)(?:\s+&quot;(.*?)&quot;)?\s*\)")
LINK = re.compile(r"\[([^\]]+)\]\(\s*([^\s)]+)(?:\s+&quot;(.*?)&quot;)?\s*\)")


def _inline(text: str) -> str:
    """行の中の形。字は先に HTML の文字にしてから、リンクや強調の要素を組む。"""
    held = []

    def hold(s: str) -> str:
        held.append(s)
        return f"{HOLD}{len(held) - 1}{HOLD_END}"

    text = re.sub(
        r"(`+)(.+?)\1",
        lambda m: hold(f"<code>{html.escape(m.group(2).strip())}</code>"),
        text,
    )
    text = re.sub(
        r"\\([\\`*_{}\[\]()#+\-.!|>~<])", lambda m: hold(html.escape(m.group(1))), text
    )
    text = re.sub(
        r"<(https?://[^\s<>]+)>",
        lambda m: hold(
            f'<a href="{html.escape(m.group(1))}">{html.escape(m.group(1))}</a>'
        ),
        text,
    )
    text = html.escape(text)

    def image(m):
        alt, url, title = m.groups()
        if not _safe(url):
            return m.group(0)
        t = f' title="{title}"' if title else ""
        return hold(f'<img src="{url}" alt="{alt}"{t} loading="lazy">')

    def link(m):
        label, url, title = m.groups()
        if not _safe(url):
            return m.group(0)
        t = f' title="{title}"' if title else ""
        return hold(f'<a href="{url}"{t}>{_emphasis(label)}</a>')

    text = IMAGE.sub(image, text)
    text = LINK.sub(link, text)
    text = _emphasis(text)
    pattern = re.compile(f"{HOLD}(\\d+){HOLD_END}")
    while pattern.search(text):
        text = pattern.sub(lambda m: held[int(m.group(1))], text)
    return text
