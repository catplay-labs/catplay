#!/usr/bin/env python3
"""Stage the application with Anyka userspace compatibility changes."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("repo", type=Path)
p.add_argument("output", type=Path)
a = p.parse_args()
repo = a.repo.resolve()
port = Path(__file__).resolve().parent
metadata = json.loads(
    subprocess.check_output(
        [
            "cargo",
            "+1.94.0",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            "armv5te-unknown-linux-gnueabi",
            "--manifest-path",
            str(repo / "Cargo.toml"),
        ]
    )
)
crates = [
    x
    for x in metadata["packages"]
    if x["name"] == "usb-gadget" and x["version"] == "0.7.6"
]
if len(crates) != 1 or not crates[0]["source"].startswith("registry+"):
    p.error("Expected usb-gadget 0.7.6 from crates.io")
source = a.output / "source"
source.mkdir(parents=True, exist_ok=True)
files = (
    subprocess.check_output(["git", "-C", str(repo), "ls-files", "-z"])
    .decode()
    .split("\0")
)
for name in filter(None, files):
    original = repo / name
    if not original.is_file():
        continue
    target = source / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(original, target)
private_usb = source / "anyka-usb-gadget"
shutil.copytree(
    Path(crates[0]["manifest_path"]).parent, private_usb, dirs_exist_ok=True
)
(private_usb / "src/function/custom/ffs.rs").chmod(0o644)
subprocess.run(
    ["git", "apply", str(port / "usb-smoke/legacy-os-descriptor.patch")],
    cwd=private_usb,
    check=True,
)
f = source / "Cargo.toml"
s = f.read_text().replace(
    "[patch.crates-io]",
    '[patch.crates-io]\nusb-gadget = { path = "anyka-usb-gadget" }',
    1,
)
f.write_text(s)
f = source / "c2a/catplay_c2a/Cargo.toml"
s = f.read_text().replace(
    "[dependencies]",
    """[dependencies]
# Build these native libraries without the stock uClibc ABI.
openssl = { version = "0.10.75", features = ["vendored"] }
dbus = { version = "0.9.9", features = ["vendored"] }
libusb1-sys = { version = "0.7.0", default-features = false, features = ["vendored"] }
turbojpeg = { version = "1.4.0", default-features = false, features = ["cmake"] }
""",
    1,
)
f.write_text(s)
shutil.copyfile(port / "stock-clock.rs", source / "c2a/catplay_c2a/src/stock_clock.rs")
f = source / "c2a/catplay_c2a/src/main.rs"
s = f.read_text().replace(
    "use std::process::exit;",
    '#[cfg(target_arch = "arm")]\nmod stock_clock;\n\nuse std::process::exit;',
    1,
)
f.write_text(s)
print(source)
