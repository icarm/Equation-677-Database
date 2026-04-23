use crate::*;

/*
    f(x, y) = x*c[0] + x^2*c[1] + y*c[2] + y^2*c[3] + c[4]
*/

pub struct FnMagma<F> {
    pub n: usize,
    pub f_def: F,
}

impl<F> Magma for FnMagma<F> where F: Fn(usize, usize) -> usize {
    type Elem = usize;

    fn elems(&self) -> impl Iterator<Item=usize> { 0..self.n }
    fn f(&self, x: usize, y: usize) -> usize { (self.f_def)(x, y) }
}

pub fn poly_search() {
    for p in 0.. {
        for (c0, c1, c2, c3, c4) in itertools::iproduct!(0..p, 0..p, 0..p, 0..p, 0..p) {
            let f_def = |x, y| ((x*c0)%p + (x*x*c1)%p + (y*c2)%p + (y*y*c3)%p + c4)%p;
            let f = FnMagma {
                n: p,
                f_def,
            };
            if f.is677() {
                present_model(p, "poly", f_def);
            }
        }
    }
}
