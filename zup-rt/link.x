INCLUDE common.x;

/* Initial stack pointer */
__stack_top__ = ORIGIN(BTCM1) + LENGTH(BTCM1);

SECTIONS
{
  .text ORIGIN(ATCM0) :
  {
    KEEP(*(.vectors));
    *(.start);
    *(.main);
    *(.text .text.*);
    . = ALIGN(4);
  } > ATCM0

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > ATCM0

  .bss : ALIGN(4)
  {
    *(.bss .bss.*);
    . = ALIGN(4);
  } > BTCM0

  .data : ALIGN(4)
  {
    *(.data .data.*);
    . = ALIGN(4);
  } > BTCM0

  .resource_table : ALIGN(4)
  {
    KEEP(*(.resource_table));
  } > BTCM0

  /* Discarded sections */
  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx.*);
  }
}
