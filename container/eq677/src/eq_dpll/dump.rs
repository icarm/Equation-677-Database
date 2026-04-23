use crate::eq_dpll::*;

impl Ctxt {
    pub fn dump_classes(&self) {
        println!("------------------------------------------------------------------------------");
        for (i, c) in self.classes.iter().enumerate() {
            let n = match &c.node {
                Node::Elem(e) => format!("Elem({e})"),
                Node::F(l, r) => format!("F(${}, ${})", l.0, r.0),
                Node::AssertEq(l, r) => format!("AssertEq({l}, ${})", r.0),
            };
            let v = match c.value {
                None => String::new(),
                Some(v) => format!(" (equal to {v})"),
            };
            let mut u = format!(" [parents = ");
            for (j, p) in c.parents.iter().enumerate() {
                u.push_str(&format!("${}", p.0));
                if j != c.parents.len() - 1 {
                    u.push_str(", ");
                }
            };
            u.push(']');
            println!("${i} := {n:20}{v:20}{u}");
        }
        println!("------------------------------------------------------------------------------");
    }

    pub fn dump_table(&self) {
        println!("------------------------------------------------------------------------------");
        for x in 0..self.n {
            for y in 0..self.n {
                let z = self.table[idx((x, y), self.n)];
                if z == ElemId::MAX {
                    print!("- ");
                } else {
                    print!("{z} ");
                }
            }
            println!("");
        }
        println!("------------------------------------------------------------------------------");
    }

    pub fn dump_pos_terms(&self) {
        println!("------------------------------------------------------------------------------");
        for x in 0..self.n {
            for y in 0..self.n {
                let z = &self.pos_terms[idx((x, y), self.n)];
                let mut zz = format!("[");
                for (i, a) in z.iter().enumerate() {
                    zz.push_str(&format!("${}", a.0));
                    if i != z.len() - 1 {
                        zz.push_str(", ");
                    }
                }
                zz.push(']');
                println!("pos_terms({x}, {y}) := {zz}");
            }
        }
        println!("------------------------------------------------------------------------------");
    }

    pub fn dump(&self) {
        println!("==============================================================================");
        self.dump_classes();
        self.dump_table();
        self.dump_pos_terms();
        println!("==============================================================================");
    }
}
