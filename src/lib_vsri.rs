/// Very Small Rolo Index
/// This is an index made for detection of gaps in continuous data with the same sampling rate.
/// Each continous segment of data will be mapped to a line using the formula y = ax + B plus
/// the number of points in the data series.
/// a - Sampling rate
/// b - Series initial point in time
/// x - sample # in the data file
/// y - time
/// 
/// This way, discovering the segment number is solving the above equation for X if the 
/// time provided is bigger than the initial point.
/// 
/// best case for sample retrival O(1)
/// worst case O(N) (N is the number of segments)
/// Space usage: 5Bytes for 64k samples. 
struct vsri {
    file_name: String,
    index_points: Vec<i32>
}

/// Reads the index from disk
fn load() {}

fn add_index_point() {}

/// Gets the first point in the index
fn min_index() {}

/// Gets the last point in the index
fn max_index() {}

/// Writes the index to disk. Creates a new file or appends to an existing one.
fn flush() {}