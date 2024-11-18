# VSRI (Very Small Rolo Index)

1. [What is VSRI](#what-is-vsri)
2. [Characteristics](#characteristics)
3. [API](#api)

## What is VSRI

VSRI is an index made for the time part of the time series.
The idea is metrics will mostly have the same sampling rate throughout their lifetime. A CPU metric sampled at once every 15 seconds, should stay like that.

With this, VSRI tries to map the time into a line. A line can be easily mathematically defined by `y = m*x + B`, and that is what mostly VSRI does. But, something about assumptions, VSRI has the capacity for detection of gaps.

How does VSRI maps time to a line?

```text
m - Sampling rate
b - Series initial point in time in (x,y)
x - sample # in the data file, this is ALWAYS sequential. There are no holes in samples
y - time
```

With VSRI, discovering the segment number is solving the above equation for `X` if the
time provided is bigger than the initial point.

## Characteristics

VSRI has the following characteristics:

- Best case for sample retrieval O(1)
- Worst case O(N) (N is the number of segments)
- Space usage: 5Bytes for 64k samples. Or 30 Bytes for 2^32 Samples

Example of content of an index:

```text
55745
59435
15,0,55745,166
15,166,58505,63
```

## API

WIP
