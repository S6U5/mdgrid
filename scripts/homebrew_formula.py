#!/usr/bin/env python3
"""Homebrew の formula(mdgrid.rb)を、リリースのアーカイブのチェックサムから作って標準出力に出す。

    python3 scripts/homebrew_formula.py v0.4.0 dist > Formula/mdgrid.rb

dist は release.yml の build が作る `mdgrid-<タグ>-<target>.tar.gz.sha256` のある置き場。
formula はビルド済みのアーカイブを落とすだけで、Rust は要らない。release.yml の homebrew のジョブが
タグごとに走らせ、tap のリポ(S6U5/homebrew-tap)に書き込む。手順は docs/design.md の「配布」。
"""

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
REPO = "https://github.com/S6U5/mdgrid"

# Homebrew の (OS, CPU) の枝と、release.yml の target。
TARGETS = {
    ("macos", "arm"): "aarch64-apple-darwin",
    ("macos", "intel"): "x86_64-apple-darwin",
    ("linux", "arm"): "aarch64-unknown-linux-gnu",
    ("linux", "intel"): "x86_64-unknown-linux-gnu",
}


def checksum(dist: pathlib.Path, archive: str) -> str:
    """`<archive>.sha256`(sha256sum の形)の最初の語。"""
    path = dist / f"{archive}.sha256"
    if not path.is_file():
        sys.exit(f"チェックサムのファイルが無い: {path}")
    digest = path.read_text().split()[0]
    if not re.fullmatch(r"[0-9a-f]{64}", digest):
        sys.exit(f"sha256 の形でない: {path}")
    return digest


def formula(tag: str, dist: pathlib.Path) -> str:
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        sys.exit(f"タグは v1.2.3 の形で渡す(試しの版は tap に出さない): {tag}")
    package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]

    def branch(os_name: str, cpu: str) -> str:
        archive = f"mdgrid-{tag}-{TARGETS[(os_name, cpu)]}.tar.gz"
        return (
            f"    on_{cpu} do\n"
            f'      url "{REPO}/releases/download/{tag}/{archive}"\n'
            f'      sha256 "{checksum(dist, archive)}"\n'
            f"    end\n"
        )

    def os_block(os_name: str) -> str:
        return f"  on_{os_name} do\n{branch(os_name, 'arm')}{branch(os_name, 'intel')}  end\n"

    return f'''# Written by scripts/homebrew_formula.py in {REPO} on each release. Do not edit by hand.
class Mdgrid < Formula
  desc "{package["description"]}"
  homepage "{package["homepage"]}"
  version "{tag[1:]}"
  license any_of: ["MIT", "Apache-2.0"]

{os_block("macos")}
{os_block("linux")}
  def install
    bin.install "mdgrid"
    man1.install "mdgrid.1"
    bash_completion.install "completions/mdgrid.bash" => "mdgrid"
    zsh_completion.install "completions/mdgrid.zsh" => "_mdgrid"
    fish_completion.install "completions/mdgrid.fish"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/mdgrid --version")
  end
end
'''


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    sys.stdout.write(formula(sys.argv[1], pathlib.Path(sys.argv[2])))
