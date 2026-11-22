#!/usr/bin/env python3
"""Upload and run the application bundle through the existing RAM session."""

import argparse
import hashlib
import io
import json
from pathlib import Path
import tarfile

from device import Device

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("--session", required=True, type=Path)
p.add_argument("--bundle", required=True, type=Path)
a = p.parse_args()
config = json.loads(a.session.read_text())
d = Device(config["host"], config["port"], config["token"], timeout=60)
stream = io.BytesIO()
with tarfile.open(fileobj=stream, mode="w:gz") as archive:
    for name in ["catplay_c2a", "run", "c2a.toml"]:
        archive.add(a.bundle / name, arcname=name)
    for library in sorted((a.bundle / "lib").iterdir()):
        if library.is_file():
            archive.add(library, arcname=f"lib/{library.name}")
data = stream.getvalue()
# Chunking limits the shell's memory use on the 64 MiB device.
d.run(": > /tmp/catplay-c2a.tar.gz")
for start in range(0, len(data), 512 * 1024):
    d.upload("/tmp/catplay-c2a-chunk", data[start : start + 512 * 1024])
    d.run(
        "cat /tmp/catplay-c2a-chunk >> /tmp/catplay-c2a.tar.gz && rm /tmp/catplay-c2a-chunk"
    )
checksum = d.run("md5sum /tmp/catplay-c2a.tar.gz").split()[0].decode()
if checksum != hashlib.md5(data).hexdigest():
    raise RuntimeError("Bundle transfer checksum mismatch")
print(f"Uploaded {len(data)} bytes", flush=True)
print(
    d.run("""set -e
if test -f /tmp/catplay-c2a.pid; then
    pid=$(cat /tmp/catplay-c2a.pid)
    if kill -0 "$pid" 2>/dev/null; then
        case "$(cat /proc/$pid/cmdline)" in
            *catplay_c2a*) kill "$pid"; sleep 1;;
            *) echo 'PID file does not identify CatPlay'; exit 1;;
        esac
    fi
fi
mkdir -p /tmp/catplay-c2a/state
tar xzf /tmp/catplay-c2a.tar.gz -C /tmp/catplay-c2a
rm /tmp/catplay-c2a.tar.gz
cd /tmp/catplay-c2a
./run >/tmp/catplay-c2a.log 2>&1 </dev/null &
echo $! >/tmp/catplay-c2a.pid
cat /tmp/catplay-c2a.pid
""").decode(),
    end="",
)
