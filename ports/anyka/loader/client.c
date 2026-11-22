// SPDX-License-Identifier: GPL-2.0-only
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define CP_LOAD 0x43500001
#define CP_EXEC 0x43500002
#define CP_UNLOAD 0x43500003
#define CP_STATUS 0x43500004
struct cp_status { unsigned long entry, dtb, nr_segments; };
#define KERNEL_ADDRESS 0x81000000UL
#define DTB_ADDRESS 0x80f00000UL
#define INITRD_ADDRESS 0x82000000UL

struct segment { void *buf; size_t bufsz; unsigned long mem; size_t memsz; };
struct load { unsigned long entry, nr_segments; struct segment *segments; };

static void die(const char *message) { perror(message); exit(1); }
static struct segment map_image(const char *path, unsigned long address)
{
    struct stat st;
    int fd = open(path, O_RDONLY);
    if (fd < 0) die(path);
    if (fstat(fd, &st) < 0) die("fstat");
    if (st.st_size <= 0 || st.st_size > 8 * 1024 * 1024) {
        fprintf(stderr, "Invalid image size: %s\n", path); exit(1);
    }
    void *data = mmap(NULL, st.st_size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (data == MAP_FAILED) die("mmap");
    close(fd);
    return (struct segment){ data, st.st_size, address, (st.st_size + 4095) & ~4095UL };
}
static int stage(int fd, struct segment *segments, unsigned long count)
{
    struct load load = {KERNEL_ADDRESS, count, segments};
    return ioctl(fd, CP_LOAD, &load);
}
static void expect_rejected(int fd, struct segment *segments, const char *name)
{
    if (!stage(fd, segments, 2)) {
        fprintf(stderr, "FAIL: accepted %s\n", name); exit(1);
    }
    printf("PASS: rejected %s (errno=%d)\n", name, errno);
}
static void selftest(int fd, const char *dtb)
{
    struct cp_status status;
    if (ioctl(fd, CP_STATUS, &status)) die("status");
    if (status.nr_segments) {
        fprintf(stderr, "Unload the staged image before running selftest\n"); exit(1);
    }
    uint32_t *kernel = calloc(1, 4096);
    if (!kernel) die("calloc");
    kernel[0x24 / 4] = 0x016f2818;
    struct segment segments[2] = {
        {kernel, 4096, KERNEL_ADDRESS, 4096},
        map_image(dtb, DTB_ADDRESS),
    };
    segments[0].mem = 0x70000000;
    expect_rejected(fd, segments, "address outside RAM");
    segments[0].mem = DTB_ADDRESS;
    expect_rejected(fd, segments, "overlapping segments");
    segments[0].mem = KERNEL_ADDRESS;
    kernel[0x24 / 4] = 0;
    expect_rejected(fd, segments, "invalid zImage magic");
    kernel[0x24 / 4] = 0x016f2818;
    if (stage(fd, segments, 2)) die("stage synthetic test image");
    puts("PASS: staged synthetic image in allocated pages; execution not requested");
    kernel[0x24 / 4] = 0;
    segments[1].mem = DTB_ADDRESS - 0x100000;
    expect_rejected(fd, segments, "invalid replacement image");
    if (ioctl(fd, CP_STATUS, &status)) die("status after invalid replacement");
    if (status.entry != KERNEL_ADDRESS || status.dtb != DTB_ADDRESS || status.nr_segments != 2) {
        fprintf(stderr, "FAIL: invalid replacement changed the staged image\n"); exit(1);
    }
    puts("PASS: failed replacement preserved the original staged image");
    if (ioctl(fd, CP_UNLOAD, 0)) die("unload synthetic image");
    puts("PASS: unloaded staged image");
    munmap(segments[1].buf, segments[1].bufsz);
    free(kernel);
}
int main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr, "Usage: %s stage ZIMAGE DTB [INITRD] | selftest DTB | status | unload | execute\n", argv[0]);
        return 2;
    }
    int fd = open("/dev/catplay_kexec", O_RDWR);
    if (fd < 0) die("/dev/catplay_kexec");
    if (!strcmp(argv[1], "selftest") && argc == 3) {
        selftest(fd, argv[2]);
    } else if (!strcmp(argv[1], "stage") && (argc == 4 || argc == 5)) {
        struct segment segments[3] = {map_image(argv[2], KERNEL_ADDRESS), map_image(argv[3], DTB_ADDRESS)};
        if (argc == 5) segments[2] = map_image(argv[4], INITRD_ADDRESS);
        if (stage(fd, segments, argc - 2)) die("stage");
        for (int i = 0; i < argc - 2; i++) {
            printf("Staged %zu bytes at 0x%08lx\n", segments[i].bufsz, segments[i].mem);
            munmap(segments[i].buf, segments[i].bufsz);
        }
        puts("Image staged. Run execute separately to boot it.");
    } else if (!strcmp(argv[1], "status") && argc == 2) {
        struct cp_status status;
        if (ioctl(fd, CP_STATUS, &status)) die("status");
        printf("segments=%lu entry=0x%08lx dtb=0x%08lx\n", status.nr_segments, status.entry, status.dtb);
    } else if (!strcmp(argv[1], "unload") && argc == 2) {
        if (ioctl(fd, CP_UNLOAD, 0)) die("unload");
        puts("Staged image unloaded.");
    } else if (!strcmp(argv[1], "execute") && argc == 2) {
        sync();
        puts("Executing staged RAM kernel.");
        fflush(stdout);
        if (ioctl(fd, CP_EXEC, 0)) die("execute");
    } else {
        fprintf(stderr, "Invalid arguments\n");
        close(fd); return 2;
    }
    close(fd);
    return 0;
}
