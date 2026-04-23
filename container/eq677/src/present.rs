use crate::*;
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref PRINT_MUTEX: Mutex<()> = Mutex::new(());
    pub static ref PRESENT_DB: Mutex<HashSet<M>> = Mutex::new(HashSet::new());
}

const CHECK_COMPOSITE: bool = false;
const SHOW_AUTOM: bool = false;

pub fn get_present_db() -> Vec<M> {
    let handle = PRESENT_DB.lock().unwrap();
    let mut v: Vec<_> = (*handle).iter().cloned().collect();
    drop(handle);
    v.sort();
    v
}

pub fn dump_present_db() {
    println!("present db:");
    for name in get_present_db() {
        print!("M({}, {}), ", name.0, name.1);
    }
    println!();
}

pub fn present_db_contains(name: M) -> bool {
    let handle = PRESENT_DB.lock().unwrap();
    handle.contains(&name)
}

pub fn present_db_add(name: M) -> bool {
    let mut handle = PRESENT_DB.lock().unwrap();
    handle.insert(name)
}

pub fn present_model(n: usize, finder: &str, f: impl Fn(usize, usize) -> usize) {
    let magma = MatrixMagma::by_fn(n, f);

    let mut globally_new_str = "";
    let mut name_str = String::new();
    if n <= 100 {
        let (name, globally_new) = db_intern(&magma);
        if !present_db_add(name) { return; }
        if globally_new {
            globally_new_str = "New ";
        }
        name_str = format!("\"{name}\" ");
    }

    let mut print_handle = PRINT_MUTEX.lock().unwrap();

    let mut colon_str = if n <= 100 { ":" } else { "" };
    let mut infos1_str = if magma.is_right_cancellative() { "" }
        else { "[non-right-cancellative] " };
    let mut infos2_str = if magma.is_idempotent() { "" }
        else { "[non-idempotent] " };

    println!("{globally_new_str}Model {name_str}{infos1_str}{infos2_str}found by {finder}{colon_str}");
    if n <= 100 {
        magma.canonicalize2().dump();
    }

    if CHECK_COMPOSITE {
        let ms = decompose(&magma);
        if ms.len() > 0 {
            println!("decomposable into:");
            for m in ms {
                println!("---");
                m.cycle_dump();
            }
            println!("---");
        }
    }
    if SHOW_AUTOM && magma.n > 1 && magma.n < 25 {
        magma.autom_dump();
        println!();
    }

    conj(&magma);
}
