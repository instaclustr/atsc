# BRRO - A Novel Approach to Monitoring TimeSeries Compression
(BR from Bromhead RO from Rolo, BRRO sounds cool (?). Ok, ok, we can get a new name!)

## Abstract

Time series data is a collection of observations (behavior) for a single subject (entity) at different time intervals (generally equally spaced as in the case of metrics, or unequally spaced as in the case of events).
For example: Max Temperature, Humidity and Wind (all three behaviors) in New York City (single entity) collected on First day of every year (multiple intervals of time)
The relevance of time as an axis makes time series data distinct from other types of data. [1]

In this document we propose a novel approach to timeseries compression, instead of relying on compression based on the properties of the samples [2] [3] or in the small segments of the sequence [4].
We propose an approach each timeseries as a digital signal and apply a set of techniques that already exist in other domains, namely in Audio compression.

One important factor in our approach is that although we rely mostly on Function Approximation (FA), and that is frequently used in Timeseries as a lossy compression, we approach this problem from a lossless prespective. (INTERNAL COMMENT: Lossy might be very interesting too!)

## State of the art

The state of the art in Timeseries compression can be found here [5] and here [6].

## Introduction

Computer systems monitoring, not general timeseries monitoring, is a process that is normally
characterized by very low frequency sampling of processes (1/20Hz to 1~2Hz). 
(INTERNAL COMMENT: Does very low frequency creates the problem for FFTs, etc or is the tools that aren't expecting such low frequencies?)
Another aspect is that in most cases, the signals don't exibith any peridicity or any harmonic components. As such, they are treated mostly as isolated samples or a short sequence of samples (delta-delta encoding).


## Background

Why are we doing this

## BRRO

What is BRRO really doing

### VSRI (Very Small Rolo Index)

This is an index technique used to address a sequential data file.

## Results

### Testing methodology

The way we test is by setting our `BRRO-server` as a backend (both read and write) for a prometheus instance. Prometheus is connected to a Instaclustr 3 node cassandra cluster
with prometheus endpoint enabled.
After the test runs for a given period of time (indicated in the results), we compare the space usage of Prometheus data directory vs the space usage of `BRRO-server` data directory.
A note on the result of the *BRRO single*. *BRRO single* is the expected output on a server that as run for a very long time. No headers, and a lot of data that can be efficiently compressed. It is not a realistic measure, but a *best case* for the current test.
All the data from every file is pushed into a single file that is then compressed. 

### BRRO-server 0.1.1 - 12/07/2023
- Signal Optimizations: None
- Metric Count: 565
- Running Time: ~14h
- Prometheus data size: 7.8 MB
- BRRO data size: 4.6 MB (69% less)
- BRRO single: 2.9 MB (268% less)

## Conclusion

BRRO is the best that you can ever have!


## References
1. https://www.influxdata.com/what-is-time-series-data/
2. Gorilla http://www.vldb.org/pvldb/vol8/p1816-teller.pdf
3. https://faun.pub/victoriametrics-achieving-better-compression-for-time-series-data-than-gorilla-317bc1f95932
4. https://www.timescale.com/blog/time-series-compression-algorithms-explained/
5. https://dzone.com/articles/time-series-compression-algorithms-and-their-appli#:~:text=Time%20series%20compression%20algorithms%20take%20advantage%20of%20specific,functions%20or%20predicting%20them%20through%20neural%20network%20models.
6. https://arxiv.org/pdf/2101.08784v1.pdf