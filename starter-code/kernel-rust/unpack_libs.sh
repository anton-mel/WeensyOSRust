#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <target_folder> <obj_folder>"
    exit 1
fi

ROOT_DIR=$(pwd)                         # Makefile Root
TARGET_FOLDER=$1                        # Archive Location
OBJ_FOLDER=$2                           # Kernel Objects

cd $TARGET_FOLDER

for lib in lib*.a; do
    ar x "$lib"                         # Unpack the .a archive

    libname=$(basename "$lib" .a)       # Remove the .a suffix
    libname="${libname#lib}"            # Remove the "lib" prefix

    # Move the .o necessary lib files
    matching_files=($libname*.$libname*.o)
    if [ ${#matching_files[@]} -gt 0 ]; then
        for obj in "${matching_files[@]}"; do
            mv "$ROOT_DIR/$TARGET_FOLDER/$obj" \
                "$ROOT_DIR/$OBJ_FOLDER/$libname.o"
        done
    fi

    # Remove the remaining .o files
    for obj in "$TARGET_DIR"/*.o; do
        rm -f "$obj"
    done
done