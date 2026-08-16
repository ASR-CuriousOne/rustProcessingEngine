## Rust CSV Parsing Engine
Tried to maximize csv ingestion and parsing speed, finally ended up with the following

1. **Memory Mapping**:  File is memory-mapped (mmap) and partitioned into equal chunks, using memchr to align thread boundaries to newlines.
2. **Zero Allocation Scanning**: Standard csv crate is bypassed in favor of a custom stack allocated byte scanner that uses memchr for SIMD comma splitting. Can switch to csv crate too if data consists of complex columns (e.g. commas inside quotes)
3. **Fast Float Parsing**: Fast-float crate parses numbers directly from raw &[u8] slices.
 
### Performance Results

Hardware: HP Omen 16 (16 Threads)

| Metric | Result |
| --- | --- |
| **Test File** | `ohlcv/test.csv` (4096 MB) |
| **Concurrency** | 16 Threads |
| **Rows Processed** | 76,820,511 |
| **Matching Rows** | 33,824,057 |
| **Total Time** | 1.16s |
| **Throughput** | 3523.24 MB/s |
