use crate::*;

use egg::*;

use std::collections::HashSet;

const FILTER_THRESHOLD: usize = 5;
const PHI: bool = false;
const G: bool = false;
const D: bool = true;

type EGraph = egg::EGraph<MagmaLang, ()>;
type RecExpr = egg::RecExpr<MagmaLang>;

define_language! {
    enum MagmaLang {
        "f" = F([Id; 2]),
        "g" = G([Id; 2]),
        "d" = D(Id),
        "phi" = Phi(Id),
        "invphi" = InvPhi(Id),
        E(usize),
        "X" = X,
        "Y" = Y,
    }
}

pub fn db_analyze() {
    for (name, x) in db() {
        println!("{name}:");
        analyze(&x, 20);
        println!();
    }
}

fn flip_re(t: &mut RecExpr) {
    for i in 0..t.len() {
        let i = i.into();

        match t[i] {
            MagmaLang::X => t[i] = MagmaLang::Y,
            MagmaLang::Y => t[i] = MagmaLang::X,
            _ => {},
        }
    }
}

fn normalize_equation(lhs: &RecExpr, rhs: &RecExpr) -> (RecExpr, RecExpr) {
    let mut lhs = lhs.clone();
    let mut rhs = rhs.clone();

    let s = format!("{lhs} {rhs}");
    let should_flip = match (s.find("X"), s.find("Y")) {
        (Some(i), Some(j)) => i > j,
        (_, Some(_)) => true,
        _ => false,
    };
    if should_flip {
        flip_re(&mut lhs);
        flip_re(&mut rhs);
    }
    (lhs, rhs)
}

fn filter_graph(eg: EGraph) -> EGraph {
    let ex = Extractor::new(&eg, MySize);

    let mut new = EGraph::new(());
    for c in eg.classes() {
        let (cost, term) = ex.find_best(c.id);
        if cost > FILTER_THRESHOLD { continue }
        let new_id = new.add_expr(&term);

        for n in &c.nodes {
            let rhs = n.join_recexprs(|x| ex.find_best(x).1);
            if MySize.cost_rec(&rhs) > FILTER_THRESHOLD { continue }
            let r = new.add_expr(&rhs);
            new.union(r, new_id);
        }
    }
    new
}

pub fn analyze(m: &MatrixMagma, count: usize) {
    if m.n < 2 { return }

    let mut eg = eggify(m);
    let mut out = None;

    // add variables.
    eg.add(MagmaLang::X);
    eg.add(MagmaLang::Y);

    for x in 0..m.n {
        for y in 0..m.n {
            let mut eg = eg.clone();

            let x_elem = eg.lookup(MagmaLang::E(x)).unwrap();
            let x_var = eg.lookup(MagmaLang::X).unwrap();
            eg.union(x_elem, x_var);

            let y_elem = eg.lookup(MagmaLang::E(y)).unwrap();
            let y_var = eg.lookup(MagmaLang::Y).unwrap();
            eg.union(y_elem, y_var);

            match out {
                None => out = Some(eg),
                Some(o) => out = Some(filter_graph(eg.egraph_intersect(&o, ()))),
            }
        }
    }

    let eg = out.unwrap();
    let ex = Extractor::new(&eg, MySize);

    if PHI {
        let x = eg.lookup(MagmaLang::X).unwrap();
        if let Some(phi_x) = eg.lookup(MagmaLang::Phi(x)) {
            let phi_def = ex.find_best(phi_x).1;
            println!("phi(X) = {phi_def}");
        }
    }

    if PHI {
        let x = eg.lookup(MagmaLang::X).unwrap();
        if let Some(phi_x) = eg.lookup(MagmaLang::Phi(x)) {
            let phi_def = ex.find_best(phi_x).1;
            println!("phi(X) = {phi_def}");
        }
    }

    let mut equations = HashSet::new();
    for c in eg.classes() {
        let lhs = ex.find_best(c.id).1;
        for n in &c.nodes {
            let rhs = n.join_recexprs(|x| ex.find_best(x).1);
            if lhs.to_string() == rhs.to_string() { continue }
            let (lhs, rhs) = normalize_equation(&lhs, &rhs);

            if !format!("{lhs} {rhs}").contains('X') { continue }

            equations.insert((lhs, rhs));
        }
    }
    let mut equations: Vec<_> = equations.into_iter().collect();
    equations.sort_by_cached_key(|(i, j)| MySize.cost_rec(i) + MySize.cost_rec(j));
    equations.truncate(count);
    for (lhs, rhs) in equations {
        println!("{lhs} = {rhs}");
    }
}

fn eggify(m: &MatrixMagma) -> EGraph {
    let mut eg = EGraph::new(());

    // add elements.
    for x in 0..m.n {
        eg.add(MagmaLang::E(x));
    }

    // add f.
    for x in 0..m.n {
        for y in 0..m.n {
            let x_ = eg.lookup(MagmaLang::E(x)).unwrap();
            let y_ = eg.lookup(MagmaLang::E(y)).unwrap();

            let fxy_ = eg.add(MagmaLang::F([x_, y_]));
            let z_ = eg.lookup(MagmaLang::E(m.f(x, y))).unwrap();
            eg.union(z_, fxy_);
        }
    }

    // add g.
    if G {
        for x in 0..m.n {
            for y in 0..m.n {
                let x_ = eg.lookup(MagmaLang::E(x)).unwrap();
                let y_ = eg.lookup(MagmaLang::E(y)).unwrap();

                let z_ = eg.lookup(MagmaLang::E(m.f(x, y))).unwrap();
                let gxz_ = eg.add(MagmaLang::G([x_, z_]));
                eg.union(y_, gxz_);
            }
        }
    }

    // add phi.
    if PHI {
        for x in 0..m.n {
            let xx = eg.lookup(MagmaLang::E(x)).unwrap();
            let yy = eg.lookup(MagmaLang::E((x+1)%m.n)).unwrap();

            let phi_xx = eg.add(MagmaLang::Phi(xx));
            eg.union(yy, phi_xx);

            let invphi_yy = eg.add(MagmaLang::InvPhi(yy));
            eg.union(xx, invphi_yy);
        }
    }

    // add d with d(x) = f(f(x,x),x).
    if D {
        for x in 0..m.n {
            let x_ = eg.lookup(MagmaLang::E(x)).unwrap();

            let d = m.f(m.f(x, x), x);
            let d_ = eg.lookup(MagmaLang::E(d)).unwrap();
            let d2 = eg.add(MagmaLang::D(x_));

            eg.union(d_, d2);
        }
    }

    eg.rebuild();
    assert_eq!(eg.classes().len(), m.n);

    eg
}

struct MySize;

impl CostFunction<MagmaLang> for MySize {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &MagmaLang, mut costs: C) -> usize where C: FnMut(Id) -> usize {
        match *enode {
            MagmaLang::F([a, b]) => costs(a) + costs(b),

            MagmaLang::G([a, b]) => costs(a) + costs(b) + 3, // often cancels out with f in useless ways.

            MagmaLang::D(a) => costs(a) + 1,

            MagmaLang::Phi(a) => costs(a) + 1,
            MagmaLang::InvPhi(a) => costs(a) + 4, // often cancels out with phi in useless ways.

            MagmaLang::E(_) => 3,

            MagmaLang::X => 1,
            MagmaLang::Y => 1,
        }
    }
}
