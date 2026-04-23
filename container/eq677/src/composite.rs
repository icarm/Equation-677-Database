use crate::*;
use std::collections::HashSet;

use egg::*;

define_language! {
    enum MagmaLang {
        "f" = F([Id; 2]),
        E(usize),
    }
}

type EGraph = egg::EGraph<MagmaLang, ()>;

pub fn decompose(m: &MatrixMagma) -> HashSet<MatrixMagma> {
    let mut out = HashSet::new();

    let eg = eggify(m);
    for x in 0..m.n {
        for y in 0..m.n {
            if x == y { continue }
            let mut eg = eg.clone();
            let x_ = eg.lookup(MagmaLang::E(x)).unwrap();
            let y_ = eg.lookup(MagmaLang::E(y)).unwrap();
            eg.union(x_, y_);
            eg.rebuild();
            if eg.classes().len() > 1 {
                let m = de_eggify(&eg);
                let m = m.canonicalize();
                if !out.contains(&m) {
                    out.insert(m);
                }
            }
        }
    }
    out
}

pub fn eggify(m: &MatrixMagma) -> EGraph {
    let mut eg = EGraph::new(());
    for x in 0..m.n {
        eg.add(MagmaLang::E(x));
    }
    for x in 0..m.n {
        for y in 0..m.n {
            let x_ = eg.lookup(MagmaLang::E(x)).unwrap();
            let y_ = eg.lookup(MagmaLang::E(y)).unwrap();
            let fxy_ = eg.add(MagmaLang::F([x_, y_]));
            let z_ = eg.lookup(MagmaLang::E(m.f(x, y))).unwrap();
            eg.union(z_, fxy_);
        }
    }
    eg.rebuild();
    assert_eq!(eg.classes().len(), m.n);

    eg
}

fn de_eggify(eg: &EGraph) -> MatrixMagma {
    let g: GeneralMagma<Id, _> = GeneralMagma {
        elems: eg.classes().map(|x| x.id).collect(),
        f_def: |x, y| eg.lookup(MagmaLang::F([x, y])).unwrap(),
    };
    g.to_matrix()
}

struct GeneralMagma<E, F> {
    elems: Vec<E>,
    f_def: F,
}

impl<E: PartialEq + Clone, F: Fn(E, E) -> E> GeneralMagma<E, F> {
    fn to_matrix(&self) -> MatrixMagma {
        MatrixMagma::by_fn(self.elems.len(), |x, y| {
            let e = (self.f_def)(self.elems[x].clone(), self.elems[y].clone());
            self.elems.iter().position(|x| x == &e).unwrap()
        })
    }
}
