use crate::*;

pub fn enumerate_dump(m: &MatrixMagma) {
    let v = enumerate(&m);
    for x in v {
        if let Some(x) = x {
            println!("{}", x.stringify());
        } else {
            println!("-");
        }
    }
}

pub fn enumerate(m: &MatrixMagma) -> Vec<Option<ETerm>> {
    let mut v: Vec<Option<ETerm>> = vec![None; m.n];
    v[0] = Some(ETerm::Zero);
    loop {
        let mut dirty = false;
        for i in 0..m.n {
            if v[i].is_none() { continue }
            for j in 0..m.n {
                if v[j].is_none() { continue }
                let new = ETerm::F(Box::new([v[i].clone().unwrap(), v[j].clone().unwrap()]));
                let vv = new.eval(m);
                if v[vv].is_none() || v[vv].as_ref().unwrap().size() > new.size() {
                    v[vv] = Some(new);
                    dirty = true;
                }
            }
        }
        if !dirty { break }
    }
    v
}

#[derive(Clone)]
pub enum ETerm {
    Zero,
    F(Box<[ETerm; 2]>),
}

impl ETerm {
    fn eval(&self, m: &MatrixMagma) -> usize {
        match self {
            ETerm::Zero => 0,
            ETerm::F(b) => {
                let [x, y] = &**b;
                let x = x.eval(m);
                let y = y.eval(m);
                m.f(x, y)
            },
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            ETerm::Zero => format!("0"),
            ETerm::F(b) => format!("f({}, {})", b[0].stringify(), b[1].stringify()),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ETerm::Zero => 1,
            ETerm::F(b) => b[0].size() + b[1].size(),
        }
    }
}
