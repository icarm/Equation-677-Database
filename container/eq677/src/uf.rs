use crate::*;
use std::collections::{HashMap, HashSet};

type E = usize;
type E2 = (E, E);
type Map = HashMap<E2, E2>;

pub fn find(a: E2, map: &Map) -> E2 {
    if map[&a] == a { return a }
    find(map[&a], map)
}

pub enum Equ {
    E255,
    E677,
}

pub fn merge(a: E2, b: E2, map: &mut Map) {
    let a = find(a, map);
    let b = find(b, map);
    if a > b {
        map.insert(a, b);
    } else {
        map.insert(b, a);
    }
}

pub fn init_uf(m: &MatrixMagma) -> Map {
    let mut map = HashMap::new();
    for p in e2_iter(m.n) {
        map.insert(p, p);
    }

    // It simplifies a bit to merge all the idempotents.
    if m.is_idempotent() {
        for x in 1..m.n {
            merge((0, 0), (x, x), &mut map);
        }
    }

    map
}

pub fn uf_to_vecs(n: usize, map: &Map) -> Vec<Vec<E2>> {
    let mut out = Vec::new();
    for xy in e2_iter(n) {
        let mut part = Vec::new();
        for ab in e2_iter(n) {
            if find(ab, &map) == xy {
                part.push(ab);
            }
        }
        if !part.is_empty() { out.push(part); }
    }
    out
}

pub fn uf(m: &MatrixMagma, eq: Equ) -> Vec<Vec<E2>> {
    let mut map = init_uf(m);

    for t in eq.traces(m) {
        let fst = t[0];
        for other in &t[1..] {
            merge(fst, *other, &mut map);
        }
    }

    uf_to_vecs(m.n, &map)
}

pub fn e2_iter(n: usize) -> impl Iterator<Item=E2> + 'static {
    (0..n).flat_map(move |x| (0..n).map(move |y| (x, y)))
}

pub fn e2_nonidem_iter(n: usize) -> impl Iterator<Item=E2> + 'static {
    (0..n).flat_map(move |x| (0..n).map(move |y| (x, y))).filter(|(x, y)| x != y)
}

fn pick_random(magmas: &[MatrixMagma]) -> MatrixMagma {
    use rand::prelude::*;

    let mut rng = rand::rng();
    let i: usize = rng.random_range(0..magmas.len());
    magmas[i].clone()
}

pub fn uf_search() {
    for (_, m) in db() {
        if m.n < 2 { continue }
        if m.n > 50 { continue }
        let uf: Vec<Vec<E2>> = uf(&m, Equ::E677);

        for k in 2..(200/m.n+2) {
            let mut mag_opts = Vec::new();
            for (_, m) in db() {
                if m.n != k { continue }
                mag_opts.push(m);
            }
            if mag_opts.is_empty() { continue }

            let mut mags = Vec::new();
            for _ in 0..uf.len() {
                let m2 = pick_random(&mag_opts);
                let m2 = m2.shuffle();
                mags.push(m2);
            }

            let mag = GenericMagma {
                elems: itertools::iproduct!(0..m.n, 0..k).collect(),
                f_def: |x: E2, y: E2| {
                    let i = uf.iter().position(|u| u.contains(&(x.0, y.0))).unwrap();
                    let mm = &mags[i];

                    let z0 = m.f(x.0, y.0);
                    let z1 = mm.f(x.1, y.1);

                    (z0, z1)
                },
            }.to_matrix();
            assert!(mag.is677());
            present_model(mag.n, "kk", |x, y| mag.f(x, y));
        }
    }
}

pub fn partial_completion_search() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    for (name, m) in db() {
        println!("====================");
        println!("{name}:");
        println!("====================");
        for x in 0..m.n {
            for y in 0..m.n {
                // These are boring:
                if x == y && m.f(x, x) == x { continue }

                let mm = partial_677_magma((x, y), &m);
                if mm.count_defined() > m.n*m.n/2 {
                    let mm = mm.canonicalize2();
                    if set.insert(mm.clone()) {
                        c_complete(&mm);
                    }
                }
            }
        }
    }
}

pub fn partial_dump_magma(m: &MatrixMagma) {
    let mut set = HashSet::new();

    for x in 0..m.n {
        for y in 0..m.n {
            // These are boring:
            if x == y && m.f(x, x) == x { continue }

            let mm = partial_677_magma((x, y), &m);
            let mm = shrink_partial_to_fit(&mm);
            let mm = mm.canonicalize2();
            if set.insert(mm.clone()) {
                println!("({x}, {y}) generate:");
                mm.dump();
            }
        }
    }
}

pub fn partial_dump_db() {
    for (name, m) in db() {
        println!();
        println!("====================");
        println!("{name}:");
        println!("====================");

        partial_dump_magma(&m);
    }
}

pub fn partial_677_magma(xy: E2, m: &MatrixMagma) -> MatrixMagma {
    let mut v = vec![xy];

    'outer: loop {
        let mut dirty = false;
        for (x, y) in v.clone() {
            for xy2 in trace677(m, x, y) {
                if !v.contains(&xy2) {
                    v.push(xy2);
                    dirty = true;
                }
            }
        }
        if !dirty { break }
    }

    let mut out = MatrixMagma::undefined(m.n);
    for (x, y) in v {
        let z = m.f(x, y);
        out.set_f(x, y, z);
    }

    out
}

// x*(y*((x*y)*x))
pub fn trace677(m: &MatrixMagma, x: E, y: E) -> [E2; 4] {
    let xy = m.f(x, y);
    let xyx = m.f(xy, x);
    let yxyx = m.f(y, xyx);
    [(x, y), (xy, x), (y, xyx), (x, yxyx)]
}

// ((x*x)*x)*x
pub fn trace255(m: &MatrixMagma, x: E) -> [E2; 3] {
    let xx = m.f(x, x);
    let xxx = m.f(xx, x);
    [(x, x), (xx, x), (xxx, x)]
}

impl Equ {
    // A trace always starts with the source pair, whose m.f-computation you are tracing.
    fn traces(self, m: &MatrixMagma) -> Vec<Vec<E2>> {
        let mut out = Vec::new();
        match self {
            Equ::E677 => {
                for x in 0..m.n {
                    for y in 0..m.n {
                        let v = trace677(m, x, y).into_iter().collect();
                        out.push(v);
                    }
                }
            },
            Equ::E255 => {
                for x in 0..m.n {
                    let v = trace255(m, x).into_iter().collect();
                    out.push(v);
                }
            },
        }
        out
    }
}

// Finds the elements that are actually used in this magma.
pub fn elements_of_partial(m: &MatrixMagma) -> Vec<usize> {
    let mut v = Vec::new();
    for x in 0..m.n {
        for y in 0..m.n {
            let z = m.f(x, y);
            if z != usize::MAX {
                if !v.contains(&x) { v.push(x); }
                if !v.contains(&y) { v.push(y); }
                if !v.contains(&z) { v.push(z); }
            }
        }
    }
    v
}

pub fn shrink_partial_to_fit(m: &MatrixMagma) -> MatrixMagma {
    let v = elements_of_partial(m);
    GenericMagma {
        elems: v,
        f_def: |x, y| m.f(x, y),
    }.to_matrix()
}

// Constructs a partial model db.
pub fn partial_db() -> HashSet<MatrixMagma> {
    let mut set: HashSet<MatrixMagma> = HashSet::new();
    for (name, m) in db() {
        println!("Looking in {name}:");
        for x in 0..m.n {
            for y in 0..m.n {
                let mm = partial_677_magma((x, y), &m);
                let mm = shrink_partial_to_fit(&mm);
                let mm = mm.canonicalize2();
                if set.insert(mm.clone()) {
                    println!("size={}, defined={}:", mm.n, mm.count_defined());
                    mm.dump();
                }
            }
        }
    }
    set
}

pub fn partial_diag_db() -> HashSet<MatrixMagma> {
    let mut set: HashSet<MatrixMagma> = HashSet::new();
    for (name, m) in db() {
        println!("Looking in {name}:");
        for x in 0..m.n {
            let mm = partial_677_magma((x, x), &m);
            let mm = shrink_partial_to_fit(&mm);
            let mm = mm.canonicalize2();
            if set.insert(mm.clone()) {
                println!("size={}, defined={}:", mm.n, mm.count_defined());
                mm.dump();
            }
        }
    }
    set
}

pub fn rebuild_c_classes(m: &MatrixMagma, map: &mut Map) {
    loop {
        let mut dirty = false;
        for p in e2_iter(m.n) {
            let r = find(p, map);
            let t1 = trace677(m, p.0, p.1);
            let t2 = trace677(m, r.0, r.1);
            for (p1, p2) in t1.into_iter().zip(t2.into_iter()) {
                if find(p1, map) != find(p2, map) {
                    dirty = true;
                    merge(p1, p2, map);
                }
            }
        }
        if !dirty { break }
    }
}

fn get_colors() -> Vec<String> {
    let mut combinations = Vec::new();

    // Define the valid color range (excluding the black codes identified previously)
    let valid_indices: Vec<u8> = (0..=255)
        .filter(|&i| !matches!(i, 0 | 8 | 16 | 232..=236))
        .collect();

    for &fg in &valid_indices {
        for &bg in &valid_indices {
            // Skip if foreground and background are the same
            if fg == bg {
                continue;
            }
            // Format: \x1b[38;5;FG;48;5;BGm
            combinations.push(format!("\x1b[38;5;{};48;5;{}m", fg, bg));
        }
    }

    shuf(Some(0), &mut combinations);
    combinations
}

pub fn leaders(n: usize, uf: &Map) -> Vec<E2> {
    e2_iter(n).filter(|x| *x == find(*x, uf)).collect()
}

pub fn colored_dump(m: &MatrixMagma, map: &Map) {
    let mut div = m.n.ilog10() as usize + 2;
    if FIXED_WIDTH { div = div.max(3); }

    let colors = get_colors();
    let l = leaders(m.n, map);
    let mut colormap = HashMap::new();
    if l.len() > colors.len() {
        println!("Warning: duplicate color usage!");
    }
    for (i, ll) in l.iter().enumerate() {
        colormap.insert(ll, colors[i%colors.len()].clone());
    }

    for x in 0..m.n {
        for y in 0..m.n {
            let z = m.f(x, y);
            let ll = find((x, y), &map);
            print!("{}", &colormap[&ll]);
            if z == usize::MAX {
                print!("{:<width$}", '-', width = div);
            } else {
                print!("{:<width$}", fmt(z), width = div);
            }
            print!("\x1b[0m");
        }
        println!();
    }

    println!("677 color traces:");
    for (i, ll) in l.iter().enumerate() {
        for a in trace677(&m, ll.0, ll.1) {
            let a = find(a, map);
            let vv = l.iter().position(|xx| *xx == a).unwrap();
            print!("{}{vv}", &colormap[&a]);
        }
        print!("\x1b[0m ");
        if i%4 == 3 { println!(); }
    }

    println!("\n255 color traces:");
    let mut traces255: Vec<Vec<usize>> = Vec::new();
    for x in 0..m.n {
        let t = trace255(&m, x).into_iter().map(|a| {
            let a = find(a, map);
            let vv = l.iter().position(|xx| *xx == a).unwrap();
            vv
        }).collect();
        if traces255.contains(&t) { continue }
        traces255.push(t.clone());
    }
    for (i, t) in traces255.iter().enumerate() {
        for vv in t {
            let a = l[*vv];
            print!("{}{vv}", &colormap[&a]);
        }
        print!("\x1b[0m ");
        if i%4 == 3 { println!(); }
    }
    println!();
}

fn shuf<T>(seed: Option<u8>, v: &mut Vec<T>) {
    use rand::seq::SliceRandom;
    use rand::rng;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    if let Some(seed) = seed {
        let seed = [seed; 32];
        let mut rng = StdRng::from_seed(seed);
        v.shuffle(&mut rng);
    } else {
        let mut rng = rng();
        v.shuffle(&mut rng);
    }
}

pub fn random_classes(m: &MatrixMagma) -> Map {
    let mut uf = init_uf(m);
    rebuild_c_classes(&m, &mut uf);

    'outer: for i in 0.. {
        // dbg!(i);
        let mut lv1 = leaders(m.n, &uf);
        assert!(lv1.len() > 1);
        let mut lv2 = lv1.clone();
        shuf(None, &mut lv1);
        shuf(None, &mut lv2);

        for l1 in lv1.iter() {
            for l2 in lv2.iter() {
                if l1 == l2 { continue }
                let mut uf2 = uf.clone();
                merge(*l1, *l2, &mut uf2);
                rebuild_c_classes(&m, &mut uf2);
                if leaders(m.n, &uf2).len() > 2 {
                    uf = uf2;
                    continue 'outer;
                }
            }
        }
        // println!("Stopped iteration with i={i}");
        break;
    }

    uf
}

fn useful_classes_from(c1: E2, c2: E2, m: &MatrixMagma) -> Vec<Map> {
    let mut uf = init_uf(m);
    for p in e2_nonidem_iter(m.n) {
        if p == c2 { break }
        merge(c1, p, &mut uf);
    }
    rebuild_c_classes(m, &mut uf);
    if find(c1, &uf) == find(c2, &uf) { return Vec::new() }

    let mut opts = vec![uf];
    for p in e2_nonidem_iter(m.n) {
        for map in std::mem::take(&mut opts) {
            let p_lead = find(p, &map);
            if p_lead == find(c1, &map) || p_lead == find(c2, &map) {
                opts.push(map);
                continue
            }
            for o in [c1, c2] {
                let mut map = map.clone();
                merge(o, p, &mut map);
                rebuild_c_classes(m, &mut map);
                if find(c1, &map) != find(c2, &map) {
                    opts.push(map);
                }
            }
        }
    }
    opts
}

pub fn useful_classes(m: &MatrixMagma) -> Vec<Map> {
    let mut opts = vec![];
    let c1 = (0, 1);
    for c2 in e2_nonidem_iter(m.n) {
        if c1 == c2 { continue }
        opts.extend(useful_classes_from(c1, c2, m));
    }

    opts
}

pub fn analyze_useful_classes(m: &MatrixMagma) {
    for uf in useful_classes(&m) {
        colored_dump(&m, &uf);
        for l in leaders(11, &uf) {
            if l.0 == l.1 { continue }
            print!("{l:?}: ");
            for x in e2_nonidem_iter(11) {
                if find(x, &uf) == l {
                    print!("{x:?}, ");
                }
            }
            dbg!(trace677(&m, l.0, l.1).into_iter().map(|a| find(a, &uf)).collect::<Vec<_>>());
            println!();
        }
        println!();
    }
}

pub fn color_dump_small_magmas() {
    let show = |m: MatrixMagma, name: M, max: usize| {
        assert!(db_intern(&m).0 == name);
        println!("\nShowing {}:\n", name);
        loop {
            let uf = random_classes(&m);
            let cnt = leaders(m.n, &uf).len();
            if dbg!(cnt) <= max {
                colored_dump(&m, &uf);
                // coloring_to_python(name, &m, &uf);
                break;
            }
        }
    };

    let m = MatrixMagma::by_fn(7, |x, y| (4*x+y)%7);
    show(m, M(7, 0), 10);

    let m = MatrixMagma::by_fn(7, |x, y| (4*x+3*y)%7);
    show(m, M(7, 1), 10);

    let m = {
        let p = 3;
        type Complex = (usize, usize);
        fn mul(x: Complex, y: Complex, p: usize) -> Complex {
            ((x.0*y.0 + (p*p - x.1*y.1))%p, (x.0*y.1 + x.1*y.0)%p)
        }

        fn add(x: Complex, y: Complex, p: usize) -> Complex {
            ((x.0 + y.0)%p, (x.1 + y.1)%p)
        }

        let f_def = |x: Complex, y: Complex| -> Complex {
            let lhs = mul(x, (1, 0), p);
            let rhs = mul(y, (2, 1), p);
            add(lhs, rhs, p)
        };

        GenericMagma {
            elems: (0..p).map(move |x| (0..p).map(move |y| (x, y))).flatten().collect(),
            f_def,
        }.to_matrix()
    };
    show(m, M(9, 0), 11);

    let m = MatrixMagma::by_fn(13, |x, y| (x*9 + y*11)%13);
    show(m, M(13, 0), 15);

    let m = M(16, 1).get();
    show(m, M(16, 1), 18);

    let m = M(19, 0).get();
    show(m, M(19, 0), 21);

    let m = M(19, 1).get();
    show(m, M(19, 1), 21);

    let m = MatrixMagma::by_fn(31, |x, y| (5*x + 27*y + 1)%31);
    show(m, M(31, 0), 31);
}

pub fn coloring_to_python(name: M, m: &MatrixMagma, uf: &Map) {
    let fmt_name = format!("magmadef_{}_{}", name.0, name.1);
    let l = leaders(m.n, uf);
    println!();
    println!();
    println!("def {fmt_name}():");
    println!("    name = \"{}/{}\"", name.0, name.1);
    println!("    n = {}", name.0);
    println!("    m = dict()");
    println!("    c = dict()");
    print!("    ");
    for i in 0..m.n {
        for j in 0..m.n {
            print!("m[({i}, {j})] = {}; ", m.f(i, j));
        }
    }
    println!();
    print!("    ");
    for i in 0..m.n {
        for j in 0..m.n {
            let z = find((i, j), uf);
            let z = l.iter().position(|x| *x == z).unwrap();
            print!("c[({i}, {j})] = {z}; ");
        }
    }

    println!();
    print!("    t = [");
    for (i, ll) in l.iter().enumerate() {
        print!("(");
        for a in trace677(&m, ll.0, ll.1) {
            let a = find(a, uf);
            let vv = l.iter().position(|xx| *xx == a).unwrap();
            print!("{vv}, ");
        }
        print!("), ");
    }
    println!("]");
    println!("    return n, name, m, c, t");
    println!();
    println!();
}
