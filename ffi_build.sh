#/bin/bash
python3 ./bindgen.py && \
RUST_BACKTRACE=full cbindgen --config cbindgen.toml --crate squire_core --output squire_core.h -v && \
echo "Exported to ./squire_core.h" && \
cargo build --verbose --features ffi --package squire_lib && \
cp squire_core.h $TARGET_DIR/squire_core.h && \
echo "Copied to $TARGET_DIR/squire_core.h"

