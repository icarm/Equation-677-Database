use crate::tinv_dpll::*;

pub fn build_ctxt(n: usize) -> Ctxt {
    let class_h = ClassH {
        value: E::MAX,
        cs: SmallVec::new(),
        score: -1,
    };
    let class_hinv = ClassHInv {
        value: E::MAX,
    };
    let mut ctxt = Ctxt {
        trail: Vec::new(),
        n: n as E,
        classes_h: std::iter::repeat(class_h)
            .take(n)
            .collect(),
        classes_hinv: std::iter::repeat(class_hinv)
            .take(n)
            .collect(),
        propagate_queue: Vec::new(),
    };
    for i in 0..ctxt.n {
        ctxt.classes_h[i as usize].score = compute_base_score(i, &ctxt);
    }
    ctxt
}
