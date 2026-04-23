use crate::tinv_dpll::*;

// Constraints

pub const C11_SCORE: i32 = 2000;
pub const C12_SCORE: i32 = 3000;
pub const C21_SCORE: i32 = 2000;
pub const C22_SCORE: i32 = 3000;

//       0 = y + h(-y + h(y + h(-y) + h(-h(-y)))); i = -y
// (C11) 0 = y + h(-y + h(y + a + h(-a))); i = -a
// (C12) 0 = y + h(-y + h(y + a + b)); i = y + a + b
//       0 = y + h(-y + c)

//       0 = y + h(-y) + h(-h(-y) + h(h(-y)) + h(-h(h(-y)))); i = -y
// (C21) 0 = y + a + h(-a + h(a) + h(-h(a))); i = a
// (C22) 0 = y + a + h(-a + b + h(-b)); i = -b
//       0 = y + a + h(-a + b + c)

// the visit_c** functions first get all the CH::C**(..) args, and then i.
// progress_c also, but then also gets v.

#[derive(Clone, Copy)]
pub enum CH {
    C11(/*y*/ E),
    C12(/*y*/ E),

    C21(/*y*/ E),
    C22(/*y*/ E, /*a*/ E),
}

pub fn progress_c(c: CH, i: E, v: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let n = ctxt.n;
    match c {
        CH::C11(y) => {
            let a = (n-i)%n;
            let yab = (y + a + v)%n;
            visit_c12(y, yab, ctxt)
        },
        CH::C12(y) => {
            // 0 = y + h(-y + v)
            // -> h(-y + v) = -y
            let n = ctxt.n;
            prove_pair((n-y+v)%n, (n-y)%n, ctxt)
        },

        CH::C21(y) => {
            let a = i;
            let b = v;
            let neg_b = (n-b)%n;
            visit_c22(y, a, neg_b, ctxt)
        },

        CH::C22(y, a) => {
            // 0 = y + a + h(-a + b + c)
            // <-> h(-a + b + c) = -y - a 
            let neg_b = i;
            let c = v;
            let b = (n-neg_b)%n;
            prove_pair((n-a + b + c)%n, (n-y + n-a)%n, ctxt)
        },
    }
}

// C1
pub fn visit_c11(y: E, neg_a: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let n = ctxt.n;
    let class = &mut ctxt.classes_h[neg_a as usize];
    let v = class.value;
    if v == E::MAX {
        ctxt.trail.push(TrailEvent::PushCH(neg_a));
        class.cs.push(CH::C11(y));
        class.score += C11_SCORE;

        Ok(())
    } else {
        let a = (n-neg_a)%n;
        let b = v;
        let yab = (y + a + b)%n;
        visit_c12(y, yab, ctxt)
    }
}

fn visit_c12(y: E, yab: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let n = ctxt.n;
    let class = &mut ctxt.classes_h[yab as usize];
    let v = class.value;
    if v == E::MAX {
        let neg_y = (n-y)%n;
        // -y = h(-y + h(yab))
        let class_hinv = &mut ctxt.classes_hinv[neg_y as usize];
        let z = class_hinv.value;
        if z != E::MAX {
            // h(z) = -y
            // -> z = -y + h(yab)
            // -> h(yab) = y+z
            return prove_pair(yab, (y+z)%n, ctxt);
        }

        ctxt.trail.push(TrailEvent::PushCH(yab));
        class.cs.push(CH::C12(y));
        class.score += C12_SCORE;

        Ok(())
    } else {
        let c = v;
        // h(-y + c) = -y
        prove_pair((n-y+c)%n, (n-y)%n, ctxt)
    }
}

// C2
pub fn visit_c21(y: E, a: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let n = ctxt.n;
    let class = &mut ctxt.classes_h[a as usize];
    let v = class.value;
    if v == E::MAX {
        ctxt.trail.push(TrailEvent::PushCH(a));
        class.cs.push(CH::C21(y));
        class.score += C21_SCORE;

        Ok(())
    } else {
        let b = v;
        let neg_b = (n-b)%n;
        visit_c22(y, a, neg_b, ctxt)
    }
}

pub fn visit_c22(y: E, a: E, neg_b: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    let n = ctxt.n;
    let class = &mut ctxt.classes_h[neg_b as usize];
    let v = class.value;
    if v == E::MAX {
        // -y - a = h(-a + b + h(-b))

        let neg_ya = (n-y + n-a)%n;
        let class_hinv = &mut ctxt.classes_hinv[neg_ya as usize];
        let z = class_hinv.value;
        if z != E::MAX {
            // h(z) = -y-a
            // -> -a + b + h(-b) = z
            // -> h(-b) = z+a-b
            return prove_pair(neg_b, (z+a+neg_b)%n, ctxt);
        }

        ctxt.trail.push(TrailEvent::PushCH(neg_b));
        class.cs.push(CH::C22(y, a));
        class.score += C22_SCORE;

        Ok(())
    } else {
        // 0 = y + a + h(-a + b + c)
        // <-> h(-a + b + c) = -y - a 
        let b = (n-neg_b)%n;
        let c = v;
        prove_pair((n-a + b + c)%n, (n-y + n-a)%n, ctxt)
    }
}
