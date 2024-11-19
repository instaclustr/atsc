# Demos

## Contents

This folder contain scripts to generate the demo `html` files.

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
