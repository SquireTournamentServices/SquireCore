python bindgen.py
cbindgen --config cbindgen.toml --crate squire_lib --output squire_core.h -v
echo "Exported to ./squire_core.h"
cargo build --features ffi --package squire_lib

