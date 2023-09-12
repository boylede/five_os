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
text:   80000000 - 8000e000     57344-bytes
 trap:  8000dfea - 8000e000??
global: 8000e000
rodata: 8000e000 - 80011fb8     16312-bytes
data:   80013000 - 80013238     568-bytes
bss:    80013238 - 80013b68     2352-bytes
 stack: 80013b68 - 80093b68     524288-bytes
 heap:  80093b68 - 88000000     133612696-bytes
################################################################
#                   Setup Memory Allocation                    #
################################################################
32620 pages x 4096-bytes
Allocation Table: 80093b68 - 8009bad4
Usable Pages: 8009c000 - 88000000
################################################################
#                  Kernel Space Identity Map                   #
################################################################
Kernel Root Page Table: 0x800dd000-0x800dd000 RW-
Kernel Dynamic Memory: 0x8009d000-0x800dd000 RW-
Allocation Bitmap: 0x80093b68-0x8009bad4 R-E
Kernel Code Section: 0x80000000-0x8000e000 R-E
Readonly Data Section: 0x8000e000-0x80011fb8 R-E
Data Section: 0x80013000-0x80013238 RW-
BSS section: 0x80013238-0x80013b68 RW-
Kernel Stack: 0x80013b68-0x80093b68 RW-
Hardware UART: 0x10000000-0x10000100 RW-
Hardware CLINT, MSIP: 0x2000000-0x2010000 RW-
Hardware PLIC: 0xc000000-0xc002000 RW-
Hardware ????: 0xc200000-0xc208000 RW-
Trap stack: 0x8009c000-0x8009d000 RW-
################################################################
#                       Allocator Bitmap                       #
################################################################
Alloc Table:    80093b68 - 8009bad4
Usable Pages:   8009c000 - 88008000
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
8009c000 => 8009cfff: 1 page(s).
8009d000 => 800dcfff: 64 page(s).
800dd000 => 800ddfff: 1 page(s).
800de000 => 800defff: 1 page(s).
800df000 => 800dffff: 1 page(s).
800e0000 => 800e0fff: 1 page(s).
800e1000 => 800e1fff: 1 page(s).
800e2000 => 800e2fff: 1 page(s).
800e3000 => 800e3fff: 1 page(s).
800e4000 => 800e4fff: 1 page(s).
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Allocated pages: 73 = 299008 bytes
Free pages: 32547 = 133312512 bytes
setting up UART receiver
~~~~~~~~~~~~~~~~~~~~~ testing allocations  ~~~~~~~~~~~~~~~~~~~~~
Boxed value = 100
String = 💖

Allocations of a box, vector, and string
inspecting 8009d000
0x8009d000: Length = 16         Taken = true
checking next: 8009d010
inspecting 8009d010
0x8009d010: Length = 16         Taken = true
checking next: 8009d020
inspecting 8009d020
0x8009d020: Length = 262112     Taken = false
checking next: 800dd000
done printing alloc table
test
test 2


Everything should now be free:
inspecting 8009d000
0x8009d000: Length = 262144     Taken = false
checking next: 800dd000
done printing alloc table
~~~~~~~~~~~~~~~~~~~~~ reached end, looping ~~~~~~~~~~~~~~~~~~~~~

````



