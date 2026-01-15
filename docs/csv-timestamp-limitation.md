# CSV Timestamp Parsing Limitation

## Current Issue
The CSV parser in `atsc/src/csv.rs` currently hardcodes all timestamps to 0:

```rust
let mut samples: Vec<Sample> = vec![];
for record in rdr.records() {
    let record = record?;
    let sample = Sample {
        timestamp: 0, // TODO: Parse actual timestamp from CSV
        value: record[0].parse::<f64>()?,
    };
    samples.push(sample);
}
```

## Impact
- VSRI compression of CSV timestamps is not fully functional
- All timestamps are stored as sequence of zeros
- VSRI correctly compresses this as a constant segment
- E2E tests document this limitation (see `test_csv_vsri_file_creation`)

## Required Work
1. Parse actual timestamp values from CSV files
2. Support multiple timestamp formats (epoch seconds, ISO 8601, etc.)
3. Handle missing timestamps (generate sequential timestamps?)
4. Update VSRI tests to verify real timestamp sequences

## Related Code
- `atsc/src/csv.rs` - CSV parser implementation
- `atsc/tests/e2e.rs` - Test documenting this limitation
- Issue #2 - "Do not ignore Step size from prometheus request"

## Priority
Medium - CSV compression works for value data, but timestamp preservation would enable better time-series analysis and VSRI functionality verification.
