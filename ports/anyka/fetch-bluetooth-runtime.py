#!/usr/bin/env python3
from pathlib import Path
import urllib.request, lzma, hashlib, subprocess, tarfile, io, re, json, argparse

parser = argparse.ArgumentParser(
    description="Fetch ARM Bluetooth runtime packages from Debian bookworm"
)
parser.add_argument("output", type=Path)
p = parser.parse_args().output
p.mkdir(parents=True, exist_ok=True)
index = p / "Packages.xz"
if not index.exists():
    urllib.request.urlretrieve(
        "https://deb.debian.org/debian/dists/bookworm/main/binary-armel/Packages.xz",
        index,
    )
packages = {}
for stanza in lzma.decompress(index.read_bytes()).decode().split("\n\n"):
    fields = {}
    for line in stanza.splitlines():
        if line and line[0] != " " and ": " in line:
            k, v = line.split(": ", 1)
            fields[k] = v
    if "Package" in fields:
        packages[fields["Package"]] = fields
queue = ["bluez", "dbus-daemon"]
seen = set()
manifest = []
while queue:
    name = queue.pop()
    if name in seen or name in ["libc6", "libgcc-s1"]:
        continue
    seen.add(name)
    entry = packages[name]
    for dep in entry.get("Depends", "").split(","):
        match = re.match(r"\s*([a-z0-9+.-]+)", dep)
        if match and (match[1].startswith("lib") or match[1] == "zlib1g"):
            queue.append(match[1])
    filename = entry["Filename"]
    local = p / Path(filename).name
    if not local.exists():
        urllib.request.urlretrieve("https://deb.debian.org/debian/" + filename, local)
    assert hashlib.sha256(local.read_bytes()).hexdigest() == entry["SHA256"]
    print(name, entry["Version"], flush=True)
    manifest.append({k: entry[k] for k in ["Package", "Version", "Filename", "SHA256"]})
    listing = subprocess.check_output(["ar", "t", str(local)], text=True).splitlines()
    data_name = next(x for x in listing if x.startswith("data.tar"))
    data = subprocess.check_output(["ar", "p", str(local), data_name])
    root = p / "sysroot"
    root.mkdir(exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        archive.extractall(root, filter="data")
(p / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
