#include <limits.h>
#include <sys/prctl.h>
#include <unistd.h>

int main(int argc, char *argv[]) {
  prctl(PR_SET_NAME, argv[1], 0, 0, 0);
  while (1) {
    sleep(UINT_MAX);
  }
  return 0;
}
