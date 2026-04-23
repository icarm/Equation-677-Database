use crate::*;

/*
    Translation-invariant models of the form:
    f(x, y) = x + h(y - x)
*/


// NOTE: this searches is deprecated due to the better https://github.com/memoryleak47/eq677/tree/main/src/tinv_dpll
//       Also because it runs out of RAM real quick.


pub fn all_perms(n: usize) -> Vec<Vec<usize>> {
    if n == 0 { return vec![Vec::new()]; }
    let mut outs = Vec::new();

    // we decide 'out[0] = a'.
    for p in all_perms(n-1) {
        for a in 0..n {
            let mut out = Vec::new();
            out.push(a);
            out.extend(p.iter().copied().map(|x| x + (x >= a) as usize));
            outs.push(out);
        }
    }
    outs
}

pub fn bad_tinv_search() {
    for p in 1.. {
        for h in all_perms(p) {
            let f_def = |x, y| (x + h[(y + p - x)%p])%p;
            let f = FnMagma {
                n: p,
                f_def,
            };
            if f.is677() {
                let mut s = String::from("tinv: f(x,y) = x + h(y-x) with h: ");
                for d in 0..p {
                    use std::fmt::Write;
                    write!(&mut s, "{}->{}", d, h[d]).unwrap();
                    if d != p-1 { write!(&mut s, ", ").unwrap(); }
                }
                present_model(p, &s, f_def);
            }
        }
    }
}
