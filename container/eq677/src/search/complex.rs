use crate::*;

type Complex = (usize, usize);

fn mul(x: Complex, y: Complex, p: usize) -> Complex {
    ((x.0*y.0 + (p*p - x.1*y.1))%p, (x.0*y.1 + x.1*y.0)%p)
}

fn add(x: Complex, y: Complex, p: usize) -> Complex {
    ((x.0 + y.0)%p, (x.1 + y.1)%p)
}

pub fn complex_linear_search() {
    complex_search(false);
}

pub fn complex_affine_search() {
    complex_search(true);
}

fn complex_search(affine: bool) {
    for p in 0.. {
        let p_aff = if affine { p } else { 1 };
        for (c0, c1, c2, c3, c4, c5, c6) in itertools::iproduct!(0..p, 0..p, 0..p, 0..p, 0..p, 0..p_aff, 0..p_aff) {
            let f_def = |x: Complex, y: Complex| -> Complex {
                let lhs = mul(x, (c0, c1), p);
                let rhs = mul(y, (c2, c3), p);
                let a = add(lhs, rhs, p);
                add(a, (c5, c6), p)
            };

            let f = GenericMagma {
                elems: (0..p).map(move |x| (0..p).map(move |y| (x, y))).flatten().collect(),
                f_def,
            };
            if f.is677() {
                let m = f.to_matrix();
                let s = format!("complex: f(x,y) = x*({c0} + {c1}i) + y*({c2} + {c3}i) + {c4} + {c5}i");
                present_model(p*p, &s, |x, y| m.f(x, y));
            }
        }
    }
}

pub struct GenericMagma<E, F> {
    pub elems: Vec<E>,
    pub f_def: F,
}

impl<E, F> Magma for GenericMagma<E, F> where F: Fn(E, E) -> E, E: Copy + Eq {
    type Elem = E;

    fn elems(&self) -> impl Iterator<Item=E> { self.elems.iter().copied() }
    fn f(&self, x: E, y: E) -> E { (self.f_def)(x, y) }
}

impl<E, F> GenericMagma<E, F> where F: Fn(E, E) -> E, E: Copy + Eq {
    pub fn to_matrix(&self) -> MatrixMagma {
        MatrixMagma::by_fn(self.elems.len(), |x, y| {
            let x = self.elems[x];
            let y = self.elems[y];
            let z = self.f(x, y);
            self.elems.iter().position(|zz| *zz == z).unwrap_or(usize::MAX)
        })
    }
}
