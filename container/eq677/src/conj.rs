use crate::*;

#[test]
fn test_db_conj() {
    for (_, m) in db() {
        conj(&m);
    }
}

pub fn conj(m: &MatrixMagma) {
    assert!(m.is677());
    assert!(m.is255());
    assert!(m.is_left_cancellative());

    conj_diag_orbit_size(m);
    conj_bijective_or_constant(m);
    conj_singleton_cycle(m);
    conj_2_orbit(m);

    // false_conj_unique_cycle_size(m);
    // false_conj_cycle_size(m);
    // false_conj_cycles_summary(m);
    // false_conj_cycles_divide_n(m);
    // false_conj_not_rigid(m);
    // false_conj_cycle2(m);
    // false_conj_d_bij(m);
    // false_conj_right_cancellative(m);
    // false_conj_exists_idempotence(m);
    // false_conj_tinv_or_semitinv(m);
}

// Conjectures:

fn bij_to_cycles(n: usize, bij: impl Fn(usize) -> usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();

    let mut seen = vec![false; n];
    for mut i in 0..n {
        if seen[i] { continue }

        let mut current = vec![i];

        loop {
            seen[i] = true;
            i = bij(i);
            if seen[i] {
                out.push(std::mem::take(&mut current));
                break
            } else {
                current.push(i);
            }
        }
    }
    out
}

fn false_conj_tinv_or_semitinv(m: &MatrixMagma) {
    if m.n < 2 || m.n > 50 /* for perf */ { return }

    let grp = m.autom_group();
    for perm in grp {
        let c = bij_to_cycles(m.n, |i| perm[i]);
        if c.iter().any(|cyc| cyc.len() >= m.n-1) { return }
    }
    assert!(false);
}

fn false_conj_d_bij(m: &MatrixMagma) {
    if m.n == 496 { return }

    // claim: for any x, {f(f(y,x),y) | y in M} = M.
    let n = m.n;
    for x in 0..n {
        let mut opts = vec![false; n];
        for y in 0..n {
            let z = m.f(m.f(y, x), y);
            opts[z] = true;
        }
        assert!(opts.iter().all(|x| *x));
    }
}

fn conj_2_orbit(m: &MatrixMagma) {
    if m.n > 40 { return } // for performance

    if !is_prime(m.n) { return }

    let grp = m.autom_group();
    let orbits = orbits(&grp);
    let mut orbit_sizes = vec![0; m.n];
    for x in 0..m.n {
       orbit_sizes[orbits[x]] += 1;
    }
    orbit_sizes.sort();
    orbit_sizes.reverse();

    // either all in one orbit
    if orbit_sizes[0] == m.n { return }

    // or one singleton element.
    assert_eq!(orbit_sizes[0], m.n - 1);
    assert_eq!(orbit_sizes[1], 1);
    assert!(orbit_sizes[2..].iter().all(|x| *x == 0));
}

fn false_conj_autom(m: &MatrixMagma) {
    let expected = match m.n {
        0 => return,
        1 => 1,
        5 => 20,
        7 => 6,
        9 => 8,
        11 => 110,
        13 => 12,
        19 => 18,
        25 => return, // sometimes 500, sometimes 12000
        31 => return, // sometimes 30, sometimes 930
        // ...
        _ => return,
    };
    let real = m.autom_group().len();
    assert_eq!(expected, real);
}

fn false_conj_one_orbit(m: &MatrixMagma) {
    if m.n % 7 == 0 { return }
    let grp = m.autom_group();
    let orbits = orbits(&grp);
    if orbits.iter().any(|x| *x != 0) {
        println!("wrong:");
        m.cycle_dump();
        dbg!(&orbits);
        assert!(false);
    }
}

// conj_cycles_summary is a stronger version of this.
fn false_conj_cycles_divide_n(m: &MatrixMagma) {
    if m.n % 7 == 0 { return } // Why %7?

    let mut s = 0;
    for x in 0..m.n {
        for z in 0..m.n {
            s += (c_mini(m, x, z) == z) as usize;
        }
    }
    assert!(s % m.n == 0);
}

fn false_conj_cycles_summary(m: &MatrixMagma) {
    if m.n % 7 == 0 { return } // Why %7?

    let a = c_summary(m, 0);
    for x in 1..m.n {
        let b = c_summary(m, x);
        assert_eq!(a, b);
    }
}

// This property seems often true, but not always true.
fn false_conj_unique_cycle_size(m: &MatrixMagma) {
    if !is_prime(m.n) { return }

    let mut out = Vec::new();
    for x in 0..m.n {
        out.extend(c_summary(m, x));
    }
    out.sort();
    out.retain(|x| *x >= 3);
    out.dedup();
    assert!(out.len() <= 1);
}

// Falsified conjectures:

pub fn is_prime(n: usize) -> bool {
    if n < 2 { return false }

    for i in 2.. {
        if n%i == 0 { return false }
        if i*i > n { break }
    }
    true
}

fn false_conj_not_rigid(m: &MatrixMagma) {
    if m.n < 2 { return }

    // for performance
    if m.n > 80 { return }

    if m.autom_stats().grpsize() <= 1.5 {
        m.dump();
        panic!();
    }
}

// Note: This is equivalent to 255, as you can see here:
// https://teorth.github.io/equational_theories/blueprint/677-chapter.html
fn conj_singleton_cycle(m: &MatrixMagma) {
    if m.n == 0 { return }

    for x in 0..m.n {
        assert!((0..m.n).any(|y| m.f(y, x) == x));
    }
}

fn conj_bijective_or_constant(m: &MatrixMagma) {
    // only applies to primitive models.
    // if decompose(&m).len() > 0 { return }

    // This is a much cheaper check though:
    if !is_prime(m.n) { return }

    assert!(m.is_diag_bijective() || m.is_diag_constant());
}

fn false_conj_exists_idempotence(m: &MatrixMagma) {
    if m.n == 0 { return }

    for x in 0..m.n {
        if m.f(x, x) == x { return }
    }
    assert!(false);
}

fn false_conj_odd(m: &MatrixMagma) {
    assert!(m.n % 2 == 1 || m.n == 0);
}

fn false_conj_right_cancellative(m: &MatrixMagma) {
    if m.n == 496 { return }

    assert!(m.is_right_cancellative());
}

fn conj_diag_orbit_size(m: &MatrixMagma) {
    if !m.is_diag_bijective() { return }

    for x in 0..m.n {
        let mut y = x;
        let mut i = 0;
        // i is the smallest positive number, s.t. S^i x = x, where S x = x*x
        loop {
            y = m.f(y, y);
            i += 1;
            if x == y { break }
        }
        // Known values: 1, 3, 4, 5, 6, 7, 12, 18.
        // We know that 2 is impossible. 0*0 = 1 /\ 1*1 = 0 -> 0=1.
        assert!(i != 8);
    }
}

fn false_conj_cycle_size(m: &MatrixMagma) {
    for x in 0..m.n {
        for z in 0..m.n {
            let i = c(m, x, z);
            // Known values for i:
            // assert!(i == 1 || i == 2 || i == 4 || i == 5 || i == 6 || i == 7 || i == 8 || i == 9
            //     || i == 10 || i == 12 || i == 14 || i == 15 || i == 18
            //     || i == 21 || i == 36 || i == 42 || i == 48 || i == 49);
        }
    }
}

fn false_conj_cycle2(m: &MatrixMagma) {
    for x in 0..m.n {
        for y in 0..m.n {
            let a = m.f(x, y);
            if a == y { continue }

            let a = m.f(x, a);
            let a = m.f(x, a);
            assert!(a != y);
        }
    }
}

// Helpers:

// returns how often I need to left-multiply x onto z, until it becomes z again.
fn c(m: &MatrixMagma, x: usize, z: usize) -> u32 {
    let mut zz = z;
    let mut i = 0;
    loop {
        zz = m.f(x, zz);
        i += 1;
        if z == zz { break }
    }
    i
}

// Finds the minimal (i.e. canonical) element from the C(x, z) cycle.
fn c_mini(m: &MatrixMagma, x: usize, z: usize) -> usize {
    let mut zz = z;
    let mut mini = z;

    loop {
        zz = m.f(x, zz);
        if z == zz { break }
        if zz < mini { mini = zz; }
    }
    mini
}

fn c_summary(m: &MatrixMagma, x: usize) -> Vec<u32> {
    let mut v = Vec::new();
    for z in 0..m.n {
        // We only consider the representatives of each cycle!
        if c_mini(m, x, z) == z {
            v.push(c(m, x, z));
        }
    }
    v.sort();
    v
}

// 255 is equivalent to this function always returning 1 or 3.
fn right_cycle(m: &MatrixMagma, x: usize) -> usize {
    let mut a = x;
    let mut c = 0;
    loop {
        a = m.f(a, x);
        c += 1;
        if a == x { break }
    }
    c
}


impl MatrixMagma {
    pub fn is_left_cancellative(&self) -> bool {
        for a in 0..self.n {
            for b in 0..self.n {
                for c in 0..self.n {
                    // a*b = a*c -> b = c.
                    if b != c && self.f(a, b) == self.f(a, c) { return false }
                }
            }
        }
        true
    }

    pub fn is_right_cancellative(&self) -> bool {
        for a in 0..self.n {
            for b in 0..self.n {
                for c in 0..self.n {
                    // b*a = c*a -> b = c
                    if b != c && self.f(b, a) == self.f(c, a) { return false }
                }
            }
        }
        true
    }

    pub fn is_diag_constant(&self) -> bool {
        for x in 0..self.n {
            for y in 0..self.n {
                if self.f(x, x) != self.f(y, y) { return false }
            }
        }
        true
    }

    pub fn is_diag_bijective(&self) -> bool {
        for x in 0..self.n {
            for y in 0..self.n {
                if x == y { continue }
                if self.f(x, x) == self.f(y, y) { return false }
            }
        }
        true
    }

    pub fn is_idempotent(&self) -> bool {
        for x in 0..self.n {
            if x != self.f(x, x) { return false }
        }
        true
    }

    // For tinv models, this is equivalent to h=h⁻¹.
    pub fn is_double_left_inverse(&self) -> bool {
        for x in 0..self.n {
            for y in 0..self.n {
                if self.f(x, self.f(x, y)) != y { return false }
            }
        }
        true
    }

    pub fn is_double_right_inverse(&self) -> bool {
        for x in 0..self.n {
            for y in 0..self.n {
                if self.f(self.f(y, x), x) != y { return false }
            }
        }
        true
    }
}
