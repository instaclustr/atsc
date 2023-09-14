function pererr = error_calc(s1, s2, sampling)
  % Calculates the error between 2 timeseries

  % Most of the times the size of the signals don't match, matlab makes a fuss
  % about it, so I adjust the size. We can probably ignore this in RUST
  s1_trimmed = s1(1:sampling:end-1);
  % A visualization of the 2 timeseries
  plot(s1_trimmed, 'b', s2, 'r')
  % Calculation of the percentage of error.
  % Point by point we subtract from the original the calculated one
  % make the absolute and divide over 100
  pererr = abs(s2-s1_trimmed)./100;
  % Output the median of the error
  median(pererr)