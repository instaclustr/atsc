function ts = remove_nan(timeseries)
% Since octave doesn't have any function that removes NaN from data,
% this function does just that.

nanIDX = find(isnan(timeseries));
while(~isempty(nanIDX))
  timeseries(nanIDX) = timeseries(nanIDX+1);
  nanIDX = find(isnan(timeseries));
end

ts=timeseries;