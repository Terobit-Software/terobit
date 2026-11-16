# Builds this script to script.wasm in this folder. `cargo x pkgen` and `cargo x rb cl` run
# it from this folder as `sh build.sh`, or `sh build.sh --final` for a final build, then
# copy script.wasm to ../main.wasm, the path the manifest points to
set -eu

profile=release-dev
if [ "${1:-}" = "--final" ]; then
    profile=release
fi

cargo build --target wasm32-unknown-unknown --profile "$profile"

# the workspace's target folder, or CARGO_TARGET_DIR if it's set
target_dir=$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
cp "$target_dir/wasm32-unknown-unknown/$profile/hello_world1.wasm" script.wasm
