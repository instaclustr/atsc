/// All the utils/code related the to file management
/// 
/// Because I was trying to fix stuff within FLaC, and it needs to be sorted way before.
/// 
/// For a READ request that needs data for MetricX from Ta to Tn this would do the following:
/// 1. Do we have metricX? -> No, stop.
/// 2. Which file has Ta, and which has Tb?
///     2.1 Select them to read
/// 3. Do holes exist in between?
///     3.1 If yes, where?
///     3.2 What is the cost of having a timestamp in a separated channel? <- Probably going this way! Have a "tick counter" based on the data timestamp.
/// 
/// Suggested internal Data Structure of the WAV file
/// 
/// +--------------------------------------------------------------------------------------------------------+
/// | HEADER | i16 @Chan1 | i16 @Chan2 | i16 @Chan3 | i16 @Chan4 | tick @Chan5 | i16 @Chan1 | i16 @Chan2 |...|
/// +--------------------------------------------------------------------------------------------------------+
///                 
///  Prometheus Point: f64 split into 4x i16 (channel 1 to 4) Timestamp: Tick into Channel 5
/// 

