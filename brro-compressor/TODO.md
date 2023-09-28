# Compressor TODO and Discussions

## How the compressor selects the compressor?

- Do we give the user the change of selecting it?
- Configuration needs to be minimal, the end user is probably a metric server and/or a database

## How do we slice the input data?

- [Streaming Reader and Writer] Size? Samples?
- Use io_uring (https://github.com/tokio-rs/tokio-uring/tree/master)

## Should we allow a plugable architecture?

- Allows to use to get external ideas
- Brings another extra layer of complexity of the code
- ~~[Carlos] Check with the shotover team how complex is to have a plugable compressor thingy~~ - It is complex, future developments.

## What to do with VSRI?

- Should we push those into the compressor or leave it outside?

## Optimization

- Take optimization out of the compressors, move it before the compressors are used (Currently it is done by each compressor).
- Optimization should `hint` the best compressor for the job
- Best idea I have is to do an histogram and select/`hint` based on that
