/* SPDX-License-Identifier: GPL-2.0-only */
/* GNU ARM preload for daemons on the stock Anyka clock_gettime64 implementation. */
extern long syscall(long number, ...);
extern int *__errno_location(void);
struct legacy_time { int seconds, nanoseconds; };
struct wide_time { long long seconds, nanoseconds; };
int clock_gettime(int clock, struct legacy_time *output)
{
    if (!output) { *__errno_location() = 14; return -1; }
    return syscall(263, clock, output) ? -1 : 0;
}
int __clock_gettime64(int clock, struct wide_time *output)
{
    struct legacy_time value;
    if (!output) { *__errno_location() = 14; return -1; }
    if (syscall(263, clock, &value)) return -1;
    output->seconds = value.seconds;
    output->nanoseconds = value.nanoseconds;
    return 0;
}
