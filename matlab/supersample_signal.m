function [x, y] = supersample_signal(ts, ts_sampling_rate, polinomial)
% This function adds data samples interpolation the original data samples
% Why? FFTs like more data, so this is a way to add more data to smooth
% out the FFTs
  warning ('off', 'all') ;
  tic
  x = (1:ts_sampling_rate:ts_sampling_rate*length(ts));
  window = polinomial * ts_sampling_rate;
  y = [];
  window_count = 0;
  for i=1:polinomial:length(ts)
    top = i+polinomial;

    if top > length(x)
      top = length(x);
    endif
    if i == top
      break;
    endif
    yi = polyfit(x(i:top),ts(i:top),3);
    yii = polyval(yi,window*window_count+1:window*(window_count+1)+1);
    y = [y(1:end-1) yii];
    window_count = window_count + 1;
  end
  toc
  plot(y, 'r', x,ts, '+');