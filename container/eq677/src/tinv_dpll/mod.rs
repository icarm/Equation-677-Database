use crate::*;
use smallvec::{SmallVec, smallvec};

mod init;
use init::*;

mod c;
use c::*;

mod run;
use run::*;
pub use run::{tinv_run, tinv_search};

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
    n: E,
    propagate_queue: Vec<(E, E)>, // contains (i, j) for h(i) = j
}

#[derive(Clone)]
enum TrailEvent {
    Decision(E, E), // contains (i, j) for h(i) = j.
    DefineClass(E), // h(i) is now defined.
    PushCH(E), // a new constraint is waiting for h(i) = ?.
}

// f(x, y) = x + h(y-x)
fn f(x: E, y: E, ctxt: &Ctxt) -> E {
    let n = ctxt.n;
     let i = (y+n-x)%n;
     let v = ctxt.classes_h[i as usize].value;
     if v == E::MAX { E::MAX } else { (x+v)%n }
}

impl Ctxt {
    pub fn matrix(&self) -> MatrixMagma {
        MatrixMagma::by_fn(self.n as usize, |x, y| {
            let z = f(x as E, y as E, self);
            if z == E::MAX { usize::MAX } else { z as usize }
        })
    }

    pub fn dump(&self) { self.matrix().dump() }
    pub fn cycle_dump(&self) { self.matrix().cycle_dump() }
}
