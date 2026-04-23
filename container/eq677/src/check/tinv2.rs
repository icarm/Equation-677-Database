use crate::*;
use z3::{ast::Int, *};
use std::fmt::Write;

pub fn tinv_chk2(m: &MatrixMagma) -> bool {
    let n = m.n;

    let mut s = String::new();
    write!(s, "(declare-datatypes () ((E ");
    for x in 0..n {
        write!(s, "e{x} ");
    }
    writeln!(s, ")))");
    writeln!(s, "(declare-fun phi (E) E)");
    writeln!(s, "(declare-fun f (E E) E)");

    // define f.
    for x in 0..n {
        for y in 0..n {
            let z = m.f(x, y);
            writeln!(s, "(assert (= e{z} (f e{x} e{y})))");
        }
    }

    // phi is only one big cycle:
    for x in 0..n {
        // This gives less info, but is empirically faster.
        if x != 0 { continue }

        write!(s, "(assert (distinct ");

        for i in 0..n {
            let mut st = format!("e{x}");
            for _ in 0..i {
                st = format!("(phi {st})");
            }
            write!(s, "{st} ");
        }
        writeln!(s, "))");
    }

    // phi injective.
    write!(s, "(assert (distinct ");
    for x in 0..n {
        write!(s, "(phi e{x}) ");
    }
    writeln!(s, "))");

    // phi homomorphism:
    for x in 0..n {
        for y in 0..n {
            let z = m.f(x, y);
            writeln!(s, "(assert (= (phi e{z}) (f (phi e{x}) (phi e{y}))))");
        }
    }

    writeln!(s, "(check-sat)");
    writeln!(s, "(get-model)");

    // println!("{s}");

    let solver = Solver::new();
    solver.from_string(s);

    matches!(solver.check(), SatResult::Sat)
}
