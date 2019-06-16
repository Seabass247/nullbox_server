#!/bin/bash

DEST="server"
TARGET="target/debug/libnull_box.so"

cargo build

cp $TARGET $DEST/"libnull_box.so"