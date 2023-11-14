#!/bin/bash
filename=$1
target/debug/brro-compressor --compressor fft --error 1 --verbose ../../wbro-july/$filename.wbro > ../../$filename.m
target/debug/brro-compressor -u --verbose ../../wbro-july/$filename.bro >> ../../$filename.m
echo "plot(Input,'b', Output,'r')" >> ../../$filename.m
echo "print -dpng $filename.png" >> ../../$filename.m
