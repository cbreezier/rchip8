rchip8
======

(Yet another) CHIP-8 emulator written in Rust.

Usage: `rchip8 <rom_file>`

There's nothing particularly special about this CHIP-8 emulator, but it does aim to be faithful to the timings
of the sub-systems. For example, it aims to run the exact number of CPU cycles before drawing the screen. It's
currently hard-coded to run the screen at 60Hz and the CPU at 700Hz, but those values are easy enough to change.
