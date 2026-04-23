use crate::*;

pub fn db_autom_search() {
    for (name, m) in db() {
        if m.n < 2 { continue }

        println!("Considering magma {name}");
        let grp = m.autom_group_mini();
        for x in &grp {
            draw_cycle(0, m.n, |i| x[i]);
            println!();
        }
        println!("group computed!");
        if grp.len() < 2 && m.n >= 25 {
            println!("The group is probably not useful enough to restrict the search space!");
            continue
        }
        let grp: Vec<Vec<E>> = grp.into_iter().map(|x| x.into_iter().map(|y| y as E).collect()).collect();
        if m.n < 16 && grp.len() > 1 {
            for g in grp {
                println!("Single-perm search for:");
                draw_cycle(0, m.n, |i| g[i] as usize);
                println!();
                c_run(m.n, vec![g]);
            }
        } else {
            c_run(m.n, grp);
        }
    }
}
