# About this crate
This is a separate crate that contains the criterion.rs benchmarks for Randy.
Separated here because it increases the f* out of compile times, and dev-deps arent optional

# Notes
## Repeatability
For repeatability you should consider setting your frequency scaling gov to `performance`
and pinning the run to a particular CPU with `taskset`.
example:
```shell
# cpupower frequency-set -g performance
# exit
$ cargo bench --norun     # compile with all CPUS
$ taskset 0x4 cargo bench # pin bench to CPU 0x4
```

# What is here
## proc
Benchmarks for various proc_info (top like) methods.

To run set `TPID` var to somthing like `/proc/1234`.
example:
```shell
$ TPID="/proc/`pgrep emacs | head -n1`" cargo bench --bench proc
```
