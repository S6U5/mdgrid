"""mdgrid の機能紹介の共通部品。要るのは python3 と cargo だけ(tmux は要らない。macOS と Linux)。

今のターミナルの中で、疑似端末の上にシェルを立て、見本(examples/)の写しと専用の設定・状態のフォルダで
mdgrid を動かしてキーを送る。字幕は画面のいちばん下の行に出す(シェルと mdgrid には1行少ない大きさを
伝え、その行には描かせない)。再生中もあとも、打ったキーはそのままシェルに届く。exit(か Ctrl+D)で
終わると、写しを消して端末を元に戻す。

紹介のスクリプトの形(all.py がまとめて流せるように、取り込んだだけでは動かさない):

    import tour
    TITLE = "題"

    def setup(t):                                    # 無くてよい
        t.file("config/mdgrid/places.toml", "...")   # 写しの中に置くファイル(t.work が写しの場所)

    def steps(t):
        t.say("字幕")
        t.run("mdgrid")       # 打って Enter
        t.type("タスク")       # 1字ずつ打つ
        t.keys("Enter", "Down", "C-s")
        t.back(8)             # BSpace を 8 回
        t.pause(1)

    if __name__ == "__main__":
        tour.main(TITLE, steps, setup)

steps は見本の保管庫(examples/vault)で始まる前提で書く(all.py は区切りごとに t.home() で戻す)。

環境変数: MDGRID_REPO(リポの場所。既定はこのフォルダの親)、TOUR_SPEED(1 が普通。2 なら倍の速さ)、TOUR_LANG(既定 ja_JP.UTF-8)。
"""

import fcntl
import os
import pty
import select
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import threading
import time
import tty

REPO = os.environ.get("MDGRID_REPO", os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SPEED = float(os.environ.get("TOUR_SPEED", "1"))

KEYS = {
    "Enter": b"\r",
    "Escape": b"\x1b",
    "Tab": b"\t",
    "BTab": b"\x1b[Z",
    "BSpace": b"\x7f",
    "Up": b"\x1b[A",
    "Down": b"\x1b[B",
    "Right": b"\x1b[C",
    "Left": b"\x1b[D",
    "Space": b" ",
}


def key_bytes(name):
    if name in KEYS:
        return KEYS[name]
    if name.startswith("C-") and len(name) == 3:
        return bytes([ord(name[2].lower()) & 0x1F])
    return name.encode()


class Tour:
    def __init__(self, title):
        self.title = title
        self.caption = title
        self.master = None
        # 字幕を描き直す合図(字幕は出力が止まったときだけ描く。mdgrid の出力の途中に挟まないため)。
        self.wake_r, self.wake_w = os.pipe()
        print("mdgrid を組む…", flush=True)
        subprocess.run(["cargo", "build", "--release", "-q"], cwd=REPO, check=True)
        self.work = tempfile.mkdtemp(prefix="mdgrid-tour.")
        os.makedirs(os.path.join(self.work, "config", "mdgrid"))
        os.makedirs(os.path.join(self.work, "state"))
        shutil.copytree(os.path.join(REPO, "examples"), os.path.join(self.work, "examples"), symlinks=True)

    # ---- 用意 ----

    def file(self, rel, text):
        p = os.path.join(self.work, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w", encoding="utf-8") as f:
            f.write(text)

    # ---- 操作(steps の中で使う) ----

    def pause(self, secs):
        time.sleep(secs / SPEED)

    def say(self, text, secs=None):
        self.caption = text
        os.write(self.wake_w, b"!")
        self.pause(secs if secs is not None else max(2.5, len(text) * 0.12))

    def send(self, data):
        os.write(self.master, data)

    def type(self, text):
        for ch in text:
            self.send(ch.encode())
            time.sleep(0.11 / SPEED)

    def keys(self, *names):
        for n in names:
            self.send(key_bytes(n))
            self.pause(0.45)

    def back(self, n):
        for _ in range(n):
            self.send(KEYS["BSpace"])
            time.sleep(0.05)
        self.pause(0.4)

    def home(self):
        """見本の保管庫(examples/vault)に戻って画面を消す(続けて流すときの区切り)。"""
        self.keys("C-l")
        self.run("cd " + os.path.join(self.work, "examples", "vault") + " && clear")

    def run(self, cmd):
        self.type(cmd)
        self.pause(0.6)
        self.keys("Enter")
        self.pause(1.2)

    # ---- 画面 ----

    def _size(self):
        try:
            rows, cols = struct.unpack("hh", fcntl.ioctl(sys.stdout.fileno(), termios.TIOCGWINSZ, b"\0" * 4))
        except OSError:
            rows, cols = 34, 120
        return max(rows, 5), max(cols, 20)

    def _resize(self, *_):
        rows, cols = self._size()
        self.rows, self.cols = rows, cols
        if self.master is not None:
            fcntl.ioctl(self.master, termios.TIOCSWINSZ, struct.pack("hhhh", rows - 1, cols, 0, 0))
            os.write(self.wake_w, b"!")

    def _width(self, s):
        import unicodedata
        return sum(2 if unicodedata.east_asian_width(c) in "WF" else 1 for c in s)

    def _draw_caption(self):
        if self.master is None:
            return
        text = " " + self.caption + " "
        # 幅に収める(はみ出す分は切る)。
        out, w = "", 0
        for c in text:
            cw = self._width(c)
            if w + cw > self.cols:
                break
            out += c
            w += cw
        bar = "\x1b[1;30;43m" + out + " " * (self.cols - w) + "\x1b[0m"
        # 保存 → 上の行だけを流れる範囲にする(下の行が流れないように) → 最下行に字幕 → 戻す。
        seq = "\x1b7\x1b[1;%dr\x1b[%d;1H%s\x1b8" % (self.rows - 1, self.rows, bar)
        os.write(sys.stdout.fileno(), seq.encode())

    # ---- 流す ----

    def play(self, steps):
        lang = os.environ.get("TOUR_LANG", "ja_JP.UTF-8")
        env = {
            "PATH": os.path.join(REPO, "target", "release") + ":" + os.environ.get("PATH", "/usr/bin:/bin"),
            "HOME": os.environ.get("HOME", self.work),
            "TERM": os.environ.get("TERM", "xterm-256color"),
            "LANG": lang,
            "XDG_CONFIG_HOME": os.path.join(self.work, "config"),
            "XDG_STATE_HOME": os.path.join(self.work, "state"),
            "PS1": "demo $ ",
        }
        if os.environ.get("COLORTERM"):
            env["COLORTERM"] = os.environ["COLORTERM"]
        shell = ["/bin/zsh", "-f"] if os.path.exists("/bin/zsh") else ["/bin/sh"]
        if shell[0].endswith("zsh"):
            env["PROMPT"] = "demo $ "
        self.rows, self.cols = self._size()
        pid, self.master = pty.fork()
        if pid == 0:
            os.chdir(os.path.join(self.work, "examples", "vault"))
            os.execve(shell[0], shell, env)
        stdin = sys.stdin.fileno()
        saved = termios.tcgetattr(stdin)
        old_winch = signal.signal(signal.SIGWINCH, self._resize)
        try:
            tty.setraw(stdin)
            os.write(sys.stdout.fileno(), b"\x1b[2J\x1b[H")
            self._resize()

            def driver():
                try:
                    self.pause(1.5)
                    steps(self)
                    self.say("おわり。ここからは自分で触れる。exit か Ctrl+D で終わる", 0)
                except OSError:
                    pass  # 先にシェルが終わった

            threading.Thread(target=driver, daemon=True).start()
            self._pump(stdin)
        finally:
            signal.signal(signal.SIGWINCH, old_winch)
            termios.tcsetattr(stdin, termios.TCSADRAIN, saved)
            # 流れる範囲を戻し、字幕の行を消す。
            os.write(sys.stdout.fileno(), ("\x1b[r\x1b[%d;1H\x1b[2K\r\n" % self.rows).encode())
            try:
                os.waitpid(pid, 0)
            except ChildProcessError:
                pass
            shutil.rmtree(self.work, ignore_errors=True)

    def _pump(self, stdin):
        out = sys.stdout.fileno()
        dirty = True
        while True:
            try:
                # 描き直しが要るときは、出力が少し止まるのを待ってから字幕を描く。
                r, _, _ = select.select([stdin, self.master, self.wake_r], [], [], 0.04 if dirty else None)
            except InterruptedError:
                continue
            if not r:
                self._draw_caption()
                dirty = False
                continue
            if self.wake_r in r:
                os.read(self.wake_r, 1024)
                dirty = True
            if self.master in r:
                try:
                    data = os.read(self.master, 65536)
                except OSError:
                    return
                if not data:
                    return
                os.write(out, data)
                dirty = True
            if stdin in r:
                data = os.read(stdin, 1024)
                if not data:
                    return
                os.write(self.master, data)


def main(title, steps, setup=None):
    """1本の紹介を流す(各スクリプトの __main__ から呼ぶ)。"""
    t = Tour(title)
    if setup:
        setup(t)
    t.play(steps)
