# BRRO - A Novel Approach to Monitoring TimeSeries Compression

(BR from Bromhead RO from Rolo, BRRO sounds cool (?). Ok, ok, we can get a new name!)

## Abstract

Time series data is a collection of observations (behavior) for a single subject (entity) at different time intervals (generally equally spaced as in the case of metrics, or unequally spaced as in the case of events).
For example: Max Temperature, Humidity and Wind (all three behaviors) in New York City (single entity) collected on First day of every year (multiple intervals of time)
The relevance of time as an axis makes time series data distinct from other types of data. [1]

In this document we propose a novel approach to timeseries compression, instead of relying on compression based on the properties of the samples [2] [3] or in the small segments of the sequence [4].
We propose an approach each timeseries as a digital signal and apply a set of techniques that already exist in other domains, namely in Audio compression.

One important factor in our approach is that although we rely mostly on Function Approximation (FA), and that is frequently used in Timeseries as a lossy compression, we approach this problem from a lossless perspective. 

> INTERNAL COMMENT: Lossy might be very interesting too!

## Prior art and State of the art

Prior art references:
1. Can't find anything that matches this space

The state of the art in Timeseries compression can be found here [5] and here [6].

## Introduction

Computer systems monitoring, not general timeseries monitoring, is a process that is normally
characterized by very low frequency sampling (0.05Hz to 1~2Hz) of processes. Processes like database processes, Operating System general health, etc generate a lot of signals [10]. 

> INTERNAL COMMENT: Does very low frequency creates the problem for FFTs, etc or is the tools that aren't expecting such low frequencies?

Another aspect is that in most cases, the signals don't exhibits any periodicity or any harmonic components. As such, they are treated mostly as isolated samples or a short sequence of samples (eg. XOR compression, delta-delta encoding, etc).

By looking at the signals as one, we can apply a series of techniques, that themselves are not novel, but are novel in this domain. For example, by leveraging some audio packing and processing we would benefit not only from the compression provided by those, but also by all the streaming functionality that exist around those formats.

Data compression is not only relevant for at-rest (stored in an hard drive) but also for in-transit. With the advent of cloud, monitoring systems that that is sent to external systems bring egress costs with it.
Also, stored data needs to be uncompressed and processed before sent to the wire. By using audio packing techniques we can benefit from the whole ecosystem of software (servers, clients) and hardware (even at mobile devices) that know how to transport and encode/decode such information.

We not only get the savings at-rest also in-transit and in compute processing since we can leverage the client side devices [9] to do that in a very efficient way (eg. parallelized: [7] using a System-on-a-Chip [8]).

## Background

Why are we doing this:

- Storage costs are high
- Egress costs are high
- Given the amount of data, techniques like averaging are used to drop points. This reduces information available.
- Techniques like the above consume a lot of Compute to walk over the points.

## BRRO

What is BRRO really doing:

### Write Path

1. Identifying the type of metric,
2. Pre-processing the signal to apply initial signal reduction techniques (eg: XOR Compression),
3. Packing the signal in an non-compressed format (eg. WAV format), this also includes things like:
    1. Doppler Shift (is it really a Doppler shift? Sounds fancier this way) the signal,
    2. Split integer and float types in types and sizes compatible with audio sampling,
    3. Identifying and exploit local opportunities to mimic audio signals
4. Generate an index to access the samples with precision (eg. Avoid full file seeking), 
5. Once a decent number of samples, apply audio compatible **open source licensed**, **non-patented** compression techniques. This might include:
    1. Channel splitting. By dividing data into several channels we can use existing channel correlation techniques to reduce the channels sizes,
    2. Blocking, divide the signal samples in blocks,
    3. Apply a modeling technique to each block that best suits the signal, eg: LPC, Polynomial Prediction, Fast Fourier Transforms, Wavelets.
6. Generate the final compressed file

While from point 4 onwards it is very much a normal audio compression process with fairly well known and document processes, steps 1. to 4. are unique to BRRO and allows us to follow up with step 5 and benefit from all the existing processes.

TODO: DESCRIBE IN DETAIL 1 to 4

> **INTERNAL COMMENT**: 5.1 is very much unique to BRRO since we search for the best way to benefit from channel correlation to improve compression vs just putting a signal into a channel because it belongs there.

### Read Path

The novel part on reading, is using the indexing. Since we don't store timestamp information on the file itself, we developed a unique indexing to allow us to stream to the part of the file that is relevant and only decompressing what is relevant.

While Streaming by blocks and decompressing by blocks is standard is multimedia files, is not known to be applied to the monitoring space. At least in the way we approach. 

> **INTERNAL COMMENT**: Databases (Filesystems?) load partially files and decompress by chunks, which *could* fit the same description. But... the concept of streaming a metric as one streams a multimedia file and benefit from the existing support for multimedia, applied at a metric/timeseries is not documented.

The read path looks like this:

1. Identifying the metric to be queried,
2. Locate the file(s) and the corresponding index,
3. Using the index locate the part of the file that we need to obtain
4. Stream the blocks over to the client
5. Decompression and extracting of the blocks by the client
    1. With the information from the index after the decompression the samples get the timestamp attached

> **INTERNAL COMMENT**: The current implementation doesn't stream, just locates and decompress locally the blocks and sends the metrics.

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
7. https://ieeexplore.ieee.org/document/8672328
8. https://www.microsemi.com/document-portal/doc_view/129825-ac376-smartfusion-csoc-implementation-of-flac-player-using-hardware-and-software-partitioning-app-note
9. https://en.wikipedia.org/wiki/List_of_hardware_and_software_that_supports_FLAC
10. https://cassandra.apache.org/doc/latest/cassandra/operating/metrics.html