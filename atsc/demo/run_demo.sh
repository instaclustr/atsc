#!/bin/bash
infilename=$1

echo "Original Size: "
du -sb $infilename

for i in 1 3; 
do 
    echo "### Error Level: $i";
    mfile="comparison-error-$i.m"

    cp $infilename tmp.wbro 

    ../../target/debug/atsc --compressor fft --error $i --verbose tmp.wbro > $mfile
    echo "FFT Size: "
    du -sb tmp.bro
    ../../target/debug/atsc -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_fft/g'  $mfile

    cp $infilename tmp.wbro 

    ../../target/debug/atsc --compressor idw --error $i tmp.wbro > /dev/null
    echo "IDW Size: "
    du -sb tmp.bro
    ../../target/debug/atsc -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_idw/g'  $mfile

    cp $infilename tmp.wbro 

    ../../target/debug/atsc --compressor polynomial --error $i tmp.wbro > /dev/null
    echo "Polynomial Size: "
    du -sb tmp.bro
    ../../target/debug/atsc -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_poly/g'  $mfile

    cp $infilename tmp.wbro 

    ../../target/debug/atsc --error $i tmp.wbro > /dev/null
    echo "Auto Size: "
    du -sb tmp.bro
    ../../target/debug/atsc -u --verbose tmp.bro >> $mfile

    sed -i -e 's/Output/output_auto/g'  $mfile

    echo "hold on;" >> $mfile
    echo "plot(Input,'g+', output_fft,'r', output_auto, 'b', output_poly, 'k')" >> $mfile
    echo "plot(Input.*((100+$i)/100), 'color','#D95319');" >> $mfile
    echo "plot(Input.*((100-$i)/100), 'color','#D95319');" >> $mfile
    echo "legend('Data','FFT Compression', 'Auto Compression', 'Poly compression', 'Upper Error', 'Lower Error')" >> $mfile
    echo "print -dpng comparison-$i.png" >> $mfile

done

rm tmp.wbro
rm tmp.bro