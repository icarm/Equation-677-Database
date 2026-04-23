use crate::*;

const PI3: bool = false;

/*
    f(x, y) = pi3(pi1(x) op pi2(y))
*/

pub fn bij_plus_search() { bij_search(|x, y| x + y); }
pub fn bij_mul_search() { bij_search(|x, y| x * y); }

fn all_perms(n: usize) -> Vec<Vec<usize>> {
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

fn bij_search(op: impl Fn(usize, usize) -> usize) {
    for p in 0.. {
        let perms = all_perms(p);
        let pi3_perms = if PI3 { &perms } else { &vec![(0..p).collect()] };
        for pi1 in &perms {
            for pi2 in &perms {
                for pi3 in pi3_perms {
                    let f_def = |x, y| pi3[op(pi1[x], pi2[y])%p];
                    let f = FnMagma {
                        n: p,
                        f_def,
                    };
                    if f.is677() {
                        present_model(p, "bij", f_def);
                    }
                }
            }
        }
    }
}
