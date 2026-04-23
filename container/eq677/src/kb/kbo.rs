use crate::*;

// returns whether we can prove s > t:
pub fn kbo(s: &Term, t: &Term) -> bool {
    if !var_gte(s, t) { return false }
    let ws = weight(s);
    let wt = weight(t);
    if ws > wt { return true }
    if ws < wt { return false }

    match (s, t) {
        (Term::F(_), Term::E(_)) => true,
        (Term::E(es), Term::E(et)) => es > et,
        (Term::F(bs), Term::F(bt)) => {
            kbo(&bs[0], &bt[0]) ||
            (bs[0] == bt[0] && kbo(&bs[1], &bt[1]))
        },
        _ => false,
    }
}

// checks whether every variable from t occurs at least equally often in s.
fn var_gte(s: &Term, t: &Term) -> bool {
    (0..var_c(t)).all(|v|
        var_count(s, v) >= var_count(t, v)
    )
}

// returns 0 if t contains no variables.
// returns v+1 if v is the highest variable in t.
fn var_c(t: &Term) -> V {
    match t {
        Term::V(v) => v+1,
        Term::E(_) => 0,
        Term::F(b) => var_c(&b[0]).max(var_c(&b[1])),
    }
}

// Counts how often the variable v occurs in t.
fn var_count(t: &Term, v: V) -> u32 {
    match t {
        Term::V(v2) => (v == *v2) as u32,
        Term::E(_) => 0,
        Term::F(b) => var_count(&b[0], v) + var_count(&b[1], v),
    }
}

fn weight(t: &Term) -> u32 {
    match t {
        Term::V(_) => 1,
        Term::E(_) => 1,
        Term::F(b) => weight(&b[0]) + weight(&b[1]) + 1,
    }
}
