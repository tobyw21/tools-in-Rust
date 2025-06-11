# Rdiff

a simple version of unix `diff` implemented in `Rust` 

To build
=========
`cargo build --release`

add 
`
[profile.release]
opt-level = <optimisation level>
over-flow-checks = true     # enable int overflow check, optional
`
to Cargo.toml

To test
========
`cargo test <which function to test>`

To run
=========
`cargo run -- <args>`

