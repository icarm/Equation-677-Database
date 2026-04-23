use crate::*;

#[derive(Clone)]
struct Class {
    main: TTerm,
    alternatives: Vec<TTerm>,
}

#[derive(Clone)]
struct Ctxt {
    classes: Vec<Class>,
    open: Vec<TTerm>,
    last_checkpoint: usize,
}

pub fn one_orbit2_run() {
    let mut ctxts = vec![new_ctxt()];
    for _ in 0..100 {
        ctxts = ctxts.into_iter().map(branch_step).flatten().collect();
        println!();
        println!();
        println!("states:");
        println!();
        for ctxt in &ctxts {
            dump_ctxt(ctxt);
        }
    }
}

fn next_level(ctxt: &mut Ctxt) -> bool {
    let n = ctxt.last_checkpoint;

    if n == ctxt.classes.len() { return false }

    for x in &ctxt.classes[n..] {
        for y in &ctxt.classes[n..] {
            ctxt.open.push(mk_f(&x.main, &y.main));
        }
    }

    for x in &ctxt.classes[0..n] {
        for y in &ctxt.classes[n..] {
            ctxt.open.push(mk_f(&x.main, &y.main));
            ctxt.open.push(mk_f(&y.main, &x.main));
        }
    }

    ctxt.last_checkpoint = ctxt.classes.len();

    true
}

fn dump_ctxt(ctxt: &Ctxt) {
    println!("-----------");
    for x in &ctxt.classes {
        print!("{} := ", stringify_term(&x.main));
        for (i, alt) in x.alternatives.iter().enumerate() {
            print!("{}", stringify_term(alt));
            if i != x.alternatives.len()-1 {
                print!(" | ");
            }
        }
        println!();
    }
    println!("-----------");
}

fn branch_step(mut ctxt: Ctxt) -> Vec<Ctxt> {
    if ctxt.open.is_empty() {
        if !next_level(&mut ctxt) {
            println!("found:");
            dump_ctxt(&ctxt);
            return Vec::new()
        }
    }

    let mut ctxts = Vec::new();
    let x = ctxt.open.pop().unwrap();

    for i in 0..ctxt.classes.len() {
        let mut ctxt = ctxt.clone();
        ctxt.classes[i].alternatives.push(x.clone());
        ctxts.push(ctxt);
    }

    ctxt.classes.push(Class {
        main: x,
        alternatives: Vec::new(),
    });
    ctxts.push(ctxt);

    ctxts.into_iter().filter(twee_chk).collect()
}

fn new_ctxt() -> Ctxt {
    Ctxt {
        classes: Vec::new(),
        open: vec![mk_X()],
        last_checkpoint: 0,
    }
}

fn twee_chk(ctxt: &Ctxt) -> bool {
    let mut eqs = Vec::new();
    let mut diseqs = Vec::new();

    {
        let x = mk_X();
        let y = mk_Y();

        // eq677
        let rhs = mk_f(&y, &mk_f(&x, &mk_f(&mk_f(&y, &x), &y)));
        eqs.push((x.clone(), rhs));

        // eq677 II
        let yx = mk_f(&y, &x);
        let rhs = mk_f(&yx, &mk_f(&mk_f(&y, &yx), &y));
        eqs.push((x.clone(), rhs));

        // eq255
        let rhs = mk_f(&mk_f(&mk_f(&x, &x), &x), &x);
        diseqs.push((x, rhs));
    }

    // equalities
    for c in &ctxt.classes {
        for alt in &c.alternatives {
            eqs.push((c.main.clone(), alt.clone()));
        }
    }

    // disequalities:
    for (i, a) in ctxt.classes.iter().enumerate() {
        for (j, b) in ctxt.classes.iter().enumerate() {
            if i < j {
                diseqs.push((a.main.clone(), b.main.clone()));
            }
        }
    }

    twee_call(&eqs, &diseqs, 50).is_ok()
}

// ========
// builders:
// ========

use egg::Symbol;

fn mk_f(x: &TTerm, y: &TTerm) -> TTerm {
    let f = Symbol::from("f");
    TTerm::Fun(f, Box::new([x.clone(), y.clone()]))
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

