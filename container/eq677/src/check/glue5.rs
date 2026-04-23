use crate::*;
use std::collections::HashSet;

pub fn glue5_chk(m: &MatrixMagma) -> bool {
    if m.n < 5 { return false }
    if !m.is_idempotent() { return false }

    for x in 0..m.n {
        for y in 0..m.n {
            if x == y { continue }
            if !induces5(m, x, y) { return false }
        }
    }
    true
}

pub fn contains5(m: &MatrixMagma) -> bool {
    for x in 0..m.n {
        for y in 0..m.n {
            if induces5(m, x, y) { return true}
        }
    }
    false
}

pub fn get_induced_submagmas(m: &MatrixMagma) -> Vec<MatrixMagma> {
    let mut v = Vec::new();
    for x in 0..m.n {
        for y in 0..m.n {
            if x == y { continue }

            let mut set = HashSet::new();
            set.insert(x);
            set.insert(y);
            complete(m, &mut set);

            if set.len() == m.n { continue }

            let sub = GenericMagma {
                elems: set.iter().copied().collect(),
                f_def: |x, y| m.f(x, y),
            }.to_matrix().canonicalize2();

            if !v.contains(&sub) {
                v.push(sub);
            }
        }
    }

    v
}

pub fn dump_induced_submagmas() {
    for (name, m) in db() {
        print!("{name}: ");
        let comps = get_induced_submagmas(&m);
        for mm in comps {
            print!("{}, ", db_intern(&mm).0);
            // present_model(mm.n, "", |x, y| mm.f(x, y));
        }
        println!();
    }
}


fn induces5(m: &MatrixMagma, x: usize, y: usize) -> bool {
    let mut set = HashSet::new();
    set.insert(x);
    set.insert(y);
    complete(m, &mut set);

    // no need to check what magma it is.
    // If it's an induced submagma of size 5, it has to be 5/0.
    set.len() == 5
}

fn complete(m: &MatrixMagma, set: &mut HashSet<usize>) {
    loop {
        let n = set.len();
        let s = set.clone();
        for x in s.iter() {
            for y in s.iter() {
                set.insert(m.f(*x, *y));
                if set.len() == m.n { return; }
            }
        }

        if n == set.len() { return }
    }
}
