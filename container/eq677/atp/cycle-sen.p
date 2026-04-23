cnf(a,axiom, X = f(Y, f(X, f(f(Y, X), Y)))).
cnf(a,axiom, X = f(f(Y, X), f(f(Y, f(Y, X)), Y))).

% We are considering the self-producing cycle C(a,a).
% We have
% - the start a*a = s.
% - the end a*e = a.
% - the next-to-last a*n = e.

cnf(a,axiom, f(a,a) = s).
cnf(a,axiom, f(a,n) = e).
cnf(a,axiom, f(a,e) = a).

% We assume it is not a singleton cycle, otherwise stuff is boring anyways.
cnf(a,axiom, a != f(a,a)).

% Goals I:
cnf(a,conjecture, f(s,a) = n).
cnf(a,conjecture, f(e,s) = n).

% Goals II:
% a, s, e, n are all distinct.
cnf(a,conjecture, a != s).
cnf(a,conjecture, a != e).
cnf(a,conjecture, a != n).

cnf(a,conjecture, s != e).
cnf(a,conjecture, s != n).

cnf(a,conjecture, e != n).
