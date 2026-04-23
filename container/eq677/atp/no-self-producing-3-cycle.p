cnf(a,axiom, X = f(Y, f(X, f(f(Y, X), Y)))).
cnf(a,axiom, X = f(f(Y, X), f(f(Y, f(Y, X)), Y))).

cnf(a,axiom, f(z0,z0) = z1).
cnf(a,axiom, f(z0,z1) = z2).
cnf(a,axiom, f(z0,z2) = z0).
cnf(a,axiom, z0 != z1).
cnf(a,axiom, z0 != z2).
cnf(a,axiom, z1 != z2).
