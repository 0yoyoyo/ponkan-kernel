#!/bin/bash

set -eu

./makefont.py -o hankaku.bin hankaku.txt
objcopy -I binary -O elf64-x86-64 -B i386:x86-64 hankaku.bin hankaku.o
ar rcs libhankaku.a hankaku.o
