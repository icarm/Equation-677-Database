use crate::semitinv_dpll::*;

pub fn build_ctxt(r: E) -> Ctxt {
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
        r,
        a: E::MAX,
        b: E::MAX,
        classes_h: std::iter::repeat(class_h)
            .take(r as usize)
            .collect(),
        classes_hinv: std::iter::repeat(class_hinv)
            .take((r+1) as usize)
            .collect(),
        propagate_queue: Vec::new(),
    };
    for i in 0..r {
        ctxt.classes_h[i as usize].score = compute_base_score(i, &ctxt);
    }
    ctxt
}
