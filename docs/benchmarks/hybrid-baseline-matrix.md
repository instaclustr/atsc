# ATSC Baseline Matrix Report

Datasets: periodic, smooth-trend, noisy-entropy, mixed-realworld
Chunk sizes: 512, 2048, 8192, 65536
Max error (%): 0.5, 1, 2, 5
Modes: auto, forced-fft, forced-poly, forced-hybrid, forced-noop
Runs per cell: 1

|dataset|chunk|error_%|mode|median_time_ms|ratio|nrmse|decision_ms|attempts_avg|attempts_p95|retries|bound_miss_rate|noop_select_rate|winner_payload_rate|fft_payload_win_rate|codec_time_share|
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|periodic|512|0.5|auto|313.916|8.2888|0.000710|0.5120|3.00|3|0|0.663|0.000|1.000|0.000|fft-f32:59.6%, polynomial:28.0%, hybrid-residual:12.4%|
|periodic|512|0.5|forced-fft|632.410|11.1028|0.011400|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-poly|113.680|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-hybrid|61.379|51.8302|0.127734|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|0.5|forced-noop|23.840|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|auto|318.655|9.2115|0.002511|0.5301|3.00|3|0|0.622|0.000|1.000|0.000|fft-f32:59.9%, polynomial:28.1%, hybrid-residual:12.1%|
|periodic|512|1.0|forced-fft|555.601|13.7563|0.012200|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-poly|119.447|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-hybrid|61.050|51.8302|0.127734|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|1.0|forced-noop|26.351|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|auto|327.261|13.4558|0.011028|0.5370|3.00|3|0|0.508|0.000|1.000|0.000|fft-f32:58.6%, polynomial:29.0%, hybrid-residual:12.5%|
|periodic|512|2.0|forced-fft|301.036|23.8804|0.017927|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-poly|119.315|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-hybrid|61.146|51.8302|0.127734|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|2.0|forced-noop|24.216|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|auto|279.710|56.5011|0.041183|0.4624|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:52.7%, polynomial:32.8%, hybrid-residual:14.5%|
|periodic|512|5.0|forced-fft|155.109|56.5011|0.041183|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-poly|115.629|8.2244|0.000546|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-hybrid|59.019|51.8302|0.127734|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|512|5.0|forced-noop|25.585|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|auto|385.704|10.0677|0.001646|2.6051|3.00|3|0|0.615|0.000|1.000|0.000|fft-f32:64.8%, polynomial:24.6%, hybrid-residual:10.6%|
|periodic|2048|0.5|forced-fft|827.005|11.2537|0.006854|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-poly|123.213|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-hybrid|64.138|121.2647|0.129287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|0.5|forced-noop|22.739|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|auto|387.256|11.7997|0.003966|2.5641|3.00|3|0|0.565|0.000|1.000|0.000|fft-f32:63.6%, polynomial:25.2%, hybrid-residual:11.2%|
|periodic|2048|1.0|forced-fft|526.437|18.4482|0.009109|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-poly|129.053|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-hybrid|71.898|121.2647|0.129287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|1.0|forced-noop|24.321|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|auto|358.602|26.4009|0.013722|2.3698|3.00|3|0|0.406|0.000|1.000|0.000|fft-f32:60.2%, polynomial:27.5%, hybrid-residual:12.3%|
|periodic|2048|2.0|forced-fft|250.062|49.8456|0.016321|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-poly|130.807|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-hybrid|71.854|121.2647|0.129287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|2.0|forced-noop|23.440|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|auto|312.121|79.1080|0.024431|2.0224|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:55.3%, polynomial:30.8%, hybrid-residual:13.9%|
|periodic|2048|5.0|forced-fft|193.550|79.1080|0.024431|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-poly|132.120|8.7798|0.000372|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-hybrid|72.375|121.2647|0.129287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|2048|5.0|forced-noop|23.842|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|auto|420.893|55.3309|0.003743|11.1073|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:64.5%, polynomial:24.4%, hybrid-residual:11.1%|
|periodic|8192|0.5|forced-fft|288.154|55.3309|0.003743|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-poly|139.989|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-hybrid|77.943|174.5590|0.127441|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|0.5|forced-noop|24.906|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|auto|366.236|81.9776|0.005616|9.4689|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:58.3%, polynomial:28.7%, hybrid-residual:13.0%|
|periodic|8192|1.0|forced-fft|233.528|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-poly|133.945|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-hybrid|76.773|174.5590|0.127441|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|1.0|forced-noop|22.886|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|auto|392.098|81.9776|0.005616|9.7750|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:57.4%, polynomial:29.5%, hybrid-residual:13.1%|
|periodic|8192|2.0|forced-fft|306.302|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-poly|169.899|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-hybrid|86.984|174.5590|0.127441|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|2.0|forced-noop|24.579|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|auto|369.539|81.9776|0.005616|9.5415|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:57.7%, polynomial:28.9%, hybrid-residual:13.4%|
|periodic|8192|5.0|forced-fft|260.247|81.9776|0.005616|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-poly|177.497|8.9403|0.000327|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-hybrid|88.083|174.5590|0.127441|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|8192|5.0|forced-noop|24.372|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|auto|469.858|58.0929|0.003521|98.7400|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:65.2%, polynomial:23.7%, hybrid-residual:11.1%|
|periodic|65536|0.5|forced-fft|327.728|58.0929|0.003521|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-poly|149.293|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-hybrid|84.954|196.0322|0.127219|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|0.5|forced-noop|26.546|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|auto|435.169|79.7700|0.004538|89.0514|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:59.9%, polynomial:27.6%, hybrid-residual:12.4%|
|periodic|65536|1.0|forced-fft|285.456|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-poly|151.577|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-hybrid|85.227|196.0322|0.127219|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|1.0|forced-noop|25.527|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|auto|417.143|79.7700|0.004538|85.7156|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:60.5%, polynomial:26.8%, hybrid-residual:12.7%|
|periodic|65536|2.0|forced-fft|280.710|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-poly|148.719|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-hybrid|85.844|196.0322|0.127219|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|2.0|forced-noop|23.670|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|auto|417.855|79.7700|0.004538|86.0290|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:60.2%, polynomial:27.4%, hybrid-residual:12.5%|
|periodic|65536|5.0|forced-fft|279.113|79.7700|0.004538|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-poly|149.753|8.9927|0.000310|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-hybrid|86.271|196.0322|0.127219|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|periodic|65536|5.0|forced-noop|23.968|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|auto|312.236|8.6077|0.000072|0.5132|3.00|3|0|0.648|0.000|1.000|0.000|fft-f32:57.9%, polynomial:29.2%, hybrid-residual:12.9%|
|smooth-trend|512|0.5|forced-fft|670.074|10.9851|0.000985|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-poly|120.712|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-hybrid|61.379|51.8302|0.005067|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|0.5|forced-noop|25.634|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|auto|301.806|9.3767|0.000226|0.4952|3.00|3|0|0.618|0.000|1.000|0.000|fft-f32:57.1%, polynomial:29.9%, hybrid-residual:12.9%|
|smooth-trend|512|1.0|forced-fft|588.010|12.4631|0.001025|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-poly|123.975|8.2244|0.000003|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-hybrid|63.950|51.8302|0.005067|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|1.0|forced-noop|25.204|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|auto|280.632|17.8451|0.000948|0.4600|3.00|3|0|0.566|0.000|1.000|0.000|fft-f32:62.8%, polynomial:22.2%, hybrid-residual:14.9%|
|smooth-trend|512|2.0|forced-fft|360.827|19.6930|0.001349|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-poly|86.767|17.4254|0.000973|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-hybrid|63.763|51.8302|0.005067|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|2.0|forced-noop|25.081|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|auto|223.440|57.6394|0.002715|0.3488|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:61.3%, polynomial:19.5%, hybrid-residual:19.2%|
|smooth-trend|512|5.0|forced-fft|146.439|57.4122|0.002753|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-poly|66.340|45.4973|0.001352|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-hybrid|64.463|51.8302|0.005067|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|512|5.0|forced-noop|23.196|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|auto|373.636|9.9849|0.000126|2.5233|3.00|3|0|0.620|0.000|1.000|0.000|fft-f32:64.0%, polynomial:24.9%, hybrid-residual:11.0%|
|smooth-trend|2048|0.5|forced-fft|855.650|10.7827|0.000633|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-poly|126.237|8.7798|0.000001|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-hybrid|70.549|121.2647|0.006159|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|0.5|forced-noop|24.975|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|auto|363.455|26.7695|0.000562|2.4529|3.00|3|0|0.573|0.000|1.000|0.000|fft-f32:67.8%, polynomial:19.9%, hybrid-residual:12.3%|
|smooth-trend|2048|1.0|forced-fft|599.166|15.3467|0.000760|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-poly|101.773|15.3987|0.000468|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-hybrid|68.866|121.2647|0.006159|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|1.0|forced-noop|24.256|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|auto|301.120|78.4246|0.000829|1.9761|3.00|3|0|0.445|0.000|1.000|0.000|fft-f32:71.5%, polynomial:14.5%, hybrid-residual:14.0%|
|smooth-trend|2048|2.0|forced-fft|272.549|41.7601|0.001308|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-poly|78.450|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-hybrid|72.064|121.2647|0.006159|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|2.0|forced-noop|25.138|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|auto|279.254|79.1080|0.002020|1.7454|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:66.1%, polynomial:16.8%, hybrid-residual:17.1%|
|smooth-trend|2048|5.0|forced-fft|202.670|79.1080|0.002020|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-poly|78.801|77.9784|0.000800|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-hybrid|71.367|121.2647|0.006159|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|2048|5.0|forced-noop|25.242|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|auto|423.688|26.6258|0.000346|11.1829|3.00|3|0|0.479|0.000|1.000|0.000|fft-f32:65.6%, polynomial:22.6%, hybrid-residual:11.8%|
|smooth-trend|8192|0.5|forced-fft|449.819|29.5324|0.000451|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-poly|127.279|10.7667|0.000229|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-hybrid|79.506|174.5590|0.005632|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|0.5|forced-noop|23.428|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|auto|322.752|93.8323|0.000580|8.6884|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:71.6%, polynomial:14.2%, hybrid-residual:14.2%|
|smooth-trend|8192|1.0|forced-fft|255.330|67.7112|0.000742|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-poly|80.098|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-hybrid|75.850|174.5590|0.005632|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|1.0|forced-noop|25.175|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|auto|335.564|93.8323|0.000580|8.5607|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:68.0%, polynomial:15.9%, hybrid-residual:16.1%|
|smooth-trend|8192|2.0|forced-fft|310.917|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-poly|102.936|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-hybrid|89.588|174.5590|0.005632|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|2.0|forced-noop|27.502|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|auto|307.493|93.8323|0.000580|8.2601|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:68.7%, polynomial:15.5%, hybrid-residual:15.8%|
|smooth-trend|8192|5.0|forced-fft|233.518|81.9776|0.000895|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-poly|81.523|93.8323|0.000580|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-hybrid|78.190|174.5590|0.005632|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|8192|5.0|forced-noop|23.675|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|auto|515.732|99.0437|0.000485|117.4983|3.00|3|0|0.417|0.000|1.000|0.000|fft-f32:60.1%, polynomial:9.5%, hybrid-residual:30.4%|
|smooth-trend|65536|0.5|forced-fft|402.488|42.6424|0.001564|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-poly|103.671|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-hybrid|203.112|185.6380|0.005135|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|0.5|forced-noop|25.777|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|auto|466.177|99.0437|0.000485|104.3532|3.00|3|0|0.333|0.000|1.000|0.000|fft-f32:53.1%, polynomial:11.0%, hybrid-residual:35.9%|
|smooth-trend|65536|1.0|forced-fft|295.612|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-poly|92.441|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-hybrid|195.030|185.6380|0.005135|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|1.0|forced-noop|26.072|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|auto|471.501|131.5984|0.003658|105.7885|3.00|3|0|0.167|0.000|1.000|0.000|fft-f32:52.9%, polynomial:11.7%, hybrid-residual:35.4%|
|smooth-trend|65536|2.0|forced-fft|290.577|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-poly|88.279|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-hybrid|179.458|192.3816|0.005144|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|2.0|forced-noop|24.855|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|auto|436.794|157.4793|0.004462|97.5315|3.00|3|0|0.083|0.000|1.000|0.000|fft-f32:53.2%, polynomial:11.3%, hybrid-residual:35.6%|
|smooth-trend|65536|5.0|forced-fft|299.333|79.7700|0.002444|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-poly|96.478|99.0437|0.000485|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-hybrid|185.250|196.0322|0.005147|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|smooth-trend|65536|5.0|forced-noop|24.994|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|auto|1867.035|1.0077|0.027575|3.4226|5.00|5|512|0.804|0.000|0.000|0.000|fft-f32:65.7%, polynomial:31.6%, hybrid-residual:2.6%|
|noisy-entropy|512|0.5|forced-fft|910.241|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-poly|418.448|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-hybrid|68.393|51.8302|0.383423|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|0.5|forced-noop|25.349|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|auto|2089.762|1.0077|0.027575|3.7123|5.00|5|512|0.804|0.000|0.000|0.000|fft-f32:65.8%, polynomial:31.5%, hybrid-residual:2.6%|
|noisy-entropy|512|1.0|forced-fft|1604.415|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-poly|748.177|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-hybrid|114.774|51.8302|0.383423|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|1.0|forced-noop|43.934|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|auto|3349.215|1.0077|0.027575|6.1979|5.00|5|512|0.804|0.000|0.000|0.000|fft-f32:66.8%, polynomial:30.5%, hybrid-residual:2.7%|
|noisy-entropy|512|2.0|forced-fft|1645.950|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-poly|814.819|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-hybrid|109.124|51.8302|0.383423|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|2.0|forced-noop|36.179|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|auto|2730.079|1.0077|0.027575|5.0732|5.00|5|512|0.804|0.000|0.000|0.000|fft-f32:66.2%, polynomial:31.2%, hybrid-residual:2.6%|
|noisy-entropy|512|5.0|forced-fft|1210.541|9.9411|0.210526|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-poly|822.404|1.0091|0.040810|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-hybrid|140.542|51.8302|0.383423|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|512|5.0|forced-noop|53.521|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|auto|4043.374|0.9979|0.000000|30.5674|5.00|5|128|0.800|0.000|0.000|0.000|fft-f32:71.1%, polynomial:26.7%, hybrid-residual:2.2%|
|noisy-entropy|2048|0.5|forced-fft|2338.373|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-poly|717.468|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-hybrid|150.580|121.2647|0.388746|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|0.5|forced-noop|47.345|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|auto|3784.679|0.9979|0.000000|28.4808|5.00|5|128|0.800|0.000|0.000|0.000|fft-f32:72.0%, polynomial:25.8%, hybrid-residual:2.3%|
|noisy-entropy|2048|1.0|forced-fft|1867.446|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-poly|600.213|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-hybrid|99.517|121.2647|0.388746|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|1.0|forced-noop|31.187|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|auto|4238.624|0.9979|0.000000|31.7971|5.00|5|128|0.800|0.000|0.000|0.000|fft-f32:71.7%, polynomial:26.2%, hybrid-residual:2.1%|
|noisy-entropy|2048|2.0|forced-fft|1700.911|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-poly|961.608|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-hybrid|152.302|121.2647|0.388746|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|2.0|forced-noop|43.502|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|auto|3902.474|0.9979|0.000000|29.4666|5.00|5|128|0.800|0.000|0.000|0.000|fft-f32:71.8%, polynomial:26.1%, hybrid-residual:2.1%|
|noisy-entropy|2048|5.0|forced-fft|1989.328|8.7330|0.201343|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-poly|707.320|0.9979|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-hybrid|134.246|121.2647|0.388746|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|2048|5.0|forced-noop|45.179|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|auto|4070.951|0.9995|0.000000|122.9747|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.5%, polynomial:22.5%, hybrid-residual:2.0%|
|noisy-entropy|8192|0.5|forced-fft|2214.918|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-poly|637.347|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-hybrid|103.296|174.5590|0.386287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|0.5|forced-noop|28.267|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|auto|2761.662|0.9995|0.000000|82.9411|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.0%, polynomial:23.0%, hybrid-residual:1.9%|
|noisy-entropy|8192|1.0|forced-fft|1731.023|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-poly|551.277|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-hybrid|95.988|174.5590|0.386287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|1.0|forced-noop|31.751|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|auto|2763.174|0.9995|0.000000|82.4009|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.1%, polynomial:23.1%, hybrid-residual:1.8%|
|noisy-entropy|8192|2.0|forced-fft|1651.207|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-poly|527.891|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-hybrid|105.029|174.5590|0.386287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|2.0|forced-noop|31.830|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|auto|2702.194|0.9995|0.000000|81.5137|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.3%, polynomial:22.8%, hybrid-residual:1.9%|
|noisy-entropy|8192|5.0|forced-fft|1606.856|8.5641|0.201148|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-poly|447.715|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-hybrid|80.994|174.5590|0.386287|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|8192|5.0|forced-noop|24.116|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|auto|3353.572|0.9999|0.000000|806.8821|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:77.1%, polynomial:21.1%, hybrid-residual:1.8%|
|noisy-entropy|65536|0.5|forced-fft|2579.366|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-poly|750.402|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-hybrid|141.893|196.0322|0.390083|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|0.5|forced-noop|43.695|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|auto|3891.195|0.9999|0.000000|946.4652|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:78.8%, polynomial:19.3%, hybrid-residual:1.9%|
|noisy-entropy|65536|1.0|forced-fft|1919.336|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-poly|668.641|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-hybrid|121.697|196.0322|0.390083|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|1.0|forced-noop|33.559|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|auto|4067.310|0.9999|0.000000|980.6359|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:78.5%, polynomial:19.7%, hybrid-residual:1.8%|
|noisy-entropy|65536|2.0|forced-fft|2134.108|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-poly|550.682|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-hybrid|98.249|196.0322|0.390083|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|2.0|forced-noop|27.712|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|auto|2771.572|0.9999|0.000000|668.9315|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:76.8%, polynomial:21.4%, hybrid-residual:1.8%|
|noisy-entropy|65536|5.0|forced-fft|1893.164|8.0955|0.199891|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-poly|639.162|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-hybrid|122.104|196.0322|0.390083|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|noisy-entropy|65536|5.0|forced-noop|32.175|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|auto|1587.841|1.4776|0.025648|2.9506|5.00|5|512|0.873|0.000|0.000|0.000|fft-f32:67.9%, polynomial:29.4%, hybrid-residual:2.7%|
|mixed-realworld|512|0.5|forced-fft|865.764|9.9411|0.042728|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-poly|463.870|1.5431|0.127074|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-hybrid|86.069|51.8302|0.204593|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|0.5|forced-noop|31.211|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|auto|1791.813|1.4776|0.025648|3.3266|5.00|5|512|0.873|0.000|0.000|0.000|fft-f32:67.8%, polynomial:29.6%, hybrid-residual:2.7%|
|mixed-realworld|512|1.0|forced-fft|927.385|9.9411|0.042728|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-poly|373.729|1.5431|0.127074|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-hybrid|78.566|51.8302|0.204593|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|1.0|forced-noop|30.072|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|auto|1651.247|1.4776|0.025648|3.0609|5.00|5|512|0.873|0.000|0.000|0.000|fft-f32:67.7%, polynomial:29.5%, hybrid-residual:2.7%|
|mixed-realworld|512|2.0|forced-fft|887.175|9.9411|0.042728|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-poly|357.442|1.5431|0.127074|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-hybrid|76.717|51.8302|0.204593|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|2.0|forced-noop|29.443|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|auto|1591.180|1.4776|0.025648|2.9560|5.00|5|512|0.873|0.000|0.000|0.000|fft-f32:67.9%, polynomial:29.4%, hybrid-residual:2.7%|
|mixed-realworld|512|5.0|forced-fft|795.736|9.9411|0.042728|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-poly|339.630|1.5431|0.127074|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-hybrid|70.136|51.8302|0.204593|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|512|5.0|forced-noop|25.773|0.9978|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|auto|1931.685|1.3539|0.021304|14.4194|5.00|5|128|0.859|0.000|0.000|0.000|fft-f32:73.4%, polynomial:24.3%, hybrid-residual:2.3%|
|mixed-realworld|2048|0.5|forced-fft|1119.929|8.7330|0.039291|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-poly|396.035|1.4116|0.122838|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-hybrid|74.098|121.2647|0.198558|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|0.5|forced-noop|24.618|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|auto|2021.660|1.3539|0.021304|15.1102|5.00|5|128|0.859|0.000|0.000|0.000|fft-f32:73.5%, polynomial:24.4%, hybrid-residual:2.2%|
|mixed-realworld|2048|1.0|forced-fft|1214.604|8.7330|0.039291|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-poly|382.726|1.4116|0.122838|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-hybrid|70.576|121.2647|0.198558|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|1.0|forced-noop|24.601|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|auto|2046.706|1.3539|0.021304|15.3293|5.00|5|128|0.859|0.000|0.000|0.000|fft-f32:73.3%, polynomial:24.5%, hybrid-residual:2.2%|
|mixed-realworld|2048|2.0|forced-fft|1545.337|8.7330|0.039291|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-poly|684.371|1.4116|0.122838|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-hybrid|137.003|121.2647|0.198558|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|2.0|forced-noop|43.764|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|auto|2161.087|1.3539|0.021304|16.2644|5.00|5|128|0.859|0.000|0.000|0.000|fft-f32:73.9%, polynomial:23.9%, hybrid-residual:2.2%|
|mixed-realworld|2048|5.0|forced-fft|1128.729|8.7330|0.039291|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-poly|381.612|1.4116|0.122838|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-hybrid|83.554|121.2647|0.198558|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|2048|5.0|forced-noop|27.446|0.9994|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|auto|2298.950|0.9995|0.000000|69.1292|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.3%, polynomial:22.8%, hybrid-residual:1.8%|
|mixed-realworld|8192|0.5|forced-fft|1415.202|8.5641|0.038217|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-poly|486.459|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-hybrid|83.369|174.5590|0.199091|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|0.5|forced-noop|27.085|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|auto|2575.945|0.9995|0.000000|77.7098|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.1%, polynomial:23.0%, hybrid-residual:1.9%|
|mixed-realworld|8192|1.0|forced-fft|1699.685|8.5641|0.038217|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-poly|517.163|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-hybrid|95.835|174.5590|0.199091|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|1.0|forced-noop|29.122|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|auto|2294.105|0.9995|0.000000|69.0077|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:75.0%, polynomial:23.1%, hybrid-residual:1.9%|
|mixed-realworld|8192|2.0|forced-fft|1430.151|8.5641|0.038217|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-poly|462.986|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-hybrid|94.812|174.5590|0.199091|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|2.0|forced-noop|30.152|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|auto|2363.431|0.9995|0.000000|70.3611|5.00|5|32|0.800|0.000|0.000|0.000|fft-f32:74.8%, polynomial:23.4%, hybrid-residual:1.9%|
|mixed-realworld|8192|5.0|forced-fft|1487.231|8.5641|0.038217|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-poly|461.773|0.9995|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-hybrid|81.693|174.5590|0.199091|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|8192|5.0|forced-noop|27.843|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|auto|2788.758|0.9999|0.000000|674.8090|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:77.6%, polynomial:20.6%, hybrid-residual:1.9%|
|mixed-realworld|65536|0.5|forced-fft|1609.544|8.0955|0.037515|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-poly|453.337|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-hybrid|86.036|196.0322|0.199823|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|0.5|forced-noop|21.498|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|auto|2780.853|0.9999|0.000000|671.4189|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:77.0%, polynomial:21.2%, hybrid-residual:1.8%|
|mixed-realworld|65536|1.0|forced-fft|1632.107|8.0955|0.037515|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-poly|445.942|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-hybrid|86.428|196.0322|0.199823|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|1.0|forced-noop|22.735|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|auto|2552.560|0.9999|0.000000|615.3834|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:76.8%, polynomial:21.4%, hybrid-residual:1.8%|
|mixed-realworld|65536|2.0|forced-fft|1669.160|8.0955|0.037515|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-poly|457.318|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-hybrid|85.052|196.0322|0.199823|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|2.0|forced-noop|22.771|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|auto|2583.604|0.9999|0.000000|623.4000|5.00|5|4|0.800|0.000|0.000|0.000|fft-f32:76.8%, polynomial:21.4%, hybrid-residual:1.8%|
|mixed-realworld|65536|5.0|forced-fft|1646.948|8.0955|0.037515|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-poly|475.431|0.9999|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-hybrid|89.637|196.0322|0.199823|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|
|mixed-realworld|65536|5.0|forced-noop|23.710|1.0000|0.000000|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|n/a|

Notes: `attempts_*`, `retries`, `bound_miss_rate`, and `codec_time_share` come from perf-telemetry in auto mode only.
Iteration metrics are currently not emitted by codecs, so per-codec avg/p95 iterations are not yet available.
