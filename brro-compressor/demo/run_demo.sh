#!/bin/bash
infilename=$1

echo "Original Size: "
du -sb $infilename

for i in 1 5 10; 
do 
    echo "### Error Level: $i";
    mfile="comparison-error-$i.m"

    cp $infilename tmp.wbro 

    ../../target/debug/brro-compressor --compressor fft --error $i --verbose tmp.wbro > $mfile
    echo "FFT Size: "
    du -sb tmp.bro
    ../../target/debug/brro-compressor -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_fft/g'  $mfile

    cp $infilename tmp.wbro 

    ../../target/debug/brro-compressor --compressor idw --error $i tmp.wbro > /dev/null
    echo "IDW Size: "
    du -sb tmp.bro
    ../../target/debug/brro-compressor -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_idw/g'  $mfile

    cp $infilename tmp.wbro 

    ../../target/debug/brro-compressor --compressor polynomial --error $i tmp.wbro > /dev/null
    echo "Polynomial Size: "
    du -sb tmp.bro
    ../../target/debug/brro-compressor -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_poly/g'  $mfile

    echo "hold on;" >> $mfile
    echo "plot(Input,'g+', output_fft,'r', output_idw, 'b', output_poly, 'k')" >> $mfile
    echo "plot(Input.*((100+$i)/100), 'color','#D95319');" >> $mfile
    echo "plot(Input.*((100-$i)/100), 'color','#D95319');" >> $mfile
    echo "legend('Data','FFT Compression', 'IDW Compression', 'Poly compression', 'Upper Error', 'Lower Error')" >> $mfile
    echo "print -dpng comparion-$1.png" >> $mfile

done

rm tmp.wbro
rm tmp.bro