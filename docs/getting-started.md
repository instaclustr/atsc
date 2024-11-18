# Getting started

How to build and/or run ATSC

## Compiling from source

1. Get [Rust](https://www.rust-lang.org/)
2. Checkout the repository:

    ```bash
    git clone https://github.com/instaclustr/atsc
    cd atsc
    ```

3. Build the project:

   ```bash
   cargo build --release
   ```

4. Get a CSV file with the proper format (or get one from [tests folder](https://github.com/instaclustr/atsc/tree/main/atsc/tests/csv)).

    **Note**: ATSC was originally built with a specific format in mind `WBRO`. For information about this format and how to use it check the [WAVBRRO](https://github.com/instaclustr/atsc/tree/main/wavbrro) library documentation.

5. Run it

    ```bash
    atsc --csv <input-file>
    ```

## Docker container

WIP

## Binary release

1. Download the latest [release](https://github.com/instaclustr/atsc/releases)
2. Get a CSV file with the proper format (or get one from [tests folder](https://github.com/instaclustr/atsc/tree/main/atsc/tests/csv))
3. Run it

```bash
atsc --csv <input-file>
```
