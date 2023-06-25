/* We want the symbol _start to be our entry point */
ENTRY(_start)
 
SECTIONS
{
  . = 0xFFFFF;

  .text : {
      *(.text)
  }
 
	/* Read-only data. */
	.rodata : {
		*(.rodata)
	}
 
	/* Read-write data (initialized) */
	.data : {
		*(.data)
	}
 
	/* Read-write data (uninitialized) and stack */
	.bss : {
		*(COMMON)
		*(.bss)
	}
 
}