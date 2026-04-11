# ls-compact — Token Saving Benchmark

Run: `sh bench.sh`

| Sample | Level | Input | Output | Saved | % |
|---|---|---|---|---|---|
| ls-output-full | lite | 169t | 167t | 2t | 1% |
| ls-output-full | full | 169t | 167t | 2t | 1% |
| ls-output-full | ultra | 169t | 23t | 146t | **86%** |
