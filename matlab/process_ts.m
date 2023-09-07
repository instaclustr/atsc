function [dc, ac, composed, fft_data] = process_ts(ts, w, freq_n, n_hold)
% ts: timeseries
% w: window size
% freq_n: Number of frequencies to find
tic
if nargin<4
  n_hold = 0;
endif

nanIDX = find(isnan(ts));
while(~isempty(nanIDX))
  ts(nanIDX) = ts(nanIDX+1);
  nanIDX = find(isnan(ts));
end

% Split the signal in DC and AC parts
%dc = movmean(ts, w);
ac = center(ts);
%ac = ts-dc;
dc = ts-ac;

%hold on;plot(dc);plot(ac);

window_n = ceil(length(ts)/w);
data_rebuild = [];
fft_store = [];
fft_data = [];
window_err = [];
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
    data_ac = ac(window_s:window_e);
  endif
  window_size = length(data_dc);

  % Process AC data
  if isempty(fft_store)
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
    fft_data = [fft_data out_fft];
    %disp("Window Frequencies: ")
    %disp(sort(window_freqs))
    out_ift = ifft(out_fft);
    if n_hold ~= 0
      fft_store = out_ift;
    endif
  elseif n_hold ~= 0
    out_ift = fft_store;
    fft_store = [];
  endif

  % Process DC data
  yi = polyfit(1:window_size,data_dc,1);
  %disp("DC points: ")
  %disp(yi)
  % Rebuild the sinal for the window
  yii = polyval(yi,1:window_size);
  % Build the dataset for the window
  window_rebuild = real(out_ift)+yii;

  % Calculate the error
  pererr = abs(data_window-window_rebuild)./data_window*100;
  mean(pererr)
  window_err = [window_err pererr];
  data_rebuild = [data_rebuild window_rebuild];


  %plot(abs(out_fft))
end
toc
composed = data_rebuild;
nnz(fft_data)
figure;
plot(window_err);

figure;
subplot(2,2,1);
plot(data_rebuild);
title('Rebuild');
subplot(2,2,2);
plot(ts, 'r');
title('Original');
subplot(2,2,[3,4]);
plot(ts,'r',data_rebuild,'b');
title('Both');


%{
wdw = ac(1:w);
f = fft(wdw);

% Create a output array
tmp_f = f;
out_fft = zeros(1, w);

% Zero out the frequency just found and around it
for i=1:freq_n*2
  [mx,ix] = max(tmp_f);
  tmp_f(ix) = 0;
  out_fft(ix) = mx;
end

out_ift = ifft(out_fft);
ift = ifft(f);

% DC component approximation
yi = polyfit(1:w,dc(1:w),1);
% Lets see the aproximattion
yii = polyval(yi,1:w);
rebuilt = real(out_ift)+yii;
x = rebuilt;

hold on;
%plot(abs(out_fft))
%plot(wdw)
%plot(real(ift))
plot(rebuilt)
plot(ts(1:w))
%}