import itertools
import sys

from sage.groups.abelian_gps.abelian_group_gap import AbelianGroupGap

def primepower_groups_l(p, n, min=1):
    if n == 0:
        yield []
    for i in range(min, n+1):
        for rest in primepower_groups_l(p, n-i, min=i):
            yield [p ** i] + rest

def groups_of_n(n):
    out = [[]]
    for (p, i) in factor(n):
        out2 = []
        for o in out:
            for g in primepower_groups_l(p, i):
                out2.append(o + g)
        out = out2

    # convert to actual groups
    ret = []
    for l in out:
        ret.append(AbelianGroupGap(l))
    return ret

def check_f(f, g):
    for x in g:
        for y in g:
            if x != f(y, f(x, f(f(y, x), y))):
                return False
    return True

def f_from_g(g):
    autos = list(g.aut())
    size = len(autos)**2 * g.order()
    print("len(autos) = " + str(len(autos)) +  ", iterating over a space of " + str(size))
    if size > 100_000:
        print("too big!")
        sys.stdout.flush()
        return

    sys.stdout.flush()
    for a in autos:
        for b in autos:
            for c in g:
                def f(x, y):
                    return a(x) * b(y) * c
                yield (f, (a, b, c))

for n in range(1, 100+1):
    for g in groups_of_n(n):
        print()
        print("looking at group:", g)
        print()
        sys.stdout.flush()
        for (f, (a, b, c)) in f_from_g(g):
            if check_f(f, g):
                print()
                print("Model found!")
                print(g)
                print("f(x, y) = " + str(a) + "(x) + " + str(b) + "(y) + " + str(c))
                ll = list(g)
                n = len(ll)
                for x in range(n):
                    for y in range(n):
                        z = f(g[x], g[y])
                        i = ll.index(z)
                        print(i, end=" ")
                    print()
                sys.stdout.flush()
