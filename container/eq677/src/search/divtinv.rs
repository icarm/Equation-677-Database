use crate::*;

// x/k in GF(p).
fn div(x: usize, k: usize, p: usize) -> usize {
    assert!(x < p);
    assert!(k < p);
    if x == 0 { return 0 }
    assert!(k != 0);

    for d in 0..p {
        if (k*d)%p == x { return d; }
    }
    panic!()
}

pub fn divtinv_search() {
    for p in 3..100 {
        if !is_prime(p) { continue }
        for d in 1..p {
            let h = |i| div(i, d, p);
            let m = MatrixMagma::by_fn(p, |x, y| (x + h((y+p-x)%p))%p);
            if m.is677() {
                let s = format!("divtinv: h(x) = x/{d}, GF({p})");
                // println!("{s}");
                present_model(p, &s, |x, y| m.f(x, y));
            }
        }
    }
}
