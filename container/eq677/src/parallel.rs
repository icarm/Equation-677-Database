use crate::*;
use rayon::prelude::*;
use std::backtrace::Backtrace;

// This flag allows me to turn off multi-thrading globally.
// Useful for flamegraphs and deterministic debugging.
const PAR: bool = !cfg!(feature = "flamegraph");

pub fn par_for_each<T>(x: &[T], f: impl Fn(&T) + Send + Sync) where T: Send + Sync + 'static {
    if PAR {
        x.par_iter().for_each(f);
    } else {
        x.iter().for_each(f);
    }
}

pub fn into_par_for_each<T>(x: Vec<T>, f: impl Fn(T) + Send + Sync) where T: Send {
    if PAR {
        x.into_par_iter().for_each(f);
    } else {
        x.into_iter().for_each(f);
    }
}

pub fn range_for_each(n: u8, f: impl Fn(u8) + Send + Sync) {
    if PAR {
        (0..n).into_par_iter().for_each(f);
    } else {
        for x in 0..n { f(x); }
    }
}

// This allows panic propagation even through rayon threads.
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("panic: {info}");
        // eprintln!("backtrace:\n{}", Backtrace::force_capture());
        std::process::abort(); // kill immediately
    }));
}
