use crate::*;
use nauty_pet::prelude::*;
use nauty_pet::canon::*;
use nauty_pet::autom::{AutomStats, TryIntoAutomStatsTraces, AutomGroup, TryIntoAutomGroupNautyDense};
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::HashMap;

type Graph = petgraph::graph::UnGraph<NodeType, EdgeType>;
pub type Group = Vec<Perm>;
pub type Perm = Vec<usize>;

/// Vertex weight in the magma → graph encoding.
///
/// `Elem(i)` carries the *original* (pre-canonicalization) label `i` of an
/// element node so we can recover the canonicalization permutation by reading
/// out the inner field after nauty/Traces finishes. We deliberately implement
/// `PartialEq`/`Ord` to treat all `Elem(_)` as a single color class — nauty
/// must be free to permute them; the inner data is carried along as opaque
/// baggage that survives the round-trip through `into_canon_traces`.
#[derive(Clone, Copy)]
enum NodeType {
    Elem(usize),
    XYZ,
}

impl PartialEq for NodeType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (NodeType::Elem(_), NodeType::Elem(_)) | (NodeType::XYZ, NodeType::XYZ),
        )
    }
}
impl Eq for NodeType {}
impl PartialOrd for NodeType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NodeType {
    fn cmp(&self, other: &Self) -> Ordering {
        use NodeType::*;
        match (self, other) {
            (Elem(_), Elem(_)) | (XYZ, XYZ) => Ordering::Equal,
            (Elem(_), XYZ) => Ordering::Less,
            (XYZ, Elem(_)) => Ordering::Greater,
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
enum EdgeType {
    X, Y, Z
}

impl MatrixMagma {
    pub fn canonicalize2(&self) -> MatrixMagma {
        self.canonicalize2_with_perm().0
    }

    /// Canonicalize and also return the permutation π that maps canonical
    /// labels back to the input's labels: canonical element `k` corresponds
    /// to `self`'s element `π[k]`. Equivalently, applying `π` as a reorder
    /// to `self` reproduces the returned canonical magma.
    pub fn canonicalize2_with_perm(&self) -> (MatrixMagma, Vec<usize>) {
        let g = graphify(self);
        let g = g.into_canon_traces();
        de_graphify_with_perm(&g)
    }

    pub fn autom_stats(&self) -> AutomStats {
        graphify(self).try_into_autom_stats_traces().unwrap()
    }

    pub fn autom_group(&self) -> Group {
        let mut a = graphify(self).try_into_autom_group_nauty_dense().unwrap().0;
        for x in &mut a {
            x.truncate(self.n);
        }
        a
    }

    pub fn autom_group_mini(&self) -> Group {
        minimize_gap(self.autom_group())
    }
}

pub fn orbits(autom: &[Vec<usize>]) -> Vec<usize> {
    let n = autom[0].len();
    let mut orbits: Vec<usize> = (0..n).collect();
    for aut in autom {
        for i in 0..n {
            let j = aut[i];
            if j < orbits[i] {
                orbits[i] = j;
            }
        }
    }
    orbits
}

fn graphify(m: &MatrixMagma) -> Graph {
    let mut g = Graph::new_undirected();
    let mut nodes = Vec::new();
    for x in 0..m.n {
        nodes.push(g.add_node(NodeType::Elem(x)));
    }
    for x in 0..m.n {
        for y in 0..m.n {
            let z = m.f(x, y);
            if z != usize::MAX {
                let xyz = g.add_node(NodeType::XYZ);
                g.add_edge(xyz, nodes[x], EdgeType::X);
                g.add_edge(xyz, nodes[y], EdgeType::Y);
                g.add_edge(xyz, nodes[z], EdgeType::Z);
            }
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::db::db;

    /// For every magma in `db`, shuffle it with several random permutations
    /// and verify that `canonicalize2_with_perm` produces a `perm` such that
    /// `shuffled.permute(perm) == canonical` (the same canonical we'd get
    /// without shuffling, since `db` magmas are already canonical).
    #[test]
    fn perm_round_trip() {
        for (name, m) in db() {
            // db magmas are already canonical, so perm-of-shuffled must
            // undo the shuffle exactly back to `m`.
            for _ in 0..5 {
                let shuffled = m.shuffle();
                let (canon, perm) = shuffled.canonicalize2_with_perm();
                assert_eq!(
                    canon, m,
                    "{name}: canonical form changed under shuffle",
                );
                assert_eq!(
                    shuffled.permute(perm.clone()), canon,
                    "{name}: shuffled.permute(perm) != canonical",
                );
            }
        }
    }
}

fn de_graphify_with_perm(g: &Graph) -> (MatrixMagma, Vec<usize>) {
    // Walk the canonicalized graph in node-index order. The k-th Elem node
    // encountered is canonical element k, and its inner field tells us which
    // original (user-side) element ended up at that canonical slot.
    let mut elem_canon_idx = HashMap::new();
    let mut perm = Vec::new();
    for idx in g.node_indices() {
        if let NodeType::Elem(orig) = g[idx] {
            elem_canon_idx.insert(idx, perm.len());
            perm.push(orig);
        }
    }

    let mut m = MatrixMagma::undefined(perm.len());

    for idx in g.node_indices() {
        if matches!(g[idx], NodeType::XYZ) {
            let mut x = None;
            let mut y = None;
            let mut z = None;
            for e in g.edges(idx) {
                match e.weight() {
                    EdgeType::X => { x = Some(e.target()); }
                    EdgeType::Y => { y = Some(e.target()); }
                    EdgeType::Z => { z = Some(e.target()); }
                }
            }
            let (x, y, z) = (x.unwrap(), y.unwrap(), z.unwrap());
            m.set_f(elem_canon_idx[&x], elem_canon_idx[&y], elem_canon_idx[&z]);
        }
    }

    (m, perm)
}
