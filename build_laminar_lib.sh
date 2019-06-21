#!/bin/bash

DEST1="server"
DEST1="server"
TARGET="target/debug/libnull_box.so"

cargo build

cp $TARGET $DEST1/"libnull_box.so"
cp $TARGET ./"libnull_box.so"