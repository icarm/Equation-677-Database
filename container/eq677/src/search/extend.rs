use crate::*;

type EE = (usize, usize);

pub fn funny_extend() {
    for q in 1.. {
        dbg!(q);

        for (l0, l1, l2, l3, l4, a, b) in itertools::iproduct!(0..q, 0..q, 0..q, 0..q, 0..q, 0..q, 0..q) {
            let l = [l0, l1, l2, l3, l4];
            let f = |x: EE, y: EE| -> EE {
                let o = (y.0 + 5 - x.0)%5;
                ((2*x.0+4*y.0)%5, (a*x.1 + b*y.1 + l[o])%q)
            };
            let m = GenericMagma {
                elems: itertools::iproduct!(0..5, 0..q).collect(),
                f_def: f,
            }.to_matrix();
            if m.is677() {
                let s = format!("{l:?}, a={a}, b={b}");
                present_model(m.n, &s, |x, y| m.f(x, y));
            }
        }
    }
}

pub fn funny_extend2() {
    for q in 1.. {
        dbg!(q);

        for (a, b) in itertools::iproduct!(0..q, 0..q) {
            let f = |x: EE, y: EE| -> EE {
                let o = (y.0 + 5 - x.0)%5;
                if o != 0 {
                    ((2*x.0+4*y.0)%5, (a*x.1 + b*y.1)%q)
                } else {
                    ((2*x.0+4*y.0)%5, (a*x.1 + b*y.1 + 1)%q)
                }
            };
            let m = GenericMagma {
                elems: itertools::iproduct!(0..5, 0..q).collect(),
                f_def: f,
            }.to_matrix();
            if m.is677() {
                let s = format!("extend(x, y) = {a}*x + {b}*y + (base(x)=base(y))");
                present_model(m.n, &s, |x, y| m.f(x, y));
            }
        }
    }
}

pub fn m5_extend_14_23() {
    let m5 = MatrixMagma::by_fn(5, |x, y| (2*x + 4*y)%5);

    for q in 2..14 {
        dbg!(q);

        let Some((_, base)) = db().into_iter().filter(|(x, _)| x.0 == q).next() else { continue };

        for (a1, b1, a2, b2) in itertools::iproduct!(0..q, 0..q, 0..q, 0..q) {
            let c1 = 0;
            let c2 = 0;
            let c3 = 0;
/*
            if a2 != 3 { continue }
            if a3 != 7 { continue }
            if b2 != 10 { continue }
            if b3 != 10 { continue }
*/
            let f = |x: EE, y: EE| -> EE {
                let o = (y.0 + 5 - x.0)%5;
                let c23 = o == 2 || o == 3;
                if x.0 == y.0 {
                    ((2*x.0+4*y.0)%5, base.f(x.1, y.1))
                } else {
                    if c23 {
                        ((2*x.0+4*y.0)%5, (a1*x.1 + b1*y.1 + c1)%q)
                    } else {
                        ((2*x.0+4*y.0)%5, (a2*x.1 + b2*y.1 + c2)%q)
                    }
                }
            };
            let m = GenericMagma {
                elems: itertools::iproduct!(0..5, 0..q).collect(),
                f_def: f,
            }.to_matrix();
            if m.is677() {
                let s = format!("ok");
                present_model(m.n, &s, |x, y| m.f(x, y));
            }
        }
    }
}

