use crate::*;

/*
    f(x, y) = (x*c0)%c1 + y*c2 + c3
*/

pub fn fakelin_search() {
    for p in 0.. {
        dbg!(p);
        for (c0, c1, c2, c3) in itertools::iproduct!(0..p, 1..p, 0..p, 0..p) {
            let f_def = |x, y| ((x*c0)%c1 + y*c2 + c3)%p;
            let f = FnMagma {
                n: p,
                f_def,
            };
            if f.is677() {
                present_model(p, "fakelin", f_def);
            }
        }
    }
}

/*
    f(x, y) = x*c0 + y*c1 + c2 + (x == c3) * c4
*/

pub fn fakelin_search2() {
    for p in 0.. {
        dbg!(p);
        for (c0, c1, c2, c3, c4) in itertools::iproduct!(0..p, 0..p, 0..p, 0..p, 1..p) {
            let f_def = |x, y| (x*c0 + y*c1 + c2 + ((x == c3) as usize) * c4)%p;
            let f = FnMagma {
                n: p,
                f_def,
            };
            if f.is677() {
                present_model(p, "fakelin2", f_def);
            }
        }
    }
}

/*
    f(x, y) = x^3*c0 + x^2*c1 + x*c2 + (c3 + x*c4 + x^2*c5 + x^3*c6)*y + c7

    Doesn't seem to find non-linear models?
*/

pub fn fakelin_search3() {
    for p in 0.. {
        dbg!(p);
        for (c0, c1, c2, c3, c4, c5, c6, c7) in itertools::iproduct!(0..p, 0..p, 0..p, 0..p, 0..p, 0..p, 0..p, 0..p) {
            let f_def = |x, y| (x*x*x*c0 + x*x*c1 + x*c2 + (c3 + x*c4 + x*x*c5 + x*x*x*c6)*y + c7)%p;

            // filter out linear models.
            if c0 == 0 && c1 == 0 && c4 == 0 && c5 == 0 && c6 == 0 { continue }

            let f = FnMagma {
                n: p,
                f_def,
            };
            if f.is677() {
                present_model(p, "fakelin3", f_def);
            }
        }
    }
}
