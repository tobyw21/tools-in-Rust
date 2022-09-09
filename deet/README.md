# Deet, deet

Credits to Standford University, reberhardt7@gmail.com provided starter code,
wrote the wrapper for gmili dwarf debugging format. The program deet is a `gdb`
format debugger rewrite in `Rust`

How to build
=============
`cargo build --release`

How to Run
=============
`cargo run <path_to_file>`

tests
============
-- test not yet implemented

TODO
============
- add stepping, aka next
- add select frame
- add print local, global variable
- add disassemble
- add attach, detach
- add x

Patches
============
2022 Sept 9
    - now `disassemble` works in Intel format! try `disassemble` command
    need to fix
        - disassemble on func/addr
        - fix the disassemble general layouts 
