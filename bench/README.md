# About this crate
This is a separate crate that contains the criterion.rs benchmarks for Randy.
Separated here because it increases the f* out of compile times, and dev-deps arent optional

# What is here

## proc
Benchmarks for various proc_info (top like) methods.

To run set `TPID` var to somthing like `/proc/1234`.
example: 
```shell
TPID="/proc/`pgrep emacs | head -n1`" cargo bench
```
