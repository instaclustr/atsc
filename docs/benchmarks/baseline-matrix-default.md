# ATSC Baseline Matrix Report

Datasets: periodic, smooth-trend, noisy-entropy, mixed-realworld
Chunk sizes: 512, 2048, 8192, 65536
Max error (%): 0.5, 1, 2, 5
Modes: auto, forced-fft, forced-poly, forced-noop
Runs per cell: 1

|dataset|chunk|error_%|mode|median_time_ms|ratio|nrmse|decision_ms|attempts_avg|attempts_p95|retries|bound_miss_rate|noop_select_rate|codec_time_share|
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|periodic|512|0.5|auto|424.087|8.2888|0.000710|0.6818|2.00|2|0|0.495|0.000|fft-f32:67.2%, polynomial:32.8%|
|periodic|512|0.5|forced-fft|955.539|11.1028|0.011400|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-poly|178.202|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-noop|32.432|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|auto|397.646|9.2115|0.002511|0.6437|2.00|2|0|0.433|0.000|fft-f32:67.2%, polynomial:32.8%|
|periodic|512|1.0|forced-fft|747.333|13.7563|0.012200|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-poly|170.329|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-noop|33.246|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|auto|387.327|13.4558|0.011028|0.6298|2.00|2|0|0.263|0.000|fft-f32:66.1%, polynomial:33.9%|
|periodic|512|2.0|forced-fft|432.505|23.8804|0.017927|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-poly|165.342|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-noop|34.382|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|auto|327.263|56.5011|0.041183|0.5222|2.00|2|0|0.000|0.000|fft-f32:60.1%, polynomial:39.9%|
|periodic|512|5.0|forced-fft|216.882|56.5011|0.041183|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-poly|172.774|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-noop|32.064|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|auto|462.685|10.0677|0.001646|3.0862|2.00|2|0|0.422|0.000|fft-f32:71.7%, polynomial:28.3%|
|periodic|2048|0.5|forced-fft|1063.632|11.2537|0.006854|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-poly|179.919|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-noop|34.440|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|auto|444.473|11.7997|0.003966|2.9047|2.00|2|0|0.348|0.000|fft-f32:70.6%, polynomial:29.4%|
|periodic|2048|1.0|forced-fft|639.228|18.4482|0.009109|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-poly|172.818|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-noop|32.135|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|auto|417.548|26.4009|0.013722|2.6723|2.00|2|0|0.109|0.000|fft-f32:68.1%, polynomial:31.9%|
|periodic|2048|2.0|forced-fft|314.191|49.8456|0.016321|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-poly|181.479|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-noop|34.522|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|auto|367.006|79.1080|0.024431|2.3010|2.00|2|0|0.000|0.000|fft-f32:62.9%, polynomial:37.1%|
|periodic|2048|5.0|forced-fft|253.756|79.1080|0.024431|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-poly|175.420|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-noop|31.237|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|auto|488.002|55.3309|0.003743|12.6299|2.00|2|0|0.000|0.000|fft-f32:71.2%, polynomial:28.8%|
|periodic|8192|0.5|forced-fft|359.954|55.3309|0.003743|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-poly|186.702|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-noop|32.129|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|auto|420.832|81.9776|0.005616|10.5289|2.00|2|0|0.000|0.000|fft-f32:66.3%, polynomial:33.7%|
|periodic|8192|1.0|forced-fft|304.861|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-poly|185.572|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-noop|30.479|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|auto|425.159|81.9776|0.005616|10.7040|2.00|2|0|0.000|0.000|fft-f32:66.5%, polynomial:33.5%|
|periodic|8192|2.0|forced-fft|311.337|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-poly|188.654|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-noop|29.584|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|auto|423.759|81.9776|0.005616|10.5622|2.00|2|0|0.000|0.000|fft-f32:66.0%, polynomial:34.0%|
|periodic|8192|5.0|forced-fft|308.916|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-poly|186.332|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-noop|32.499|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|auto|569.531|58.0929|0.003521|117.7080|2.00|2|0|0.000|0.000|fft-f32:71.1%, polynomial:28.9%|
|periodic|65536|0.5|forced-fft|413.792|58.0929|0.003521|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-poly|208.571|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-noop|28.456|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|auto|511.019|79.7700|0.004538|103.5769|2.00|2|0|0.000|0.000|fft-f32:67.8%, polynomial:32.2%|
|periodic|65536|1.0|forced-fft|384.445|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-poly|212.878|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-noop|31.061|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|auto|508.311|79.7700|0.004538|103.4760|2.00|2|0|0.000|0.000|fft-f32:67.8%, polynomial:32.2%|
|periodic|65536|2.0|forced-fft|376.989|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-poly|210.378|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-noop|32.345|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|auto|517.888|79.7700|0.004538|104.1527|2.00|2|0|0.000|0.000|fft-f32:67.6%, polynomial:32.4%|
|periodic|65536|5.0|forced-fft|362.490|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-poly|228.131|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-noop|30.131|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|auto|370.840|8.6077|0.000072|0.5876|2.00|2|0|0.473|0.000|fft-f32:65.4%, polynomial:34.6%|
|smooth-trend|512|0.5|forced-fft|893.681|10.9851|0.000985|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-poly|164.157|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-noop|33.917|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|auto|358.258|9.3767|0.000226|0.5730|2.00|2|0|0.428|0.000|fft-f32:64.5%, polynomial:35.5%|
|smooth-trend|512|1.0|forced-fft|769.662|12.4631|0.001025|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-poly|160.211|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-noop|31.199|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|auto|297.732|17.8451|0.000948|0.4696|2.00|2|0|0.349|0.000|fft-f32:72.6%, polynomial:27.4%|
|smooth-trend|512|2.0|forced-fft|471.607|19.6930|0.001349|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-poly|116.875|17.4254|0.000973|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-noop|30.576|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|auto|244.897|57.6394|0.002715|0.3563|2.00|2|0|0.000|0.000|fft-f32:75.1%, polynomial:24.9%|
|smooth-trend|512|5.0|forced-fft|197.468|57.4122|0.002753|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-poly|92.290|45.4973|0.001352|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-noop|33.271|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|auto|442.457|9.9849|0.000126|2.9334|2.00|2|0|0.430|0.000|fft-f32:71.3%, polynomial:28.7%|
|smooth-trend|2048|0.5|forced-fft|1085.797|10.7827|0.000633|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-poly|179.729|8.7798|0.000001|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-noop|33.602|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|auto|398.198|26.7695|0.000562|2.6208|2.00|2|0|0.359|0.000|fft-f32:76.6%, polynomial:23.4%|
|smooth-trend|2048|1.0|forced-fft|753.728|15.3467|0.000760|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-poly|139.058|15.3987|0.000468|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-noop|32.644|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|auto|340.640|78.4246|0.000829|2.1925|2.00|2|0|0.168|0.000|fft-f32:82.5%, polynomial:17.5%|
|smooth-trend|2048|2.0|forced-fft|345.922|41.7601|0.001308|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-poly|101.617|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-noop|31.029|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|auto|293.349|79.1080|0.002020|1.7503|2.00|2|0|0.000|0.000|fft-f32:79.1%, polynomial:20.9%|
|smooth-trend|2048|5.0|forced-fft|241.939|79.1080|0.002020|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-poly|100.943|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-noop|30.227|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|auto|465.544|26.6258|0.000346|12.2973|2.00|2|0|0.219|0.000|fft-f32:73.7%, polynomial:26.3%|
|smooth-trend|8192|0.5|forced-fft|522.641|29.5324|0.000451|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-poly|165.201|10.7667|0.000229|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-noop|34.194|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|auto|360.531|93.8323|0.000580|9.5061|2.00|2|0|0.000|0.000|fft-f32:82.8%, polynomial:17.2%|
|smooth-trend|8192|1.0|forced-fft|321.693|67.7112|0.000742|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-poly|108.585|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-noop|31.082|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|auto|328.696|93.8323|0.000580|8.5305|2.00|2|0|0.000|0.000|fft-f32:81.4%, polynomial:18.6%|
|smooth-trend|8192|2.0|forced-fft|293.557|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-poly|107.782|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-noop|31.616|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|auto|323.083|93.8323|0.000580|8.3783|2.00|2|0|0.000|0.000|fft-f32:80.8%, polynomial:19.2%|
|smooth-trend|8192|5.0|forced-fft|303.995|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-poly|105.350|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-noop|43.445|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|auto|557.243|99.0437|0.000485|122.3118|2.00|2|0|0.125|0.000|fft-f32:86.7%, polynomial:13.3%|
|smooth-trend|65536|0.5|forced-fft|521.463|42.6424|0.001564|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-poly|126.627|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-noop|34.761|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|auto|432.314|99.0437|0.000485|89.6282|2.00|2|0|0.000|0.000|fft-f32:81.8%, polynomial:18.2%|
|smooth-trend|65536|1.0|forced-fft|388.817|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-poly|128.907|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-noop|34.301|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|auto|404.701|99.0437|0.000485|85.5184|2.00|2|0|0.000|0.000|fft-f32:81.9%, polynomial:18.1%|
|smooth-trend|65536|2.0|forced-fft|372.573|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-poly|132.788|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-noop|34.979|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|auto|402.134|99.0437|0.000485|84.2764|2.00|2|0|0.000|0.000|fft-f32:82.0%, polynomial:18.0%|
|smooth-trend|65536|5.0|forced-fft|373.687|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-poly|122.705|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-noop|30.822|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|auto|1856.293|1.0077|0.027575|3.4230|4.00|4|512|0.754|0.000|fft-f32:66.4%, polynomial:33.6%|
|noisy-entropy|512|0.5|forced-fft|996.678|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-poly|501.396|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-noop|30.866|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|auto|1843.568|1.0077|0.027575|3.3943|4.00|4|512|0.754|0.000|fft-f32:66.6%, polynomial:33.4%|
|noisy-entropy|512|1.0|forced-fft|981.272|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-poly|498.225|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-noop|29.809|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|auto|1828.171|1.0077|0.027575|3.3624|4.00|4|512|0.754|0.000|fft-f32:66.7%, polynomial:33.3%|
|noisy-entropy|512|2.0|forced-fft|989.382|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-poly|503.618|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-noop|31.082|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|auto|1843.391|1.0077|0.027575|3.3971|4.00|4|512|0.754|0.000|fft-f32:66.4%, polynomial:33.6%|
|noisy-entropy|512|5.0|forced-fft|972.916|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-poly|494.845|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-noop|28.703|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|auto|2209.274|0.9979|0.000000|16.4589|4.00|4|128|0.750|0.000|fft-f32:72.5%, polynomial:27.5%|
|noisy-entropy|2048|0.5|forced-fft|1311.174|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-poly|506.382|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-noop|29.317|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|auto|2197.560|0.9979|0.000000|16.3610|4.00|4|128|0.750|0.000|fft-f32:72.5%, polynomial:27.5%|
|noisy-entropy|2048|1.0|forced-fft|1306.195|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-poly|508.458|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-noop|28.762|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|auto|2251.270|0.9979|0.000000|16.7704|4.00|4|128|0.750|0.000|fft-f32:72.7%, polynomial:27.3%|
|noisy-entropy|2048|2.0|forced-fft|1350.220|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-poly|518.738|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-noop|29.400|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|auto|2348.197|0.9979|0.000000|17.5551|4.00|4|128|0.750|0.000|fft-f32:73.0%, polynomial:27.0%|
|noisy-entropy|2048|5.0|forced-fft|1322.801|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-poly|518.063|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-noop|29.960|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|auto|2692.293|0.9995|0.000000|80.8310|4.00|4|32|0.750|0.000|fft-f32:76.0%, polynomial:24.0%|
|noisy-entropy|8192|0.5|forced-fft|1664.577|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-poly|540.432|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-noop|27.529|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|auto|2677.041|0.9995|0.000000|80.3765|4.00|4|32|0.750|0.000|fft-f32:75.8%, polynomial:24.2%|
|noisy-entropy|8192|1.0|forced-fft|1662.582|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-poly|545.198|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-noop|27.030|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|auto|2681.542|0.9995|0.000000|80.4993|4.00|4|32|0.750|0.000|fft-f32:75.9%, polynomial:24.1%|
|noisy-entropy|8192|2.0|forced-fft|1666.063|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-poly|595.229|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-noop|32.087|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|auto|2667.779|0.9995|0.000000|80.0536|4.00|4|32|0.750|0.000|fft-f32:75.8%, polynomial:24.2%|
|noisy-entropy|8192|5.0|forced-fft|1680.884|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-poly|555.538|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-noop|31.206|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|auto|3216.840|0.9999|0.000000|774.1527|4.00|4|4|0.750|0.000|fft-f32:77.1%, polynomial:22.9%|
|noisy-entropy|65536|0.5|forced-fft|2069.698|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-poly|603.399|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-noop|29.620|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|auto|3267.471|0.9999|0.000000|782.0200|4.00|4|4|0.750|0.000|fft-f32:76.4%, polynomial:23.6%|
|noisy-entropy|65536|1.0|forced-fft|2075.808|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-poly|613.994|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-noop|28.771|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|auto|3236.657|0.9999|0.000000|779.3106|4.00|4|4|0.750|0.000|fft-f32:77.2%, polynomial:22.8%|
|noisy-entropy|65536|2.0|forced-fft|2076.733|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-poly|604.452|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-noop|28.592|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|auto|3182.329|0.9999|0.000000|767.1604|4.00|4|4|0.750|0.000|fft-f32:76.9%, polynomial:23.1%|
|noisy-entropy|65536|5.0|forced-fft|2044.486|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-poly|602.164|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-noop|30.728|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|auto|1711.838|1.7369|0.012712|3.1780|4.00|4|512|0.869|0.000|fft-f32:69.9%, polynomial:30.1%|
|mixed-realworld|512|0.5|forced-fft|979.937|9.9407|0.016427|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-poly|396.526|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-noop|30.205|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|auto|1678.612|1.8380|0.012715|3.1112|3.93|4|512|0.857|0.000|fft-f32:70.2%, polynomial:29.8%|
|mixed-realworld|512|1.0|forced-fft|968.941|10.2778|0.016429|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-poly|398.175|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-noop|30.975|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|auto|1531.853|2.1099|0.012845|2.8353|3.67|4|451|0.825|0.000|fft-f32:69.9%, polynomial:30.1%|
|mixed-realworld|512|2.0|forced-fft|873.925|11.4341|0.016465|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-poly|398.426|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-noop|30.359|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|auto|866.627|4.6181|0.021138|1.5581|2.78|4|281|0.656|0.000|fft-f32:68.6%, polynomial:31.4%|
|mixed-realworld|512|5.0|forced-fft|523.369|20.7854|0.021138|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-poly|411.955|1.7965|0.098606|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-noop|30.783|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|auto|2370.845|1.4207|0.008411|17.7653|4.00|4|128|0.834|0.000|fft-f32:74.9%, polynomial:25.1%|
|mixed-realworld|2048|0.5|forced-fft|1379.944|8.7327|0.014993|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-poly|465.532|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-noop|30.835|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|auto|2301.160|1.4207|0.008411|17.1390|4.00|4|128|0.834|0.000|fft-f32:74.6%, polynomial:25.4%|
|mixed-realworld|2048|1.0|forced-fft|1415.409|8.7327|0.014993|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-poly|459.268|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-noop|32.110|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|auto|1894.081|2.3461|0.011016|14.1105|3.55|4|123|0.780|0.000|fft-f32:78.1%, polynomial:21.9%|
|mixed-realworld|2048|2.0|forced-fft|1197.835|9.9641|0.015382|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-poly|458.263|1.4178|0.082626|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-noop|28.930|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|auto|546.562|25.4942|0.027581|3.7134|2.08|2|6|0.519|0.000|fft-f32:59.0%, polynomial:41.0%|
|mixed-realworld|2048|5.0|forced-fft|342.748|54.2849|0.027581|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-poly|460.928|1.5097|0.083468|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-noop|31.301|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|auto|2816.125|0.9995|0.000000|84.6583|4.00|4|32|0.750|0.000|fft-f32:75.8%, polynomial:24.2%|
|mixed-realworld|8192|0.5|forced-fft|1737.038|8.5638|0.013588|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-poly|558.810|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-noop|31.311|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|auto|2822.191|0.9995|0.000000|84.7553|4.00|4|32|0.750|0.000|fft-f32:76.1%, polynomial:23.9%|
|mixed-realworld|8192|1.0|forced-fft|1721.333|8.5638|0.013588|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-poly|562.865|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-noop|30.886|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|auto|1154.489|25.6392|0.019536|33.4569|3.00|3|32|0.667|0.000|fft-f32:82.8%, polynomial:17.2%|
|mixed-realworld|8192|2.0|forced-fft|612.800|25.6392|0.019536|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-poly|562.526|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-noop|32.242|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|auto|516.354|81.9748|0.025899|13.5109|2.00|2|0|0.500|0.000|fft-f32:56.5%, polynomial:43.5%|
|mixed-realworld|8192|5.0|forced-fft|327.007|81.9748|0.025899|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-poly|593.631|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-noop|31.152|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|auto|3326.242|1.7800|0.006091|802.6275|4.00|4|4|0.875|0.000|fft-f32:80.3%, polynomial:19.7%|
|mixed-realworld|65536|0.5|forced-fft|2152.600|8.0953|0.008577|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-poly|497.130|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-noop|32.034|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|auto|2212.825|10.0142|0.009765|526.8869|3.00|3|4|0.667|0.000|fft-f32:89.3%, polynomial:10.7%|
|mixed-realworld|65536|1.0|forced-fft|1595.148|10.0142|0.009765|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-poly|495.528|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-noop|29.818|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|auto|677.640|53.2659|0.019059|144.8951|2.00|2|0|0.500|0.000|fft-f32:63.7%, polynomial:36.3%|
|mixed-realworld|65536|2.0|forced-fft|466.562|53.2659|0.019059|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-poly|498.309|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-noop|28.694|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|auto|598.973|79.7672|0.021336|126.2197|2.00|2|0|0.500|0.000|fft-f32:57.4%, polynomial:42.6%|
|mixed-realworld|65536|5.0|forced-fft|385.445|79.7672|0.021336|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-poly|498.653|1.7697|0.095274|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-noop|28.387|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|

Notes: `attempts_*`, `retries`, `bound_miss_rate`, and `codec_time_share` come from perf-telemetry in auto mode only.
Iteration metrics are currently not emitted by codecs, so per-codec avg/p95 iterations are not yet available.
