#!/usr/bin/env python3
"""Stage stock-kernel Bluetooth modules with the audited socket layout."""

import argparse
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("sdk", type=Path)
p.add_argument("output", type=Path)
a = p.parse_args()
if (
    subprocess.check_output(
        ["git", "-C", str(a.sdk), "rev-parse", "HEAD"], text=True
    ).strip()
    != "a7c6e75c154db2b5c270603d76f85fa66e0a3127"
):
    p.error("Unexpected SDK revision")
kernel = a.sdk / "os/kernel"
a.output.mkdir(parents=True, exist_ok=True)
for folder, target in [("net/bluetooth", "bluetooth"), ("drivers/bluetooth", "hci")]:
    shutil.copytree(
        kernel / folder,
        a.output / target,
        dirs_exist_ok=True,
        ignore=shutil.ignore_patterns("*.o", "*.ko", "*.mod.c", "*.cmd"),
    )
for source in ["lib/crc16.c", "crypto/cmac.c", "crypto/ecb.c"]:
    shutil.copyfile(kernel / source, a.output / Path(source).name)
(a.output / "compat.h").write_text("""#include <net/sock.h>
/* Stock lock_sock_nested reads/writes owned at 112. IPv6 is required. */
_Static_assert(__builtin_offsetof(struct sock, sk_lock.owned) == 112, "stock sock lock ABI");
""")
(a.output / "Makefile").write_text("""obj-m += bluetooth/ hci/ crc16.o cmac.o ecb.o
subdir-ccflags-y := -include $(src)/compat.h
""")
print(a.output)
