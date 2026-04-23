use crate::*;

/*
    f(r, r) = r
    f(i, r) = a + i
    f(r, j) = b + j
    f(i, j) = i + h(j-i)

    h(i) := (t=a)? r : t, where t := alpha*i + beta

*/
// Does not seem to have models.

pub fn linear_semitinv_search() {
    for n in 2.. {
        let r = n-1;
        for (a, b, alpha, beta) in itertools::iproduct!(0..r, 0..r, 0..r, 0..r) {
            let h = |i| {
                let t = (alpha*i + beta)%r;
                if t == a { r } else { t }
            };
            let f_def = |x, y| {
                if (x, y) == (r, r) { return r }
                if y == r { return (a + x)%r }
                if x == r { return (b + y)%r }
                let ht = h((y + r - x)%r);
                if ht == r { r } else { (x+ht)%r }
            };
            let f = FnMagma {
                n,
                f_def,
            };
            if f.is677() {
                present_model(n, "semitinv", f_def);
            }
        }
    }
}

/*
    f(r, r) = r
    f(i, r) = a + i
    f(r, j) = b + j
    f(i, j) = i + h(j-i)

    h(i) := (t=a)? r : t, where t := perm[i]

*/
// has models.

pub fn general_semitinv_search() {
    for n in 2.. {
        let r = n-1;
        for (a, b) in itertools::iproduct!(0..r, 0..r) {
            for perm in all_perms(r) {
                let h = |i| {
                    let t = perm[i];
                    if t == a { r } else { t }
                };
                let f_def = |x, y| {
                    if (x, y) == (r, r) { return r }
                    if y == r { return (a + x)%r }
                    if x == r { return (b + y)%r }
                    let ht = h((y + r - x)%r);
                    if ht == r { r } else { (x+ht)%r }
                };
                let f = FnMagma {
                    n,
                    f_def,
                };
                if f.is677() {
                    dbg!(a);
                    dbg!(b);
                    dbg!(r);
                    dbg!(&perm);
                    present_model(n, "general-semitinv", f_def);
                }
            }
        }
    }
}

