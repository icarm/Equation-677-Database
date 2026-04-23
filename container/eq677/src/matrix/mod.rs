use crate::*;

mod canon;

mod canon2;
pub use canon2::*;

mod minimize;
pub use minimize::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
// This magma might be partial; we use usize::MAX to encoding missing entries.
pub struct MatrixMagma {
    pub n: usize, // size
    pub data: Box<[usize]>, // of length n^2
}

impl Magma for MatrixMagma {
    type Elem = usize;
    fn elems(&self) -> impl Iterator<Item=usize> {
        0..self.n
    }

    fn f(&self, x: usize, y: usize) -> usize {
        assert!(x < self.n);
        assert!(y < self.n);
        self.data[MatrixMagma::idx(x, y, self.n)]
    }
}

impl MatrixMagma {
    pub fn zeros(n: usize) -> Self {
        MatrixMagma {
            n,
            data: std::iter::repeat(0).take(n*n).collect(),
        }
    }

    pub fn shuffle(&self) -> MatrixMagma {
        use rand::seq::SliceRandom;
        use rand::rng;

        let mut vec: Vec<usize> = (0..self.n).collect();
        let mut rng = rng();

        vec.shuffle(&mut rng);

        self.permute(vec)
    }

    pub fn undefined(n: usize) -> Self {
        MatrixMagma {
            n,
            data: std::iter::repeat(usize::MAX).take(n*n).collect(),
        }
    }

    pub fn parse(s: &str) -> Self {
        let s = s.replace(",", " ");
        let s = s.trim();
        let mut lines: Vec<_> = s.lines().collect();

        let mut m = MatrixMagma::zeros(lines.len());

        for (x, line) in lines.into_iter().enumerate() {
            for (y, s) in line.split_whitespace().enumerate() {
                if s == "-" {
                    m.set_f(x, y, usize::MAX);
                } else {
                    let i: usize = s.parse().unwrap();
                    m.set_f(x, y, i);
                }
            }
        }

        m
    }

    pub fn by_fn(n: usize, f: impl Fn(usize, usize) -> usize) -> Self {
        let mut m = MatrixMagma::zeros(n);
        for x in 0..n {
            for y in 0..n {
                m.set_f(x, y, f(x, y));
            }
        }
        m
    }

    pub fn set_f(&mut self, x: usize, y: usize, val: usize) {
        self.data[MatrixMagma::idx(x, y, self.n)] = val;
    }

    fn idx(x: usize, y: usize, n: usize) -> usize {
        x + n * y
    }

    pub fn count_defined(&self) -> usize {
        let mut c = 0;
        for x in 0..self.n {
            for y in 0..self.n {
                c += (self.f(x, y) != usize::MAX) as usize;
            }
        }
        c
    }

    pub fn to_string(&self) -> String {
        if self.n == 0 { return String::new() }

        let mut s = String::new();

        let mut div = self.n.ilog10() as usize + 2;
        if FIXED_WIDTH { div = div.max(3); }

        for x in 0..self.n {
            for y in 0..self.n {
                let z = self.f(x, y);
                if z == usize::MAX {
                    s.push_str(&format!("{:<width$}", '-', width = div));
                } else {
                    s.push_str(&format!("{:<width$}", fmt(z), width = div));
                }
            }
            s.push('\n');
        }
        s
    }

    pub fn dump(&self) {
        print!("{}", self.to_string());
    }

    // Prints a magma, but displays every row x as the bijection spanned by
    // `y => x*y` written in cycle notation.
    pub fn cycle_dump(&self) {
        let ctxt = self;
        for x in 0..ctxt.n {
            print!("{x}: ");
            draw_cycle(x, ctxt.n, |i| ctxt.f(x, i));
            println!();
        }
    }

    pub fn column_cycle_dump(&self) {
        assert!(self.is_right_cancellative());
        let ctxt = self;
        for x in 0..ctxt.n {
            draw_cycle(x, ctxt.n, |i| ctxt.f(i, x));
            print!(":{}\n", fmt(x));
        }
    }

    fn zxy(&self, z: usize, x: usize) -> usize {
        for y in 0..self.n {
            if self.f(x, y) == z { return y }
        }
        unreachable!()
    }

    pub fn z_cycle_dump(&self) {
        assert!(self.is_right_cancellative());
        let ctxt = self;
        for z in 0..ctxt.n {
            print!("{} = ", fmt(z));
            draw_cycle(z, ctxt.n, |i| self.zxy(z, i));
            println!();
        }
    }

    pub fn autom_dump(&self) {
        println!("AUTOM:");
        let g = self.autom_group_mini();
        for x in g {
            draw_cycle(0, self.n, |i| x[i]);
            println!();
        }
    }

    pub fn is_total(&self) -> bool {
        !self.data.contains(&usize::MAX)
    }
}

pub fn bij_to_cycles(x: usize, n: usize, sort: bool, f: impl Fn(usize) -> usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut seen = vec![false; n];
    let mut current = Vec::new();
    for i in (x..n).chain(0..x) {
        if seen[i] { continue }
        let mut cc = i;
        loop {
            current.push(cc);
            seen[cc] = true;
            cc = f(cc);
            if cc == usize::MAX {
                current.push(usize::MAX);
                break;
            }
            if seen[cc] {
                out.push(current);
                current = Vec::new();
                break;
            }
        }
    }
    if sort {
        out.sort_by_key(|x| x.len());
    }
    out
}

pub fn draw_cycle_string(x: usize, n: usize, f: impl Fn(usize) -> usize) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    for cycle in bij_to_cycles(x, n, true, f) {
        write!(out, "(");
        for a in cycle {
            if a == usize::MAX {
                write!(out, "... ");
            } else {
                write!(out, "{} ", fmt(a));
            }
        }
        let _ = out.pop();
        write!(out, ") ");
    }
    let _ = out.pop();
    out
}

pub fn draw_cycle(x: usize, n: usize, f: impl Fn(usize) -> usize) {
    print!("{}", draw_cycle_string(x, n, f));
}

pub fn cartesian(m0: &MatrixMagma, m1: &MatrixMagma) -> MatrixMagma {
    MatrixMagma::by_fn(m0.n * m1.n, |x, y| {
        let (x0, x1) = (x%m0.n, x/m0.n);
        let (y0, y1) = (y%m0.n, y/m0.n);

        let (z0, z1) = (m0.f(x0, y0), m1.f(x1, y1));

        if z0 == usize::MAX || z1 == usize::MAX { usize::MAX }
        else { z0 + z1*m0.n }
    })
}


pub const FIXED_WIDTH: bool = true;
pub fn fmt(x: usize) -> String {
    if FIXED_WIDTH {
        format!("{x:02}")
    } else {
        x.to_string()
    }
}
