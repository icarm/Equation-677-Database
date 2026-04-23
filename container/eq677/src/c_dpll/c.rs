use crate::c_dpll::*;

// Constraints

pub const C11_SCORE: i32 = 2000;
pub const C12_SCORE: i32 = 3000;
pub const C21_SCORE: i32 = 2000;
pub const C22_SCORE: i32 = 3000;
pub const CHOSEN_SCORE: i32 = 1000;
pub const X0_SCORE: i32 = 1000;

#[derive(Clone, Copy)]
pub enum CXY {
    C11(/*a*/ E),           // a = b*(a*(ba*b))
                            //            x*y
    C12(/*b*/ E),           // a = b*(a*bab)
                            //        x*y

    C21(/*a*/ E),           // a = ba * ((b*ba) * b)
                            //            x*y
    C22(/*a*/ E, /*ba*/ E), // a = ba * (bba * b)
                            //            x  * y
}

#[derive(Clone, Copy)]
pub struct CXZ(pub /*a*/ E, pub /*b*/ E); // z = x * (a * b)

pub fn progress_c(c: CXY, x: E, y: E, e: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    match c {
        CXY::C11(a) => {
            let _ba = x;
            let b = y;
            let bab = e;
            visit_c12(a, b, bab, ctxt)
        }
        CXY::C12(b) => {
            let a = x;
            let bab = y;
            let abab = e;
            prove_triple(b, abab, a, ctxt)
        },
        CXY::C21(a) => {
            let b = x;
            let ba = y;
            let bba = e;
            visit_c22(a, b, ba, bba, ctxt)
        },
        CXY::C22(a, ba) => {
            let bba = x;
            let b = y;
            let bbab = e;
            prove_triple(ba, bbab, a, ctxt)
        },
    }
}

// C1
pub fn visit_c11(a: E, b: E, ba: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let class = &mut ctxt.classes_xy[idx(ba, b, ctxt.n)];
    let v = class.value;
    if v == E::MAX {
        ctxt.trail.push(TrailEvent::PushCXY(ba, b));
        class.cs.push(CXY::C11(a));
        class.score += C11_SCORE;

        Ok(())
    } else {
        let bab = v;
        visit_c12(a, b, bab, ctxt)
    }
}

fn visit_c12(a: E, b: E, bab: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let class_xy = &mut ctxt.classes_xy[idx(a, bab, ctxt.n)];
    let v = class_xy.value;
    if v == E::MAX {
        // a = b*(a*bab)
        let class_xz = &mut ctxt.classes_xz[idx(b, a, ctxt.n)];
        let z = class_xz.value;
        if z != E::MAX {
            return prove_triple(a, bab, z, ctxt);
        }

        ctxt.trail.push(TrailEvent::PushCXY(a, bab));
        class_xy.cs.push(CXY::C12(b));
        class_xy.score += C12_SCORE;

        ctxt.trail.push(TrailEvent::PushCXZ(b, a));
        class_xz.cs.push(CXZ(a, bab));

        Ok(())
    } else {
        let abab = v;
        prove_triple(b, abab, a, ctxt)
    }
}

// C2
pub fn visit_c21(a: E, b: E, ba: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let class = &mut ctxt.classes_xy[idx(b, ba, ctxt.n)];
    let v = class.value;
    if v == E::MAX {
        ctxt.trail.push(TrailEvent::PushCXY(b, ba));
        class.cs.push(CXY::C21(a));
        class.score += C21_SCORE;
        Ok(())
    } else {
        let bba = v;
        visit_c22(a, b, ba, bba, ctxt)
    }
}

fn visit_c22(a: E, b: E, ba: E, bba: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let class_xy = &mut ctxt.classes_xy[idx(bba, b, ctxt.n)];
    let v = class_xy.value;
    if v == E::MAX {
        // a = ba * (bba * b)
        let class_xz = &mut ctxt.classes_xz[idx(ba, a, ctxt.n)];
        let z = class_xz.value;
        if z != E::MAX {
            return prove_triple(bba, b, z, ctxt);
        }

        ctxt.trail.push(TrailEvent::PushCXY(bba, b));
        class_xy.cs.push(CXY::C22(a, ba));
        class_xy.score += C22_SCORE;

        ctxt.trail.push(TrailEvent::PushCXZ(ba, a));
        class_xz.cs.push(CXZ(bba, b));

        Ok(())
    } else {
        let bbab = v;
        prove_triple(ba, bbab, a, ctxt)
    }
}
