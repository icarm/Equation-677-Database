use crate::*;

use std::collections::BTreeSet;

mod kbo;
use kbo::*;

mod simplify;
use simplify::*;

mod cps;
use cps::*;

pub type E = u8;
pub type V = usize;

// typically oriented lhs -> rhs.
pub type Rule = (Term, Term, /*oriented*/ bool);

struct ER(BTreeSet<Rule>);

#[derive(PartialEq, Eq)]
pub enum Term {
    V(V),
    E(E),
    F(Box<[Term; 2]>),
}

pub fn completion(er: &mut ER) {
    loop {
        simplify(er);
        if !compute_cps(er) { break }
    }
}
