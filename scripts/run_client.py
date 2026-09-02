# This script runs the client after building it and copying assets to the build directory.

import os
import shutil
import subprocess
import sys

# Constants
CWD = os.getcwd() # Current Working Directory
CLIENT_DIR = os.path.join(CWD, "client") # Path to the client dir
CLIENT_ASSETS_DIR = os.path.join(CLIENT_DIR, "assets")
OUT_DIR = os.path.join(CWD, "out", "client")

# Build the client
print("Building client...")
build_proc = subprocess.run(["cargo", "build", "--release", "-p", "client"], check=False)

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
shutil.copytree(CLIENT_ASSETS_DIR, os.path.join(OUT_DIR, "assets"))

# Get the binary name
bin = "client"
if os.name == "nt":
    bin += ".exe"

# Copy the binary to the output directory
print("Copying", bin+"...")
shutil.copy(os.path.join(CWD, "target", "release", bin), os.path.join(OUT_DIR, bin))

# Start run
print("Running", bin+"...")
run_proc = subprocess.run([os.path.join(OUT_DIR, bin)], check=False, cwd=OUT_DIR)

# Check if the run succeeded
if run_proc.returncode != 0:
    print("Run exited with code", run_proc.returncode)
    sys.exit(1)

print("Run succeeded")
