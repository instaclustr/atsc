# Demos

## What are demos?

ATSC is a lossy compressor tool, as such, the output time series will not match the input one.

With this in mind it could be important to visualize the difference between output and input and verify if the error introduced is within the expectations.

So demos were created with the intent of having a quick way to display input vs output for a provided metric. The demo scripts will run the ATSC compressor with 2 different error levels and with all the available compression options.

The output HTML files (for error levels 1% and 3%) can then be used to visualize and compare the different results with the input file and evaluate the result of ATSC compression.

Three demo output files are provided so that comparison can be made without needing to run the compressor even once if a quick evaluation is needed. Or if you are curious what is this all about!

## Contents

This folder contains scripts to generate the demo `html` files.

The demo scripts generate 2 comparison files. One for all compressors (`FFT`, `IDW` and `Polynomial`) with an error of 1% and another with a 3% error.

In this folder there are 3 comparisons from some of the available uncompressed files in [tests folder](https://github.com/instaclustr/atsc/tree/v0.7/atsc/tests).

The files are the following:

* IOWait metrics with 1% and 3% error from a CSV file
  * comparison-error-1-csv-iowait.html
  * comparison-error-3-csv-iowait.html
* Java Heap Usage metrics with 1% and 3% error from a wbro file
  * comparison-error-1-heap.html
  * comparison-error-3-heap.html
* OS Memory Usage metrics with 1% and 3% error from a wbro file
  * comparison-error-1-memory.html
  * comparison-error-3-memory.html

## Create your own demo files

1. change into the current directory:

    ```bash
        cd atsc/demo
    ```

2. Execute the Demo. **Note**: If using a `wbro` file, run `run_demo.sh`, if using a `csv` file run `run_demo_csv.sh`

    ```bash
        ./run_demo_csv.sh INPUT_FILE
    ```
