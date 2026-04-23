use crate::*;
use smallvec::{SmallVec, smallvec};

mod init;
use init::*;

mod c;
use c::*;

mod run;
use run::*;
pub use run::{semitinv_run, semitinv_search};

// identifies an element.
type E = u8;

#[derive(Clone)]
struct ClassH {
    value: E, // is E::MAX when undecided.
    cs: SmallVec<[CH; 7]>, // the constraints that currently wait on us.
    score: i32, // a cache for the base_score.
}

#[derive(Clone)]
struct ClassHInv {
    value: E,
}

#[derive(Clone)]
struct Ctxt {
    trail: Vec<TrailEvent>,
    classes_h: Box<[ClassH]>,
    classes_hinv: Box<[ClassHInv]>,
    a: E,
    b: E,
    r: E, // n = r+1
    propagate_queue: Vec<(E, E)>, // contains (i, j) for h(i) = j
}

#[derive(Clone)]
enum TrailEvent {
    Decision(E, E), // contains (i, j) for h(i) = j.
    DefineClass(E), // h(i) is now defined.
    PushCH(E), // a new constraint is waiting for h(i) = ?.
}

fn f(x: E, y: E, ctxt: &Ctxt) -> E {
    match try_f(x, y, ctxt) {
        Ok(z) => z,
        Err(_) => E::MAX,
    }
}

impl Ctxt {
    pub fn matrix(&self) -> MatrixMagma {
        MatrixMagma::by_fn(self.r as usize + 1, |x, y| {
            let z = f(x as E, y as E, self);
            if z == E::MAX { usize::MAX } else { z as usize }
        })
    }

    pub fn dump(&self) { self.matrix().dump() }
    pub fn cycle_dump(&self) { self.matrix().cycle_dump() }
}
