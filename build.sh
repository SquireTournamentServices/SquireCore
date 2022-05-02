#/bin/bash

python3 ./bindgen.py
cbindgen > ./squire_core.h
cargo build --verbose

