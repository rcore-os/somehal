/* 
 * Test linker script that works for both architectures
 * The actual OUTPUT_ARCH will be determined by the target
 */

__SMP = 4;
STACK_SIZE = 0x40000;

INCLUDE "somehal.x"

SECTIONS
{
   
}
