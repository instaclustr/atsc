## Version: 0.1 Released: 11/07/2023

Check Github issues for open... issues.

## Description

The objective of this repository is creating a PoC for an end-to-end monitoring system that relies on the properties of the harmonic signals to provide a much greater compression than traditional means.
Altough monitoring signals are not harmonic, they do look similar to a viewer. Given that they look similar, do they exibit the same characteristics? If not, can we aproximate enough that we can benefit
from that without losing information?

Also learning RUST while I go at this.

## Methodology

1. Write to WAV (It's a uncompressed format. Header and binary samples)
2. Compress to FLAC - Compare sizes
3. Serve FLAC file via Prometheus remote READ
4. Write WAV files via Prometheus Write Requests <- I'm here now!
5. Compare results and native prometheus files with FLAC files.

## Results

At this point in time, CPU utilization can be clearly approximated by an harmonic signal, and compressed greatly using traditional signal processing techniques.

## Programs and description

This repository contains one main program and other programs that serve different purposes, some are for just testing, others do some actual work.

The main program is in `main.rs` and implements a lousy prometheus READ and WRITE storage.

The other programs do the following:

1. `monitoring_agents.rs` UPDATE: Use prometheus directly, since remote write is supported. Is a monitoring agent that generates WAV files. Those need to be converted manualy to flac after the program termination. The use of this program is to gather real system data for compression and having something to read from.
2. `improved_flac_server.rs` Reading FLAC files without full decompression, support for seeking and only decompressing part of the file needed. Mostly used to test stuff around the FLAC format.
3. `flac_reader_tester.rs` Compare raw data with the compressed FLAC to make sure there is no information loss.
4. `symphonia_test.rs` Test the reading of FLAC files with the same framework as the main code uses.

## How to make this work?

1. Launch `flac_sysmonitor`. 
2. Run a prometheus and Grafana server. Make sure that prometheus remote is pointing to `flac_sysmonitor`.
3. After everyday (It is default for now) compress the generated data with `sox` (https://github.com/chirlu/sox) to FLAC with the following command: `sox <input>.wav <output>.flac`.

Example:
```
# Remote read and Write
remote_write:
   - url: "http://localhost:9201/api/write"

remote_read:
   - url: "http://localhost:9201/api/read"
     read_recent: true
     name: "flac_server"
```

Make Prometheus server a source of your grafana and check the data.