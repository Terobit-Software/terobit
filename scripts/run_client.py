# This script runs the client after building it and copying assets to the build directory.
# Usage: python scripts/run_client.py [--final]
#   (default) builds with the fast-compiling "release-dev" profile
#   --final   builds with the fully optimized "release" profile
#   the client is run with --dev unless --final is passed

import os
import shutil
import subprocess
import sys

# Constants
CWD = os.getcwd() # Current Working Directory
CLIENT_DIR = os.path.join(CWD, "bins", "terobit_client") # Path to the client dir
CLIENT_ASSETS_DIR = os.path.join(CLIENT_DIR, "assets")
OUT_DIR = os.path.join(CWD, "out", "client")
FINAL = "--final" in sys.argv[1:]
PROFILE = "release" if FINAL else "release-dev"
CLIENT_FLAGS = [] if FINAL else ["--dev"] # Runtime flags passed to the client

# Build the client
print("Building client with profile", PROFILE+"...")
build_proc = subprocess.run(["cargo", "build", "--profile", PROFILE, "-p", "terobit_client"], check=False)

# Check if the build succeeded
if build_proc.returncode != 0:
    print("Build failed with code", build_proc.returncode, "- not running client")
    sys.exit(1)

print("Build succeeded")

print("Copying assets...")
if not os.path.exists(OUT_DIR):
    # Create the output directory if it doesn't exist
    print("Folder", OUT_DIR, "does not exist, creating...")
    os.makedirs(OUT_DIR)
else:
    # Clean the output directory if it already exists
    print("Folder", OUT_DIR, "already exists, cleaning...")
    shutil.rmtree(OUT_DIR)
    os.makedirs(OUT_DIR)

# Copy the assets to the output directory
if not os.path.exists(CLIENT_ASSETS_DIR):
    print("Assets folder", CLIENT_ASSETS_DIR, "does not exist, skipping...")
else:
    shutil.copytree(CLIENT_ASSETS_DIR, os.path.join(OUT_DIR, "assets"))

# Get the binary name
bin = "terobit_client"
if os.name == "nt":
    bin += ".exe"

# Copy the binary to the output directory
print("Copying", bin+"...")
shutil.copy(os.path.join(CWD, "target", PROFILE, bin), os.path.join(OUT_DIR, bin))

# Start run
print("Running", bin+"...")
run_proc = subprocess.run([os.path.join(OUT_DIR, bin), *CLIENT_FLAGS], check=False, cwd=OUT_DIR)

# Check if the run succeeded
if run_proc.returncode != 0:
    print("Run exited with code", run_proc.returncode)
    sys.exit(1)

print("Run succeeded")
