#include "stdlib.h"

int main();

void _start() {
    main();

    _exit(0);
}

int main() {

    create(0x080300C2);

    int counter = 0;
    while (1)
    {
        counter++;
        print("Hello World!\n");
        print("Counter: ");
        printu32(counter);
        print("\n");
        yield();
    }
}

