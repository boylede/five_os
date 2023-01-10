# five os

A Rust OS for RISC-V, based on the write-up by Stephen Marz:

https://osblog.stephenmarz.com/ch1.html



## Current Status
* UART communication
* Page-grained allocation
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
Base ISA Width: 64
Extensions: ACDFIMSU
~~~~~~~~~~~~~~~~~~~~~~~~~~ Extensions ~~~~~~~~~~~~~~~~~~~~~~~~~~
Atomic
Compressed
Double-precision floating point
single-precision Floating point
rv32I/64I/128I base isa
integer Multiply/divide
Supervisor Mode
User Mode
################################################################
#                  Static Layout Sanity Check                  #
################################################################
text:	80000000 - 8000a000	40960-bytes
 trap:	80009dfa - 8000a000??
global:	8000a000
rodata:	8000a000 - 8000d3e8	13288-bytes
data:	8000e000 - 8000e2c8	712-bytes
bss:	8000e2c8 - 8000ebd8	2320-bytes
 stack:	8000ebd8 - 8008ebd8	524288-bytes
 heap:	8008ebd8 - 88000000	133633064-bytes
################################################################
#                   Setup Memory Allocation                    #
################################################################
32625 pages x 4096-bytes
Allocation Table: 8008ebd8 - 80096b49
Usable Pages: 80097000 - 88008000
################################################################
#                  Kernel Space Identity Map                   #
################################################################
Kernel root page table: 800d7000
Dynamic Memory: 80097000 -> 800d7000  RW
Allocation bitmap: 8008ebd8 -> 80096b49  RE
Kernel code section: 80000000 -> 8000a000  RE
Readonly data section: 8000a000 -> 8000d3e8  RE
Data section: 8000e000 -> 8000e2c8  RW
BSS section: 8000e2c8 -> 8000ebd8  RW
Kernel stack: 8000ebd8 -> 8008ebd8  RW
Hardware UART: 10000000 -> 10000100  RW
Hardware CLINT, MSIP: 2000000 -> 200ffff  RW
Hardware PLIC: c000000 -> c002000  RW
Hardware ???: c200000 -> c208000  RW
Trap stack: 800df000 -> 800e0000  RW
################################################################
#                       Allocator Bitmap                       #
################################################################
Alloc Table:	8008ebd8 - 80096b49
Usable Pages:	80097000 - 88008000
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
80097000 => 800d6fff: 64 page(s).
800d7000 => 800d7fff: 1 page(s).
800d8000 => 800d8fff: 1 page(s).
800d9000 => 800d9fff: 1 page(s).
800da000 => 800dafff: 1 page(s).
800db000 => 800dbfff: 1 page(s).
800dc000 => 800dcfff: 1 page(s).
800dd000 => 800ddfff: 1 page(s).
800de000 => 800defff: 1 page(s).
800df000 => 800dffff: 1 page(s).
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Allocated pages: 73 = 299008 bytes
Free pages: 32552 = 133332992 bytes
################################################################
#                        entering kmain                        #
################################################################
setting up UART receiver
~~~~~~~~~~~~~~~~~~~~~ reached end, looping ~~~~~~~~~~~~~~~~~~~~~

````



