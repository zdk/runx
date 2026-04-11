# git-compact — Token Saving Benchmark

Run: `sh bench.sh`

| Sample | Level | Input | Output | Saved | % |
|---|---|---|---|---|---|
| git-diff-full | lite | 2,781t | 1,701t | 1,080t | 38% |
| git-diff-full | full | 2,781t | 1,701t | 1,080t | 38% |
| git-diff-full | ultra | 2,781t | 156t | 2,625t | **94%** |
| git-log-full | lite | 911t | 199t | 712t | 78% |
| git-log-full | full | 911t | 199t | 712t | 78% |
| git-log-full | ultra | 911t | 100t | 811t | **89%** |
| git-show-full | lite | 1,192t | 625t | 567t | 47% |
| git-show-full | full | 1,192t | 625t | 567t | 47% |
| git-show-full | ultra | 1,192t | 123t | 1,069t | **89%** |
| git-status-full | lite | 127t | 5t | 122t | **96%** |
| git-status-full | full | 127t | 5t | 122t | **96%** |
| git-status-full | ultra | 127t | 5t | 122t | **96%** |
