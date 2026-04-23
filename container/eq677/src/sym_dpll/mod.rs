use crate::*;

// TODO things to add:
// - somehow improve rebuilding (the "usages" datastructure thing seems suboptimal)

mod api;
pub use api::*;

mod init;
pub use init::*;

type Map<T, K> = fxhash::FxHashMap<T, K>;

fn threading_depth(n: usize) -> usize { n+1 }

// e-class Id.
// ids less than ctxt.n correspond to ElemIds aswell.
type Id = usize;

#[derive(Clone, PartialEq, Eq)]
enum Mode {
    Forward,
    Backtrack,
    Done,
}

#[derive(Clone)]
enum TrailEvent {
    DecisionPoint(Id, /*remaining options*/ Vec<Id>),
    RmXYZ(Id, Id, Id),
    AddXYZ(Id, Id, Id),
    Equate(/*x*/ Id, /*y*/ Id), // equates x <- y.
    Defresh(Id),
}

#[derive(Clone)]
struct Class {
    uf: Id, // unionfind entry (pointer to leader)
    uf_rev: Vec<Id>, // c.uf_rev.contains(d) <-> (c != d && d.uf == c)
    usages: Vec<(Id, Id, Id)>, // This one is kept immutable after the "add"-phase is done.
}

#[derive(Clone)]
struct Ctxt {
    // Note: f(x, y) = z
    xyz: Map<(Id, Id), Id>,
    xzy: Map<(Id, Id), Id>,

    classes: Vec<Class>,

    n: usize,
    dirty_stack: Vec<(Id, Id)>,

    fresh: Vec<bool>,

    trail: Vec<TrailEvent>,
    mode: Mode,

    depth: usize, // The number of DecisionPoints on the current trail
}

fn choose_branch_id(ctxt: &Ctxt) -> Option<(Id, Id)> {
    let mut best: Option<((Id, Id), usize)> = None;
    for x in 0..ctxt.n {
        for y in 0..ctxt.n {
            let z = ctxt.xyz[&(x, y)];
            if z < ctxt.n { continue; }
            let score = ctxt.classes[z].usages.len(); // Is this a good heuristic?
            if best.map(|(_, score2) | score2 < score).unwrap_or(true) {
                best = Some(((x, y), score));
            }
        }
    }
    Some(best?.0)
}

fn submit_model(ctxt: &Ctxt) {
    present_model(ctxt.n, "sym_dpll", |x, y| ctxt.xyz[&(x, y)]);
}

fn infeasible_decision((x, y): (Id, Id), z: Id, ctxt: &Ctxt) -> bool {
    ctxt.xzy[&(x, z)] < ctxt.n
}

fn get_options((x, y): (Id, Id), ctxt: &Ctxt) -> Vec<Id> {
    let mut found_fresh = false;
    let mut options = Vec::new();
    for e in 0..ctxt.n {
        if infeasible_decision((x, y), e, &ctxt) { continue; }
        if ctxt.fresh[e] {
            if found_fresh { continue; }
            else { found_fresh = true; }
        }
        options.push(e);
    }
    options
}

fn backtrack(ctxt: &mut Ctxt) {
    loop {
        if ctxt.trail.is_empty() { ctxt.mode = Mode::Done; return; }
        match ctxt.trail.pop().unwrap() {
            TrailEvent::DecisionPoint(z, options) => {
                ctxt.depth -= 1;
                activate_option(z, options, ctxt);
                return;
            },
            TrailEvent::RmXYZ(x, y, z) => {
                ctxt.xyz.insert((x, y), z);
                ctxt.xzy.insert((x, z), y);
            },
            TrailEvent::AddXYZ(x, y, z) => {
                assert_eq!(ctxt.xyz.remove(&(x, y)), Some(z));
                assert_eq!(ctxt.xzy.remove(&(x, z)), Some(y));
            },
            TrailEvent::Equate(x, y) => {
                ctxt.classes[y].uf = y;
                assert_eq!(ctxt.classes[x].uf_rev.pop(), Some(y));
            },
            TrailEvent::Defresh(x) => { ctxt.fresh[x] = true; },
        }
    }
}

fn threaded_forward(ctxt: &mut Ctxt) {
    let Some((x, y)) = choose_branch_id(ctxt) else {
        submit_model(&ctxt);
        ctxt.mode = Mode::Backtrack;
        return;
    };

    if ctxt.fresh[x] {
        ctxt.trail.push(TrailEvent::Defresh(x));
        ctxt.fresh[x] = false;
    }
    if ctxt.fresh[y] {
        ctxt.trail.push(TrailEvent::Defresh(y));
        ctxt.fresh[y] = false;
    }

    let options = get_options((x, y), ctxt);
    let z = ctxt.xyz[&(x, y)];
    into_par_for_each(options, |e| {
        let mut ctxt = ctxt.clone();
        activate_option(z, vec![e], &mut ctxt);
        mainloop(ctxt);
    });
    ctxt.mode = Mode::Done;
}

fn forward(ctxt: &mut Ctxt) {
    if ctxt.depth < threading_depth(ctxt.n) {
        threaded_forward(ctxt);
        return;
    }

    let Some((x, y)) = choose_branch_id(ctxt) else {
        submit_model(&ctxt);
        ctxt.mode = Mode::Backtrack;
        return;
    };

    if ctxt.fresh[x] {
        ctxt.trail.push(TrailEvent::Defresh(x));
        ctxt.fresh[x] = false;
    }
    if ctxt.fresh[y] {
        ctxt.trail.push(TrailEvent::Defresh(y));
        ctxt.fresh[y] = false;
    }

    let options = get_options((x, y), ctxt);
    let z = ctxt.xyz[&(x, y)];
    activate_option(z, options, ctxt);
}

fn mainloop(mut ctxt: Ctxt) {
    loop {
        match ctxt.mode {
            Mode::Forward => forward(&mut ctxt),
            Mode::Backtrack => backtrack(&mut ctxt),
            Mode::Done => return,
        }
    }
}

// Counts the number of found PosIds.
fn progress(ctxt: &Ctxt) -> usize {
    let mut c = 0;
    for x in 0..ctxt.n {
        for y in 0..ctxt.n {
            c += (ctxt.xyz[&(x, y)] < ctxt.n) as usize;
        }
    }
    c
}

fn activate_option(z: Id, mut options: Vec<Id>, ctxt: &mut Ctxt) {
    let Some(e) = options.pop() else {
        // dbg!(ctxt.depth);
        // dbg!(progress(ctxt));

        ctxt.mode = Mode::Backtrack;
        return;
    };

    ctxt.mode = Mode::Forward;

    if ctxt.fresh[e] {
        ctxt.trail.push(TrailEvent::Defresh(e));
        ctxt.fresh[e] = false;
    }

    ctxt.trail.push(TrailEvent::DecisionPoint(z, options));
    ctxt.depth += 1;

    union(z, e, ctxt);
    rebuild(ctxt);
}

pub fn sym_run(n: usize) {
    let models = new_ctxts(n);
    for ctxt in models {
        mainloop(ctxt);
    }
}
