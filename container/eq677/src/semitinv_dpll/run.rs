use crate::semitinv_dpll::*;

use std::sync::atomic::{AtomicUsize, Ordering};

const USE_COUNTER: bool = false;

fn threading_depth(r: E) -> E { 2 }

static RUNS_STARTED: AtomicUsize = AtomicUsize::new(0);
static RUNS_FINISHED: AtomicUsize = AtomicUsize::new(0);

pub fn semitinv_run(n: usize) {
    if n < 2 { return }

    let r = (n-1) as E;
    let ctxt = build_ctxt(r);
    let iters: Vec<_> = itertools::iproduct!(0..r, 0..r, 0..r).collect();
    into_par_for_each(iters, |(a, b, h_nega)| {
        let mut ctxt = ctxt.clone();
        ctxt.a = a;
        ctxt.b = b;

        if prove_pair((r-a)%r, h_nega, &mut ctxt).is_err() { return }

        // r = h(b + a + h(-a)).
        let c = (h_nega + a + b)%r;
        if prove_pair(c, r, &mut ctxt).is_err() { return }

        if prove_pair((a+b)%r, (r-b)%r, &mut ctxt).is_err() { return }
        if spawn_cs(0, r, a, &mut ctxt).is_err() { return }
        if spawn_cs(r, 0, b, &mut ctxt).is_err() { return }
        if propagate(&mut ctxt).is_err() { return }

        // prerun is unnecessary due to the large amount of model splitting.
        main_branch(&mut ctxt);
    });
}

pub fn semitinv_search() {
    for i in 0.. {
        semitinv_run(i);
    }
}

fn inc_counter(ctr: &AtomicUsize) {
    if !USE_COUNTER { return }

    let t = PRINT_MUTEX.lock().unwrap();
    ctr.fetch_add(1, Ordering::SeqCst);

    let started = RUNS_STARTED.load(Ordering::SeqCst);
    let finished = RUNS_FINISHED.load(Ordering::SeqCst);

    let running = started - finished;
    println!("running: {running}, finished: {finished}");
}

fn prerun(depth: E, ctxt: &mut Ctxt) {
    // No need to have a trail, we won't backtrack in the prerun.
    ctxt.trail.clear();

    if depth >= threading_depth(ctxt.r) {
        inc_counter(&RUNS_STARTED);

        main_branch(ctxt);

        inc_counter(&RUNS_FINISHED);
        return;
    }

    let Some(i) = select_p(ctxt) else {
        submit_model(ctxt);
        return;
    };

    range_for_each(ctxt.r+1, |v| {
        if infeasible_decision(i, v, ctxt) { return }
        let c = &mut ctxt.clone();

        if prove_pair(i, v, c).is_err() { return }
        if propagate(c).is_err() { return }

        prerun(depth+1, c);
    });
}

fn pos_score(i: E, ctxt: &Ctxt) -> i32 {
    let ii = i as i32;
    let ni = ctxt.r as i32;
    ni - ii
}

pub fn compute_base_score(i: E, ctxt: &Ctxt) -> i32 {
    let class = &ctxt.classes_h[i as usize];
    let cs_score = class.cs.iter().map(|c| score_c(*c)).sum::<i32>();
    let pos_score = pos_score(i, ctxt);

    cs_score + pos_score
}

// returns None if we are done.
fn select_p(ctxt: &Ctxt) -> Option<E> {
    // Should hold here:
    // check_score(ctxt);

    let mut best = E::MAX;
    let mut best_score = -1;

    for i in 0..ctxt.r {
        let class = &ctxt.classes_h[i as usize];
        let score = class.score;
        if (class.value == E::MAX) & (score > best_score) {
            best = i;
            best_score = score;
        }
    }

    if best_score == -1 { None }
    else { Some(best) }
}

fn submit_model(ctxt: &Ctxt) {
    let n = ctxt.r+1;
    {
        let r = ctxt.r;
        let a = ctxt.a;
        let b = ctxt.b;
        let h = |i: E| ctxt.classes_h[i as usize].value;
        let c = (0..r).find(|i| h(*i) == r).unwrap();
        println!("-----");
        dbg!(r);
        dbg!(a);
        dbg!(b);
        dbg!(c);
        for i in 0..r { print!("h({i}) = {}; ", h(i)); }
        println!();

        // facts:
        assert!(h((a+b)%r) == (r-b)%r);
        assert!(c == (a + b + h((r-a)%r))%r);
        assert!((r-a)%r != c);
        if b == dbg!((2*c)%r) { assert!((2*a)%r == b); }

        println!("-----");
    }
    present_model(n as usize, "semitinv_dpll", |x, y| f(x as E, y as E, ctxt) as usize );
}

fn main_branch(ctxt: &mut Ctxt) {
    let Some(i) = select_p(ctxt) else {
        submit_model(ctxt);
        return main_backtrack(ctxt);
    };

    if branch_options(i, 0, ctxt).is_ok() { return main_propagate(ctxt); }
    else { return main_backtrack(ctxt); }
}

// e_start is the next thing to try. If that doesn't work, we iterate from there.
fn branch_options(i: E, e_start: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    for e in e_start..=ctxt.r {
        if infeasible_decision(i, e, ctxt) { continue }

        ctxt.trail.push(TrailEvent::Decision(i, e));
        prove_pair(i, e, ctxt)?;
        return Ok(());
    }
    Err(())
}

fn main_backtrack(ctxt: &mut Ctxt) {
    ctxt.propagate_queue.clear();

    loop {
        let Some(event) = ctxt.trail.pop() else { return };
        match event {
            TrailEvent::Decision(i, e) => {
                if branch_options(i, e+1, ctxt).is_ok() { return main_propagate(ctxt); }
            },
            TrailEvent::DefineClass(i) => {
                let z = std::mem::replace(&mut ctxt.classes_h[i as usize].value, E::MAX);
                ctxt.classes_hinv[z as usize].value = E::MAX;
            },
            TrailEvent::PushCH(i) => {
                let class = &mut ctxt.classes_h[i as usize];
                let c = class.cs.pop().unwrap();
                class.score -= score_c(c);
            }
        }
    }
}

pub fn main_propagate(ctxt: &mut Ctxt) {
    match propagate(ctxt) {
        Ok(()) => return main_branch(ctxt),
        Err(()) => return main_backtrack(ctxt),
    }
}

// we proved h(i) = v.
// i may not be r, but v may.
pub fn prove_pair(i: E, v: E, ctxt: &mut Ctxt) -> Result<(), ()> {
    assert!(i < ctxt.r);

    let i_ref = &mut ctxt.classes_h[i as usize].value;
    let i_v = *i_ref;
    if i_v == v { return Ok(()) }
    if i_v != E::MAX { return Err(()) }

    let hi_ref = &mut ctxt.classes_hinv[v as usize].value;
    if *hi_ref != E::MAX { return Err(()); }

    *i_ref = v;
    *hi_ref = i;
    ctxt.trail.push(TrailEvent::DefineClass(i));
    ctxt.propagate_queue.push((i, v));
    Ok(())
}

pub fn propagate(ctxt: &mut Ctxt) -> Result<(), ()> {
    let r = ctxt.r;
    let n = r+1;
    while let Some((i, v)) = ctxt.propagate_queue.pop() {
        spawn_cs(0, i, v, ctxt)?;

        let len = ctxt.classes_h[i as usize].cs.len();
        for j in 0..len {
            let c = ctxt.classes_h[i as usize].cs[j];
            progress_c(c, ctxt)?;
        }
    }

    Ok(())
}

fn check_score(ctxt: &Ctxt) {
    for i in 0..ctxt.r {
        let actual = compute_base_score(i, ctxt);
        let stored = ctxt.classes_h[i as usize].score;
        assert_eq!(actual, stored);
    }
}

// h(i) = v is disallowed.
fn infeasible_decision(i: E, v: E, ctxt: &Ctxt) -> bool {
    assert!(i != ctxt.r);
    assert!(ctxt.classes_h[i as usize].value == E::MAX);

    if ctxt.classes_hinv[v as usize].value != E::MAX { return true }
    if ctxt.a == v { return true }

    false
}
