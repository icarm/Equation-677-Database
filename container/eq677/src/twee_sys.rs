use crate::*;

use std::process::{Command, Stdio};
use std::fmt::Write;
use std::io::Write as IoWrite;
use egg::Symbol;

#[derive(Clone)]
pub enum TTerm {
    Var(Symbol),
    Fun(Symbol, Box<[TTerm]>),
}

pub type Equation = (TTerm, TTerm);
pub type Diseq = (TTerm, TTerm);

// returning Err means unsat.
pub fn twee_call(eqs: &[Equation], diseq: &[Diseq], max_cps: u32) -> Result<Vec<Equation>, ()> {
    let mut cmd = Command::new("twee").args(&["-", "--max-cps", &max_cps.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let input = stringify(eqs, diseq);

    // println!("in:\n\n{input}\n\n");

    let mut stdin = cmd.stdin.take().unwrap();
    write!(stdin, "{}", input).unwrap();
    drop(stdin);

    let out = cmd.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(&out.stdout);

    // println!("out:\n\n{out}\n\n");

    twee_parse(&out)
}

// =========
// stringify:
// =========

fn stringify(eqs: &[Equation], diseq: &[Diseq]) -> String {
    // this keeps twee running for a bit.
    let mut s = String::from("cnf(a,axiom, private_a != private_b).\n");

    for (lhs, rhs) in eqs {
        writeln!(&mut s, "cnf(a,axiom, {} = {}).", stringify_term(lhs), stringify_term(rhs)).unwrap();
    }

    for (lhs, rhs) in diseq {
        writeln!(&mut s, "cnf(a,axiom, {} != {}).", stringify_term(lhs), stringify_term(rhs)).unwrap();
    }

    s
}

pub fn stringify_term(t: &TTerm) -> String {
    match t {
        TTerm::Var(v) => v.to_string(),
        TTerm::Fun(symb, args) => {
            if args.is_empty() { return symb.to_string() };

            let mut s = format!("{symb}(");
            for (i, a) in args.iter().enumerate() {
                s.push_str(&stringify_term(a));
                if i != args.len() - 1 {
                    s.push_str(", ");
                }
            }
            s.push(')');
            s
        },
    }
}

// =========
// parse:
// =========

fn twee_parse(s: &str) -> Result<Vec<Equation>, ()> {
    if s.contains("RESULT: Unsatisfiable") { return Err(()) }

    let s = s.split("Here is the final rewrite system:\n").last().unwrap();
    let s = s.split("RESULT: GaveUp").next().unwrap();

    let mut out = Vec::new();

    for line in s.split("\n") {
        if line.trim().is_empty() { continue }

        let mut it = 
            if line.contains("->") { line.split("->") }
            else                   { line.split("=")  };
        let a = parse_tterm(it.next().unwrap());
        let b = parse_tterm(it.next().unwrap());
        out.push((a, b));
    }

    Ok(out)
}

fn parse_tterm(s: &str) -> TTerm {
    let s = s.replace(",", " , ").replace("(", " ( ").replace(")", " ) ");
    let toks = s.trim().split(" ").filter(|x| x.trim() != "").collect::<Vec<_>>();
    let (t, []) = assemble_tterm(&toks) else { panic!() };
    t
}

fn assemble_tterm<'a>(mut toks: &'a [&'a str]) -> (TTerm, &'a [&'a str]) {
    let t0 = toks[0];

    let chr = t0.chars().next().unwrap();
    assert!(chr.is_alphanumeric());
    let symb_t0 = Symbol::new(t0);

    if chr.is_uppercase() {
        (TTerm::Var(symb_t0), &toks[1..])
    } else if chr.is_lowercase() {
        let Some(&"(") = toks.get(1) else {
            return (TTerm::Fun(symb_t0, Box::new([])), &toks[1..]);
        };
        toks = &toks[2..];

        let mut args = Vec::new();
        loop {
            let (subterm, toks2) = assemble_tterm(toks);
            args.push(subterm);
            toks = toks2;
            if toks[0] != "," { break; }
            toks = &toks[1..];
        }
        assert!(toks[0] == ")");
        toks = &toks[1..];

        (TTerm::Fun(symb_t0, args.into_boxed_slice()), toks)
    } else { panic!() }
}
