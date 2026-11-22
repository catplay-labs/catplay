// SPDX-License-Identifier: GPL-2.0-only
/* Set both ends of the USB console to raw mode before starting a shell. */
#include <fcntl.h>
#include <stdio.h>
#include <termios.h>
#include <unistd.h>

int main(void)
{
    int fd = open("/dev/ttyGS0", O_RDWR | O_NOCTTY);
    struct termios attributes;
    if (fd < 0 || tcgetattr(fd, &attributes)) {
        perror("USB console");
        return 1;
    }
    cfmakeraw(&attributes);
    if (tcsetattr(fd, TCSANOW, &attributes)) {
        perror("USB console attributes");
        return 1;
    }
    for (int target = 0; target < 3; target++)
        if (dup2(fd, target) < 0)
            return 1;
    if (fd > 2)
        close(fd);
    puts("CATPLAY_USB_CONSOLE_READY");
    fflush(stdout);
    execl("/bin/sh", "sh", "-i", (char *)0);
    perror("USB shell");
    return 1;
}
