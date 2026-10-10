#!/usr/bin/env python3
# app-manual プラグイン(S6U5)の tui_shot.py を写したもの。Pages の workflow が説明書の画面を撮るのに使う。
"""Take screenshots of a terminal (TUI) app for a Markdown manual.

Standard library only. The app runs in a pseudo-terminal; its output goes through a small
terminal emulator (enough for ratatui / crossterm style apps), and each screen is saved as
SVG and plain text. Usage:

    python3 tui_shot.py <scenarios.toml|.json> [--out manual] [--lang ja] [--only ID]

Scenario file (TOML needs Python 3.11+; JSON with the same keys works on older Pythons):

    [app]
    command = ["./target/release/app", "examples/data"]
    cols = 100              # terminal size
    rows = 30
    env = { TZ = "UTC" }    # added on top of a minimal environment
    isolate = true          # HOME / XDG_* point at a fresh temp folder
    stage = ["examples"]    # copy these into stage_dir before every shot and run there:
    stage_dir = "/tmp/app-demo"   #   the real files are never written, the path shown is neutral
    timeout = 10            # seconds per wait
    title = "App manual"
    intro = "One paragraph shown at the top of the manual."

    [[shot]]
    id = "start"            # file name: <id>.svg / <id>.txt
    title = "Open a folder"
    keys = ["j", "Enter", "text:hello", "C-s"]
    wait = "Saved"          # optional: text to wait for after the keys
    command = ["./app", "other.db"]   # optional: launch this shot differently
    ready = "text"          # optional: overrides [app].ready for this shot
    text = "Explanation shown under the screenshot."

Output: <out>/<lang>/images/<id>.svg and <id>.txt, plus <out>/<lang>/shots.md (a gallery for
checking). The manual's chapters are written by hand (or by an agent) and link to images/<id>.svg.

title, intro, text, ready and wait may also be tables by language, e.g.
title = { en = "Help", ja = "ヘルプ" }; --lang picks one (falling back to en). --env KEY=VALUE
(repeatable) overrides the environment for this run, e.g. --env LANG=en_US.UTF-8.

Every shot starts the app from scratch, so shots do not depend on each other. Keys:
single characters, names (Enter, Esc, Tab, S-Tab, Backspace, Delete, Up, Down, Left,
Right, S-Up, S-Down, S-Left, S-Right, Home, End, PageUp, PageDown, Space, F1..F12), C-x for Ctrl, A-x for Alt,
"text:..." for literal text, and "sleep:0.5" for a pause. The child is always killed,
even on errors or timeouts.
"""

import argparse
import fcntl
import json
import os
import pty
import re
import select
import shutil
import signal
import struct
import sys
import tempfile
import termios
import time
import unicodedata
from html import escape

# --------------------------------------------------------------------------- terminal

PALETTE = [
    "#1e1e1e", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
]
DEFAULT_FG = "#d4d4d4"
DEFAULT_BG = "#1e1e1e"


def color256(n):
    if n < 16:
        return PALETTE[n]
    if n < 232:
        n -= 16
        steps = [0, 95, 135, 175, 215, 255]
        return "#%02x%02x%02x" % (steps[n // 36], steps[(n // 6) % 6], steps[n % 6])
    v = 8 + (n - 232) * 10
    return "#%02x%02x%02x" % (v, v, v)


def char_width(ch):
    if unicodedata.combining(ch) or unicodedata.category(ch) in ("Mn", "Me", "Cf"):
        return 0
    return 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1


class Cell:
    __slots__ = ("ch", "fg", "bg", "bold", "dim", "italic", "under", "rev", "cont")

    def __init__(self):
        self.ch = " "
        self.fg = None
        self.bg = None
        self.bold = self.dim = self.italic = self.under = self.rev = False
        self.cont = False  # right half of a wide character


class Screen:
    """A minimal VT100/xterm emulator: cursor moves, erase, SGR colors, alternate screen."""

    def __init__(self, rows, cols):
        self.rows, self.cols = rows, cols
        self.reset_style()
        self.grid = self.blank()
        self.y = self.x = 0
        self.saved = (0, 0)
        self.top, self.bottom = 0, rows - 1
        self.replies = []  # bytes to send back to the app (cursor position reports)
        self.hidden_half = False  # the next space is the hidden right half of a widened emoji
        self.buf = ""

    def blank(self):
        return [[Cell() for _ in range(self.cols)] for _ in range(self.rows)]

    def reset_style(self):
        self.fg = self.bg = None
        self.bold = self.dim = self.italic = self.under = self.rev = False

    def styled(self, ch):
        c = Cell()
        c.ch, c.fg, c.bg = ch, self.fg, self.bg
        c.bold, c.dim, c.italic, c.under, c.rev = self.bold, self.dim, self.italic, self.under, self.rev
        return c

    def erase_cell(self):
        c = Cell()
        c.bg = self.bg
        return c

    def feed(self, data):
        self.buf += data
        i, s = 0, self.buf
        while i < len(s):
            ch = s[i]
            if ch == "\x1b":
                end = self.escape(s, i)
                if end is None:  # incomplete sequence: keep for the next read
                    break
                i = end
                continue
            self.control(ch) if ord(ch) < 0x20 or ch == "\x7f" else self.put(ch)
            i += 1
        self.buf = s[i:]

    def control(self, ch):
        self.hidden_half = False
        if ch == "\r":
            self.x = 0
        elif ch == "\n":
            self.linefeed()
        elif ch == "\b":
            self.x = max(0, self.x - 1)
        elif ch == "\t":
            self.x = min(self.cols - 1, (self.x // 8 + 1) * 8)

    def linefeed(self):
        if self.y == self.bottom:
            del self.grid[self.top]
            self.grid.insert(self.bottom, [self.erase_cell() for _ in range(self.cols)])
        else:
            self.y = min(self.rows - 1, self.y + 1)

    def put(self, ch):
        if self.hidden_half:
            self.hidden_half = False
            if ch == " ":  # ratatui wrote the hidden half of "✈️": it is already covered
                return
        w = char_width(ch)
        row = self.grid[self.y]
        if ch == "\ufe0f":
            # Emoji presentation selector: ratatui (unicode-width) counts "✈️" as two cells, so the
            # previous character becomes wide and the selector itself takes no cell.
            x = self.x - 1
            if 0 <= x and self.x < self.cols and not row[x].cont and not row[self.x].cont:
                row[x].ch += ch
                cont = self.styled("")
                cont.cont = True
                row[self.x] = cont
                self.x += 1
                self.hidden_half = True
            else:
                self.attach(ch)
            return
        if (ch == " " and self.x < self.cols and row[self.x].cont and self.x > 0
                and row[self.x - 1].ch.endswith("\ufe0f")):
            # ratatui sometimes repaints the hidden right half of such an emoji with a space;
            # keep the emoji (a terminal that draws it one cell wide shows that space instead).
            self.x += 1
            return
        if w == 0:
            self.attach(ch)
            return
        if self.x + w > self.cols:
            self.x = 0
            self.linefeed()
        row = self.grid[self.y]
        # Overwriting half of a wide character erases its other half, as real terminals do.
        if row[self.x].cont and self.x > 0:
            row[self.x - 1] = self.erase_cell()
        for k in range(self.x + 1, min(self.cols, self.x + w + 1)):
            if row[k].cont:
                row[k] = self.erase_cell()
        self.grid[self.y][self.x] = self.styled(ch)
        if w == 2:
            cont = self.styled("")
            cont.cont = True
            self.grid[self.y][self.x + 1] = cont
        self.x += w  # may equal cols: the next character wraps (pending wrap)

    def attach(self, ch):
        """Keep a zero-width character (combining mark) with the previous character."""
        if self.x > 0:
            prev = self.x - 1
            while prev > 0 and self.grid[self.y][prev].cont:
                prev -= 1
            self.grid[self.y][prev].ch += ch

    def escape(self, s, i):
        self.hidden_half = False
        if i + 1 >= len(s):
            return None
        kind = s[i + 1]
        if kind == "[":
            m = re.compile(r"\x1b\[([?>=]?)([0-9;:]*)([ -/]*)([@-~])").match(s, i)
            if not m:
                return None if len(s) - i < 64 else i + 2
            self.csi(m.group(1), m.group(2), m.group(4))
            return m.end()
        if kind in "]PX^_":  # OSC / DCS etc.: skip to BEL or ST
            for j in range(i + 2, len(s)):
                if s[j] == "\x07":
                    return j + 1
                if s[j] == "\x1b" and j + 1 < len(s) and s[j + 1] == "\\":
                    return j + 2
            return None
        if kind == "7":
            self.saved = (self.y, self.x)
        elif kind == "8":
            self.y, self.x = self.saved
        elif kind in "()":
            return i + 3 if i + 2 < len(s) else None
        return i + 2

    def csi(self, private, params, final):
        args = [int(p) if p.isdigit() else 0 for p in params.replace(":", ";").split(";")] if params else []

        def arg(n, default=1):
            v = args[n] if len(args) > n else 0
            return v if v else default

        if private == "?":
            if final in "hl" and any(a in (47, 1047, 1049) for a in args):
                self.grid = self.blank()
                self.y = self.x = 0
            return
        if private:
            if final == "c":
                pass
            return
        if final in "Hf":
            self.y = min(self.rows - 1, arg(0) - 1)
            self.x = min(self.cols - 1, arg(1) - 1)
        elif final == "A":
            self.y = max(0, self.y - arg(0))
        elif final == "B":
            self.y = min(self.rows - 1, self.y + arg(0))
        elif final == "C":
            self.x = min(self.cols - 1, self.x + arg(0))
        elif final == "D":
            self.x = max(0, self.x - arg(0))
        elif final == "G":
            self.x = min(self.cols - 1, arg(0) - 1)
        elif final == "d":
            self.y = min(self.rows - 1, arg(0) - 1)
        elif final == "J":
            mode = arg(0, 0)
            rows = range(self.rows)
            if mode == 0:
                self.erase_line(0)
                rows = range(self.y + 1, self.rows)
            elif mode == 1:
                self.erase_line(1)
                rows = range(0, self.y)
            for r in rows:
                self.grid[r] = [self.erase_cell() for _ in range(self.cols)]
        elif final == "K":
            self.erase_line(arg(0, 0))
        elif final == "X":
            for c in range(self.x, min(self.cols, self.x + arg(0))):
                self.grid[self.y][c] = self.erase_cell()
        elif final == "P":
            row = self.grid[self.y]
            del row[self.x:self.x + arg(0)]
            row.extend(self.erase_cell() for _ in range(self.cols - len(row)))
        elif final == "@":
            row = self.grid[self.y]
            for _ in range(arg(0)):
                row.insert(self.x, self.erase_cell())
            del row[self.cols:]
        elif final == "r":
            self.top = arg(0) - 1
            self.bottom = min(self.rows - 1, arg(1, self.rows) - 1)
            self.y = self.x = 0
        elif final == "s":
            self.saved = (self.y, self.x)
        elif final == "u":
            self.y, self.x = self.saved
        elif final == "m":
            self.sgr(args or [0])
        elif final == "n" and arg(0) == 6:
            self.replies.append(("\x1b[%d;%dR" % (self.y + 1, self.x + 1)).encode())
        elif final == "c":
            self.replies.append(b"\x1b[?1;2c")

    def erase_line(self, mode):
        row = self.grid[self.y]
        x = min(self.x, self.cols - 1)
        cols = range(x, self.cols) if mode == 0 else range(0, x + 1) if mode == 1 else range(self.cols)
        for c in cols:
            row[c] = self.erase_cell()

    def sgr(self, a):
        i = 0
        while i < len(a):
            p = a[i]
            if p == 0:
                self.reset_style()
            elif p == 1:
                self.bold = True
            elif p == 2:
                self.dim = True
            elif p == 3:
                self.italic = True
            elif p == 4:
                self.under = True
            elif p == 7:
                self.rev = True
            elif p == 22:
                self.bold = self.dim = False
            elif p == 23:
                self.italic = False
            elif p == 24:
                self.under = False
            elif p == 27:
                self.rev = False
            elif 30 <= p <= 37:
                self.fg = PALETTE[p - 30]
            elif 90 <= p <= 97:
                self.fg = PALETTE[p - 90 + 8]
            elif 40 <= p <= 47:
                self.bg = PALETTE[p - 40]
            elif 100 <= p <= 107:
                self.bg = PALETTE[p - 100 + 8]
            elif p == 39:
                self.fg = None
            elif p == 49:
                self.bg = None
            elif p in (38, 48) and i + 1 < len(a):
                if a[i + 1] == 5 and i + 2 < len(a):
                    col = color256(a[i + 2])
                    i += 2
                elif a[i + 1] == 2 and i + 4 < len(a):
                    col = "#%02x%02x%02x" % tuple(a[i + 2:i + 5])
                    i += 4
                else:
                    col = None
                if p == 38:
                    self.fg = col
                else:
                    self.bg = col
            i += 1

    def text(self):
        lines = []
        for row in self.grid:
            lines.append("".join(c.ch for c in row if not c.cont).rstrip())
        while lines and not lines[-1]:
            lines.pop()
        return "\n".join(lines) + "\n"

    def contains(self, needle):
        return needle in "\n".join("".join(c.ch for c in row if not c.cont) for row in self.grid)

    def svg(self, cell_w=8.4, cell_h=18, font=14):
        w, h = self.cols * cell_w + 16, self.rows * cell_h + 16
        out = [
            '<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" viewBox="0 0 %d %d">' % (w, h, w, h),
            '<rect width="100%%" height="100%%" fill="%s" rx="6"/>' % DEFAULT_BG,
            '<g font-family="Menlo, Consolas, \'DejaVu Sans Mono\', \'Noto Sans Mono CJK JP\', monospace" '
            'font-size="%d" xml:space="preserve">' % font,
        ]
        def colors(cell):
            if cell.rev:
                return (cell.bg or DEFAULT_BG), (cell.fg or DEFAULT_FG)
            return cell.fg or DEFAULT_FG, cell.bg

        for r, row in enumerate(self.grid):
            y = 8 + r * cell_h
            # Backgrounds first, one rectangle per run of the same colour (no seams between cells).
            c = 0
            while c < self.cols:
                bg = colors(row[c])[1] if not row[c].cont else None
                if row[c].cont:
                    c += 1
                    continue
                start = c
                c += 1
                while c < self.cols and (row[c].cont or colors(row[c])[1] == bg):
                    c += 1
                if bg:
                    out.append('<rect x="%.1f" y="%d" width="%.1f" height="%d" fill="%s"/>'
                               % (8 + start * cell_w, y, (c - start) * cell_w, cell_h, bg))
            for c, cell in enumerate(row):
                if cell.cont:
                    continue
                fg = colors(cell)[0]
                span = 2 if c + 1 < self.cols and row[c + 1].cont else 1
                if cell.ch.strip():
                    attrs = ' fill="%s"' % fg
                    if cell.bold:
                        attrs += ' font-weight="bold"'
                    if cell.dim:
                        attrs += ' opacity="0.6"'
                    if cell.italic:
                        attrs += ' font-style="italic"'
                    if cell.under:
                        attrs += ' text-decoration="underline"'
                    out.append('<text x="%.1f" y="%.1f" textLength="%.1f" lengthAdjust="spacingAndGlyphs"%s>%s</text>'
                               % (8 + c * cell_w, y + cell_h * 0.75, span * cell_w, attrs, escape(cell.ch)))
        out.append("</g></svg>")
        return "\n".join(out) + "\n"


# --------------------------------------------------------------------------- keys

NAMED = {
    "Enter": "\r", "Esc": "\x1b", "Tab": "\t", "S-Tab": "\x1b[Z", "Backspace": "\x7f",
    "Delete": "\x1b[3~", "Up": "\x1b[A", "Down": "\x1b[B", "Right": "\x1b[C", "Left": "\x1b[D",
    "S-Up": "\x1b[1;2A", "S-Down": "\x1b[1;2B", "S-Right": "\x1b[1;2C", "S-Left": "\x1b[1;2D",
    "Home": "\x1b[H", "End": "\x1b[F", "PageUp": "\x1b[5~", "PageDown": "\x1b[6~", "Space": " ",
    "F1": "\x1bOP", "F2": "\x1bOQ", "F3": "\x1bOR", "F4": "\x1bOS", "F5": "\x1b[15~",
    "F6": "\x1b[17~", "F7": "\x1b[18~", "F8": "\x1b[19~", "F9": "\x1b[20~", "F10": "\x1b[21~",
    "F11": "\x1b[23~", "F12": "\x1b[24~",
}


def key_bytes(key):
    if key.startswith("text:"):
        return key[5:].encode()
    if key in NAMED:
        return NAMED[key].encode()
    if key.startswith("C-") and len(key) == 3:
        return bytes([ord(key[2].lower()) & 0x1F])
    if key.startswith("A-") and len(key) > 2:
        return b"\x1b" + key_bytes(key[2:])
    if len(key) >= 1:
        return key.encode()
    raise ValueError("unknown key: %r" % key)


# --------------------------------------------------------------------------- running

class Child:
    """The app in a pseudo-terminal. Always killed on close()."""

    def __init__(self, command, rows, cols, env, cwd):
        self.screen = Screen(rows, cols)
        self.pid, self.fd = pty.fork()
        if self.pid == 0:  # child
            try:
                os.chdir(cwd)
                os.execvpe(command[0], command, env)
            finally:
                os._exit(127)
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        self.decoder_rest = b""

    def pump(self, seconds):
        end = time.time() + seconds
        while True:
            left = end - time.time()
            if left <= 0:
                return
            r, _, _ = select.select([self.fd], [], [], min(left, 0.05))
            if not r:
                continue
            try:
                data = os.read(self.fd, 65536)
            except OSError:
                return
            if not data:
                return
            data = self.decoder_rest + data
            try:
                text = data.decode("utf-8")
                self.decoder_rest = b""
            except UnicodeDecodeError as e:
                text = data[:e.start].decode("utf-8")
                self.decoder_rest = data[e.start:]
            self.screen.feed(text)
            for reply in self.screen.replies:
                os.write(self.fd, reply)
            self.screen.replies.clear()

    def wait_for(self, needle, timeout):
        end = time.time() + timeout
        while time.time() < end:
            self.pump(0.1)
            if needle is None or self.screen.contains(needle):
                self.pump(0.3)  # let the frame settle
                return True
        return False

    def send(self, key):
        if key.startswith("sleep:"):
            self.pump(float(key[6:]))
            return
        os.write(self.fd, key_bytes(key))
        self.pump(0.15)

    def close(self):
        # The child leads its own session (pty.fork), so signal the whole process group:
        # a shell wrapper's own children are stopped too.
        for sig in (signal.SIGTERM, signal.SIGKILL):
            try:
                os.killpg(self.pid, sig)
            except ProcessLookupError:
                break
            except PermissionError:
                os.kill(self.pid, sig)
            for _ in range(20):
                pid, _ = os.waitpid(self.pid, os.WNOHANG)
                if pid:
                    break
                time.sleep(0.05)
            else:
                continue
            break
        try:  # anything left in the group (children that ignored SIGTERM) goes too
            os.killpg(self.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass
        try:
            os.close(self.fd)
        except OSError:
            pass


def stage_root(app):
    """Where a shot runs. A fixed stage_dir gives neutral, repeatable paths on screen; it must be
    absent or a real directory owned by this user (not a symlink), so a shared /tmp cannot be used
    to redirect the copy or the clean-up. Without stage_dir a private random folder is used."""
    root = app.get("stage_dir")
    if not root:
        return tempfile.mkdtemp(prefix="app-manual-stage-")
    try:
        st = os.lstat(root)
    except FileNotFoundError:
        os.makedirs(root, mode=0o700)
        return root
    import stat as _stat
    if _stat.S_ISLNK(st.st_mode) or not _stat.S_ISDIR(st.st_mode) or st.st_uid != os.getuid():
        raise RuntimeError("stage_dir %s exists and is not a directory owned by you; remove it or "
                           "choose another path" % root)
    shutil.rmtree(root)
    os.makedirs(root, mode=0o700)
    return root


def stage(app, cwd, argv):
    """Copy the [app].stage paths (relative to cwd) into a fresh stage folder and run there, so the
    shot can never change the real sample data (and, with stage_dir, the path shown is neutral).
    A relative program path that exists in cwd (./target/release/app) is made absolute first."""
    root = stage_root(app)
    for rel in app["stage"]:
        src, dst = os.path.join(cwd, rel), os.path.join(root, rel)
        if os.path.isdir(src):
            shutil.copytree(src, dst, symlinks=True)
        else:
            os.makedirs(os.path.dirname(dst) or root, exist_ok=True)
            shutil.copy2(src, dst)
    prog = argv[0]
    if not os.path.isabs(prog) and os.sep in prog and os.path.exists(os.path.join(cwd, prog)):
        argv = [os.path.join(cwd, prog)] + argv[1:]
    return root, argv


def unstage(root):
    """Remove the stage folder (only what stage() created: a real directory owned by this user)."""
    import stat as _stat
    try:
        st = os.lstat(root)
    except FileNotFoundError:
        return
    if _stat.S_ISDIR(st.st_mode) and not _stat.S_ISLNK(st.st_mode) and st.st_uid == os.getuid():
        shutil.rmtree(root, ignore_errors=True)


def load(path):
    with open(path, "rb") as f:
        raw = f.read()
    if path.endswith(".json"):
        return json.loads(raw)
    try:
        import tomllib
    except ImportError:
        sys.exit("reading TOML needs Python 3.11+ (or write the scenarios as JSON)")
    return tomllib.loads(raw.decode("utf-8"))


def base_env(isolate, extra, tmp):
    env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "TERM": "xterm-256color",
           "COLORTERM": "truecolor", "LANG": os.environ.get("LANG", "C.UTF-8")}
    if isolate:
        for k, sub in (("HOME", "home"), ("XDG_CONFIG_HOME", "config"), ("XDG_STATE_HOME", "state"),
                       ("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache")):
            d = os.path.join(tmp, sub)
            os.makedirs(d, exist_ok=True)
            env[k] = d
    else:
        env["HOME"] = os.environ.get("HOME", tmp)
    env.update({k: str(v) for k, v in (extra or {}).items()})
    return env


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("scenarios")
    ap.add_argument("--out", default="manual")
    ap.add_argument("--lang", default=None, help="sub-folder of --out (e.g. ja, en)")
    ap.add_argument("--only", default=None, help="take only the shot with this id")
    ap.add_argument("--env", action="append", default=[], metavar="KEY=VALUE",
                    help="extra environment for the app (repeatable), e.g. --env LANG=en_US.UTF-8")
    a = ap.parse_args()
    cli_env = dict(e.split("=", 1) for e in a.env)

    spec = load(a.scenarios)
    app = spec.get("app", {})
    command = app["command"]
    rows, cols = int(app.get("rows", 30)), int(app.get("cols", 100))
    timeout = float(app.get("timeout", 10))
    cwd = os.path.abspath(app.get("cwd", "."))
    out = os.path.join(a.out, a.lang) if a.lang else a.out
    images = os.path.join(out, "images")
    os.makedirs(images, exist_ok=True)

    shots = [s for s in spec.get("shot", []) if a.only in (None, s["id"])]
    if not shots:
        sys.exit("no shots to take")
    failed = []
    for shot in shots:
        tmp = tempfile.mkdtemp(prefix="app-manual-")
        child = None
        staged = None
        try:
            env = base_env(app.get("isolate", True),
                           {**app.get("env", {}), **shot.get("env", {}), **cli_env}, tmp)
            run_cwd, argv = cwd, list(shot.get("command", command))
            if app.get("stage"):
                run_cwd, argv = stage(app, cwd, argv)
                staged = run_cwd
            child = Child(argv, rows, cols, env, run_cwd)
            ready = pick(shot.get("ready", app.get("ready")), a.lang) or None
            if not child.wait_for(ready, timeout):
                raise RuntimeError("the app did not show %r" % ready)
            for key in shot.get("keys", []):
                child.send(key)
            wait = pick(shot.get("wait"), a.lang) or None
            if not child.wait_for(wait, timeout):
                raise RuntimeError("the screen did not show %r" % wait)
            with open(os.path.join(images, shot["id"] + ".svg"), "w", encoding="utf-8") as f:
                f.write(child.screen.svg())
            with open(os.path.join(images, shot["id"] + ".txt"), "w", encoding="utf-8") as f:
                f.write(child.screen.text())
            print("shot:", shot["id"])
        except Exception as e:  # keep going; report at the end
            failed.append((shot["id"], str(e)))
            print("FAILED:", shot["id"], e, file=sys.stderr)
        finally:
            if child:
                child.close()
            shutil.rmtree(tmp, ignore_errors=True)
            if staged:
                unstage(staged)

    if a.only is None:
        write_gallery(out, app, spec.get("shot", []), failed, a.lang)
    if failed:
        sys.exit(1)


def pick(value, lang):
    """A text field is a string, or a table of strings by language ({ ja = "…", en = "…" })."""
    if isinstance(value, dict):
        return value.get(lang) or value.get("en") or next(iter(value.values()), "")
    return value or ""


def write_gallery(out, app, shots, failed, lang=None):
    """shots.md: every screenshot with its scene text, for checking. The manual's chapters are
    written separately and link to images/<id>.svg."""
    bad = {i for i, _ in failed}
    keys_label = "キー" if lang == "ja" else "Keys"
    lines = ["# " + (pick(app.get("title"), lang) or "Manual"), ""]
    if app.get("intro"):
        lines += [pick(app["intro"], lang).strip(), ""]
    lines += ["<!-- generated by app-manual/tui_shot.py: the screenshot gallery. The manual's chapters link to images/. -->", ""]
    for s in shots:
        title = pick(s.get("title"), lang) or s["id"]
        lines += ["## " + title, ""]
        if s.get("keys"):
            lines += [keys_label + ": " + " ".join("`%s`" % k for k in s["keys"] if not k.startswith("sleep:")), ""]
        if s["id"] in bad:
            lines += ["(this screenshot could not be taken)", ""]
        else:
            lines += ["![%s](images/%s.svg)" % (title, s["id"]), ""]
        if s.get("text"):
            lines += [pick(s["text"], lang).strip(), ""]
    with open(os.path.join(out, "shots.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
