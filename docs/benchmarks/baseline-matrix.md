# ATSC Baseline Matrix Report

Datasets: periodic, smooth-trend, noisy-entropy, mixed-realworld
Chunk sizes: 512, 2048, 8192, 65536
Max error (%): 0.5, 1, 2, 5
Modes: auto, forced-fft, forced-poly, forced-noop
Runs per cell: 1

|dataset|chunk|error_%|mode|median_time_ms|ratio|nrmse|decision_ms|attempts_avg|attempts_p95|retries|bound_miss_rate|codec_time_share|
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
|periodic|512|0.5|auto|992.097|9.3742|0.001913|1.8135|3.00|3|0|0.264|noop:1.2%, fft-f32:87.5%, polynomial:11.3%|
|periodic|512|0.5|forced-fft|1594.241|11.1028|0.011400|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-poly|162.839|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-noop|31.598|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|auto|803.200|12.1160|0.006306|1.4523|3.00|3|0|0.156|noop:1.4%, fft-f32:84.9%, polynomial:13.7%|
|periodic|512|1.0|forced-fft|1147.160|13.7563|0.012200|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-poly|176.948|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-noop|34.745|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|auto|534.273|23.3320|0.017369|0.9352|3.00|3|0|0.016|noop:2.2%, fft-f32:75.9%, polynomial:21.9%|
|periodic|512|2.0|forced-fft|444.224|23.8804|0.017927|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-poly|165.136|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-noop|33.519|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|auto|320.586|56.5011|0.041183|0.5163|3.00|3|0|0.000|noop:3.9%, fft-f32:57.9%, polynomial:38.1%|
|periodic|512|5.0|forced-fft|200.812|56.5011|0.041183|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-poly|159.939|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-noop|30.185|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|auto|1134.469|11.3034|0.002705|8.2918|3.00|3|0|0.214|noop:1.0%, fft-f32:88.8%, polynomial:10.3%|
|periodic|2048|0.5|forced-fft|1790.748|11.2537|0.006854|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-poly|180.379|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-noop|32.042|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|auto|793.044|18.4660|0.008571|5.6799|3.00|3|0|0.026|noop:1.6%, fft-f32:82.9%, polynomial:15.6%|
|periodic|2048|1.0|forced-fft|720.995|18.4482|0.009109|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-poly|172.186|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-noop|39.116|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|auto|424.092|49.8456|0.016321|2.7855|3.00|3|0|0.000|noop:3.0%, fft-f32:67.1%, polynomial:30.0%|
|periodic|2048|2.0|forced-fft|287.960|49.8456|0.016321|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-poly|165.306|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-noop|33.294|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|auto|352.017|79.1080|0.024431|2.2390|3.00|3|0|0.000|noop:3.6%, fft-f32:60.6%, polynomial:35.8%|
|periodic|2048|5.0|forced-fft|229.829|79.1080|0.024431|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-poly|171.380|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-noop|31.230|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|auto|477.358|55.3309|0.003743|12.4474|3.00|3|0|0.000|noop:2.5%, fft-f32:69.3%, polynomial:28.2%|
|periodic|8192|0.5|forced-fft|349.664|55.3309|0.003743|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-poly|182.437|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-noop|29.687|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|auto|409.504|81.9776|0.005616|10.4486|3.00|3|0|0.000|noop:3.0%, fft-f32:63.7%, polynomial:33.3%|
|periodic|8192|1.0|forced-fft|281.526|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-poly|185.681|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-noop|28.495|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|auto|405.858|81.9776|0.005616|10.2727|3.00|3|0|0.000|noop:3.1%, fft-f32:64.2%, polynomial:32.8%|
|periodic|8192|2.0|forced-fft|277.304|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-poly|179.632|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-noop|29.804|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|auto|404.826|81.9776|0.005616|10.2460|3.00|3|0|0.000|noop:3.1%, fft-f32:63.7%, polynomial:33.2%|
|periodic|8192|5.0|forced-fft|273.506|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-poly|172.764|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-noop|29.701|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|auto|533.293|58.0929|0.003521|110.8120|3.00|3|0|0.000|noop:2.4%, fft-f32:68.7%, polynomial:28.9%|
|periodic|65536|0.5|forced-fft|401.672|58.0929|0.003521|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-poly|207.574|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-noop|30.768|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|auto|495.063|79.7700|0.004538|100.6566|3.00|3|0|0.000|noop:2.8%, fft-f32:64.6%, polynomial:32.6%|
|periodic|65536|1.0|forced-fft|343.627|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-poly|196.606|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-noop|31.219|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|auto|470.811|79.7700|0.004538|95.9160|3.00|3|0|0.000|noop:2.7%, fft-f32:65.2%, polynomial:32.1%|
|periodic|65536|2.0|forced-fft|351.608|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-poly|196.250|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-noop|30.759|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|auto|494.225|79.7700|0.004538|100.9286|3.00|3|0|0.000|noop:2.4%, fft-f32:65.8%, polynomial:31.7%|
|periodic|65536|5.0|forced-fft|332.867|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-poly|195.348|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-noop|29.753|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|auto|989.864|9.1991|0.000131|1.8116|3.00|3|0|0.281|noop:1.2%, fft-f32:87.5%, polynomial:11.3%|
|smooth-trend|512|0.5|forced-fft|1585.351|10.9851|0.000985|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-poly|162.248|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-noop|35.457|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|auto|865.855|10.6878|0.000405|1.5665|3.00|3|0|0.212|noop:1.3%, fft-f32:86.0%, polynomial:12.7%|
|smooth-trend|512|1.0|forced-fft|1274.754|12.4631|0.001025|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-poly|161.066|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-noop|31.788|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|auto|518.709|24.8460|0.001287|0.9132|3.00|3|0|0.000|noop:2.1%, fft-f32:84.1%, polynomial:13.8%|
|smooth-trend|512|2.0|forced-fft|444.491|19.6930|0.001349|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-poly|118.921|17.4254|0.000973|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-noop|33.545|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|auto|234.315|57.6394|0.002715|0.3516|3.00|3|0|0.000|noop:5.5%, fft-f32:71.3%, polynomial:23.2%|
|smooth-trend|512|5.0|forced-fft|179.180|57.4122|0.002753|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-poly|88.755|45.4973|0.001352|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-noop|31.506|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|auto|1142.707|10.8317|0.000204|8.3665|3.00|3|0|0.227|noop:0.9%, fft-f32:89.5%, polynomial:9.6%|
|smooth-trend|2048|0.5|forced-fft|1845.093|10.7827|0.000633|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-poly|167.240|8.7798|0.000001|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-noop|32.344|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|auto|806.312|40.2517|0.000671|5.8510|3.00|3|0|0.076|noop:1.4%, fft-f32:88.3%, polynomial:10.3%|
|smooth-trend|2048|1.0|forced-fft|1009.111|15.3467|0.000760|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-poly|140.855|15.3987|0.000468|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-noop|30.555|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|auto|409.775|78.4246|0.000829|2.7058|3.00|3|0|0.000|noop:3.1%, fft-f32:81.9%, polynomial:15.0%|
|smooth-trend|2048|2.0|forced-fft|327.799|41.7601|0.001308|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-poly|97.451|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-noop|31.213|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|auto|294.659|79.1080|0.002020|1.7850|3.00|3|0|0.000|noop:4.4%, fft-f32:75.3%, polynomial:20.3%|
|smooth-trend|2048|5.0|forced-fft|241.414|79.1080|0.002020|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-poly|99.920|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-noop|32.277|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|auto|612.640|43.9737|0.000442|16.7343|3.00|3|0|0.000|noop:1.9%, fft-f32:79.6%, polynomial:18.4%|
|smooth-trend|8192|0.5|forced-fft|519.330|29.5324|0.000451|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-poly|169.862|10.7667|0.000229|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-noop|30.543|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|auto|356.255|93.8323|0.000580|9.4288|3.00|3|0|0.000|noop:3.4%, fft-f32:79.1%, polynomial:17.5%|
|smooth-trend|8192|1.0|forced-fft|316.721|67.7112|0.000742|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-poly|104.093|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-noop|28.512|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|auto|337.278|93.8323|0.000580|8.6854|3.00|3|0|0.000|noop:3.9%, fft-f32:77.9%, polynomial:18.2%|
|smooth-trend|8192|2.0|forced-fft|278.037|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-poly|107.159|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-noop|32.161|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|auto|323.123|93.8323|0.000580|8.4620|3.00|3|0|0.000|noop:3.9%, fft-f32:76.9%, polynomial:19.1%|
|smooth-trend|8192|5.0|forced-fft|291.743|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-poly|107.722|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-noop|31.408|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|auto|510.466|99.0437|0.000485|112.0029|3.00|3|0|0.000|noop:2.7%, fft-f32:84.2%, polynomial:13.1%|
|smooth-trend|65536|0.5|forced-fft|471.088|42.6424|0.001564|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-poly|121.679|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-noop|32.470|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|auto|389.069|99.0437|0.000485|82.6372|3.00|3|0|0.000|noop:3.0%, fft-f32:79.8%, polynomial:17.2%|
|smooth-trend|65536|1.0|forced-fft|344.164|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-poly|123.358|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-noop|31.197|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|auto|392.279|99.0437|0.000485|82.1705|3.00|3|0|0.000|noop:3.2%, fft-f32:79.5%, polynomial:17.3%|
|smooth-trend|65536|2.0|forced-fft|349.403|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-poly|118.768|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-noop|31.307|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|auto|387.898|99.0437|0.000485|81.0242|3.00|3|0|0.000|noop:3.3%, fft-f32:78.8%, polynomial:17.9%|
|smooth-trend|65536|5.0|forced-fft|349.483|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-poly|118.501|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-noop|31.283|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|auto|1336.330|0.9978|0.000000|2.5606|3.00|3|0|0.333|noop:0.8%, fft-f32:69.1%, polynomial:30.1%|
|noisy-entropy|512|0.5|forced-fft|1831.984|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-poly|505.387|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-noop|31.192|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|auto|1336.896|0.9978|0.000000|2.5613|3.00|3|0|0.333|noop:0.8%, fft-f32:69.1%, polynomial:30.2%|
|noisy-entropy|512|1.0|forced-fft|1863.486|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-poly|500.694|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-noop|32.220|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|auto|1315.222|0.9978|0.000000|2.5218|3.00|3|0|0.333|noop:0.8%, fft-f32:68.9%, polynomial:30.3%|
|noisy-entropy|512|2.0|forced-fft|1862.618|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-poly|488.477|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-noop|30.042|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|auto|1333.367|0.9978|0.000000|2.5556|3.00|3|0|0.333|noop:0.8%, fft-f32:69.0%, polynomial:30.3%|
|noisy-entropy|512|5.0|forced-fft|1809.874|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-poly|484.682|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-noop|28.903|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|auto|1642.918|0.9994|0.000000|12.6568|3.00|3|0|0.333|noop:0.6%, fft-f32:75.0%, polynomial:24.4%|
|noisy-entropy|2048|0.5|forced-fft|2488.849|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-poly|492.689|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-noop|28.664|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|auto|1671.440|0.9994|0.000000|12.8735|3.00|3|0|0.333|noop:0.6%, fft-f32:74.6%, polynomial:24.7%|
|noisy-entropy|2048|1.0|forced-fft|2444.764|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-poly|484.479|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-noop|33.970|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|auto|1663.998|0.9994|0.000000|12.8140|3.00|3|0|0.333|noop:0.6%, fft-f32:75.0%, polynomial:24.4%|
|noisy-entropy|2048|2.0|forced-fft|2520.054|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-poly|505.334|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-noop|31.012|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|auto|1657.577|0.9994|0.000000|12.7650|3.00|3|0|0.333|noop:0.6%, fft-f32:75.1%, polynomial:24.3%|
|noisy-entropy|2048|5.0|forced-fft|2555.232|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-poly|488.614|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-noop|30.069|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|auto|2204.744|0.9999|0.000000|68.0861|3.00|3|0|0.333|noop:0.5%, fft-f32:78.1%, polynomial:21.4%|
|noisy-entropy|8192|0.5|forced-fft|3418.143|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-poly|559.822|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-noop|31.044|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|auto|2055.119|0.9999|0.000000|63.5666|3.00|3|0|0.333|noop:0.5%, fft-f32:77.4%, polynomial:22.1%|
|noisy-entropy|8192|1.0|forced-fft|3296.431|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-poly|572.115|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-noop|31.066|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|auto|2033.667|0.9999|0.000000|62.8106|3.00|3|0|0.333|noop:0.5%, fft-f32:77.7%, polynomial:21.7%|
|noisy-entropy|8192|2.0|forced-fft|3192.086|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-poly|559.005|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-noop|31.056|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|auto|2073.438|0.9999|0.000000|64.0732|3.00|3|0|0.333|noop:0.5%, fft-f32:77.9%, polynomial:21.6%|
|noisy-entropy|8192|5.0|forced-fft|3314.135|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-poly|568.641|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-noop|30.511|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|auto|2500.039|1.0000|0.000000|619.8438|3.00|3|0|0.333|noop:0.5%, fft-f32:78.9%, polynomial:20.6%|
|noisy-entropy|65536|0.5|forced-fft|3927.531|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-poly|632.708|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-noop|32.326|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|auto|2638.371|1.0000|0.000000|654.1437|3.00|3|0|0.333|noop:0.4%, fft-f32:79.2%, polynomial:20.4%|
|noisy-entropy|65536|1.0|forced-fft|4048.519|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-poly|639.830|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-noop|32.197|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|auto|2656.815|1.0000|0.000000|658.6594|3.00|3|0|0.333|noop:0.4%, fft-f32:79.3%, polynomial:20.3%|
|noisy-entropy|65536|2.0|forced-fft|3976.660|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-poly|633.145|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-noop|36.103|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|auto|2486.387|1.0000|0.000000|615.9930|3.00|3|0|0.333|noop:0.4%, fft-f32:78.9%, polynomial:20.6%|
|noisy-entropy|65536|5.0|forced-fft|3933.969|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-poly|603.691|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-noop|32.287|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|auto|1354.273|0.9978|0.000000|2.5924|3.00|3|0|0.333|noop:0.8%, fft-f32:68.8%, polynomial:30.4%|
|mixed-realworld|512|0.5|forced-fft|1845.887|9.9411|0.042675|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-poly|484.995|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-noop|29.303|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|auto|1337.930|0.9978|0.000000|2.5674|3.00|3|0|0.333|noop:0.8%, fft-f32:68.8%, polynomial:30.4%|
|mixed-realworld|512|1.0|forced-fft|1842.840|9.9411|0.042675|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-poly|505.797|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-noop|31.140|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|auto|1310.371|0.9978|0.000000|2.5146|3.00|3|0|0.333|noop:0.8%, fft-f32:68.9%, polynomial:30.3%|
|mixed-realworld|512|2.0|forced-fft|1859.701|9.9411|0.042675|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-poly|488.555|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-noop|29.335|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|auto|1334.946|0.9978|0.000000|2.5588|3.00|3|0|0.333|noop:0.8%, fft-f32:68.9%, polynomial:30.3%|
|mixed-realworld|512|5.0|forced-fft|1892.999|9.9411|0.042675|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-poly|496.925|0.9918|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-noop|32.983|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|auto|1671.163|0.9994|0.000000|12.8780|3.00|3|0|0.333|noop:0.6%, fft-f32:74.9%, polynomial:24.5%|
|mixed-realworld|2048|0.5|forced-fft|2461.971|8.7330|0.039427|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-poly|503.074|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-noop|33.060|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|auto|1659.217|0.9994|0.000000|12.7853|3.00|3|0|0.333|noop:0.6%, fft-f32:74.8%, polynomial:24.6%|
|mixed-realworld|2048|1.0|forced-fft|2475.473|8.7330|0.039427|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-poly|506.337|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-noop|31.429|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|auto|1654.668|0.9994|0.000000|12.7420|3.00|3|0|0.333|noop:0.6%, fft-f32:74.8%, polynomial:24.6%|
|mixed-realworld|2048|2.0|forced-fft|2436.389|8.7330|0.039427|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-poly|497.237|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-noop|31.058|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|auto|1647.355|0.9994|0.000000|12.6950|3.00|3|0|0.333|noop:0.6%, fft-f32:74.9%, polynomial:24.4%|
|mixed-realworld|2048|5.0|forced-fft|2468.466|8.7330|0.039427|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-poly|514.945|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-noop|32.394|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|auto|1958.210|0.9999|0.000000|60.5495|3.00|3|0|0.333|noop:0.5%, fft-f32:77.8%, polynomial:21.7%|
|mixed-realworld|8192|0.5|forced-fft|3066.373|8.5641|0.038280|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-poly|530.134|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-noop|28.151|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|auto|1971.133|0.9999|0.000000|60.9515|3.00|3|0|0.333|noop:0.5%, fft-f32:77.6%, polynomial:21.8%|
|mixed-realworld|8192|1.0|forced-fft|3136.444|8.5641|0.038280|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-poly|574.351|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-noop|30.174|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|auto|2011.869|0.9999|0.000000|62.2056|3.00|3|0|0.333|noop:0.5%, fft-f32:78.0%, polynomial:21.5%|
|mixed-realworld|8192|2.0|forced-fft|3069.764|8.5641|0.038280|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-poly|533.992|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-noop|28.455|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|auto|1956.791|0.9999|0.000000|60.4482|3.00|3|0|0.333|noop:0.5%, fft-f32:77.7%, polynomial:21.7%|
|mixed-realworld|8192|5.0|forced-fft|3126.684|8.5641|0.038280|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-poly|550.840|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-noop|29.815|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|auto|2533.559|1.0000|0.000000|628.1008|3.00|3|0|0.333|noop:0.4%, fft-f32:79.4%, polynomial:20.1%|
|mixed-realworld|65536|0.5|forced-fft|3861.054|8.0955|0.037622|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-poly|588.834|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-noop|30.036|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|auto|2449.847|1.0000|0.000000|607.4758|3.00|3|0|0.333|noop:0.4%, fft-f32:79.7%, polynomial:19.9%|
|mixed-realworld|65536|1.0|forced-fft|3909.656|8.0955|0.037622|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-poly|603.049|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-noop|28.707|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|auto|2399.812|1.0000|0.000000|594.7925|3.00|3|0|0.333|noop:0.4%, fft-f32:79.1%, polynomial:20.4%|
|mixed-realworld|65536|2.0|forced-fft|3854.816|8.0955|0.037622|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-poly|596.877|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-noop|29.898|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|auto|2180.869|1.8030|0.027610|531.1370|3.00|3|0|0.167|noop:0.5%, fft-f32:77.0%, polynomial:22.5%|
|mixed-realworld|65536|5.0|forced-fft|2705.581|8.5935|0.038389|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-poly|596.715|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-noop|31.162|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|

Notes: `attempts_*`, `retries`, `bound_miss_rate`, and `codec_time_share` come from perf-telemetry in auto mode only.
Iteration metrics are currently not emitted by codecs, so per-codec avg/p95 iterations are not yet available.
