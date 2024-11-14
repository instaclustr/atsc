#!/bin/bash
filename=$2
error=$1

cp ../../wbro-july/$filename.wbro tmp.wbro 

target/debug/atsc --compressor fft --error $error --verbose tmp.wbro > ../../comparison-$filename.m
target/debug/atsc -u --verbose tmp.bro >> ../../comparison-$filename.m

sed -i -e 's/Output/output_fft/g'  ../../comparison-$filename.m

cp ../../wbro-july/$filename.wbro tmp.wbro 

target/debug/atsc --compressor idw --error $error --verbose tmp.wbro > /dev/null
target/debug/atsc -u --verbose tmp.bro >> ../../comparison-$filename.m

sed -i -e 's/Output/output_idw/g'  ../../comparison-$filename.m

cp ../../wbro-july/$filename.wbro tmp.wbro 

target/debug/atsc --compressor polynomial --error $error --verbose tmp.wbro > /dev/null
target/debug/atsc -u --verbose tmp.bro >> ../../comparison-$filename.m

sed -i -e 's/Output/output_poly/g'  ../../comparison-$filename.m

echo "plot(Input,'b', output_fft,'r', output_idw, 'g', output_poly, 'k')" >> ../../comparison-$filename.m
echo "print -dpng $filename.png" >> ../../comparison-$filename.m

rm tmp.wbro
rm tmp.bro