#!/usr/bin/env python3
"""Bundle the Bluetooth daemons, reusing the application's GNU runtime."""

import argparse
from pathlib import Path
import re
import shutil
import subprocess
import uuid

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("sysroot", type=Path)
parser.add_argument("app_bundle", type=Path)
parser.add_argument("clock_library", type=Path)
parser.add_argument("output", type=Path)
a = parser.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
libdir = a.output / "lib"
libdir.mkdir(exist_ok=True)
for source, name in [
    ("usr/libexec/bluetooth/bluetoothd", "bluetoothd"),
    ("usr/bin/dbus-daemon", "dbus-daemon"),
    ("usr/bin/btmgmt", "btmgmt"),
]:
    shutil.copyfile(a.sysroot / source, a.output / name)
    (a.output / name).chmod(0o755)
queue = [
    f
    for f in a.output.iterdir()
    if f.is_file() and f.name in ["bluetoothd", "dbus-daemon", "btmgmt"]
]
seen = set()
while queue:
    item = queue.pop()
    dynamic = subprocess.check_output(["readelf", "-d", str(item)], text=True)
    for name in re.findall(r"\(NEEDED\).*\[([^\]]+)\]", dynamic):
        if name in seen or (a.app_bundle / "lib" / name).exists():
            continue
        source = next(
            (
                a.sysroot / folder / name
                for folder in ["lib/arm-linux-gnueabi", "usr/lib/arm-linux-gnueabi"]
                if (a.sysroot / folder / name).is_file()
            ),
            None,
        )
        if source is None:
            raise RuntimeError(f"Missing library {name}")
        target = libdir / name
        shutil.copyfile(source.resolve(), target)
        target.chmod(0o755)
        seen.add(name)
        queue.append(target)
shutil.copyfile(a.clock_library, libdir / "libstock-clock.so")
(
    a.output / "system.conf"
).write_text("""<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>system</type>
  <listen>unix:path=/tmp/catplay-bt/system_bus_socket</listen>
  <auth>EXTERNAL</auth>
  <policy user="root">
    <allow own="*"/>
    <allow send_destination="*"/>
    <allow receive_sender="*"/>
  </policy>
</busconfig>
""")
(a.output / "machine-id").write_text(uuid.uuid4().hex + "\n")
shutil.copyfile(Path(__file__).parent / "start-bluetooth.sh", a.output / "start")
(a.output / "start").chmod(0o755)
print(f"Packaged Bluetooth daemons with {len(seen)} additional libraries")
