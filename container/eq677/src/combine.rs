use crate::*;


fn good(m: &MatrixMagma) -> bool {
    if !m.is_idempotent() { return false }
    if m.n < 2 { return false }
    if m.n > 20 { return false }

    for x in 0..m.n {
        for y in 0..m.n {
            if x == y { continue }
            let mut eq = false;
            let f = |x, y| {
                if x == y { eq = true; }
                m.f(x, y)
            };
            assert!(x == m.f(y, m.f(x, m.f(m.f(y, x), y))));
            if eq { return false }
        }
    }
    true
}

fn goodies() -> Vec<MatrixMagma> {
    let mut goodies = Vec::new();
    for (name, m) in db() {
        if good(&m) {
            goodies.push(m);
        }
    }
    goodies
}

pub fn combine_search() {
    let goodies = goodies();
    for (_, a) in db() {
        if a.n < 2 { continue }
        if a.n > 20 { continue }
        for (_, b) in db() {
            if a.n != b.n { continue }

            for g in &goodies {
                let a = a.shuffle();
                let b = b.shuffle();
                let g = g.shuffle();
                let m = GenericMagma {
                    elems: itertools::iproduct!(0..g.n, 0..a.n).collect(),
                    f_def: |x: (usize, usize), y: (usize, usize)| {
                        let z0 = g.f(x.0, y.0);
                        let z1 = 
                            if x.0 == y.0 { a.f(x.1, y.1) }
                            else { b.f(x.1, y.1) };
                        (z0, z1)
                    },
                }.to_matrix();
                if m.is677() {
                    dbg!(a.n);
                    dbg!(g.n);
                    present_model(m.n, "k", |x, y| m.f(x, y));
                }
            }
        }
    }
}

pub fn duo_combine_search() {
    let goodies = goodies();

    for a0 in &goodies  {
        for b0 in &goodies {
            if a0.n != b0.n { continue }

            for a1 in &goodies  {
                for b1 in &goodies {
                    if a1.n != b1.n { continue }

                    let m = GenericMagma {
                        elems: itertools::iproduct!(0..a0.n, 0..a1.n).collect(),
                        f_def: |x: (usize, usize), y: (usize, usize)| {
                            let z0 = if x.1 == y.1 { a0.f(x.0, y.0) }
                                     else { b0.f(x.0, y.0) };
                            let z1 = if x.0 == y.0 { a1.f(x.1, y.1) }
                                     else { b1.f(x.1, y.1) };
                            (z0, z1)
                        },
                    }.to_matrix();
                    if m.is677() {
                        present_model(m.n, "k", |x, y| m.f(x, y));
                    }
                }
            }
        }
    }
}
