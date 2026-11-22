#!/usr/bin/env python3
"""Collect the ARM executable and its shared-library closure into one bundle."""

import argparse
from pathlib import Path
import re
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("binary", type=Path)
p.add_argument("output", type=Path)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
libdir = a.output / "lib"
libdir.mkdir(exist_ok=True)
binary = a.output / "catplay_c2a"
subprocess.run(
    ["arm-linux-gnueabi-strip", "-o", str(binary), str(a.binary)], check=True
)
search = [
    Path("/lib/arm-linux-gnueabi"),
    Path("/usr/lib/arm-linux-gnueabi"),
    Path("/usr/arm-linux-gnueabi/lib"),
]
search += list(Path("/usr/lib/gcc-cross/arm-linux-gnueabi").glob("*"))
queue = [binary]
seen = set()
while queue:
    item = queue.pop()
    dynamic = subprocess.check_output(
        ["arm-linux-gnueabi-readelf", "-d", str(item)], text=True
    )
    for name in re.findall(r"\(NEEDED\).*\[([^\]]+)\]", dynamic):
        if name in seen:
            continue
        source = next((base / name for base in search if (base / name).is_file()), None)
        if source is None:
            raise SystemExit(f"Missing ARM runtime library: {name}")
        target = libdir / name
        shutil.copyfile(source.resolve(), target)
        target.chmod(0o755)
        seen.add(name)
        queue.append(target)
        print(name)
if "ld-linux.so.3" not in seen:
    raise SystemExit("Expected GNU ARM dynamic loader")
(a.output / "run").write_text("""#!/bin/sh
set -eu
case "$0" in */*) bundle=${0%/*};; *) bundle=.;; esac
bundle=$(CDPATH= cd -- "$bundle" && pwd)
cd "$bundle"
exec "$bundle/lib/ld-linux.so.3" --library-path "$bundle/lib" "$bundle/catplay_c2a" "$@"
""")
(a.output / "run").chmod(0o755)
print(f"Runtime closure: {len(seen)} libraries")
