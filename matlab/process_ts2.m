function [dc, ac, composed, fft_data] = process_ts2(timeseries, w, freq_n, ss)
% ts: timeseries
% w: window size
% freq_n: Number of frequencies to find

% Clear errors from data
nanIDX = find(isnan(timeseries));
while(~isempty(nanIDX))
  timeseries(nanIDX) = timeseries(nanIDX+1);
  nanIDX = find(isnan(timeseries));
end

% Supersample the timeseries
[x, ts] = supersample_signal(timeseries, ss, 3);
tic
% Split the signal in DC and AC parts
%dc = movmean(ts, w);
ac = center(ts);
%ac = ts-dc;
dc = ts-ac;

% Window (Probably not needed)
% hn = hann (w)';

%hold on;plot(dc);plot(ac);

window_n = ceil(length(ts)/w);
data_rebuild = [];
fft_store = [];
fft_data = [];
window_err = [];
dc_store = [];
% Process the whole signal
for i=1:window_n
  window_s = (i-1)*w + 1;
  window_e = i*w;
  if i == window_n
    data_window = ts(window_s:end);
    data_dc = dc(window_s:end);
    data_ac = ac(window_s:end);
  else
    data_window = ts(window_s:window_e);
    %data_dc = movmean(data_window, w/10);
    data_dc = dc(window_s:window_e);
    %data_ac = data_window - data_dc;
    data_ac = ac(window_s:window_e);%.*hn;
  endif
  window_size = length(data_dc);

% Process AC data
  f = fft(data_ac);
  tmp_f = f;
  out_fft = zeros(1, window_size);
  window_freqs = [];
  if freq_n > window_size/2
    freq_n = floor(window_size/2);
  endif
  for i=1:freq_n*2
    [mx,ix] = max(tmp_f);
    window_freqs(i,:) = [real(ix) mx];
    tmp_f(ix) = 0;
    out_fft(ix) = mx;
  end
  % Process DC data
  yi = polyfit(1:window_size,data_dc,1);
  dc_store = [dc_store yi];
  fft_data = [fft_data out_fft];
end
toc
% Decompressing
tic
for j=1:window_n
  window_s = (j-1)*w + 1;
  window_e = j*w;
  if j == window_n
    data_fft = fft_data(window_s:end);
    %original_data = timeseries(window_s:end);
  else
    data_fft = fft_data(window_s:window_e);
    %original_data = timeseries(window_s:window_e);
  endif
  window_size = length(data_fft);
  out_ift = ifft(data_fft);
  % Process DC data
  %yi = polyfit(1:window_size,center(out_ift),1);
  % Rebuild the sinal for the window
  yii = polyval(dc_store((2*j)-1:2*j),1:window_size);
  % Build the dataset for the window
  window_rebuild = real(out_ift)+yii;
  data_rebuild = [data_rebuild window_rebuild];
end
toc

% Calculate the error
pererr = abs(timeseries-data_rebuild(1:ss:end-1))./timeseries*100;
mean(pererr)
figure;
plot(pererr, '+');

composed = data_rebuild;


figure;
subplot(2,2,1);
plot(data_rebuild);
title('Rebuild');
subplot(2,2,2);
plot(ts, 'r');
title('Original');
subplot(2,2,[3,4]);
plot(x,timeseries,'r',data_rebuild,'b');
title('Both');