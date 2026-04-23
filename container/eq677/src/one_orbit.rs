use crate::*;

use egg::Symbol;

#[derive(Clone)]
struct Ctxt {
    m: MatrixMagma,
    n: usize,
    nonfresh: usize,
}

pub fn one_orbit_run(n: usize) {
    let mut ctxt = Ctxt {
        m: MatrixMagma::undefined(n),
        n,
        nonfresh: 0,
    };

    run(&mut ctxt);
}

fn run(ctxt: &mut Ctxt) {
    let Some((x, y)) = heuristic(&ctxt) else {
        present_model(ctxt.n, "one_orbit", |x, y| ctxt.m.f(x, y));
        return;
    };
    if x >= ctxt.nonfresh { ctxt.nonfresh += 1; }
    if y >= ctxt.nonfresh { ctxt.nonfresh += 1; }

    for z in 0..=ctxt.nonfresh {
        if z >= ctxt.n { break }

        let mut ctxt = ctxt.clone();
        if z >= ctxt.nonfresh { ctxt.nonfresh += 1; }
        ctxt.m.set_f(x, y, z);
        if propagate(&mut ctxt).is_ok() {
            run(&mut ctxt);
        }
    }
}

// What question to ask next?
fn heuristic(ctxt: &Ctxt) -> Option<(usize, usize)> {
    for x in 0..ctxt.n {
        for y in 0..ctxt.n {
            if ctxt.m.f(x, y) == usize::MAX {
                return Some((x, y));
            }
        }
    }
    None
}

fn propagate(ctxt: &mut Ctxt) -> Result<(), ()> {
    let mut eqs = Vec::new();
    let mut diseqs = Vec::new();

    {
        let x = mk_X();
        let y = mk_Y();

        let rhs = mk_f(&y, &mk_f(&x, &mk_f(&mk_f(&y, &x), &y)));
        eqs.push((x.clone(), rhs));

        let yx = mk_f(&y, &x);
        let rhs = mk_f(&yx, &mk_f(&mk_f(&y, &yx), &y));
        eqs.push((x, rhs));
    }

    // equalities

    {
        let x = mk_X();
        let rhs = mk_e(0, &x);
        eqs.push((x, rhs));
    }

    for a in 0..ctxt.n {
        for b in 0..ctxt.n {
            let z = ctxt.m.f(a, b);
            if z != usize::MAX {
                let x = mk_X();
                let lhs = mk_e(a, &x);
                let rhs = mk_e(b, &x);
                let comb = mk_f(&lhs, &rhs);

                let z = mk_e(z, &x);
                eqs.push((comb, z));
            }
        }
    }


    // disequalities:
    for a in 0..ctxt.n {
        for b in 0..ctxt.n {
            if a < b {
                let x = mk_X();
                let lhs = mk_e(a, &x);
                let rhs = mk_e(b, &x);
                diseqs.push((lhs, rhs));
            }
        }
    }

    let facts = twee_call(&eqs, &diseqs, 50)?;

    // TODO: actually propagate
    Ok(())
}


// ========
// builders:
// ========

fn mk_f(x: &TTerm, y: &TTerm) -> TTerm {
    let f = Symbol::from("f");
    TTerm::Fun(f, Box::new([x.clone(), y.clone()]))
}

fn mk_e(i: usize, x: &TTerm) -> TTerm {
    let e = Symbol::from(&format!("e{i}"));
    TTerm::Fun(e, Box::new([x.clone()]))
}

#[allow(non_snake_case)]
fn mk_X() -> TTerm {
    let x = Symbol::from("X");
    TTerm::Var(x)
}

#[allow(non_snake_case)]
fn mk_Y() -> TTerm {
    let y = Symbol::from("Y");
    TTerm::Var(y)
}
