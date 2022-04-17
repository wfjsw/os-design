arm-none-eabi-gcc -o out -nostdlib -fPIC -mthumb -msingle-pic-base -mpic-register=r9 -n -T./kernel.ld main.c -lgcc
