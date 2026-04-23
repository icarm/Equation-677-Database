use crate::*;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use lazy_static::lazy_static;

lazy_static! {
    static ref DB_REF: RwLock<Vec<DB>> = RwLock::new(init_db());
}

include!(concat!(env!("OUT_DIR"), "/db_sources.rs"));

pub fn db_search() {
    for (name, m) in db() {
        let s = format!("db: {name}");
        present_model(m.n as usize, &s, |x, y| m.f(x, y));
    }
}

pub fn db_cart_search() {
    for (name1, m1) in db() {
        for (name2, m2) in db() {
            if m1.n * m2.n > 1000 { continue }
            if m1.n <= 1 { continue }
            if m2.n <= 1 { continue }

            let m = cartesian(&m1, &m2);
            let s = format!("db-cartesian: {name1} ⨯ {name2}");
            present_model(m.n, &s, |x, y| m.f(x, y));
        }
    }
}

pub fn db() -> Vec<(M, MatrixMagma)> {
    let mut out = Vec::new();
    let db = DB_REF.read().unwrap();
    for (i, dbs) in db.iter().enumerate() {
        for (j, m) in dbs.magmas.iter().enumerate() {
            out.push((M(i, j), m.clone()));
        }
    }
    out
}

#[test]
fn db_canon() {
    for (name, m) in db() {
        if m.canonicalize2() != m {
            panic!("magma {name} is not canonicalized!");
        }
    }
}

#[test]
fn db_naming() {
    for (name, m) in db() {
        let n = m.n;
        if name.0 != n {
            println!("M is {name}, but n={n}");
            assert!(false);
        }
    }
}

#[test]
fn db_unique() {
    use std::collections::HashMap;

    let mut map: HashMap<MatrixMagma, M> = HashMap::default();
    for (name, m) in db() {
        if let Some(name2) = map.insert(m, name.clone()) {
            panic!("Redundant Magmas {name} = {name2}");
        }
    }
}

#[test]
fn db_is677() {
    for (_, m) in db() {
        assert!(m.is677());
    }
}

#[test]
fn db_is255() {
    for (_, m) in db() {
        assert!(m.is255());
    }
}

pub fn init_db() -> Vec<DB> {
    let mut handle = Vec::new();
    for ((i, j), s) in DB_SOURCES.iter() {
        let (i, j) = (*i, *j);
        let m = MatrixMagma::parse(s);
        while handle.len() <= i {
            handle.push(DB {
                magmas: Default::default(),
                map: Default::default(),
            });
        }
        handle[i].magmas.push(m.clone());
        handle[i].map.insert(m, j);
    }
    handle
}

// Each magma size has its own DB.
#[derive(Clone)]
struct DB {
    magmas: Vec<MatrixMagma>,
    map: HashMap<MatrixMagma, usize>,
}

pub fn db_get(M(i, j): M) -> MatrixMagma {
    let handle = DB_REF.read().unwrap();
    handle[i].magmas[j].clone()
}

pub fn db_intern(m: &MatrixMagma) -> (M, /*fresh*/ bool) {
    let m = m.canonicalize2();
    let i = m.n;
    if i > 100 { panic!("Interning is only supported up to n=100"); }

    let handle = DB_REF.read().unwrap();
    if let Some(j) = handle[i].map.get(&m) { return (M(i, *j), false); }
    drop(handle);

    let mut handle = DB_REF.write().unwrap();
    if let Some(j) = handle[i].map.get(&m) { return (M(i, *j), false); }
    while handle.len() <= i {
        handle.push(DB {
            magmas: Default::default(),
            map: Default::default(),
        });
    }
    let j = handle[i].magmas.len();
    handle[i].magmas.push(m.clone());
    let s = m.to_string();
    handle[i].map.insert(m, j);
    let path = format!("db/{i}/{j}");
    std::fs::write(&path, s).unwrap();
    (M(i, j), true)
}


#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
// The name of a magma (up to isomorphism).
pub struct M(pub usize, pub usize);

impl M {
    pub fn get(self) -> MatrixMagma {
        db_get(self)
    }
}

use std::fmt::*;

impl Display for M {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl Debug for M {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "M({}, {})", self.0, self.1)
    }
}

// models of the form f(x, y) = (ax + by + c)%n in Z/pZ.
pub static AFFINE_MODELS_MOD: &[M] = &[
    M(5, 0),
    M(7, 0), M(7, 1),
    M(11, 0), M(11, 1), M(11, 2), M(11, 3),
    M(13, 0),
    M(19, 0), M(19, 1),
    M(31, 0), M(31, 1), M(31, 2), M(31, 3), M(31, 4), M(31, 5),
    M(35, 0), M(35, 1),
    M(37, 0), M(37, 1),
    M(41, 7), M(41, 8), M(41, 9), M(41, 10),
    M(43, 0), M(43, 1), M(43, 2), M(43, 3),
    M(49, 4), M(49, 5),
    M(55, 0), M(55, 1), M(55, 2), M(55, 3),
    M(61, 0), M(61, 1), M(61, 2), M(61, 3),
    M(65, 0),
    M(67, 0), M(67, 1),
    M(71, 0), M(71, 1), M(71, 2), M(71, 3),
    M(73, 0), M(73, 1),
    M(77, 0), M(77, 1), M(77, 2), M(77, 3), M(77, 4), M(77, 5), M(77, 6), M(77, 7),
    M(91, 0), M(91, 1),
    M(95, 0), M(95, 1),
    M(97, 0), M(97, 1),

];

// could miss some models with p=2, or p=3 (and cartesian products thereof).
// affine (there called "linear") models in the sense of Lemma 13.3: https://teorth.github.io/equational_theories/blueprint/677-chapter.html
pub static AFFINE_MODELS_13_3: &[M] = &[
    M(5, 0),
    M(7, 0), M(7, 1),
    M(9, 0),
    M(11, 0), M(11, 1), M(11, 2), M(11, 3),
    M(13, 0),
    M(16, 0), M(16, 1),
    M(19, 0), M(19, 1),
    M(25, 3),
    M(31, 0), M(31, 1), M(31, 2), M(31, 3), M(31, 4), M(31, 5),
    M(35, 0), M(35, 1),
    M(37, 0), M(37, 1),
    M(41, 7), M(41, 8), M(41, 9), M(41, 10),
    M(43, 0), M(43, 1), M(43, 2), M(43, 3),
    M(45, 0),
    M(49, 0), M(49, 1), M(49, 2), M(49, 3), M(49, 4), M(49, 5),
    M(55, 0), M(55, 1), M(55, 2), M(55, 3),
    M(61, 0), M(61, 1), M(61, 2), M(61, 3),
    M(63, 0), M(63, 1),
    M(65, 0),
    M(67, 0), M(67, 1),
    M(71, 0), M(71, 1), M(71, 2), M(71, 3),
    M(73, 0), M(73, 1),
    M(77, 0), M(77, 1), M(77, 2), M(77, 3), M(77, 4), M(77, 5), M(77, 6), M(77, 7),
    M(80, 0), M(80, 1),
    M(81, 0), M(81, 1),
    M(91, 0), M(91, 1),
    M(95, 0), M(95, 1),
    M(97, 0), M(97, 1),
    M(99, 0), M(99, 1), M(99, 2), M(99, 3),
];

// not a complete list, obtained via glue5_chk from the db.
// might get outdated repeatedly. Using glue5_chk to check for gluing models is safer.
pub static GLUE5_MODELS: &[M] = &[
    M(5, 0),
    M(21, 0), M(21, 1),
    M(25, 0), M(25, 1), M(25, 2), M(25, 3), M(25, 4), M(25, 5), M(25, 6), M(25, 7), M(25, 9), M(25, 10), M(25, 11), M(25, 12), M(25, 13), M(25, 14), M(25, 15), M(25, 16), M(25, 17), M(25, 18), M(25, 19), M(25, 20), M(25, 21), M(25, 22), M(25, 26),
    M(41, 0), M(41, 1), M(41, 2), M(41, 3), M(41, 4), M(41, 5), M(41, 6), M(41, 11),
    M(61, 4), M(61, 5), M(61, 6), M(61, 7), M(61, 8), M(61, 9), M(61, 10), M(61, 11), M(61, 12), M(61, 13), M(61, 14), M(61, 15), M(61, 16), M(61, 17), M(61, 18), M(61, 19), M(61, 20), M(61, 21), M(61, 22), M(61, 23), M(61, 24), M(61, 25), M(61, 26), M(61, 27), M(61, 28), M(61, 29), M(61, 30), M(61, 31), M(61, 32), M(61, 33), M(61, 34),
    M(65, 7), M(65, 8), M(65, 9), M(65, 10), M(65, 11), M(65, 12), M(65, 13), M(65, 14), M(65, 15),
    M(81, 4), M(81, 5), M(81, 6), M(81, 7), M(81, 8), M(81, 9), M(81, 10), M(81, 11), M(81, 12), M(81, 13), M(81, 14), M(81, 15), M(81, 16), M(81, 17), M(81, 18), M(81, 19),
    M(85, 0), M(85, 1), M(85, 2), M(85, 3), M(85, 4), M(85, 5), M(85, 6),
];

pub static LINEAR_EXTENSIONS: &[M] = &[
    M(25, 8),
    M(35, 12), M(35, 13), M(35, 14), M(35, 15), M(35, 16), M(35, 17), M(35, 18), M(35, 19), M(35, 20), M(35, 21),
    M(45, 1), M(45, 2), M(45, 3), M(45, 4),
    M(49, 7), M(49, 8), M(49, 9), M(49, 10), M(49, 11), M(49, 12),
    M(63, 2), M(63, 3), M(63, 4), M(63, 5),
    M(65, 1), M(65, 2), M(65, 3), M(65, 4), M(65, 5), M(65, 6),
    M(77, 8), M(77, 9), M(77, 10), M(77, 11), M(77, 12), M(77, 13), M(77, 14), M(77, 15), M(77, 16), M(77, 17), M(77, 18), M(77, 19), M(77, 20), M(77, 21), M(77, 22), M(77, 23), M(77, 24), M(77, 25), M(77, 26), M(77, 27), M(77, 28), M(77, 29), M(77, 30), M(77, 31), M(77, 32), M(77, 33), M(77, 34), M(77, 35), M(77, 36), M(77, 37), M(77, 38), M(77, 39), M(77, 40), M(77, 41), M(77, 42), M(77, 43), M(77, 44), M(77, 45), M(77, 46), M(77, 47), M(77, 48), M(77, 49), M(77, 50),
    M(81, 2), M(81, 3),
    M(91, 2), M(91, 3), M(91, 4), M(91, 5),
    M(99, 4), M(99, 5), M(99, 6), M(99, 7), M(99, 8), M(99, 9), M(99, 10), M(99, 11), M(99, 12), M(99, 13), M(99, 14), M(99, 15), M(99, 16), M(99, 17), M(99, 18), M(99, 19), M(99, 20), M(99, 21), M(99, 22), M(99, 23), M(99, 24), M(99, 25), M(99, 26), M(99, 27), M(99, 28), M(99, 29), M(99, 30), M(99, 31), M(99, 32), M(99, 33),
];

// state 14.12.2025, all magmas from the db of size < 80 were checked whether they were tinv (using chk_tinv2), and added to TINV accordingly.
pub static TINV: &[M] = &[
    M(5, 0),
    M(11, 0), M(11, 1), M(11, 2), M(11, 3),
    M(21, 0),
    M(29, 0),
    M(31, 0), M(31, 1), M(31, 2), M(31, 3), M(31, 4),
    M(41, 0), M(41, 1), M(41, 2), M(41, 3), M(41, 4), M(41, 5), M(41, 6), M(41, 7), M(41, 8), M(41, 9), M(41, 10), M(41, 11),
    M(55, 0), M(55, 1), M(55, 2), M(55, 3),
    M(61, 0), M(61, 1), M(61, 2), M(61, 3), M(61, 4), M(61, 5), M(61, 6), M(61, 7), M(61, 8), M(61, 9), M(61, 10), M(61, 11), M(61, 12), M(61, 13), M(61, 14), M(61, 15), M(61, 16), M(61, 17), M(61, 18), M(61, 19), M(61, 20), M(61, 21), M(61, 22), M(61, 23), M(61, 24), M(61, 25), M(61, 26), M(61, 27), M(61, 28), M(61, 29), M(61, 30), M(61, 31), M(61, 32), M(61, 33), M(61, 34),
    M(65, 7), M(65, 8), M(65, 9), M(65, 10), M(65, 11), M(65, 12), M(65, 13), M(65, 14), M(65, 15),
    M(71, 0), M(71, 1), M(71, 2), M(71, 3),
    M(81, 4), M(81, 5), M(81, 6), M(81, 7),
];

pub fn find_affine_models() {
    for p in 0..101 {
        affine_run(p);
    }
    for p in 0..11 {
        affmat_run(p);
    }

    { // Some other models for p=2 or p=3 could be missing.
        let add = |name| {
            let m = db_get(name);
            present_model(m.n, "add", |x, y| m.f(x, y));
        };
        add(M(16, 0));
        add(M(16, 1));
        add(M(81, 1));
    }

    let d: Vec<_> = get_present_db().into_iter().map(db_get).collect();
    for m1 in d.iter() {
        for m2 in d.iter() {
            let m = cartesian(m1, m2);
            if m1.n * m2.n > 100 { continue }
            present_model(m.n, "add", |x, y| m.f(x, y));
        }
    }

    dump_present_db();
}

pub fn find_tinv_models() {
    for (name, m) in db() {
        if name.0 < 2 { continue }
        println!("{name}");
        if tinv_chk2(&m) {
            present_model(m.n, "tinv-chk", |x, y| m.f(x, y));
        }
    }

    dump_present_db();
}

pub fn find_gluing_models() {
    for (name, m) in db() {
        if name.0 < 2 { continue }
        println!("{name}");
        if glue5_chk(&m) {
            present_model(m.n, "glue5-chk", |x, y| m.f(x, y));
        }
    }

    dump_present_db();
}


pub fn dump_potentially_interesting_models() {
    for (name, _) in db() {
        if name.0 < 2 { continue; }
        if AFFINE_MODELS_13_3.contains(&name) { continue; }
        if GLUE5_MODELS.contains(&name) { continue; }
        if LINEAR_EXTENSIONS.contains(&name) { continue; }
        println!("{name}");
    }
}

#[test]
fn affine_subset() {
    for x in AFFINE_MODELS_MOD {
        assert!(AFFINE_MODELS_13_3.contains(x));
    }
}
