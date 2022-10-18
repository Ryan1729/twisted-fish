[gdb]
path=./rust-gdb

[commands]
Compile twisted-fish=shell cargo b --bin twisted-fish --profile debugging
Load twisted-fish=file target/debugging/twisted-fish
Run twisted-fish=file target/debugging/twisted-fish;run&