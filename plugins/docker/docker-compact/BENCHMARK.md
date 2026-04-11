# docker-compact — Token Saving Benchmark

Run: `sh bench.sh`

| Sample | Level | Input | Output | Saved | % |
|---|---|---|---|---|---|
| docker-images-full | lite | 1,468t | 616t | 852t | 58% |
| docker-images-full | full | 1,468t | 616t | 852t | 58% |
| docker-images-full | ultra | 1,468t | 518t | 950t | **64%** |
| docker-ps-full | lite | 271t | 168t | 103t | 38% |
| docker-ps-full | full | 271t | 168t | 103t | 38% |
| docker-ps-full | ultra | 271t | 41t | 230t | **84%** |
