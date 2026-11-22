// SPDX-License-Identifier: GPL-2.0
// Execute from tmpfs with a static C runtime while replacing the mounted root.
#define _XOPEN_SOURCE 700
#define _DEFAULT_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <mtd/mtd-user.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/reboot.h>
#include <sys/stat.h>
#include <unistd.h>

static int readers_stopped;

static void resume_readers(void)
{
    if (readers_stopped)
        kill(-1, SIGCONT);
}

static void die(const char *message)
{
    perror(message);
    exit(1);
}

static void transfer(int fd, unsigned char *data, size_t size, int writing)
{
    while (size) {
        ssize_t n = writing ? write(fd, data, size) : read(fd, data, size);
        if (n < 0 && errno == EINTR)
            continue;
        if (n <= 0)
            die(writing ? "write" : "read");
        data += n;
        size -= n;
    }
}

int main(int argc, char **argv)
{
    struct stat st;
    struct mtd_info_user info;
    unsigned char verify[4096];
    char name[32] = {0};
    int writing = argc == 3 && !strcmp(argv[1], "--write-and-reboot");
    if (argc != 3 || (!writing && strcmp(argv[1], "--check"))) {
        fprintf(stderr, "Usage: %s --check|--write-and-reboot IMAGE\n", argv[0]);
        return 2;
    }
    setvbuf(stdout, NULL, _IONBF, 0);
    if (atexit(resume_readers))
        die("register cleanup");
    int image = open(argv[2], O_RDONLY);
    if (image < 0 || fstat(image, &st))
        die("image");
    if (!S_ISREG(st.st_mode) || st.st_size < 4096 || st.st_size > 0x880000) {
        fprintf(stderr, "Image size/type rejected\n");
        return 1;
    }
    unsigned char *data = malloc(st.st_size);
    if (!data)
        die("malloc");
    transfer(image, data, st.st_size, 0);
    close(image);
    if (memcmp(data, "hsqs", 4)) {
        fprintf(stderr, "Expected squashfs image\n");
        return 1;
    }
    int sysname = open("/sys/class/mtd/mtd5/name", O_RDONLY);
    if (sysname < 0 || read(sysname, name, sizeof(name) - 1) < 0)
        die("MTD name");
    close(sysname);
    int flash = open("/dev/mtd5", writing ? O_RDWR : O_RDONLY);
    if (flash < 0 || ioctl(flash, MEMGETINFO, &info))
        die("MTD info");
    if (strcmp(name, "ROOTFS\n") || info.type != MTD_NORFLASH ||
        info.size != 0x880000 || info.erasesize != 4096) {
        fprintf(stderr, "Unexpected ROOTFS geometry\n");
        return 1;
    }
    printf("ROOTFS preflight passed: %ld image bytes, %u partition bytes\n",
           (long)st.st_size, info.size);
    if (!writing)
        return 0;
    // Give the HTTP launch command time to return before stopping flash readers.
    sync();
    sleep(3);
    readers_stopped = 1;
    if (kill(-1, SIGSTOP))
        die("stop userspace");
    for (uint32_t offset = 0; offset < info.size; offset += info.erasesize) {
        struct erase_info_user erase = { offset, info.erasesize };
        if (ioctl(flash, MEMERASE, &erase))
            die("erase ROOTFS");
    }
    puts("ROOTFS erased");
    transfer(flash, data, st.st_size, 1);
    // MTD writes complete synchronously. Linux 4.4 mtdchar has no fsync
    // operation, so fsync would return EINVAL before verification/reboot.
    if (lseek(flash, 0, SEEK_SET) != 0)
        die("seek ROOTFS");
    for (size_t offset = 0; offset < (size_t)st.st_size; offset += sizeof(verify)) {
        size_t length = st.st_size - offset;
        if (length > sizeof(verify))
            length = sizeof(verify);
        transfer(flash, verify, length, 0);
        if (memcmp(verify, data + offset, length)) {
            fprintf(stderr, "ROOTFS verification failed at %zu\n", offset);
            return 1;
        }
    }
    puts("ROOTFS verified; rebooting");
    close(flash);
    reboot(RB_AUTOBOOT);
    die("reboot");
}
