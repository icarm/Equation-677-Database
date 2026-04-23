use crate::*;
use std::collections::HashSet;

// produces perm^k
fn compose_rep(perm: &[usize], k: usize) -> Perm {
    let n = perm.len();
    (0..n).map(|mut x| {
        for _ in 0..k {
            x = perm[x];
        }
        x
    }).collect()
}

// only minimizes a bit.
pub fn minimize(mut group: Group) -> Group {
    let n = group[0].len();
    let mut set: HashSet<Perm> = group.iter().cloned().collect();
    for a in group.iter() {
        if set.contains(a) {
            for k in 2..n {
                let rep = compose_rep(&a, k);
                if &rep == a { break }

                set.remove(&rep);
            }
        }
    }
    set.into_iter().collect()
}

pub fn minimize_gap(group: Group) -> Group {
    use std::process::{Stdio, Command};
    use std::io::Write;

    // dbg!(&group);
    let n = group[0].len();
    let mut input = String::from("MinimalGeneratingSet(Group([");
    for x in &group {
        input.push_str("PermList([");
        for a in x {
            input.push_str(&(a+1).to_string());
            input.push_str(", ");
        }
        input.pop().unwrap();
        input.pop().unwrap();
        input.push_str("]),");
    }
    input.pop().unwrap();
    input.push_str("]));");

    let mut cmd = Command::new("gap").args(["-q"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();
    write!(stdin, "{}", input).unwrap();
    drop(stdin);

    let out = cmd.wait_with_output().unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    let out = out.trim();

    let cycle_sets = parse_gap_permutations_cycles(out);
    cycle_sets
        .into_iter()
        .map(|x| cycles_to_full_perm(n, x))
        .collect()
}

fn parse_gap_permutations_cycles(input: &str) -> Vec<Vec<Vec<usize>>> {
    let mut perms = Vec::new();
    let mut cycles = Vec::new();
    let mut current_cycle = Vec::new();
    let mut num = String::new();

    let mut paren_depth = 0;

    for c in input.chars() {
        match c {
            '(' => {
                paren_depth += 1;
                current_cycle = Vec::new();
                num.clear();
            }
            ')' => {
                if !num.is_empty() {
                    current_cycle.push(num.parse().unwrap());
                    num.clear();
                }
                paren_depth -= 1;
                cycles.push(current_cycle.clone());
            }
            ',' => {
                if paren_depth > 0 {
                    if !num.is_empty() {
                        current_cycle.push(num.parse().unwrap());
                        num.clear();
                    }
                } else {
                    // separating permutations
                    if !cycles.is_empty() {
                        perms.push(cycles.clone());
                        cycles.clear();
                    }
                }
            }
            '0'..='9' => {
                if paren_depth > 0 {
                    num.push(c);
                }
            }
            '[' | ']' => {
                if c == ']' && !cycles.is_empty() {
                    perms.push(cycles.clone());
                    cycles.clear();
                }
            }
            _ => {} // ignore whitespace
        }
    }

    perms
}

fn cycles_to_full_perm(n: usize, cycles: Vec<Vec<usize>>) -> Vec<usize> {
    // identity permutation
    let mut perm: Vec<usize> = (1..=n).collect();

    // apply cycles: (a b c) means a→b, b→c, c→a
    for cycle in cycles {
        if cycle.len() < 2 {
            continue;
        }
        for i in 0..cycle.len() {
            let from = cycle[i];
            let to = cycle[(i + 1) % cycle.len()];
            perm[from - 1] = to;
        }
    }

    // to get back to 0-indexed.
    for i in 0..n {
        perm[i] -= 1;
    }

    perm
}
