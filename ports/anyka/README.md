# Anyka AK3918AV130 bring-up

This directory contains the CatPlay development port for the Anyka version of
the Carlinkit Mini Ultra. A persistent stock-kernel image now boots CatPlay,
Bluetooth, Wi-Fi, and the USB gadget automatically. Phone pairing and CarPlay
streaming still need validation. It also contains an ARMv5 build environment,
a stock-kernel compatibility probe, a device transport, and a RAM-only kexec
loader. The loader
loads on the stock kernel and passes image staging/unloading tests. A diagnostic
kernel builds, but the first execution attempt disconnected USB without
producing a serial console. Replacement-kernel boot is unconfirmed.

A USB serial root shell now works on the stock kernel without a UART adapter.
The vendor controller needs a host USB reset after changing gadget drivers;
otherwise Linux retains the old iPhone identity. This explains a USB observation
from bring-up, but does not establish that the replacement kernel booted.

## Hardware checked

The first unit reports `platform:AK3918AN`, `custom:suding`, and stock system
version `20260612170300CA`. Its hardware and kernel report:

- ARM926EJ-S / ARMv5TEJ, approximately 60 MiB usable RAM, 16 MiB SPI NOR.
- Linux 4.4.302-cip94, ARM EABI5, uClibc userspace.
- USB controller `20200000.usb`, with the vendor `ak_udc` module.
- AIC8800 radio over SDIO, interface `wlan0`, and I2C nodes 0 and 1.

Marketing names and the `product_Model` field do not identify the chip reliably.
The authentication chip responds on `/dev/i2c-0` at `0x10`. CatPlay's Rust I2C
backend reads its 608-byte certificate and returns a 64-byte challenge response.
Bluetooth access still needs verification.

The kernel's `sys_kexec_load` and `sys_kexec_file_load` resolve to
`sys_ni_syscall`. Installing a userspace kexec executable will not enable RAM
boot. A module loader or another boot mechanism is required.

## Build the compatibility probe

The build needs Docker and this SDK revision:

```sh
git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/Link-WQS/AnyCloud39AV130_SDK_V1.03.git anyka-sdk
git -C anyka-sdk sparse-checkout set os/kernel
git -C anyka-sdk rev-parse HEAD
# Expected: a7c6e75c154db2b5c270603d76f85fa66e0a3127
ports/anyka/build-probe.sh anyka-sdk /absolute/path/to/private-build
```

The script rejects a different SDK revision. If the upstream branch advances,
fetch and check out the recorded commit first. Keep build output and device
backups outside this repository. The SDK already supplies `localversion-cip`;
adding `-cip94` to `CONFIG_LOCALVERSION` would duplicate the suffix.

The probe's module ABI version string is `4.4.302-cip94 mod_unload ARMv5`.
Its module structure is 416 bytes, with init and exit pointers at offsets 208
and 384. These match the stock `ak_udc.ko`. This comparison alone does not prove
that every SDK structure matches the vendor kernel.

`catplay_anyka_probe` logs the kernel version and the presence of restart
functions. It does not change registers, flash, boot arguments, or USB bindings. Loading
and unloading this probe have both passed on the first dongle.

## Device access

Join the dongle's Wi-Fi using your network manager. On the tested unit the device
address is `192.168.50.100`. Reading its identity has no shell side effects:

```sh
python3 ports/anyka/device.py identify
```

The stock `submition.cgi` unconditionally executes
`rm -rf /usr/data/lib` before evaluating the filename. This removes persistent
vendor binary overrides. Do not use it as a general shell endpoint. Earlier
bring-up requests used it before this side effect was identified, so backups
made afterward cannot establish what that directory originally contained.

The transport's bootstrap needs explicit authorization for one call to that
handler. After authorization, the command is:

```sh
python3 ports/anyka/device.py --session /private/path/anyka-session.json \
    bootstrap --allow-stock-bootstrap
```

Bootstrap uses `downfile.cgi` chunk modes `.1` and `.2` to write and checksum a
unique script in `/tmp`. Mode `.0`, which clears the upload directory, is never
used. A single submission call executes that script. It starts BusyBox httpd
on the dongle's Wi-Fi address, port 8081, with a random Basic authentication
password and a CGI under `/tmp`. The session file is created with mode 0600.
The server disappears at reboot. Never commit or share the session file.

Normal commands use that authenticated RAM server:

```sh
python3 ports/anyka/device.py --session /private/path/anyka-session.json \
    run 'id; cat /proc/version'
python3 ports/anyka/device.py --session /private/path/anyka-session.json \
    upload /private/build/catplay_anyka_probe.ko /tmp/catplay_anyka_probe.ko
python3 ports/anyka/device.py --session /private/path/anyka-session.json \
    run 'insmod /tmp/catplay_anyka_probe.ko && dmesg | grep catplay_anyka'
python3 ports/anyka/device.py --session /private/path/anyka-session.json \
    run 'rmmod catplay_anyka_probe && dmesg | grep catplay_anyka'
python3 ports/anyka/device.py --session /private/path/anyka-session.json stop
```

Uploads through the RAM server are restricted to files directly under `/tmp`
and verified with a device-side checksum. Shell commands run as root; review
them before execution. A shell command can still write persistent files.

## USB console without a UART adapter

Build modules compatible with the stock kernel, plus a static shell launcher:

```sh
ports/anyka/build-usb-console.sh anyka-sdk /private/usb-console-build
```

Upload `usb-console/{u_serial,usb_f_acm,usb_f_serial,usb_f_obex,g_serial}.ko`
and `usb-shell` to `/tmp` using the RAM transport. Through that transport, load
the modules and replace the stock gadget:

```sh
insmod /tmp/u_serial.ko
insmod /tmp/usb_f_acm.ko
insmod /tmp/usb_f_serial.ko
insmod /tmp/usb_f_obex.ko
echo '' > /sys/kernel/config/usb_gadget/anyka/UDC
insmod /tmp/g_serial.ko
chmod 700 /tmp/usb-shell
/tmp/usb-shell </dev/null >/tmp/usb-shell.log 2>&1 &
echo $! > /tmp/catplay-usb-shell.pid
```

Wi-Fi remains available. The host can retain the cached iPhone descriptors
despite the driver replacement. Find the dongle's bus and device numbers with
`lsusb`, then reset that device explicitly. For example:

```sh
python3 ports/anyka/usb-console.py reset /dev/bus/usb/006/029
```

The reset checks the cached USB identity and, for the fake iPhone, the known
development serial number. Device numbers change on re-enumeration. An
`ENODEV` reset result is expected when Linux removes the old identity.
Within a few seconds the gadget appears as `0525:a4a7`, with `/dev/ttyACM0`:

```sh
python3 ports/anyka/usb-console.py command /dev/ttyACM0 'id; cat /proc/version'
```

This path returned `uid=0(root)` on the first unit. It provides a console after
the gadget starts; it cannot capture kernel failures before USB initializes.
To restore stock USB, restart stock firmware through the Wi-Fi transport:

```sh
sync
reboot -f
```

These changes also disappear at power cycle. Later USB ABI findings below mean
that removing SDK gadget modules through native cleanup callbacks requires
additional validation; restarting clears the temporary gadget setup.

## Authentication on the stock kernel

This diagnostic cross-compiles the repository's actual `catplay_mfi` crate. It
needs Rust, rustup, Docker, and internet access for the target and dependencies:

```sh
ports/anyka/build-mfi-smoke.sh /private/mfi-build
python3 ports/anyka/device.py --session /private/anyka-session.json \
    upload /private/mfi-build/catplay-mfi-smoke /tmp/catplay-mfi-smoke
python3 ports/anyka/device.py --session /private/anyka-session.json \
    run 'chmod 700 /tmp/catplay-mfi-smoke && /tmp/catplay-mfi-smoke --challenge'
```

The program defaults to reading the certificate; `--challenge` also requests
one response to a fixed 32-byte test digest. It prints lengths, without printing
certificate or signature contents. On the tested dongle:

```text
MFi certificate read: 608 bytes
MFi challenge response: 64 bytes
MFi diagnostic elapsed: 154 ms
```

The build targets `armv5te-unknown-linux-musleabi` with a static libc. The stock
kernel's custom `clock_gettime64` backport returns success without populating
the result for clocks other than `CLOCK_REALTIME`. Raw syscall 403 reproduced
this for `CLOCK_MONOTONIC`; raw syscall 263 returned a valid legacy timestamp.
The SDK contains the same defective implementation. This caused both GNU and
musl Rust builds to fail in `Instant::now` before the workaround.

`stock-clock.rs` overrides musl's weak `clock_gettime` function and converts
legacy ARM timestamps to musl's ABI. This is restricted to the ARM musl smoke
test. It retains the legacy syscall's 32-bit seconds range. The diagnostic
kernel build also applies a patch to populate the 64-bit result for every
supported clock with explicit 64-bit nanoseconds. That kernel builds; its new
clock implementation has not been tested on hardware.

The stock kernel already contains FunctionFS code. Loading another `usb_f_fs`
fails with a duplicate `ffs_lock` export. SDK USB networking modules load, but
creating an NCM instance exposed an incompatible `usb_function_instance`
layout and caused a kernel oops. Native disassembly reads `set_inst_name` and
`free_func_inst` at offsets 84 and 88; SDK headers put them at 80 and 84. Matching
module version strings does not establish compatibility for these structures.
Adding the missing instance slot fixed NCM instance creation. Binding NCM then
failed in `register_netdevice`, exposing a separate network structure mismatch.
Do not treat that single slot adjustment as a complete stock-module ABI fix.
Native FunctionFS instance creation and mounting did pass after a clean reboot.

The networking audit found that the SDK also inserts two wireless-handler
pointers absent from the stock `net_device` layout. Private header copies now
remove those eight bytes and reserve the stock USB instance slot. The builder
checks the networking offsets against disassembly, including operations,
addresses, device parent, and the 992-byte aligned private-data boundary.
With both adjustments, NCM binds and carries traffic on the tested firmware.

## USB networking on stock firmware

This route provides root access over the existing USB cable while retaining
Wi-Fi. It uses temporary RAM files and the stock kernel. Build the audited
modules and configure the dongle from an existing Wi-Fi RAM session:

```sh
ports/anyka/build-stock-usb.sh anyka-sdk /private/stock-usb-build
python3 ports/anyka/setup-usb-network.py \
    --session /private/wifi-session.json --build /private/stock-usb-build \
    --output-session /private/usb-session.json
```

The setup script checks the exact tested firmware identity, uploads both
modules, creates an NCM gadget, sets device address `192.168.51.2/24`, and starts
an authenticated RAM server on that address. The output session is mode 0600.
It replaces the temporary USB identity with `0525:a4a1`, serial
`catplay-anyka-probe`. Stock USB returns at reboot.

If the host retains the previous USB identity, use `lsusb` to find its current
bus/device number and run `usb-console.py reset` as described above. Device
numbers can change after enumeration. Configure the new host USB Ethernet
interface as `192.168.51.1/24`, without a gateway. For example:

```sh
nmcli connection add type ethernet ifname '*' con-name catplay-anyka-usb \
    802-3-ethernet.mac-address 02:43:50:00:00:01 \
    ipv4.method manual ipv4.addresses 192.168.51.1/24 ipv4.never-default yes \
    ipv6.method disabled connection.autoconnect no
nmcli connection up catplay-anyka-usb
python3 ports/anyka/device.py --session /private/usb-session.json run 'id'
```

Match the diagnostic MAC address because interface ordering changes when the
vendor function precedes NCM. The tested host already has that
NetworkManager profile; reuse it rather than creating another. USB root access,
the MFi certificate/challenge diagnostic, and a 512 KiB bidirectional payload
transfer all passed. The payload matched byte for byte and the interface
reported no packet errors. These results validate the development link, not
CarPlay operation or every kernel ABI path.

The host can restrict raw USB access after the gadget changes identity. The
prepared udev rule grants the active desktop session access only to the
development vendor/product/serial combination. Install it from your terminal:

```sh
ports/anyka/install-usb-access.sh
```

The script requests `sudo` locally, installs the rule, reloads udev, and applies
it to matching devices. It refuses to replace a different existing rule. The
rule passed `udevadm verify`. The installer has run on the development host;
the USB device now grants the active user read/write access after enumeration.

## Native FunctionFS transport test

The SDK's Linux `ffs-test` program builds as a static ARMv5 executable with
`build-ffs-test.sh SDK OUTPUT`. On the tested stock kernel it writes the v2
descriptors and strings successfully, creating `ep0`, `ep1`, and `ep2`.
An `ffs.iap2` function was added to the same temporary configuration as NCM.
Host enumeration and the bulk traffic test both passed after installing the
host access rule. Once the configuration has enumerated, run:

```sh
python3 ports/anyka/test-ffs-host.py /dev/bus/usb/BUS/DEVICE
```

Use the current USB numbers from `lsusb`. The helper checks the diagnostic
identity, claims only the vendor interface, and transfers a 32 KiB zero pattern
in each direction. It leaves the NCM interfaces attached to their host driver.
This is a transport integration test, not an iAP2 or CarPlay protocol session.

## Rust USB integration test

`build-usb-smoke.sh OUTPUT` builds a static ARMv5 diagnostic using CatPlay's
`usb-gadget` 0.7.6 and Tokio 1.49.0. It includes the stock clock workaround,
an iAP2-compatible vendor interface, and NCM networking. Build output contains
a private patched dependency copy; the shared Cargo cache stays unchanged.

Linux 4.4 FunctionFS requires `Reserved1 = 0` in its input OS compatibility
descriptor. Upstream `usb-gadget` writes 1, causing registration to fail with
`EINVAL`. The private patch selects 0 for Linux 4.4. Host queries still receive
the expected iAP2 compatible ID. This patch must also be applied when building
the full application for the stock kernel.

After stopping the previous FunctionFS owner and unbinding its gadget, upload
and start `catplay-usb-smoke` through the Wi-Fi RAM transport. Configure its
reported device network interface as `192.168.51.2/24`, then activate the host
profile above. The test replaces the USB development gadget and runs until
terminated. Run these host checks with the current USB device numbers:

```sh
python3 ports/anyka/test-ffs-host.py /dev/bus/usb/BUS/DEVICE --echo
python3 ports/anyka/test-ffs-host.py /dev/bus/usb/BUS/DEVICE --os-descriptor
```

Both checks passed on the attached dongle: 20 varied payloads, 4,856 bytes total,
matched byte for byte, and the Microsoft OS string and iAP2 compatibility
descriptor were readable. Authenticated root access over the same NCM gadget
also passed. The diagnostic uses Tokio's asynchronous endpoint APIs. The
dependency's blocking `recv_and_fetch_timeout` panics on an idle timeout and
should not be used for this test.

Stop the old FunctionFS process before unbinding its gadget. On this kernel,
unbind can wait indefinitely while that process keeps the endpoint files open.

## Car-facing iPhone driver backport

The car-facing application uses the custom `g_iphone` driver, including
`/sys/class/iphone`; the FunctionFS result above does not validate that path.
An unmodified build against the pinned SDK fails on newer headers such as
`linux/usb/webusb.h`, `linux/usb/role.h`, and `linux/container_of.h`.
Its bundled composite implementation also uses newer SuperSpeed fields.

`build-stock-iphone.sh SDK OUTPUT` builds `g_iphone` and all four iAP2
support modules. Source copies and the Linux 4.4 composite implementation are
staged privately. The backport preserves the iPhone configurations, bus-power
behavior, device version, and vendor requests before configuration. Its USB
function layout uses the audited stock offsets. The shared iAP2 modules can
also be built separately with `build-stock-iap2.sh SDK OUTPUT`.

The compatibility code adapts class attributes, ID allocation, poll types,
checked flexible-array sizes, and driver attachment. Audio alternate-setting
selection supports this controller's full/high speeds. Shutdown joins USB
producers and drains pending work using APIs available in Linux 4.4.

All five modules compiled, resolved every imported symbol, loaded on the
attached dongle, and later unloaded successfully with no configured host.
`g_iphone` bound to `20200000.usb`. The host read all four configurations
(39, 149, 62, and 117 bytes) and received the expected capability response
`01000000` both before and after configuration. The power request also passed.
These checks exercise the car-facing driver; a CarPlay protocol session has
not passed.

The stock controller acknowledges a zero-length EP0 request without completing
the HID-owned request. `SET_IDLE` therefore left HID control pending and stalled
the next report-descriptor read. The backport queues these acknowledgements
through the composite-owned request. Live testing now passes all 208 report
descriptor bytes and `SET_IDLE`/`GET_IDLE`, including configuration restoration.

During disconnect, the tested host's Linux 7.2.8 Apple charging driver
`apple_mfi_fastcharge` faulted in `power_supply_unregister` and then
`devres_release_group`. USB enumeration stopped on that host bus. A host reboot
is required before further USB testing. The dongle remains reachable over
Wi-Fi; resetting it again does not resolve the host fault.

For laptop tests, load the module with `diagnostic_identity=1`. This uses the
existing `0525:a4a1` / `catplay-anyka-probe` development identity, avoiding the
host Apple charging driver while preserving interface descriptors and vendor
requests. The existing development udev rule applies. The parameter defaults
to false for the car-facing identity. After loading dependencies, use:

```sh
insmod /tmp/catplay-g_iphone.ko diagnostic_identity=1
echo anyka > /sys/class/iphone/create
echo 20200000.usb > /sys/class/iphone/anyka/udc
echo 1 > /sys/class/iphone/anyka/bind
```

Run `test-iphone-host.py /dev/bus/usb/BUS/DEVICE` for the descriptor and vendor
checks. Add `--hid` for the 208-byte report descriptor and HID idle tests.
The helper restores the initial USB configuration. It rejects the Apple
identity to prevent repeating the host charging-driver fault. A gadget reset
can be needed when replacing the previous development identity.

The Anyka role adapter accepts an optional absolute `role_helper` executable.
It invokes that file with `host` or `device` and waits for its exit status,
without a shell. With no helper, device mode works and host mode returns
`EOPNOTSUPP`. Upload `usb-role.sh` as `/tmp/catplay-usb-role`, make it executable,
and load `g_iphone` with `role_helper=/tmp/catplay-usb-role`. The helper exchanges
the stock `ak_hcd.ko` and `ak_udc.ko` modules and checks the resulting controller
binding. The gadget lifecycle remains owned by `g_iphone`.

Manual host/device exchange and the actual USB vendor role-switch request now
pass on the tested unit. Add `--role-switch` to the host test to send that
request after the other checks. This disconnects USB. With a computer that
cannot become a car accessory, the driver times out after five seconds and
rebinds the gadget after a further two seconds. The device number changes.
The backport waits 100 ms before removing the device controller so the
zero-length status acknowledgement can reach the host. Without that wait,
the host receives `EPROTO` even though the controller switches successfully.
These tests validate controller switching and timeout recovery; they do not
validate a successful car accessory probe or a CarPlay session.

## Application and Bluetooth bring-up

The full `catplay_c2a` application now cross-compiles and runs on the stock
ARMv5 kernel. The tested build uses Rust 1.94.0, GCC 12 soft-float ARM tools,
and `--no-default-features` to omit jemalloc. The current UI dependencies still
compile. OpenSSL, D-Bus client code, libusb, and TurboJPEG build from their
vendored sources; other native libraries use Debian bookworm armel packages.

The stripped executable is about 9.7 MiB. Eleven GNU runtime libraries add
about 6.7 MiB. `bundle/run` invokes the bundled loader explicitly, so the
application does not require the stock uClibc loader to understand GNU binaries.
Both GNU clock entry points, `clock_gettime` and `__clock_gettime64`, use the
working legacy syscall through `stock-clock.rs`. This retains the legacy
32-bit seconds limit. A kernel clock fix remains the long-term solution.

Build and upload with:

```sh
ports/anyka/build-c2a.sh /private/c2a-build
python3 ports/anyka/deploy-c2a.py --session /private/ram-session.json \
    --bundle /private/c2a-build/bundle
```

`prepare-c2a.py` stages the current tracked source files in a private workspace
and applies the FunctionFS dependency and clock changes there. The build needs
Rust 1.94.0 on the host for Cargo metadata. The separate Docker image uses the
same compiler version. The environment, application build, runtime closure,
and deployment were exercised directly. A fresh run of the combined Docker
wrapper remains pending because the host kernel fault blocks new container
starts on the development host.

The deployment helper sends 512 KiB chunks to limit shell memory usage. It
starts the program in `/tmp/catplay-c2a`, logs to `/tmp/catplay-c2a.log`, and
records its PID in `/tmp/catplay-c2a.pid`. The sample configuration stores new
HomeKit state under that temporary directory. Development logs can contain
device identifiers; keep them outside the repository.

The application exposed a second Linux 4.4 compatibility issue: this UDC has
no `function` sysfs attribute. `PhoneGadgetDriver` now falls back to
`USB_UDC_DRIVER` in the same UDC's `uevent` file when the attribute is absent.
This removed the repeated bind/unbind failures in the real application.

### Bluetooth kernel and UART

The stock kernel has no Bluetooth stack. Build loadable modules using:

```sh
ports/anyka/build-bluetooth.sh anyka-sdk /private/bluetooth-build
ports/anyka/build-bluetooth-uart.sh /private/bluetooth-uart
```

The modules are `crc16`, `cmac`, `ecb`, `bluetooth`, `rfcomm`, and `hci_uart`.
Load them in that order. Upload `bluetooth-uart`, make it executable, and run
it in the background with its output redirected. It must retain the UART file
descriptor. `--up` also attempts to power on the controller; it can report
`EALREADY` while the kernel's initial setup is in progress.

The SDK configuration disables IPv6, while the stock kernel enables it.
An uncorrected Bluetooth build loads but faults when creating sockets:
`sock.sk_lock.owned` is at offset 80 in that build and 112 in the native kernel.
The build enables IPv6 and asserts offset 112. Native `lock_sock_nested` and
the corrected module access the same field. With that correction, RFCOMM and
raw HCI sockets work without the earlier fault.

The AIC radio responds through `/dev/ttySAK1` using H4 at its existing 115200
baud setting, without hardware flow control. The helper creates `hci0`; the
controller reports its real Bluetooth address and stock device name. No
additional radio firmware download was needed in this test.

### Bluetooth userspace

Build the development daemon bundle against the application runtime:

```sh
ports/anyka/build-bluetooth-runtime.sh /private/c2a-build/bundle /private/bt-runtime
```

The fetcher downloads the armel packages from Debian bookworm, checks package
SHA-256 hashes against the package index, and records a manifest. The packager
collects D-Bus, BlueZ, and their library closure. These binaries were tested
from the [Debian BlueZ package](https://packages.debian.org/bookworm/armel/bluez)
and [D-Bus daemon package](https://packages.debian.org/bookworm/armel/dbus-daemon).

Extract this bundle under `/tmp/catplay-bt` through the RAM transport. The
combined development bundles exceed the stock 30 MiB temporary filesystem
limit; `mount -o remount,size=48m /tmp` provided room for this run. After loading
the kernel modules and attaching UART1, run `/tmp/catplay-bt/start`. It starts a
root-only D-Bus bus and BlueZ with optional plugins disabled. Its GNU daemons
use `libstock-clock.so` to avoid the vendor clock bug. The tested BlueZ version
is 5.66 and D-Bus is 1.14.10.

Start the application with the development bus address:

```sh
cd /tmp/catplay-c2a
DBUS_SYSTEM_BUS_ADDRESS=unix:path=/tmp/catplay-bt/system_bus_socket ./run
```

The live application reads the 608-byte MFi certificate, registers its iAP2
Bluetooth service, applies its CarPlay HCI settings, advertises Bonjour on
`wlan0`, and reaches `WaitingForCar`. Its AirPlay receiver listens on port 5000;
a Wi-Fi `GET /info RTSP/1.0` request returned `200 OK` and a 2,694-byte binary
property list. A phone pairing session and actual CarPlay streaming remain
untested. This is a running development port, not a finished flash image.

## Recovery from a stock driver failure

A software stock reboot restored the device after the first configfs failure.
For failures that can hold driver locks, an explicit recovery module calls the
native `emergency_restart` function without walking driver shutdown callbacks:

```sh
ports/anyka/build-recovery.sh anyka-sdk /private/recovery-build
```

Upload `recovery/catplay_anyka_restart.ko` to `/tmp` through the RAM transport.
Sync the filesystems before explicitly loading it with `restart=1`:

```sh
sync
insmod /tmp/catplay_anyka_restart.ko restart=1
```

This immediately drops the connection and restarts the dongle. Loading without
the parameter fails without restarting. The module successfully recovered the
test unit after the NCM bind failure, restoring stock USB and Wi-Fi without a
physical power cycle. It needs a responsive stock kernel and root access;
it cannot recover a hung replacement kernel. The tested firmware has no
`/proc/sysrq-trigger`. Recreate the RAM transport session after restart.

## RAM loader

Build the module and a statically linked ARMv5 client:

```sh
ports/anyka/build-loader.sh anyka-sdk /absolute/path/to/private-build
```

This reuses the pinned SDK's GPL kexec page allocator, segment loader, and ARM
relocation assembly. The module exposes `/dev/catplay_kexec` with separate load,
status, unload, and execute operations. Only normal boot is supported. Crash
kernels and preserve-context requests are not exposed. Target addresses must be
page aligned, lie within the verified lower 60 MiB of RAM, and occupy at most
16 MiB in total. A zImage signature and a device tree are required.

Upload `loader/catplay_anyka_kexec.ko`, `catplay-kexec`, and a stock DTB into
`/tmp` through the RAM transport. Then run:

```sh
insmod /tmp/catplay_anyka_kexec.ko
chmod 700 /tmp/catplay-kexec
/tmp/catplay-kexec selftest /tmp/catplay-test.dtb
/tmp/catplay-kexec status
```

The self-test rejects addresses outside RAM, overlapping segments, and an
invalid zImage. It stages and unloads a synthetic image without execution, and
checks that a failed replacement preserves the original staged image. These
tests passed on the first dongle. `status` reported zero segments afterward.

## Diagnostic kernel

The build uses private files from the stock firmware backup. Do not distribute
these artifacts as part of the public source tree.

```sh
ports/anyka/build-ramboot.sh anyka-sdk /private/capture /private/ramboot-build
```

The capture directory must contain `rootfs/` and `mtd3-dtb.bin`. The script applies
the included ACM compile fix to the SDK checkout. It enables an embedded
initramfs and the USB serial gadget, and disables MTD flash support. The
initramfs contains the stock BusyBox, uClibc, and USB controller module. The
patched DTB limits memory to the actual 64 MiB and replaces the stock boot
arguments with an initramfs boot.

The first zImage was 2,752,160 bytes. It and a 59,232-byte DTB staged successfully
at `0x81000000` and `0x80f00000`. Every symbol required by the stock USB controller
module is exported by the diagnostic kernel. Staging is performed with:

```sh
/tmp/catplay-kexec stage /tmp/catplay-zImage /tmp/catplay-ramboot.dtb
/tmp/catplay-kexec status
```

`/tmp/catplay-kexec execute` starts the handoff. It disconnects Wi-Fi and can hang
the dongle. The first attempt caused host USB disconnection, with no USB serial
device appearing within 60 seconds. No conclusion about kernel startup is
possible from that observation alone. A power cycle restored stock firmware on the
tested unit. All six firmware partitions, including both environment copies,
matched their pre-test backup checksums afterward. The RAM development server
was restored successfully. Later testing reproduced the missing serial device
on the running stock kernel and fixed it with a host USB reset. Further RAM boot
tests must account for this controller behavior. The initramfs now uses the
same static shell launcher verified on the stock kernel. A second boot attempt
with that launcher still produced no USB device during a 50-second observation;
there was no remaining device to reset. The replacement kernel remains unconfirmed.

## Persistent stock-kernel image

`package-rootfs.py` builds a squashfs ROOTFS image from a private stock partition
backup and the application/module bundles described above. Run it in the
`Dockerfile.image` environment, which adds squashfs-tools to the toolchain.
`build-rootfs.sh` wraps packaging and compiles the static installer. Its inputs
are the stock ROOTFS backup, application bundle, Bluetooth bundle, iPhone
modules directory, Bluetooth modules directory, UART helper, and output path.
Pass `--stock`, `--app`, `--bluetooth`, `--modules`, `--bluetooth-modules`,
`--uart`, `--net-modules`, and a fresh `--output` directory.
The stock kernel has no USB networking, but after a role switch `iap2_scan`
must bind `cdc_ncm` to the car's NCM interface. Copy the vendor-built
`usr/lib/usbnet.ko` and `usr/lib/cdc_ncm.ko` from the stock ROOTFS into one
directory and pass it as `build-rootfs.sh`'s `NET_MODULES` argument
(`--net-modules` for `package-rootfs.py`). `usb-role.sh` loads them in host
mode. Modules from `build-usbnet.sh` hang this kernel; see the car test notes.
`--diagnostic-identity` selects the
computer test identity at boot. Build `bluetooth-uart` from the current source;
it explicitly selects 115200 baud without relying on the vendor daemon.
It also restores `/usr/data/catplay/bluetooth-address` before attaching Linux's
HCI stack. If absent, it derives and persists a local address from the board's
Wi-Fi address. A cold controller otherwise reports a shared default address.
The AIC address command is described in the
[AWTRIX AIC8800 initialization documentation](https://blueforcer.github.io/awtrix-ng/tc002/developers/tc002/bluetooth/).

The output contains `catplay-rootfs.squashfs`, `catplay-bt.tar.gz`, a manifest,
and a private authenticated `device-session.json`. Keep all output private:
the image includes stock binaries and generated management credentials.
The builder rejects a system image larger than the existing 8.5 MiB partition.
Bluetooth supporting libraries expand from DATA into tmpfs; application code
and its libraries execute directly from flash. `bluetoothd` also stays on flash.

The boot scripts initialize the stock radio drivers, start the `CatPlay-Anyka`
access point (development password `catplay-dev`), run DHCP and the authenticated
development command server, start D-Bus and BlueZ, and load `g_iphone` with the
role helper. BusyBox init starts CatPlay and restarts it if it exits. Pairing
state lives under `/usr/data/catplay/state`. BlueZ pairing keys live under
`/usr/data/catplay/bluetooth`, linked from `/var/lib/bluetooth` at boot.

Install the Bluetooth archive as `/usr/data/catplay-bt.tar.gz` before writing
the system image. `flash-rootfs.c` builds as a static ARM executable with the
toolchain. Its `--check IMAGE` validates the image and the tested ROOTFS
partition geometry without writing. `--write-and-reboot IMAGE` loads the image
into RAM, stops other userspace flash readers, erases ROOTFS, writes the image,
compares every written byte, and reboots. Run it from tmpfs, with the image in
tmpfs and enough RAM available for a second image copy. It writes `/dev/mtd5`;
it does not change the bootloader, environment, device tree, or kernel.

After reboot, join `CatPlay-Anyka` and use the generated session file with
`device.py`. For a build using the diagnostic identity, writing `0` to
`/usr/data/catplay/diagnostic-identity` selects the car identity on the next
boot. Writing `1` selects the computer identity. This override persists in DATA.

On the tested dongle, the persistent image boots automatically, reads the MFi
certificate, registers iAP2 Bluetooth, reaches `WaitingForCar`, and answers
AirPlay `/info` with `200 OK`. Full ROOTFS readback matches the built image.
The corrected installer verifies its write and reboots successfully. HomeKit
state and a Bluetooth-storage probe survived a subsequent software reboot.
The car USB identity also enumerates. Available idle RAM is about 30 MiB.
Phone pairing, saved bond restoration, and CarPlay streaming remain untested.

`monitor-device.py --session SESSION --output DIRECTORY` captures USB
enumeration from host sysfs, host kernel events, device uptime and boot ID,
processes, memory, and application/kernel/Bluetooth logs. It polls the existing
authenticated console and sends no USB control requests. Run it before
reconnecting the dongle; the saved Wi-Fi profile must reconnect to its AP.
Device logs in tmpfs disappear at power loss unless captured separately.

For tests away from the computer, set `/usr/data/catplay/capture-enabled` to
`1`, copy the `catplay-capture` binary from the build output to
`/usr/data/catplay/catplay-capture`, and reboot. The helper records kernel messages,
application output, boot IDs, app process IDs, memory, and USB role/status
changes under `/usr/data/catplay/capture`. The launcher also records child
exit statuses and enables Rust backtraces while capture is enabled.
Each boot captures for 20 minutes. Kernel and application logs each retain a
64 KiB current file and one previous file; the event log uses 16 KiB files.
Records sync to flash once per second. Existing logs remain across boots.
Set the flag to `0` and reboot to stop automatic capture.

The helper's rotation and size limits passed host checks. On the dongle, a
controlled USB role-switch timeout kept the same boot ID and app PID; a
controlled application termination recorded exit status 143 and a new app PID
under the same boot ID. These are diagnostic checks, not evidence of the cause
of the repeated CarPlay consent prompt reported during the first car test.

## Car test

On November 20, 2026, wireless CarPlay worked end to end through this dongle in
a 2026 Honda Pilot (Clarion head unit, AirPlay SDK 210.81): the car connects
over USB, the iPhone pairs over Bluetooth and joins the dongle's Wi-Fi, and the
CarPlay UI appears on the car's screen. These issues had to be fixed:

1. **USB networking.** The stock kernel has no `usbnet` or `cdc_ncm`, so the
   role-switch accessory probe failed with `ensure_driver_bound(cdc_ncm) failed
   -19`. Modules built from the SDK sources hung the kernel during enumeration,
   because the SDK lacks the vendor's USB core changes. The stock rootfs ships
   vendor-built `/usr/lib/usbnet.ko` and `cdc_ncm.ko`. `usb-role.sh` loads them
   on the switch to host mode and unloads them on the switch back, as the stock
   `sdDongle` does. Pass a directory holding those two files as `NET_MODULES`.
   `build-usbnet.sh` remains only as a record of the failed approach.
2. **Missing UDC during the role switch.** `rmmod ak_udc` removes
   `/sys/class/udc/20200000.usb`. The UDC function check now treats a missing
   UDC directory as no conflict (`phone_gadget_driver.rs`).
3. **No `/sbin/ip`.** BusyBox 1.30.1 has no `ip` applet and its `ifconfig`
   lacks IPv6. `ncm_helper.rs` falls back to `SIOCSIFADDR`/`SIOCDIFADDR` and
   `SIOCSIFFLAGS` ioctls.
4. **Honda `/info` audio types.** The head unit lists `spokenAudio` and
   `measurement`. `AudioType` gains both, plus a `#[serde(other)]` catch-all.
5. **Unknown iAP2 messages.** After `StartMediaLibraryInformation` the head
   unit sends payloads the CSM registry cannot decode. They are now logged and
   skipped instead of ending the session.
6. **Overlay frames shown late.** The head unit's decoder holds the newest
   frame until another arrives, so the pairing PIN appeared only after the
   iPhone timed out. The overlay keyframe now repeats every 500 ms.
7. **Bluetooth pairing.** With BR/EDR Secure Connections enabled, pairing ends
   in Simple Pairing Complete status 5 (Authentication Failure) right after
   numeric comparison. `run-persistent.sh` sets
   `CATPLAY_BT_SECURE_CONNECTIONS=0`; the app then powers the controller off,
   disables Secure Connections, and powers it on. This `bluetoothd` build has
   no `SecureConnections` option in `main.conf`.

The capture helper now lives in `/usr/data/catplay/catplay-capture` so it can
change without rewriting ROOTFS, and the image has about 184 KiB free.

### Car test procedure

- The head unit remembers the dongle as **iPhone SIM**. After a failed session
  it can stop enumerating the dongle entirely (the UDC stays `not attached`)
  until the device is deleted in the car's settings and approved again.
- A laptop near the car can join `CatPlay-Anyka` on a second Wi-Fi interface
  and relay the console (`ssh -L 18081:192.168.50.100:8081`). `log-stream.rs`
  pushes the app log and kernel messages to a TCP listener as they are written;
  flash-based capture loses the last second before a hang. `bt-diag.rs` reads
  controller settings over the mgmt socket and streams HCI monitor frames.
  `ncm-probe.rs` sends ping, UDP, and TCP traffic over `usb0` step by step.

## Remaining work

1. **Teardown hang.** When the app tears down a session itself while `usb0` is
   up (for example after a transmitter error), the kernel hard-hangs right
   after the app stops its Bonjour services on `usb0`. The head unit then cuts
   power. Sessions the car ends are unaffected. The cause is unknown; a UART
   console would show it.
2. **Car audio on iOS 27.** With a Honda head unit (SDK 210.81) and iOS
   27.2.1, the iPhone never sent an audio `SETUP`, so audio played from the
   phone and CarPlay was missing from the AirPlay menu. The car accepts only
   PCM 44.1 kHz on AltAudio, so `filter_audio_sample_rates` removed every Opus
   format and left wireless AltAudio without a compressed format. Keeping Opus
   48 kHz on AltAudio fixes it (see
   [#66](https://github.com/catplay-labs/catplay/issues/66)). Tested on
   November 22, 2026: music plays over `MainHighAudio` (1000 ms latency), and
   navigation prompts play over AltAudio as PCM 48 kHz, which the car accepts
   even though it advertises only 44.1 kHz. Filling `bluetoothIDs` in `/info`
   and handling `RECORD` made no difference.

   If pairing fails after the phone was deleted on the iPhone, remove the
   dongle's stale bond in `/usr/data/catplay/bluetooth/<adapter>/<phone>` and
   reboot. While a stale bond exists, the dongle retries the reconnect every
   10 s and BlueZ reports `br-connection-busy`.
3. Measure memory use during a long CarPlay session.
4. Add a setting that switches media audio between buffered `MainHighAudio`
   (AAC, about 1 s of buffer) and unbuffered `MainAudio` PCM, for lower
   playback lag. Measure the lag difference in the car first.

Replacement-kernel startup remains unconfirmed. It needs an independent
early-boot observation before further handoff experiments.

A UART adapter can show boot failures. It does not by itself restore damaged
SPI flash. Establish a recovery procedure before changing the bootloader.

Sources:

- [Anyka port discussion](https://github.com/catplay-labs/catplay-firmware/issues/2)
- [AV130 SDK](https://github.com/Link-WQS/AnyCloud39AV130_SDK_V1.03)
- [Linux kexec module approach](https://github.com/amonakov/kexec-module)
