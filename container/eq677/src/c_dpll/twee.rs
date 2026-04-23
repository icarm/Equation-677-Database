use crate::c_dpll::*;

pub fn twee_prop(ctxt: &mut Ctxt) -> Result<(), ()> {
    let m = MatrixMagma::by_fn(ctxt.n as usize, |x, y| {
        let i = idx(x as E, y as E, ctxt.n);
        let z = ctxt.classes_xy[i].value;
        if z == E::MAX { usize::MAX } else { z as usize }
    });
    let mut touched = false;
    for e in twee_analyze(&m) {
        if let (GTerm::E(_), GTerm::E(_)) = e { return Err(()) }
        if let (GTerm::F(b), GTerm::E(z)) = e && let [GTerm::E(x), GTerm::E(y)] = &*b {
            let (x, y, z) = (*x as E, *y as E, z as E);

            if ctxt.classes_xy[idx(x, y, ctxt.n)].value != z {
                prove_triple(x, y, z, ctxt)?;
                touched = true;
            }
        }
    }

    if touched {
        return propagate(ctxt);
    } else {
        Ok(())
    }
}

