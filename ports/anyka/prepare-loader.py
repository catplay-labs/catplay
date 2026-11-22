#!/usr/bin/env python3
"""Copy the pinned SDK's normal kexec page allocator into an out-of-tree module."""
import argparse
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('sdk', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
revision = subprocess.check_output(['git', '-C', str(args.sdk), 'rev-parse', 'HEAD'], text=True).strip()
if revision != 'a7c6e75c154db2b5c270603d76f85fa66e0a3127':
    raise SystemExit('SDK revision mismatch')
kernel = args.sdk / 'os/kernel' 
source = kernel / 'kernel/kexec_core.c'
text = source.read_text()
# Exclude crash handling and its built-in initcalls. Crash staging is rejected by
# our ioctl; retain upstream normal page allocation, collision handling, and cleanup.
prefix = text[:text.index('DEFINE_MUTEX(kexec_mutex);')]
body = text[text.index('#define KIMAGE_NO_DEST'):text.index('void crash_kexec(')]
args.output.mkdir(parents=True, exist_ok=True)
(args.output / 'core.c').write_text(prefix + '\nDEFINE_MUTEX(kexec_mutex);\nstruct resource crashk_res;\n' + body)
shutil.copyfile(kernel / 'kernel/kexec.c', args.output / 'load.c')
shutil.copyfile(kernel / 'arch/arm/kernel/relocate_kernel.S', args.output / 'relocate.S')
for name in ['driver.c', 'arm.c', 'Makefile']:
    shutil.copyfile(Path(__file__).parent / 'loader' / name, args.output / name)
print(f'Prepared loader source: {args.output}')
