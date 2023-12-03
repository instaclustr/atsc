#!/bin/bash
infilename=$1


for i in 1 2 3 5 10; 
do 
    echo $i;
    mfile="comparison-$filename-error-$i.m"

    cp ../../wbro-july/$filename.wbro tmp.wbro 

    target/debug/brro-compressor --compressor fft --error $i --verbose tmp.wbro > ../../$mfile
    target/debug/brro-compressor -u --verbose tmp.bro >> ../../$mfile

    sed -i -e 's/Output/output_fft/g'  ../../comparison-$filename.m

    cp ../../wbro-july/$filename.wbro tmp.wbro 

    target/debug/brro-compressor --compressor idw --error $i tmp.wbro > /dev/null
    target/debug/brro-compressor -u --verbose tmp.bro >> ../../$mfile

    sed -i -e 's/Output/output_idw/g'  ../../comparison-$filename.m

    cp ../../wbro-july/$filename.wbro tmp.wbro 

    target/debug/brro-compressor --compressor polynomial --error $i tmp.wbro > /dev/null
    target/debug/brro-compressor -u --verbose tmp.bro >> ../../$mfile

    sed -i -e 's/Output/output_poly/g'  ../../$mfile

    echo "plot(Input,'b', output_fft,'r', output_idw, 'g', output_poly, 'k')" >> ../../$mfile
    echo "print -dpng $filename.png" >> ../../$mfile

done

rm tmp.wbro
rm tmp.bro