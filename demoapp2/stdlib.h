#ifndef __CHOCOS_STDLIB_H__
#define __CHOCOS_STDLIB_H__

int syscall(int id, int arg1, int arg2, int arg3) {
    int ret;
    asm volatile(
        "MOV %%r0, %1 \n\t"
        "MOV %%r1, %2  \n\t"
        "MOV %%r2, %3  \n\t"
        "MOV %%r3, %4  \n\t"
        "SVC 0"
        : "=r"(ret)
        : "r"(id), "r"(arg1), "r"(arg2), "r"(arg3)
        : "r0", "r1", "r2", "r3");
}

void print(const char * str) {
    syscall(4, (int)str, 0, 0);
}

void printu32(unsigned int num) {
    syscall(7, num, 0, 0);
}

void yield() {
    syscall(1, 0, 0, 0);
}

void create(unsigned int addr) {
    syscall(6, addr, 0, 0);
}

void _exit(int return_code) {
    syscall(5, return_code, 0, 0);
    while(1)
        ;
}

#endif
