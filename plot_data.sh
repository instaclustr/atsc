#!/bin/bash
filename=$2
compressor=$1
cp ../../wbro-july/$filename.wbro tmp.wbro 
target/debug/atsc --compressor $compressor --error 1 --verbose tmp.wbro > ../../$filename-$compressor.m
target/debug/atsc -u --verbose tmp.bro >> ../../$filename-$compressor.m
echo "plot(Input,'b', Output,'r')" >> ../../$filename-$compressor.m
echo "print -dpng $filename.png" >> ../../$filename-$compressor.m
rm tmp.wbro
rm tmp.bro