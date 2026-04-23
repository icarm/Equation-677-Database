use crate::*;

// In this file I want to document common combination of properties that might hint towards a model class.

// satisfied by 11/0, 25/25, 29/0.
pub fn prop_combo(m: &MatrixMagma) -> bool {
    let f = |x, y| m.f(x, y);
    for x in 0..m.n {
        for y in 0..m.n {
            if f(x, f(x, y)) != f(f(y, x), x) { return false }
            if x != f(f(f(y, f(x, y)), x), y) { return false }
            if f(f(x, y), y) != f(f(f(x, f(x, y)), x), x) { return false }
        }
    }
    true
}

// Satisfied by a huge amount of magmas.
pub fn prop_combo2(m: &MatrixMagma) -> bool {
    let f = |x, y| m.f(x, y);
    for x in 0..m.n {
        for y in 0..m.n {
            if x != f(f(f(x, y), y), y) { return false }
            if f(x, f(x, y)) != f(y, f(y, x)) { return false }
        }
    }
    true
}

