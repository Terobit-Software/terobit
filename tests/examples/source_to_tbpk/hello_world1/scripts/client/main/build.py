# Builds this script to script.wasm in this folder. `cargo x pkgen` and `cargo x rb cl` run
# it from this folder as `python build.py`, or `python build.py --final` for a final build,
# then copy script.wasm to ../main.wasm, the path the manifest points to
import json
import shutil
import subprocess
import sys
from pathlib import Path

profile = "release" if sys.argv[1:2] == ["--final"] else "release-dev"

subprocess.run(
    ["cargo", "build", "--target", "wasm32-unknown-unknown", "--profile", profile],
    check=True,
)

# the workspace's target folder, or CARGO_TARGET_DIR if it's set
metadata = subprocess.run(
    ["cargo", "metadata", "--format-version", "1", "--no-deps"],
    check=True,
    capture_output=True,
    text=True,
).stdout
target_dir = Path(json.loads(metadata)["target_directory"])
shutil.copyfile(
    target_dir / "wasm32-unknown-unknown" / profile / "hello_world1.wasm", "script.wasm"
)
