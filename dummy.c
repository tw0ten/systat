#include <stdio.h>
#include <unistd.h>
#include <sys/prctl.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
    prctl(PR_SET_NAME, argv[1], 0, 0, 0);
    while (1) {
        sleep(86400);
    }
    return 0;
}
