/* STM32F446RE memory layout */
MEMORY
{
  /* Flash memory starts at 0x08000000 and has a size of 512K */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  
  /* RAM starts at 0x20000000 and has a size of 128K */
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* This is where the call stack will be allocated */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);