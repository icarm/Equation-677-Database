use crate::*;
use std::fmt::Write;

type H = Vec<usize>;

pub fn tinv_chk(m: &MatrixMagma) -> bool {
    let n = m.n;
    let g = m.autom_group();
    g.iter().any(|p| {
        let cycles = bij_to_cycles(0, m.n, false, |i| p[i]);
        cycles.len() == 1
    })
}

pub fn tinv_infos(m: &MatrixMagma) -> Option<Vec<H>> {
    let n = m.n;
    let g = m.autom_group();

    let mut outs = Vec::new();
    for p in g.iter() {
        let mut cycles = bij_to_cycles(0, m.n, false, |i| p[i]);
        if cycles.len() != 1 { continue }
        let p = cycles.pop().unwrap();

        let m2 = m.permute(p);

        let h: H = (0..n).map(|i| m2.f(0, i)).collect();
        if outs.contains(&h) { continue }

        // checks:
        {
            let m3 = MatrixMagma::by_fn(n, |x, y| (x + h[(y+n-x)%n])%n);
            assert!(m3 == m2);
            // assert!(m3.canonicalize2() == m.canonicalize2());
        }
        outs.push(h);
    }

    if outs.is_empty() { return None }
    outs.sort();
    Some(outs)
}

pub fn tinv_dump() {
    for name in TINV {
        println!("{name}:");
        let m = db_get(*name);
        for h in tinv_infos(&m).unwrap() {
            println!("h = {}", draw_cycle_string(0, m.n, |i| h[i]));
        }
    }
}
