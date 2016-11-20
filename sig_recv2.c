#define _GNU_SOURCE
#include <stdio.h>
#include <signal.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>

static volatile sig_atomic_t sigCnt = 0;
static volatile sig_atomic_t allDone = 0;
static volatile int handlerSleepTime;

static void siginfoHandler(int sig, siginfo_t *si, void *ucontext) {
    if (sig == SIGINT || sig == SIGTERM) {
	allDone = 1;
	return;
    }

    sigCnt++;
    printf("Caught signal %d\n", sig);

    printf("\tsi_signo = %d, si_code = %d (%s), ", si -> si_signo, si -> si_code,
	   (si -> si_code == SI_USER) ? "SI_USER" :
	   (si -> si_code == SI_QUEUE) ? "SI_QUEUE" : "other");
    printf("si_value = %d\n", si -> si_value.sival_int);
    printf("\tsi_pid = %ld, si_uid = %ld\n", (long)si -> si_pid, (long)si -> si_uid);

    sleep(handlerSleepTime);
}

int main(int argc, char *argv[]) {
    struct sigaction sa;
    int sig;
    sigset_t prevMask, blockMask;

    printf("%s: PID is %ld\n", argv[0], (long)getpid());

    handlerSleepTime = atoi(argv[2]);

    sa.sa_sigaction = siginfoHandler;
    sa.sa_flags = SA_SIGINFO;
    /* during execution of the handler, all other signals are blocked */
    sigfillset(&sa.sa_mask);

    for (sig = 1; sig<NSIG; sig++) {
	if (sig != SIGTSTP && sig != SIGQUIT) {
	    sigaction(sig, &sa, NULL);
	}
    }

    if (argc > 1) {
	sigfillset(&blockMask);
	sigdelset(&blockMask, SIGINT);
	sigdelset(&blockMask, SIGTERM);

	if (sigprocmask(SIG_SETMASK, &blockMask, &prevMask) == -1) {
	    perror("sigprocmask error");
	    exit(-1);
	}

	printf("%s: signals blocked - sleeping %s seconds\n", argv[0], argv[1]);
	sleep(atoi(argv[1]));
	printf("%s: sleep complete\n", argv[0]);

	if (sigprocmask(SIG_SETMASK, &prevMask, NULL) == -1) {
	    perror("sigprocmask error");
	    exit(-1);
	}	
    }

    while (!allDone)
	pause();

    return 0;
}

