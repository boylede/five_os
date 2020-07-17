# five os

A Rust OS for RISC-V, based on the write-up by Stephen Marz:

https://osblog.stephenmarz.com/ch1.html



## Current Status
* UART communication
* Page-grained allocation
* Generate and walk page tables for use with the MMU, including Sv32, Sv39, and Sv48 virtual address modes. 
* 

## Printout

Currently the OS prints the following status information from QEMU:

````
       _________   ______  ____  ____
      / __/  _/ | / / __/ / __ \/ __/
     / _/_/ / | |/ / _/  / /_/ /\ \
    /_/ /___/ |___/___/  \____/___/

--- CPU INFO ---
Vendor: 0
Architecture: 0
Implementation: 0
--- MISA INFO ---
Base ISA Width: 64
Extensions: ACDFIMSU
--- Extensions ---
Atomic
Compressed
Double-precision floating point
single-precision Floating point
rv32I/64I/128I base isa
integer Multiply/divide
Supervisor Mode
User Mode
----------- Static Layout ---------------
text:   80000000 - 80009008     36872-bytes
 trap:  800082bc - 80009008
global: 80009008
rodata: 80009010 - 8000ac70     7264-bytes
data:   8000b000 - 8000b000     0-bytes
bss:    8000b000 - 8000b0b8     184-bytes
 stack: 8000b0b8 - 8008b0b8     524288-bytes
 heap:  8008b0b8 - 88000000     133648200-bytes
----------- Dynamic Layout --------------
32628 pages x 4096-bytes
Allocation Table: 8008b0b8 - 8009302c
Usable Pages: 80094000 - 88008000
----------- Page Table --------------
Alloc Table:    8008b0b8 - 8009302c
Usable Pages:   80094000 - 88008000
   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
80094000 => 80293fff: 512 page(s).
80294000 => 80294fff: 1 page(s).
80295000 => 802d4fff: 64 page(s).
802d5000 => 802d5fff: 1 page(s).
   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Allocated pages: 578 = 2367488 bytes
Free pages: 32050 = 131276800 bytes
----------------------------------------
reached end
````



