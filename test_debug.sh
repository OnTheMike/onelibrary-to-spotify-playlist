#!/bin/bash
# Test script to run the app with debug logging

export RUST_LOG=debug

# Run with example.xml
echo "Running with example.xml and debug logging..."
./target/release/onelibrary-to-spotify-playlist --file example.xml 2>&1 | tee debug_output.log

echo ""
echo "Debug output saved to debug_output.log"
