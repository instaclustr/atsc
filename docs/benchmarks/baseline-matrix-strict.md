# ATSC Baseline Matrix Report

Datasets: periodic, smooth-trend, noisy-entropy, mixed-realworld
Chunk sizes: 512, 2048, 8192, 65536
Max error (%): 0.5, 1, 2, 5
Modes: auto, forced-fft, forced-poly, forced-noop
Runs per cell: 1

|dataset|chunk|error_%|mode|median_time_ms|ratio|nrmse|decision_ms|attempts_avg|attempts_p95|retries|bound_miss_rate|noop_select_rate|codec_time_share|
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|periodic|512|0.5|auto|398.060|8.2888|0.000710|0.6415|2.00|2|0|0.495|0.000|fft-f32:67.2%, polynomial:32.8%|
|periodic|512|0.5|forced-fft|908.341|11.1028|0.011400|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-poly|167.807|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-noop|33.499|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|auto|393.995|9.2115|0.002511|0.6343|2.00|2|0|0.433|0.000|fft-f32:66.7%, polynomial:33.3%|
|periodic|512|1.0|forced-fft|756.161|13.7563|0.012200|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-poly|167.489|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-noop|32.251|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|auto|368.374|13.4558|0.011028|0.5947|2.00|2|0|0.263|0.000|fft-f32:66.2%, polynomial:33.8%|
|periodic|512|2.0|forced-fft|428.929|23.8804|0.017927|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-poly|169.531|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-noop|35.161|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|auto|337.353|56.5011|0.041183|0.5351|2.00|2|0|0.000|0.000|fft-f32:60.2%, polynomial:39.8%|
|periodic|512|5.0|forced-fft|218.353|56.5011|0.041183|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-poly|173.231|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-noop|33.629|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|auto|543.466|10.0677|0.001646|3.6365|2.00|2|0|0.422|0.000|fft-f32:71.8%, polynomial:28.2%|
|periodic|2048|0.5|forced-fft|1105.761|11.2537|0.006854|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-poly|179.072|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-noop|31.266|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|auto|445.390|11.7997|0.003966|2.9434|2.00|2|0|0.348|0.000|fft-f32:70.8%, polynomial:29.2%|
|periodic|2048|1.0|forced-fft|632.752|18.4482|0.009109|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-poly|178.149|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-noop|33.130|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|auto|422.443|26.4009|0.013722|2.7098|2.00|2|0|0.109|0.000|fft-f32:67.8%, polynomial:32.2%|
|periodic|2048|2.0|forced-fft|324.767|49.8456|0.016321|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-poly|180.686|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-noop|32.837|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|auto|382.937|79.1080|0.024431|2.4303|2.00|2|0|0.000|0.000|fft-f32:62.5%, polynomial:37.5%|
|periodic|2048|5.0|forced-fft|254.093|79.1080|0.024431|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-poly|174.466|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-noop|33.990|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|auto|497.738|55.3309|0.003743|12.9726|2.00|2|0|0.000|0.000|fft-f32:71.5%, polynomial:28.5%|
|periodic|8192|0.5|forced-fft|375.269|55.3309|0.003743|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-poly|198.380|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-noop|35.243|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|auto|452.511|81.9776|0.005616|11.1440|2.00|2|0|0.000|0.000|fft-f32:65.3%, polynomial:34.7%|
|periodic|8192|1.0|forced-fft|351.819|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-poly|197.149|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-noop|35.119|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|auto|446.199|81.9776|0.005616|11.1933|2.00|2|0|0.000|0.000|fft-f32:65.7%, polynomial:34.3%|
|periodic|8192|2.0|forced-fft|319.718|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-poly|194.352|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-noop|34.019|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|auto|443.865|81.9776|0.005616|11.1727|2.00|2|0|0.000|0.000|fft-f32:66.2%, polynomial:33.8%|
|periodic|8192|5.0|forced-fft|318.463|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-poly|194.666|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-noop|44.487|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|auto|598.518|58.0929|0.003521|124.4889|2.00|2|0|0.000|0.000|fft-f32:71.2%, polynomial:28.8%|
|periodic|65536|0.5|forced-fft|454.980|58.0929|0.003521|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-poly|216.499|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-noop|33.814|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|auto|509.344|79.7700|0.004538|102.4294|2.00|2|0|0.000|0.000|fft-f32:68.5%, polynomial:31.5%|
|periodic|65536|1.0|forced-fft|385.396|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-poly|211.187|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-noop|33.451|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|auto|513.429|79.7700|0.004538|104.4390|2.00|2|0|0.000|0.000|fft-f32:68.1%, polynomial:31.9%|
|periodic|65536|2.0|forced-fft|380.200|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-poly|213.460|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-noop|33.813|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|auto|523.340|79.7700|0.004538|106.5137|2.00|2|0|0.000|0.000|fft-f32:66.7%, polynomial:33.3%|
|periodic|65536|5.0|forced-fft|391.061|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-poly|244.081|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-noop|34.717|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|auto|381.865|8.6077|0.000072|0.6103|2.00|2|0|0.473|0.000|fft-f32:65.4%, polynomial:34.6%|
|smooth-trend|512|0.5|forced-fft|930.075|10.9851|0.000985|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-poly|171.809|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-noop|34.180|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|auto|387.171|9.3767|0.000226|0.6122|2.00|2|0|0.428|0.000|fft-f32:64.8%, polynomial:35.2%|
|smooth-trend|512|1.0|forced-fft|831.115|12.4631|0.001025|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-poly|172.309|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-noop|37.102|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|auto|321.806|17.8451|0.000948|0.5113|2.00|2|0|0.349|0.000|fft-f32:72.9%, polynomial:27.1%|
|smooth-trend|512|2.0|forced-fft|493.928|19.6930|0.001349|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-poly|124.306|17.4254|0.000973|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-noop|33.948|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|auto|264.370|57.6394|0.002715|0.3985|2.00|2|0|0.000|0.000|fft-f32:74.9%, polynomial:25.1%|
|smooth-trend|512|5.0|forced-fft|208.042|57.4122|0.002753|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-poly|98.329|45.4973|0.001352|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-noop|35.671|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|auto|483.724|9.9849|0.000126|3.1810|2.00|2|0|0.430|0.000|fft-f32:71.2%, polynomial:28.8%|
|smooth-trend|2048|0.5|forced-fft|1239.100|10.7827|0.000633|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-poly|199.863|8.7798|0.000001|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-noop|33.540|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|auto|421.427|26.7695|0.000562|2.7532|2.00|2|0|0.359|0.000|fft-f32:76.7%, polynomial:23.3%|
|smooth-trend|2048|1.0|forced-fft|796.028|15.3467|0.000760|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-poly|140.806|15.3987|0.000468|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-noop|32.942|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|auto|353.941|78.4246|0.000829|2.2793|2.00|2|0|0.168|0.000|fft-f32:82.8%, polynomial:17.2%|
|smooth-trend|2048|2.0|forced-fft|352.796|41.7601|0.001308|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-poly|102.481|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-noop|33.668|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|auto|316.490|79.1080|0.002020|1.8915|2.00|2|0|0.000|0.000|fft-f32:79.3%, polynomial:20.7%|
|smooth-trend|2048|5.0|forced-fft|256.450|79.1080|0.002020|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-poly|103.453|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-noop|33.784|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|auto|489.878|26.6258|0.000346|12.8117|2.00|2|0|0.219|0.000|fft-f32:73.0%, polynomial:27.0%|
|smooth-trend|8192|0.5|forced-fft|552.225|29.5324|0.000451|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-poly|175.440|10.7667|0.000229|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-noop|32.774|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|auto|378.716|93.8323|0.000580|9.8776|2.00|2|0|0.000|0.000|fft-f32:83.1%, polynomial:16.9%|
|smooth-trend|8192|1.0|forced-fft|343.987|67.7112|0.000742|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-poly|114.391|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-noop|35.279|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|auto|357.241|93.8323|0.000580|9.2184|2.00|2|0|0.000|0.000|fft-f32:80.6%, polynomial:19.4%|
|smooth-trend|8192|2.0|forced-fft|326.934|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-poly|149.972|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-noop|37.659|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|auto|385.212|93.8323|0.000580|9.6328|2.00|2|0|0.000|0.000|fft-f32:80.0%, polynomial:20.0%|
|smooth-trend|8192|5.0|forced-fft|309.849|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-poly|111.136|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-noop|33.367|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|auto|549.135|99.0437|0.000485|118.8143|2.00|2|0|0.125|0.000|fft-f32:86.0%, polynomial:14.0%|
|smooth-trend|65536|0.5|forced-fft|544.966|42.6424|0.001564|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-poly|125.282|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-noop|33.394|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|auto|413.639|99.0437|0.000485|86.8089|2.00|2|0|0.000|0.000|fft-f32:82.2%, polynomial:17.8%|
|smooth-trend|65536|1.0|forced-fft|388.898|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-poly|126.757|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-noop|34.307|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|auto|417.295|99.0437|0.000485|87.5237|2.00|2|0|0.000|0.000|fft-f32:82.2%, polynomial:17.8%|
|smooth-trend|65536|2.0|forced-fft|373.595|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-poly|123.959|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-noop|33.132|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|auto|423.249|99.0437|0.000485|88.6062|2.00|2|0|0.000|0.000|fft-f32:81.4%, polynomial:18.6%|
|smooth-trend|65536|5.0|forced-fft|395.062|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-poly|133.423|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-noop|36.079|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|auto|2039.987|0.9919|0.000000|3.7686|4.02|4|512|0.751|0.018|noop:0.0%, fft-f32:66.6%, polynomial:33.4%|
|noisy-entropy|512|0.5|forced-fft|1108.681|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-poly|542.925|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-noop|31.054|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|auto|1911.275|0.9919|0.000000|3.5248|4.02|4|512|0.751|0.018|noop:0.0%, fft-f32:66.5%, polynomial:33.5%|
|noisy-entropy|512|1.0|forced-fft|1036.355|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-poly|518.178|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-noop|32.194|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|auto|1997.610|0.9919|0.000000|3.6720|4.02|4|512|0.751|0.018|noop:0.0%, fft-f32:66.6%, polynomial:33.4%|
|noisy-entropy|512|2.0|forced-fft|1059.234|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-poly|515.276|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-noop|30.272|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|auto|1905.121|0.9919|0.000000|3.5167|4.02|4|512|0.751|0.018|noop:0.0%, fft-f32:66.7%, polynomial:33.3%|
|noisy-entropy|512|5.0|forced-fft|1026.961|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-poly|530.599|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-noop|31.043|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|auto|2323.715|0.9979|0.000000|17.2859|4.00|4|128|0.750|0.000|fft-f32:72.6%, polynomial:27.4%|
|noisy-entropy|2048|0.5|forced-fft|1396.008|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-poly|535.675|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-noop|29.541|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|auto|2325.612|0.9979|0.000000|17.3370|4.00|4|128|0.750|0.000|fft-f32:72.8%, polynomial:27.2%|
|noisy-entropy|2048|1.0|forced-fft|1407.970|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-poly|532.737|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-noop|32.251|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|auto|2333.259|0.9979|0.000000|17.4245|4.00|4|128|0.750|0.000|fft-f32:72.7%, polynomial:27.3%|
|noisy-entropy|2048|2.0|forced-fft|1405.976|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-poly|536.723|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-noop|30.865|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|auto|2384.366|0.9979|0.000000|17.4810|4.00|4|128|0.750|0.000|fft-f32:72.9%, polynomial:27.1%|
|noisy-entropy|2048|5.0|forced-fft|1420.348|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-poly|543.246|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-noop|29.718|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|auto|2767.452|0.9995|0.000000|82.9924|4.00|4|32|0.750|0.000|fft-f32:75.8%, polynomial:24.2%|
|noisy-entropy|8192|0.5|forced-fft|1720.400|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-poly|563.160|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-noop|29.940|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|auto|2798.876|0.9995|0.000000|84.1383|4.00|4|32|0.750|0.000|fft-f32:75.9%, polynomial:24.1%|
|noisy-entropy|8192|1.0|forced-fft|1753.374|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-poly|560.264|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-noop|30.494|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|auto|2846.838|0.9995|0.000000|85.5129|4.00|4|32|0.750|0.000|fft-f32:75.6%, polynomial:24.4%|
|noisy-entropy|8192|2.0|forced-fft|1710.011|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-poly|557.627|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-noop|32.599|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|auto|2729.131|0.9995|0.000000|81.8912|4.00|4|32|0.750|0.000|fft-f32:75.8%, polynomial:24.2%|
|noisy-entropy|8192|5.0|forced-fft|1736.799|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-poly|562.272|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-noop|30.436|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|auto|3366.344|0.9999|0.000000|811.0492|4.00|4|4|0.750|0.000|fft-f32:77.2%, polynomial:22.8%|
|noisy-entropy|65536|0.5|forced-fft|2193.916|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-poly|623.960|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-noop|30.501|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|auto|3283.573|0.9999|0.000000|790.5154|4.00|4|4|0.750|0.000|fft-f32:77.0%, polynomial:23.0%|
|noisy-entropy|65536|1.0|forced-fft|2106.036|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-poly|614.907|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-noop|29.803|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|auto|3295.266|0.9999|0.000000|794.3519|4.00|4|4|0.750|0.000|fft-f32:77.1%, polynomial:22.9%|
|noisy-entropy|65536|2.0|forced-fft|2153.144|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-poly|625.763|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-noop|30.029|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|auto|3257.166|0.9999|0.000000|784.7807|4.00|4|4|0.750|0.000|fft-f32:77.5%, polynomial:22.5%|
|noisy-entropy|65536|5.0|forced-fft|2090.772|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-poly|627.916|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-noop|28.242|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|auto|1743.796|0.9946|0.000000|3.2732|4.48|5|512|0.777|0.477|noop:0.3%, fft-f32:69.6%, polynomial:30.1%|
|mixed-realworld|512|0.5|forced-fft|981.949|9.9407|0.016427|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-poly|401.986|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-noop|32.308|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|auto|1715.809|1.0655|0.000387|3.2176|4.37|5|512|0.771|0.438|noop:0.3%, fft-f32:69.9%, polynomial:29.8%|
|mixed-realworld|512|1.0|forced-fft|988.211|10.2778|0.016429|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-poly|414.834|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-noop|29.000|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|auto|1570.363|1.2532|0.002453|2.9312|4.03|5|451|0.752|0.359|noop:0.3%, fft-f32:69.7%, polynomial:30.0%|
|mixed-realworld|512|2.0|forced-fft|886.904|11.4341|0.016465|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-poly|404.539|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-noop|30.254|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|auto|891.329|3.8904|0.021138|1.6140|2.82|4|281|0.646|0.045|noop:0.1%, fft-f32:68.5%, polynomial:31.4%|
|mixed-realworld|512|5.0|forced-fft|499.591|20.7854|0.021138|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-poly|401.391|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-noop|27.546|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|auto|2250.598|0.9984|0.000000|16.9870|4.34|5|128|0.769|0.336|noop:0.2%, fft-f32:74.7%, polynomial:25.1%|
|mixed-realworld|2048|0.5|forced-fft|1380.649|8.7327|0.014993|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-poly|453.157|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-noop|27.243|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|auto|2205.307|0.9984|0.000000|16.6203|4.34|5|128|0.769|0.336|noop:0.2%, fft-f32:74.6%, polynomial:25.2%|
|mixed-realworld|2048|1.0|forced-fft|1357.561|8.7327|0.014993|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-poly|457.471|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-noop|26.548|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|auto|1831.458|1.6127|0.007857|13.7320|3.77|5|123|0.734|0.219|noop:0.1%, fft-f32:78.1%, polynomial:21.7%|
|mixed-realworld|2048|2.0|forced-fft|1165.030|9.9641|0.015382|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-poly|441.616|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-noop|34.134|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|auto|533.847|21.6699|0.027581|3.5749|2.09|2|6|0.517|0.008|noop:0.0%, fft-f32:59.1%, polynomial:40.9%|
|mixed-realworld|2048|5.0|forced-fft|318.596|54.2849|0.027581|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-poly|433.716|1.5097|0.083468|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-noop|27.614|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|auto|2720.812|0.9995|0.000000|81.7816|4.00|4|32|0.750|0.000|fft-f32:76.1%, polynomial:23.9%|
|mixed-realworld|8192|0.5|forced-fft|1703.352|8.5638|0.013588|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-poly|543.229|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-noop|25.993|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|auto|2709.489|0.9995|0.000000|81.3963|4.00|4|32|0.750|0.000|fft-f32:76.1%, polynomial:23.9%|
|mixed-realworld|8192|1.0|forced-fft|1699.916|8.5638|0.013588|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-poly|538.443|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-noop|28.816|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|auto|1144.461|25.6392|0.019536|33.0360|3.00|3|32|0.667|0.000|fft-f32:83.0%, polynomial:17.0%|
|mixed-realworld|8192|2.0|forced-fft|601.616|25.6392|0.019536|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-poly|549.912|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-noop|27.849|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|auto|513.112|81.9748|0.025899|13.4453|2.00|2|0|0.500|0.000|fft-f32:56.4%, polynomial:43.6%|
|mixed-realworld|8192|5.0|forced-fft|372.195|81.9748|0.025899|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-poly|551.983|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-noop|28.954|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|auto|3149.436|1.0000|0.000000|770.5366|4.50|5|4|0.778|0.500|noop:0.2%, fft-f32:79.6%, polynomial:20.2%|
|mixed-realworld|65536|0.5|forced-fft|2109.161|8.0953|0.008577|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-poly|498.995|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-noop|30.091|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|auto|2160.051|10.0142|0.009765|514.2668|3.00|3|4|0.667|0.000|fft-f32:89.8%, polynomial:10.2%|
|mixed-realworld|65536|1.0|forced-fft|1584.017|10.0142|0.009765|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-poly|503.302|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-noop|32.786|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|auto|739.995|53.2659|0.019059|160.8240|2.00|2|0|0.500|0.000|fft-f32:62.7%, polynomial:37.3%|
|mixed-realworld|65536|2.0|forced-fft|458.833|53.2659|0.019059|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-poly|500.487|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-noop|29.957|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|auto|645.521|79.7672|0.021336|135.2986|2.00|2|0|0.500|0.000|fft-f32:57.0%, polynomial:43.0%|
|mixed-realworld|65536|5.0|forced-fft|396.095|79.7672|0.021336|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-poly|494.248|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-noop|32.275|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|

Notes: `attempts_*`, `retries`, `bound_miss_rate`, and `codec_time_share` come from perf-telemetry in auto mode only.
Iteration metrics are currently not emitted by codecs, so per-codec avg/p95 iterations are not yet available.
