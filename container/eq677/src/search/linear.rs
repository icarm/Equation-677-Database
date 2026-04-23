use crate::*;

/*
  x*y = (ax + by)%p

  x = y*(x*((y*x)*y))
    = y*(x*((ay + bx)*y))
    = y*(x*(aay + bax + by))
    = y*(ax + baay + bbax + bby)
    = ay + bax + bbaay + bbbax + bbby
  => 0 = a + bbaa + bbb
  => 1 = ba + bbba
*/

fn modpow(mut b: usize, mut e: usize, p: usize) -> usize {
    let mut res = 1;
    while (e > 0) {
        if (e & 1 == 1) {
            res = res*b % p;
        }
        b = b * b % p;
        e >>= 1;
    }
    res
}

pub fn linear_search() {
    for p in 0.. {
        for b in 0..p {
            let b4 = (b + b*b*b)%p;
            for a in 0..p {
                // Optimization via Fermats little theorem.
                // Only works for `p` prime though!
                // let a = modpow(b4, p-2, p);

                if (a*b4)%p != 1 { continue }
                if (a + a*a*b*b + b*b*b)%p != 0 { continue }

                present_model(p, &format!("linear: f(x,y) = (x*{a} + y*{b})%{p}"), |x, y| (x*a + y*b)%p);
            }
        }
    }
}

pub fn piecewise_linear_search() {
    for p in 0.. {
        dbg!(p);

        let mut residues = vec![false; p];
        for x in 0..p {
            residues[(x*x)%p] = true;
        }
        // for (a1, b1, c1, a2, b2, c2) in itertools::iproduct!(0..p, 0..p, 0..p, 0..p, 0..p, 0..p) {
        for (a1, a2) in itertools::iproduct!(0..p, 0..p) {
            let b1 = (1+p-a1)%p;
            let b2 = (1+p-a2)%p;
            let c1 = 0;
            let c2 = 0;
            // if b1 != (1+p-a1)%p { continue }
            // if b2 != (1+p-a2)%p { continue }
            // if c1 != 0 { continue }
            // if c2 != 0 { continue }

            let f = |x, y| {
                if residues[(y+p-x)%p] {
                    (a1*x + b1*y + c1)%p
                } else {
                    (a2*x + b2*y + c2)%p
                }
            };
            let m = MatrixMagma::by_fn(p, f);
            if m.is677() {
                present_model(p, "piecewise linear: ", |x, y| m.f(x, y));
            }
        }
    }
}
