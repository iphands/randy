#[cfg(feature = "timings")]
#[macro_export]
macro_rules! timings {
    ($name:expr, $func:tt $(, $optional:expr)*) => {
        {
            use std::time::{Instant};
            let now = Instant::now();
            let ret = $func($($optional, )*);
            let elapsed = now.elapsed();
            let tabs = if $name.len() < 7 { "\t\t" } else { "\t" };
            println!("{}:{}millis: {}\tnanos: {}", $name, tabs, elapsed.as_millis(), elapsed.as_nanos());
            ret
        }
    };
}

#[cfg(not(feature = "timings"))]
#[macro_export]
macro_rules! timings {
    ($name:expr, $func:tt $(, $optional:expr)*) => {
        $func($($optional, )*)
    };
}

#[macro_export]
macro_rules! split_to_strs {
    ($str:expr, $delimiter:literal) => {
        $str.split($delimiter).collect::<Vec<&str>>()
    };
}

#[macro_export]
macro_rules! split_spc_to_strs {
    ($str:expr) => {
        $str.split_ascii_whitespace().collect::<Vec<&str>>()
    };
}
