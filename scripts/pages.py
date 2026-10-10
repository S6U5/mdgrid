#!/usr/bin/env python3
"""GitHub Pages のサイトを作る(日英)。

    python3 scripts/pages.py [出力のフォルダ(既定 site)] [--shots 画面のフォルダ(既定 target/shots)]

先に画面を撮る(本物の mdgrid を疑似端末で動かす。cargo build --release のあと):

    python3 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang ja --env LANG=ja_JP.UTF-8
    python3 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang en --env LANG=en_US.UTF-8

作るもの:
- index.html    言語を選ぶ入口(x-default)。覚えた選択かブラウザの言語で ja/・en/ へ移る
- ja/・en/      入口(説明書・カタログ・画面の一覧へ)と画面の一覧(gallery.html)。hreflang で互いを示す
- manual/ja・en 説明書(mdBook)。docs/manual/<言語>/*.md と参照のページ(キー・設定・安全・.base)を
                1冊にまとめ、ページの間のリンクを本の中に直す。本の外を指すリンクは GitHub の上のファイルへ
- catalog/      見た目のカタログ(docs/catalog/index.html をそのまま。SR-38)
- gallery.html  前の画面の一覧の URL(言語を決めて ja/・en/ の画面の一覧へ移る)
- images/ja・en 撮った画面の SVG

画面の画像はリポに入れない(撮るたびに履歴が太るため)。mdbook が要る。Python 3.11 以上(tomllib)。
"""

import html
import json
import os
import re
import subprocess
import shutil
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LANGS = ("ja", "en")
REPO = "https://github.com/S6U5/mdgrid"
RELEASE = REPO + "/releases/latest/download/"

TEXT = {
    "ja": {
        "title": "mdgrid",
        "lead": "フロントマター付きの Markdown のノートを、ターミナルの表で見て直す道具。",
        "manual": "説明書",
        "manual_desc": "はじめかた・考え方・作業の手引き・見た目・困ったとき、キーと設定の一覧。",
        "catalog": "見た目のカタログ",
        "catalog_desc": "テーマと部品の形を触って選び、config.toml に貼る設定を作る。",
        "gallery": "画面の一覧",
        "gallery_desc": "本物の mdgrid の画面を、場面ごとに見る(説明書と同じ写し)。",
        "repo": "リポジトリ",
        "demos": "動き(GIF)",
        "demos_desc": "リリースに付けた録画。",
        "back": "入口へ",
        "filter": "場面を絞る",
        "count": "{n} 場面",
        "keys": "押したキー",
        "lang": "言語",
    },
    "en": {
        "title": "mdgrid",
        "lead": "View and edit Markdown notes with frontmatter as a table in the terminal.",
        "manual": "Manual",
        "manual_desc": "Getting started, concepts, how-tos, appearance, troubleshooting, and the key and setting lists.",
        "catalog": "Look catalog",
        "catalog_desc": "Try themes and part shapes, then copy the settings into config.toml.",
        "gallery": "Screen gallery",
        "gallery_desc": "Real mdgrid screens, scene by scene (the same shots as the manual).",
        "repo": "Repository",
        "demos": "In motion (GIF)",
        "demos_desc": "Recordings attached to the latest release.",
        "back": "Home",
        "filter": "Filter scenes",
        "count": "{n} scenes",
        "keys": "Keys pressed",
        "lang": "Language",
    },
}

DEMOS = [
    ("demo-values.gif", "型ごとの入力と保存", "Typed inputs and saving"),
    ("demo-new-note.gif", "新しいノートのフォーム", "New note form"),
    ("demo-export.gif", "CSV と --print に書き出す", "Export to CSV and --print"),
    ("demo-places.gif", "登録した表を切り替える", "Switching registered tables"),
    ("demo-relations.gif", "リレーション", "Relations"),
    ("demo-relmap.gif", "関係マップ", "Relation map"),
    ("demo-workspace.gif", "ワークスペース", "Workspaces"),
    ("demo-view-tabs.gif", "ビューのタブを整える", "Arranging view tabs"),
    ("demo-look.gif", "見た目を選んで保存", "Picking and saving a look"),
    ("demo-tree.gif", "親子の字下げ", "Parent/child indent"),
    ("demo-wbs.gif", "WBS", "WBS"),
    ("demo-themes.gif", "色のテーマ", "Color themes"),
]

CSS = """
:root{--bg:#f6f7f9;--surface:#fff;--fg:#1c2027;--muted:#5f6774;--line:#dcdfe5;--accent:#3657d6;--on-accent:#fff;color-scheme:light}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#111318;--surface:#191c23;--fg:#e3e5ea;--muted:#9aa1ad;--line:#2a2f39;--accent:#8fa5ff;--on-accent:#111318;color-scheme:dark}}
:root[data-theme="dark"]{--bg:#111318;--surface:#191c23;--fg:#e3e5ea;--muted:#9aa1ad;--line:#2a2f39;--accent:#8fa5ff;--on-accent:#111318;color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.6 system-ui,-apple-system,"Segoe UI","Hiragino Sans","Noto Sans JP",sans-serif}
.wrap{max-width:1180px;margin:0 auto;padding:28px 16px 64px;display:grid;gap:24px}
h1{margin:0;font-size:28px}h2{margin:0;font-size:18px}
p{margin:0}.muted{color:var(--muted)}
a{color:var(--accent)}
.bar{display:flex;flex-wrap:wrap;gap:8px 14px;align-items:center}
.chips{display:flex;gap:6px}
button.opt{font:inherit;font-size:13px;padding:3px 10px;border-radius:999px;border:1px solid var(--line);background:transparent;color:var(--fg);cursor:pointer}
.opt{font:inherit;font-size:13px;padding:3px 10px;border-radius:999px;border:1px solid var(--line);color:var(--fg);text-decoration:none}
.opt.on{background:var(--accent);border-color:var(--accent);color:var(--on-accent)}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:14px}
.card{display:grid;gap:6px;background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:16px;text-decoration:none;color:var(--fg)}
.card:hover{border-color:var(--accent)}
.shots{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(100%,520px),1fr));gap:16px}
figure{margin:0;background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:12px;display:grid;gap:8px;min-width:0}
figure img{width:100%;height:auto;border-radius:8px;display:block}
figcaption{display:grid;gap:4px;font-size:14px}
figcaption b{font-size:15px}
code{font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12.5px}
input[type=search]{font:inherit;padding:4px 10px;border-radius:8px;border:1px solid var(--line);background:var(--surface);color:var(--fg);min-width:0;width:min(100%,320px)}
"""

SITE = "https://s6u5.github.io/mdgrid/"
NAMES = {"ja": "日本語", "en": "English"}
KEY = "mdgrid-site-lang"

# 言語の切り替え(data-pick)を押したら、選んだ言語を覚える(入口の振り分けが使う)。読み書きの失敗は無視。
REMEMBER = """
document.addEventListener("click", function (e) {
  var a = e.target.closest && e.target.closest("a[data-pick]");
  if (a) { try { localStorage.setItem("%s", a.getAttribute("data-pick")); } catch (x) {} }
});
""" % KEY

# 画面の一覧の絞り込み。
FIND = """
var f = document.getElementById("find");
if (f) f.addEventListener("input", function () {
  var w = f.value.trim().toLowerCase();
  document.querySelectorAll("figure[data-search]").forEach(function (x) { x.hidden = w !== "" && x.dataset.search.indexOf(w) < 0; });
});
"""


def other_lang(l: str) -> str:
    return "en" if l == "ja" else "ja"


def alternates(rel: str) -> str:
    """`rel` は ja の版のサイトの中の道筋(例 "ja/gallery.html")。ja・en と x-default(入口)の hreflang。"""
    out = [f'<link rel="alternate" hreflang="{l}" href="{SITE}{rel.replace("ja/", l + "/", 1)}">' for l in LANGS]
    out.append(f'<link rel="alternate" hreflang="x-default" href="{SITE}">')
    return "\n".join(out)


def page(lang: str, title: str, body: str, desc: str, head: str = "", script: str = "") -> str:
    return f"""<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title>
<meta name="description" content="{html.escape(desc)}">
{head}
<style>{CSS}</style>
</head>
<body>
<div class="wrap">
{body}
</div>
<script>{REMEMBER}{script}</script>
</body>
</html>
"""


def lang_bar(lang: str, file: str) -> str:
    """同じページの別の言語へのリンク(今の言語は印だけ)。"""
    t = TEXT[lang]
    items = []
    for l in LANGS:
        if l == lang:
            items.append(f'<span class="opt on" aria-current="true">{NAMES[l]}</span>')
        else:
            items.append(f'<a class="opt" href="../{l}/{file}" hreflang="{l}" lang="{l}" data-pick="{l}">{NAMES[l]}</a>')
    return f'<div class="bar"><span class="muted">{t["lang"]}</span><div class="chips">{"".join(items)}</div></div>'


def build(out: Path, shots_dir: Path) -> None:
    scen = tomllib.loads((ROOT / "docs/manual-scenarios.toml").read_text(encoding="utf-8"))
    if out.exists():
        shutil.rmtree(out)
    (out / "catalog").mkdir(parents=True)
    shutil.copy(ROOT / "docs/catalog/index.html", out / "catalog/index.html")
    shots = []
    for s in scen["shot"]:
        sid = s["id"]
        missing = [l for l in LANGS if not (shots_dir / l / "images" / f"{sid}.svg").exists()]
        if missing:
            raise SystemExit(f"画面が無い: {sid}({shots_dir}/{missing[0]}/images。先に tui_shot.py で撮る)")
        shots.append(s)
    for l in LANGS:
        d = out / "images" / l
        d.mkdir(parents=True)
        for s in shots:
            shutil.copy(shots_dir / l / "images" / f"{s['id']}.svg", d / f"{s['id']}.svg")
        book(l, out / "manual" / l, shots_dir / l / "images")
        decorate_book(l, out / "manual" / l)

    # リリースに付いている録画だけを並べる(workflow が名前の一覧を MDGRID_RELEASE_ASSETS に渡す。
    # 無ければ全部)。新しい台本の録画は次のリリースまで無いので、切れたリンクを出さない。
    names = os.environ.get("MDGRID_RELEASE_ASSETS")
    have = set(names.split()) if names else None
    shown = [d for d in DEMOS if have is None or d[0] in have]

    for l in LANGS:
        (out / l).mkdir()
        (out / l / "index.html").write_text(landing(l, shown), encoding="utf-8")
        (out / l / "gallery.html").write_text(gallery(l, shots), encoding="utf-8")
    (out / "index.html").write_text(chooser(), encoding="utf-8")
    # 前の画面の一覧の URL(README の古いリンク)は、言語を決めて新しい URL へ。
    (out / "gallery.html").write_text(chooser("gallery.html"), encoding="utf-8")
    print(json.dumps({"out": str(out), "shots": len(shots)}))


def chooser(file: str = "") -> str:
    """入口(x-default): 覚えた選択か、?lang= か、ブラウザの言語(navigator.languages)で /ja/・/en/ へ移る。
    合わなければ選ぶ画面のまま。移るのは location.replace(戻るで戻されない)。JS が無くても両方へのリンクが見える。"""
    links = "".join(
        f'<a class="card" href="{l}/{file}" hreflang="{l}" lang="{l}" data-pick="{l}"><h2>{NAMES[l]}</h2>'
        f'<p class="muted">{html.escape(TEXT[l]["lead"])}</p></a>'
        for l in LANGS
    )
    route = """
(function () {
  var langs = ["ja", "en"], pick = null;
  try { var q = new URLSearchParams(location.search).get("lang"); if (langs.indexOf(q) >= 0) pick = q; } catch (e) {}
  if (!pick) { try { var s = localStorage.getItem("%s"); if (langs.indexOf(s) >= 0) pick = s; } catch (e) {} }
  if (!pick) {
    var nav = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language || ""];
    for (var i = 0; i < nav.length && !pick; i++) { var p = String(nav[i]).slice(0, 2).toLowerCase(); if (langs.indexOf(p) >= 0) pick = p; }
  }
  if (pick) location.replace(pick + "/%s" + location.hash);
})();
""" % (KEY, file)
    body = f"""
<h1>mdgrid</h1>
<p class="muted">Choose a language · 言語を選ぶ</p>
<div class="cards">{links}</div>
"""
    head = alternates("ja/" + file).replace(f'hreflang="x-default" href="{SITE}"', f'hreflang="x-default" href="{SITE}{file}"')
    return page("en", "mdgrid", body, TEXT["en"]["lead"], head, route)


def landing(lang: str, shown) -> str:
    t = TEXT[lang]
    i = 1 if lang == "ja" else 2
    demos = "".join(
        f'<a class="card" href="{RELEASE}{d[0]}"><b>{html.escape(d[i])}</b><code>{d[0]}</code></a>' for d in shown
    )
    body = f"""
<h1>mdgrid</h1>
<p class="muted">{html.escape(t["lead"])}</p>
{lang_bar(lang, "")}
<div class="cards">
  <a class="card" href="../manual/{lang}/"><h2>{t["manual"]}</h2><p class="muted">{html.escape(t["manual_desc"])}</p></a>
  <a class="card" href="../catalog/?lang={lang}"><h2>{t["catalog"]}</h2><p class="muted">{html.escape(t["catalog_desc"])}</p></a>
  <a class="card" href="gallery.html"><h2>{t["gallery"]}</h2><p class="muted">{html.escape(t["gallery_desc"])}</p></a>
  <a class="card" href="{REPO}"><h2>{t["repo"]}</h2><p class="muted"><code>{REPO}</code></p></a>
</div>
{f'<h2>{t["demos"]}</h2><p class="muted">{t["demos_desc"]}</p><div class="cards">{demos}</div>' if shown else ""}
"""
    return page(lang, "mdgrid", body, t["lead"], alternates("ja/"))


def gallery(lang: str, shots) -> str:
    t = TEXT[lang]
    figs = []
    for s in shots:
        sid = s["id"]
        # 題と説明は {ja, en} の表か、両方に同じ文字。
        pick = lambda v: v.get(lang, sid) if isinstance(v, dict) else v
        title = pick(s.get("title", sid))
        text = pick(s.get("text", ""))
        keys = " ".join(s.get("keys", []))
        search = " ".join([sid, keys, title, text]).lower()
        k = f'<code class="muted">{t["keys"]}: {html.escape(keys)}</code>' if keys else ""
        figs.append(
            f'<figure id="{sid}" data-search="{html.escape(search)}">'
            f'<a href="../images/{lang}/{sid}.svg"><img loading="lazy" alt="{html.escape(title)}" src="../images/{lang}/{sid}.svg"></a>'
            f'<figcaption><b>{html.escape(title)}</b><span class="muted">{html.escape(text)}</span>{k}</figcaption></figure>'
        )
    body = f"""
<p><a href="./">← {t["back"]}</a></p>
<h1>{t["gallery"]}</h1>
<p class="muted">{html.escape(t["gallery_desc"])} {t["count"].format(n=len(shots))}</p>
{lang_bar(lang, "gallery.html")}
<input id="find" type="search" aria-label="{t["filter"]}" placeholder="{t["filter"]}">
<div class="shots">{''.join(figs)}</div>
"""
    return page(lang, f'{t["gallery"]} · mdgrid', body, t["gallery_desc"], alternates("ja/gallery.html"), FIND)


def decorate_book(lang: str, dest: Path) -> None:
    """mdBook の各ページに、hreflang(ja・en・x-default)と、右上の同じページの別の言語への切り替えを書き足す。"""
    other = other_lang(lang)
    for f in dest.glob("*.html"):
        name = f.name
        text = f.read_text(encoding="utf-8")
        alt = "\n".join(
            [f'<link rel="alternate" hreflang="{l}" href="{SITE}manual/{l}/{name}">' for l in LANGS]
            + [f'<link rel="alternate" hreflang="x-default" href="{SITE}">']
        )
        text = text.replace("</head>", alt + "\n</head>", 1)
        switch = (
            f'<a class="lang-switch" href="../{other}/{name}" hreflang="{other}" lang="{other}" data-pick="{other}" '
            f'title="{NAMES[other]}" style="font-size:14px;margin-inline-end:10px;text-decoration:none">{NAMES[other]}</a>'
        )
        text = text.replace('<div class="right-buttons">', '<div class="right-buttons">' + switch, 1)
        text = text.replace("</body>", f"<script>{REMEMBER}</script>\n</body>", 1)
        f.write_text(text, encoding="utf-8")


# ---------- 説明書(mdBook) ----------

# 本に入れるページ(題は各ページの最初の見出しから)。参照のページは docs/ の言語ごとの版を本の名前で入れる。
PAGES = ["index.md", "getting-started.md", "concepts.md", "tasks.md", "appearance.md", "themes.md", "faq.md"]
REFS = {
    "ja": {"keys.md": "docs/keys.ja.md", "config.md": "docs/config.ja.md", "safety.md": "docs/safety.ja.md", "obsidian-bases.md": "docs/obsidian-bases.ja.md"},
    "en": {"keys.md": "docs/keys.md", "config.md": "docs/config.md", "safety.md": "docs/safety.md", "obsidian-bases.md": "docs/obsidian-bases.md"},
}
BOOK_TITLE = {"ja": "mdgrid の説明書", "en": "mdgrid manual"}
REF_HEAD = {"ja": "参照", "en": "Reference"}
LINK = re.compile(r"(\]\()([^)\s]+)(\))")


def first_heading(text: str, default: str) -> str:
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return default


def rewrite(text: str, src: str, lang: str) -> str:
    """`src`(リポの中のパス)のページのリンクを、本の中のページ・画像・カタログ・GitHub に直す。"""
    book_of = {v: k for k, v in REFS[lang].items()}
    other = "en" if lang == "ja" else "ja"
    other_of = {v: k for k, v in REFS[other].items()}
    base = os.path.dirname(src)

    def fix(m: re.Match) -> str:
        target = m.group(2)
        if re.match(r"^[a-z]+:", target) or target.startswith("#"):
            return m.group(0)
        path, _, frag = target.partition("#")
        frag = "#" + frag if frag else ""
        repo = os.path.normpath(os.path.join(base, path))
        if repo.endswith("/shots.md") and repo.startswith("docs/manual/"):
            # 画面の一覧は、サイトの画面の一覧へ。
            new = f"../../{lang}/gallery.html"
        elif repo.startswith(f"docs/manual/{lang}/images/"):
            new = "images/" + os.path.basename(repo)
        elif repo.startswith(f"docs/manual/{lang}/") and repo.endswith(".md"):
            new = os.path.basename(repo)
        elif repo.startswith(f"docs/manual/{other}/") and repo.endswith(".md"):
            new = f"../{other}/" + os.path.basename(repo).replace(".md", ".html")
        elif repo in book_of:
            new = book_of[repo]
        elif repo in other_of:
            new = f"../{other}/" + other_of[repo].replace(".md", ".html")
        elif repo == "docs/catalog/index.html":
            new = "../../catalog/?lang=" + lang
        elif repo.startswith(".."):
            return m.group(0)
        else:
            new = f"{REPO}/blob/main/{repo}"
        return m.group(1) + new + frag + m.group(3)

    out, fence = [], False
    for line in text.split("\n"):
        if line.lstrip().startswith("```"):
            fence = not fence
        out.append(line if fence else LINK.sub(fix, line))
    return "\n".join(out)


def book(lang: str, dest: Path, images: Path) -> None:
    src = ROOT / "target" / "book" / lang
    if src.exists():
        shutil.rmtree(src)
    (src / "src").mkdir(parents=True)
    shutil.copytree(images, src / "src" / "images")
    summary = [f"# {BOOK_TITLE[lang]}", ""]
    for name in PAGES:
        rel = f"docs/manual/{lang}/{name}"
        text = (ROOT / rel).read_text(encoding="utf-8")
        # リポで読む人への「Pages で読める」の案内は、Pages の本では外す。
        text = re.sub(r" · (画面つきでは|Read it with the screenshots at) .*$", "", text, flags=re.M)
        (src / "src" / name).write_text(rewrite(text, rel, lang), encoding="utf-8")
        title = first_heading(text, name)
        summary.append(f"[{title}]({name})" if name == "index.md" else f"- [{title}]({name})")
    summary += ["", f"# {REF_HEAD[lang]}", ""]
    for name, rel in REFS[lang].items():
        text = (ROOT / rel).read_text(encoding="utf-8")
        (src / "src" / name).write_text(rewrite(text, rel, lang), encoding="utf-8")
        summary.append(f"- [{first_heading(text, name)}]({name})")
    (src / "src" / "SUMMARY.md").write_text("\n".join(summary) + "\n", encoding="utf-8")
    (src / "book.toml").write_text(
        f'[book]\ntitle = "{BOOK_TITLE[lang]}"\nlanguage = "{lang}"\n\n'
        f'[output.html]\ngit-repository-url = "{REPO}"\nsite-url = "/mdgrid/manual/{lang}/"\n',
        encoding="utf-8",
    )
    subprocess.run(["mdbook", "build", str(src), "--dest-dir", str(dest.resolve())], check=True)


if __name__ == "__main__":
    args = sys.argv[1:]
    shots = ROOT / "target" / "shots"
    if "--shots" in args:
        i = args.index("--shots")
        shots = Path(args[i + 1]).resolve()
        del args[i : i + 2]
    build(Path(args[0]) if args else ROOT / "site", shots)
