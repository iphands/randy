# About this crate
This is a separate crate that contains the criterion.rs benchmarks for Randy.
Separated here because it increases the f* out of compile times, and dev-deps arent optional

# Notes
## Repeatability
For repeatability you should consider setting your frequency scaling gov to `performance`
and pinning the run to a particular CPU with `taskset`.
example:
```shell
sudo cpupower frequency-set -g performance

cargo bench --norun     # compile with all CPUS
taskset 0x4 cargo bench # pin bench to CPU 0x4
```

# What is here
## macro
I wrote a wrapper macro that I try to almost fully disable at compile time...
unless the timings feature is on. I want to see if the pass through impl of the macro
causes any noticeable perf diff. Seems like NO :D
example:
```shell
cargo bench --bench macro
```

## proc
Benchmarks for various proc_info (top like) methods.

To run set `TPID` var to something like `/proc/1234`.
example:
```shell
TPID="/proc/`pgrep emacs | head -n1`" cargo bench --bench proc
```
