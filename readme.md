# five os

A Rust OS for RISC-V, based on the write-up by Stephen Marz:

https://osblog.stephenmarz.com/ch1.html

## Current Status

* UART communication
* Page-grained allocation
* Simple bump-allocator
* Generate and walk page tables for use with the MMU, including Sv32, Sv39, and Sv48 virtual address modes. 
* Trap handler pass to rust code

## Printout

Currently the OS prints the following status information from QEMU:

````
                    _________   ______  ____  ____
                   / __/  _/ | / / __/ / __ \/ __/
                  / _/_/ / | |/ / _/  / /_/ /\ \
                 /_/ /___/ |___/___/  \____/___/

################################################################
#                           CPU INFO                           #
################################################################
Vendor: 0 | Architecture: 0 | Implementation: 0
~~~~~~~~~~~~~ Machine Instruction Set Architecture ~~~~~~~~~~~~~
Reported base width: 64
Extensions: ACDFHIMSU
~~~~~~~~~~~~~~~~~~~~~~~~~~ Extensions ~~~~~~~~~~~~~~~~~~~~~~~~~~
Atomics (A)
Compressed (C)
Double-precision floating point (D)
Single-precision floating point (F)
Hypervisor (H)
Integer base ISA (I)
Integer multiply & divide (M)
Supervisor Mode (S)
User Mode (U)
################################################################
#                  Static Layout Sanity Check                  #
################################################################
text:   80000000 - 8000f000     61440-bytes
 trap:  8000e276 - 8000f000??
global: 8000f000
rodata: 8000f000 - 80012e48     15944-bytes
data:   80014000 - 80014238     568-bytes
bss:    80014238 - 80014b68     2352-bytes
 stack: 80014b68 - 80094b68     524288-bytes
 heap:  80094b68 - 88000000     133608600-bytes
################################################################
#                   Setup Memory Allocation                    #
################################################################
32619 pages x 4096-bytes
Allocation Table: 80094b68 - 8009cad3
Usable Pages: 8009d000 - 88008000
################################################################
#                  Kernel Space Identity Map                   #
################################################################
Kernel Root Page Table: 0x800de000-0x800de000 RW-
Kernel Dynamic Memory: 0x8009e000-0x800de000 RW-
Allocation Bitmap: 0x80094b68-0x8009cad3 R-E
Kernel Code Section: 0x80000000-0x8000f000 R-E
Readonly Data Section: 0x8000f000-0x80012e48 R-E
Data Section: 0x80014000-0x80014238 RW-
BSS section: 0x80014238-0x80014b68 RW-
Kernel Stack: 0x80014b68-0x80094b68 RW-
Hardware UART: 0x10000000-0x10000100 RW-
Hardware CLINT, MSIP: 0x2000000-0x2010000 RW-
Hardware PLIC: 0xc000000-0xc002000 RW-
Hardware ????: 0xc200000-0xc208000 RW-
Trap stack: 0x8009d000-0x8009e000 RW-
################################################################
#                       Allocator Bitmap                       #
################################################################
Alloc Table:    80094b68 - 8009cad3
Usable Pages:   8009d000 - 88008000
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
8009d000 => 8009dfff: 1 page(s).
8009e000 => 800ddfff: 64 page(s).
800de000 => 800defff: 1 page(s).
800df000 => 800dffff: 1 page(s).
800e0000 => 800e0fff: 1 page(s).
800e1000 => 800e1fff: 1 page(s).
800e2000 => 800e2fff: 1 page(s).
800e3000 => 800e3fff: 1 page(s).
800e4000 => 800e4fff: 1 page(s).
800e5000 => 800e5fff: 1 page(s).
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Allocated pages: 73 = 299008 bytes
Free pages: 32546 = 133308416 bytes
[bookmark sigil] Leaving kinit
################################################################
#                        entering kmain                        #
################################################################
setting up UART receiver
~~~~~~~~~~~~~~~~~~~~~ testing allocations  ~~~~~~~~~~~~~~~~~~~~~
Boxed value = 100
String = 💖

Allocations of a box, vector, and string
inspecting 8009e000
0x8009e000: Length = 16         Taken = true
checking next: 8009e010
inspecting 8009e010
0x8009e010: Length = 16         Taken = true
checking next: 8009e020
inspecting 8009e020
0x8009e020: Length = 262112     Taken = false
checking next: 800de000
done printing alloc table
test
test 2


Everything should now be free:
inspecting 8009e000
0x8009e000: Length = 262144     Taken = false
checking next: 800de000
done printing alloc table
~~~~~~~~~~~~~~~~~~~~~ reached end, looping ~~~~~~~~~~~~~~~~~~~~~


````



