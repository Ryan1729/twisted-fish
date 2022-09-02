[gdb]
path=./rust-gdb

[commands]
Compile rename-me=shell cargo b --bin rename-me --profile debugging
Run rename-me=file target/debugging/rename-me;run&