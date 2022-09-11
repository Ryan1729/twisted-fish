[gdb]
path=./rust-gdb

[commands]
Compile rename-me=shell cargo b --bin twisted-fish --profile debugging
Run twisted-fish=file target/debugging/twisted-fish;run&