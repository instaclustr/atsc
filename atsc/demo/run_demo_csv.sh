#!/bin/bash
infilename=$1
echo "Original Size: "
du -sb $infilename
for i in 1 3; do
  echo "### Error Level: $i"
  htmlfile="comparison-error-$i-csv.html"
  cp $infilename tmp.csv
  ../../target/debug/atsc --csv --compressor fft --error $i --verbose tmp.csv > input.txt
  echo "FFT Size: "
  du -sb tmp.bro
  ../../target/debug/atsc -u --verbose tmp.bro > tmp_fft.txt
  cp $infilename tmp.csv
  ../../target/debug/atsc --csv --compressor idw --error $i tmp.csv > /dev/null
  echo "IDW Size: "
  du -sb tmp.bro
  ../../target/debug/atsc -u --verbose tmp.bro > tmp_idw.txt
  cp $infilename tmp.csv
  ../../target/debug/atsc --csv --compressor polynomial --error $i tmp.csv > /dev/null
  echo "Polynomial Size: "
  du -sb tmp.bro
  ../../target/debug/atsc -u --verbose tmp.bro > tmp_poly.txt

  # Create HTML file
  echo "<!DOCTYPE html>" > $htmlfile
  echo "<html lang=\"en\">" >> $htmlfile
  echo "<head>" >> $htmlfile
  echo "<meta charset=\"UTF-8\">" >> $htmlfile
  echo "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">" >> $htmlfile
  echo "<title>Comparison Error Level $i</title>" >> $htmlfile
  echo "<script src=\"https://cdn.jsdelivr.net/npm/chart.js\"></script>" >> $htmlfile
  echo "<script src=\"https://cdn.jsdelivr.net/npm/chartjs-plugin-zoom\"></script>" >> $htmlfile
  echo "</head>" >> $htmlfile
  echo "<body>" >> $htmlfile
  echo "<canvas id=\"myChart\" width=\"400\" height=\"200\"></canvas>" >> $htmlfile
  echo "<script>" >> $htmlfile

  # Read data from tmp files and convert to JavaScript arrays
  
  file_content=$(<input.txt)
  array_data=$(echo $file_content | grep -oP '\[.*?\]')
  js_array="const inputData = $array_data;"
  echo "$js_array" >> $htmlfile

  file_content=$(<tmp_fft.txt)
  array_data=$(echo $file_content | grep -oP '\[.*?\]')
  js_array="const fftData = $array_data;"
  echo "$js_array" >> $htmlfile

   file_content=$(<tmp_idw.txt)
  array_data=$(echo $file_content | grep -oP '\[.*?\]')
  js_array="const idwData = $array_data;"
  echo "$js_array" >> $htmlfile

  file_content=$(<tmp_poly.txt)
  array_data=$(echo $file_content | grep -oP '\[.*?\]')
  js_array="const polyData = $array_data;"
  echo "$js_array" >> $htmlfile

  # JavaScript code to create the chart
  echo "const ctx = document.getElementById('myChart').getContext('2d');" >> $htmlfile
  echo "const myChart = new Chart(ctx, {" >> $htmlfile
  echo "  type: 'line'," >> $htmlfile
  echo "  data: {" >> $htmlfile
  echo "    labels: Array.from({length: inputData.length}, (_, i) => i + 1)," >> $htmlfile
  echo "    datasets: [" >> $htmlfile
  echo "      { label: 'Data', data: inputData, borderColor: 'green', borderWidth: 1 }," >> $htmlfile
  echo "      { label: 'FFT Compression', data: fftData, borderColor: 'red', borderWidth: 1 }," >> $htmlfile
  echo "      { label: 'IDW Compression', data: idwData, borderColor: 'blue', borderWidth: 1 }," >> $htmlfile
  echo "      { label: 'Poly Compression', data: polyData, borderColor: 'black', borderWidth: 1 }" >> $htmlfile
  echo "    ]" >> $htmlfile
  echo "  }," >> $htmlfile
  echo "  options: {" >> $htmlfile
  echo "    scales: {" >> $htmlfile
  echo "      y: { beginAtZero: true }" >> $htmlfile
  echo "    }," >> $htmlfile
  echo "    plugins: {" >> $htmlfile
  echo "      zoom: {" >> $htmlfile
  echo "        pan: {" >> $htmlfile
  echo "          enabled: true," >> $htmlfile
  echo "          mode: 'xy'" >> $htmlfile
  echo "        }," >> $htmlfile
  echo "        zoom: {" >> $htmlfile
  echo "          wheel: {" >> $htmlfile
  echo "            enabled: true" >> $htmlfile
  echo "          }," >> $htmlfile
  echo "          pinch: {" >> $htmlfile
  echo "            enabled: true" >> $htmlfile
  echo "          }," >> $htmlfile
  echo "          mode: 'xy'" >> $htmlfile
  echo "        }" >> $htmlfile
  echo "      }" >> $htmlfile
  echo "    }" >> $htmlfile
  echo "  }" >> $htmlfile
  echo "});" >> $htmlfile
  echo "</script>" >> $htmlfile
  echo "</body>" >> $htmlfile
  echo "</html>" >> $htmlfile

  rm tmp.csv
  rm tmp.bro
  rm tmp.wbro
  rm tmp_fft.txt
  rm tmp_idw.txt
  rm tmp_poly.txt
  rm input.txt
done