#!/usr/bin/env python3
"""GitHub Pages のサイト(scripts/pages.py が作る site/)を、公開の前にブラウザで確かめる。

    python3 scripts/check_site.py [site]

要るもの: Playwright の Python 版と Chromium(pip install playwright && playwright install chromium)。
確かめること:
- 全部のページの相対のリンクと画像の行き先があること
- サイトの地図(sitemap.xml)が XML として読め、公開するページとちょうど同じ URL を持つこと
- 入口の振り分け: 日本語のブラウザは /ja/、英語は /en/、ほかの言語は選ぶ画面のまま、選んだ言語は覚える
- 説明書: 右上の切り替えが同じページの別の言語を指し、hreflang(ja・en・x-default)がある
- 入口と画面の一覧の画像と録画が読める、画面の一覧の絞り込み
- 入口の部品: 場面の切り替え、仕組みの図、場面のタブ、テーマ、写すボタン、明るい・暗いの切り替え
- 見た目のカタログ: 見本の画面が描かれる、幅 390px で縮んで横にはみ出さない、拡大の切り替え、広い幅では切り替えを出さない
- 入口が幅 390px で横にはみ出さない
- どのページにもコンソールのエラー(とページの例外)が無い
落ちたら理由を並べて終了コード 1。
"""

import functools
import glob
import http.server
import os
import re
import sys
import threading
from xml.etree import ElementTree

from playwright.sync_api import sync_playwright

FAILS: list[str] = []


def check(ok: bool, what: str) -> None:
    print(("ok   " if ok else "FAIL ") + what)
    if not ok:
        FAILS.append(what)


def links(site: str) -> None:
    bad = []
    for f in glob.glob(os.path.join(site, "**", "*.html"), recursive=True):
        text = open(f, encoding="utf-8").read()
        for m in re.finditer(r'(?:href|src)="([^"#?]+)', text):
            u = m.group(1)
            if re.match(r"^[a-z]+:", u) or u.startswith("/"):
                continue
            p = os.path.normpath(os.path.join(os.path.dirname(f), u))
            if not os.path.exists(p) and not os.path.exists(os.path.join(p, "index.html")):
                bad.append(f"{os.path.relpath(f, site)} → {u}")
    check(not bad, f"相対のリンクと画像の行き先がある({len(bad)} 件が無い){': ' + ', '.join(bad[:5]) if bad else ''}")


SITE = "https://s6u5.github.io/mdgrid/"
SITEMAP_NS = "{http://www.sitemaps.org/schemas/sitemap/0.9}"


def site_pages(site: str) -> set:
    """サイトの地図に載るべきページの URL(scripts/pages.py の決まりを、作った site/ のファイルから別に数える):
    HTML のうち、言語を選んで移るだけの根の入口と前の URL、mdBook の目次の部品・404・印刷の版を除いたもの。
    index.html はフォルダの形、説明書は mdBook のファイル名のまま(hreflang と同じ形)。"""
    out = set()
    for f in glob.glob(os.path.join(site, "**", "*.html"), recursive=True):
        rel = os.path.relpath(f, site).replace(os.sep, "/")
        if rel in ("index.html", "gallery.html") or os.path.basename(rel) in ("toc.html", "404.html", "print.html"):
            continue
        if not rel.startswith("manual/") and rel.endswith("index.html"):
            rel = rel[: -len("index.html")]
        out.add(SITE + rel)
    return out


def sitemap(site: str) -> None:
    """sitemap.xml が XML の urlset で、どの URL もサイトの中のページを指し、重ならず、公開するページとちょうど同じ。"""
    path = os.path.join(site, "sitemap.xml")
    if not os.path.exists(path):
        check(False, "sitemap.xml がある")
        return
    try:
        root = ElementTree.parse(path).getroot()
    except ElementTree.ParseError as e:
        check(False, f"sitemap.xml が XML として読める({e})")
        return
    check(root.tag == SITEMAP_NS + "urlset", f"sitemap.xml の根が sitemaps.org の urlset(今: {root.tag})")
    urls = [(e.text or "").strip() for e in root.iter(SITEMAP_NS + "loc")]
    bad = []
    for u in urls:
        rel = u[len(SITE):] if u.startswith(SITE) else None
        if rel is None or rel.startswith("/") or ".." in rel.split("/"):
            bad.append(u)
            continue
        p = os.path.join(site, rel)
        # フォルダの形(末尾の /)は index.html、ほかはファイルそのもの(末尾の / の無いフォルダは移されるので外れ)。
        if not (os.path.isfile(os.path.join(p, "index.html")) if rel == "" or rel.endswith("/") else os.path.isfile(p)):
            bad.append(u)
    check(not bad, f"sitemap.xml の URL がどれもサイトの中のページを指す({len(bad)} 件が外れ){': ' + ', '.join(bad[:5]) if bad else ''}")
    check(len(urls) == len(set(urls)), "sitemap.xml の URL が重ならない")
    want = site_pages(site)
    miss, extra = sorted(want - set(urls)), sorted(set(urls) - want)
    check(
        not miss and not extra,
        f"sitemap.xml が公開するページとちょうど同じ(無い {len(miss)} 件・余計 {len(extra)} 件){': ' + ', '.join((miss + extra)[:5]) if miss or extra else ''}",
    )


def serve(site: str):
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=site)
    handler.log_message = lambda *a: None
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd, f"http://127.0.0.1:{httpd.server_address[1]}/"


def errors_of(page) -> list[str]:
    errs: list[str] = []
    page.on("console", lambda m: errs.append(m.text) if m.type == "error" else None)
    page.on("pageerror", lambda e: errs.append(str(e)))
    return errs


def all_images_load(page) -> tuple[int, int]:
    """画像を全部すぐに読ませて、(読めた数, 全部の数)。"""
    page.evaluate("document.querySelectorAll('img').forEach(i => i.loading = 'eager')")
    page.wait_for_function(
        "[...document.images].every(i => i.complete)", timeout=60000
    )
    return page.evaluate(
        "[[...document.images].filter(i => i.naturalWidth > 0).length, document.images.length]"
    )


def browser_checks(base: str) -> None:
    with sync_playwright() as p:
        b = p.chromium.launch()

        # 入口の振り分け。
        for loc, want in [("ja-JP", "/ja/"), ("en-US", "/en/"), ("fr-FR", None)]:
            ctx = b.new_context(locale=loc)
            pg = ctx.new_page()
            pg.goto(base)
            pg.wait_for_timeout(300)
            path = pg.url[len(base) - 1 :]
            if want:
                check(path.startswith(want), f"{loc} のブラウザで / → {want}(今: {path})")
            else:
                check(path in ("/", ""), f"{loc} のブラウザで / は選ぶ画面のまま(今: {path})")
            ctx.close()
        ctx = b.new_context(locale="ja-JP")
        pg = ctx.new_page()
        pg.goto(base + "ja/")
        pg.click('nav.top a[data-pick="en"]')
        pg.wait_for_url("**/en/")
        pg.goto(base)
        pg.wait_for_timeout(300)
        check(pg.url.endswith("/en/"), f"English を選んだあと / → /en/(今: {pg.url})")
        ctx.close()

        ctx = b.new_context(locale="en-US", viewport={"width": 1280, "height": 900})
        # どのページにもコンソールのエラーが無い。
        pages = [
            "", "ja/", "en/", "ja/gallery.html", "en/gallery.html", "gallery.html",
            "catalog/?lang=ja", "catalog/?lang=en",
            "manual/ja/index.html", "manual/en/index.html", "manual/ja/tasks.html", "manual/en/keys.html",
        ]
        for u in pages:
            pg = ctx.new_page()
            errs = errors_of(pg)
            pg.goto(base + u)
            pg.wait_for_load_state("load")
            pg.wait_for_timeout(200)
            check(not errs, f"{u or '/'} にコンソールのエラーが無い{': ' + errs[0][:120] if errs else ''}")
            pg.close()

        # 説明書の言語の切り替えと hreflang。
        pg = ctx.new_page()
        pg.goto(base + "manual/ja/tasks.html")
        href = pg.get_attribute("a.lang-switch", "href")
        check(href == "../en/tasks.html", f"説明書の切り替えが同じページの英語版を指す(今: {href})")
        langs = pg.eval_on_selector_all('link[rel="alternate"]', "ls => ls.map(l => l.hreflang).sort().join(',')")
        check(langs == "en,ja,x-default", f"説明書に hreflang ja・en・x-default(今: {langs})")
        pg.close()

        # 画像と録画、画面の一覧の絞り込み。
        for u in ["ja/", "en/", "ja/gallery.html", "en/gallery.html"]:
            pg = ctx.new_page()
            pg.goto(base + u)
            ok, total = all_images_load(pg)
            check(total > 0 and ok == total, f"{u} の画像が全部読める({ok}/{total})")
            pg.close()
        pg = ctx.new_page()
        pg.goto(base + "en/gallery.html")
        total = pg.locator("figure[data-search]").count()
        pg.fill("#find", "calendar")
        shown = pg.locator("figure[data-search]:not([hidden])").count()
        check(0 < shown < total, f"画面の一覧の絞り込み(calendar で {shown}/{total})")
        pg.close()

        # 入口の部品: 場面の切り替え、仕組みの図(status を直すと書かれる1行が出て、元に戻せる)、場面のタブ、テーマ、写すボタン。
        for lang in ("ja", "en"):
            pg = ctx.new_page()
            errs = errors_of(pg)
            pg.goto(base + lang + "/")
            pg.click(".steps .step >> nth=2")
            on = pg.eval_on_selector_all(".frames a", "fs => fs.map(f => f.classList.contains('on'))")
            check(on == [False, False, True], f"{lang}/ の場面の切り替えで3枚目が出る(今: {on})")
            row = pg.locator("#model tbody tr").nth(1)
            was = row.locator("select").input_value()
            row.locator("select").select_option("done" if was != "done" else "todo")
            check(pg.locator("#model .file .l.del").count() == 1 and pg.locator("#model .file .l.add").count() == 1,
                  f"{lang}/ の図で status を直すと、消える行と書く行が1行ずつ出る")
            check("1" in (pg.text_content("#model .tally b") or ""), f"{lang}/ の図の未保存が 1 になる")
            pg.click("#model .tally button")
            check(pg.locator("#model .file .l.del").count() == 0 and row.locator("select").input_value() == was,
                  f"{lang}/ の図を元に戻せる")
            pg.click(".steps .step >> nth=1")
            check("equipment.csv" in (pg.text_content(".term .bar-t span") or ""), f"{lang}/ の枠の題が場面のコマンドになる")
            check(pg.locator("#ai .pane-t").count() == 2 and pg.locator("#ai .pane-t img").count() == 1, f"{lang}/ の分割した画面の段に、図の pane と本物の画面がある")
            pg.click("#tab-db-detail")
            check(pg.is_visible("#db-detail") and pg.is_hidden("#db-filter"), f"{lang}/ のデータベースの段のタブが切り替わる")
            pg.click("#tab-scene-people")
            check(pg.is_visible("#scene-people") and pg.is_hidden("#scene-tasks"), f"{lang}/ の場面のタブが切り替わる")
            pg.click('[data-look="dracula"]')
            check("theme-dracula.svg" in (pg.get_attribute("#look-img", "src") or "") and 'theme = "dracula"' in (pg.text_content("#look-code") or ""),
                  f"{lang}/ のテーマを押すと画面と設定の行が替わる")
            pg.wait_for_function("document.getElementById('look-img').complete && document.getElementById('look-img').naturalWidth > 0")
            copy = pg.locator(".hero .cmd button[data-copy]")
            check(copy.count() == 1 and pg.get_attribute(".hero .cmd code", "data-text") == "cargo install mdgrid --locked",
                  f"{lang}/ の入れるコマンドに写すボタンがある")
            pg.click("#mode")
            check(pg.evaluate("document.documentElement.getAttribute('data-theme')") in ("light", "dark"), f"{lang}/ の明るい・暗いの切り替えが効く")
            check(not errs, f"{lang}/ の部品を触ってもコンソールのエラーが無い{': ' + errs[0][:120] if errs else ''}")
            pg.close()

        # カタログ: 広い幅では等倍で切り替えを出さない。
        pg = ctx.new_page()
        pg.goto(base + "catalog/?lang=en")
        check(pg.locator("#term .scr").count() == 1, "カタログの見本の画面が描かれる")
        check("tasks" in (pg.text_content("#term") or ""), "カタログの見本の画面に表の中身がある")
        check(pg.is_hidden("#fitbtn"), "広い幅ではカタログの拡大の切り替えを出さない")
        check(pg.locator("#catalog .card").count() > 20, "カタログの部品の見本が並ぶ")
        pg.close()
        ctx.close()

        # スマホの幅。
        ctx = b.new_context(locale="ja-JP", viewport={"width": 390, "height": 844})
        pg = ctx.new_page()
        errs = errors_of(pg)
        pg.goto(base + "catalog/?lang=ja")
        pg.wait_for_timeout(300)
        zoom = pg.evaluate("document.querySelector('#term .scr').style.zoom")
        check(zoom not in ("", None) and float(zoom) < 1, f"390px でカタログの見本の画面が縮む(zoom {zoom})")
        sw = pg.evaluate("document.documentElement.scrollWidth")
        check(sw <= 390, f"390px でカタログが横にはみ出さない(幅 {sw})")
        pg.click("#fitbtn")
        zoom2 = pg.evaluate("document.querySelector('#term .scr').style.zoom")
        check(zoom2 == "", f"「拡大して見る」で等倍になる(zoom {zoom2!r})")
        # スマホの Safari の字の自動の拡大を止めている(縮めた見本の画面で半角の字だけが大きくならない)。
        adj = pg.evaluate("(() => { const s = getComputedStyle(document.documentElement); return s.webkitTextSizeAdjust || s.textSizeAdjust || ''; })()")
        check(adj == "100%", f"カタログが字の自動の拡大を止めている(text-size-adjust {adj!r})")
        check(not errs, f"390px のカタログにコンソールのエラーが無い{': ' + errs[0][:120] if errs else ''}")
        for u in ["ja/", "en/", "en/gallery.html", "manual/ja/tasks.html"]:
            pg.goto(base + u)
            pg.wait_for_timeout(200)
            sw = pg.evaluate("document.documentElement.scrollWidth")
            check(sw <= 390, f"390px で {u} が横にはみ出さない(幅 {sw})")
        ctx.close()
        b.close()


def main() -> int:
    site = sys.argv[1] if len(sys.argv) > 1 else "site"
    links(site)
    sitemap(site)
    httpd, base = serve(site)
    try:
        browser_checks(base)
    finally:
        httpd.shutdown()
    if FAILS:
        print(f"\n{len(FAILS)} 件が落ちた", file=sys.stderr)
        return 1
    print("\n全部通った")
    return 0


if __name__ == "__main__":
    sys.exit(main())
