MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  /* These values correspond to the LM3S6965, one of the few devices QEMU can emulate */
  /* FLASH : ORIGIN = 0x00000000, LENGTH = 256K */

  /*
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
  */

  FLASH : ORIGIN = 0x20000000, LENGTH = 45056
  RAM : ORIGIN = 0x2000B000, LENGTH = 20480

}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* _stack_start = ORIGIN(RAM) + LENGTH(RAM); */
_stack_start = 0x2000F500;

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */
_stext = ORIGIN(FLASH) + 0x200;

/* Example of putting non-initialized variables into custom RAM locations. */
/* This assumes you have defined a region RAM2 above, and in the Rust
   sources added the attribute `#[link_section = ".ram2bss"]` to the data
   you want to place there. */
/* Note that the section will not be zero-initialized by the runtime! */
/* SECTIONS {
     .ram2bss (NOLOAD) : ALIGN(4) {
       *(.ram2bss);
       . = ALIGN(4);
     } > RAM2
   } INSERT AFTER .bss;
*/


/* MAGIC VALUE USED TO RESET SP/PC TO VALID ADDRESS WHEN BOOT ON SRAM */
SECTIONS {
   .magic 0x200001e0 : AT(0x200001e0) {
      LONG(0xD000F8DF);
      LONG(0x2000F500);
      LONG(0xF1E8F85F);
   } > FLASH
} INSERT AFTER .vector_table
