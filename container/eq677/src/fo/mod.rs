type V = usize;
type E = usize;

fn run(n: usize) {
    mainloop(initial(n));
}

fn mainloop(st: State) {
    let st = propagate(st);
    let f = heur(&st);
    for x in branch(&st, &f) {
        mainloop(x);
    }
}

// will run some theorem prover, and filter the results it proved.
fn propagate(st: State) -> State { todo!() }
 
fn initial(n: usize) -> State { todo!() }

fn heur(st: &State) -> Formula { todo!() }

fn branch(st: &State, f: &Formula) -> Vec<State> { todo!() }

struct State {
    facts: Vec<Formula>,
}

enum Formula {
    Forall(V, Box<Formula>),
    Exists(V, Box<Formula>),
    Not(Box<Formula>),
    And(Box<[Formula; 2]>),
    Or(Box<[Formula; 2]>),
    Equal(Term, Term),
}

enum Term {
    F(Box<[Term; 2]>),
    Elem(E),
}
