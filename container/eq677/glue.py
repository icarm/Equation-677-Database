# This generates a random gluing model of multiple 5 or 11-sized models glued together.
# The steiner system is the affine plane of order 5 or 11.

import random

# If you want another model, change this seed.
random.seed(1)

# Choose block-size of 5 or 11
n = 5

def choose_block_magma():
    if n == 5: return "5/0"
    if n == 11: return random.choice(["11/0", "11/1", "11/2", "11/3"])
    return None

def load_magma(m):
    l = open("db/" + m).read()
    lines = l.split("\n")
    m = dict()
    for i, line in enumerate(lines):
        line = line.strip()
        for j, v in enumerate([x for x in line.split(" ") if x]):
            m[(i, j)] = int(v)
    return m

def check677(m):
    points = list(set(m.values()))
    for x in points:
        for y in points:
            f = lambda x, y: m[(x, y)]
            assert(x == f(y, f(x, f(f(y, x), y))))

# funny equivalent way to write 677
def check677_alt(m):
    points = list(set(m.values()))
    for x in points:
        for y in points:
            l = lambda x, y: m[(x, y)]
            r = lambda x, y: m[(y, x)]

            a = y
            for fn in [l, r]:
                for b in [x, y]:
                    a = fn(a, b)
            assert(x == a)

def dump(m):
    points = list(set(m.values()))
    for x in points:
        for y in points:
            z = m[(x, y)]
            print(points.index(z), end=" ")
        print()


def generate_projective_plane_order_4():
    def gf4_add(a, b):
        return a ^ b

    def gf4_mul(a, b):
        # Multiplication table for GF(4) with primitive polynomial x^2 + x + 1
        table = [
            [0, 0, 0, 0],
            [0, 1, 2, 3],
            [0, 2, 3, 1],
            [0, 3, 1, 2]
        ]
        return table[a][b]

    n = 4
    blocks = []

    # 1. Non-vertical lines: y = m*x + b using GF(4) math
    for m in range(n):
        for b in range(n):
            line = []
            for x in range(n):
                # y = gf4_add(gf4_mul(m, x), b)
                y = gf4_mul(m, x) ^ b
                line.append(x * n + y)
            line.append(16 + m)
            blocks.append(line)

    # 2. Vertical lines: x = c
    for c in range(n):
        line = []
        for y in range(n):
            line.append(c * n + y)
        line.append(20)
        blocks.append(line)

    # 3. Line at infinity
    blocks.append(list(range(16, 21)))

    return list(range(21)), blocks

def generate_affine_plane():
    points = [(x, y) for x in range(n) for y in range(n)]
    blocks = []

    for m in range(n):
        for b in range(n):
            line = []
            for x in range(n):
                y = (m * x + b) % n
                line.append((x, y))
            blocks.append(line)

    for c in range(n):
        line = []
        for y in range(n):
            line.append((c, y))
        blocks.append(line)

    return points, blocks

def random_glue(points, blocks):
    m = dict()
    for a in points:
        m[(a, a)] = a

    for block in blocks:
        m5 = load_magma(choose_block_magma())
        random.shuffle(block)
        for x in range(n):
            for y in range(n):
                m[(block[x], block[y])] = block[m5[(x, y)]]
    return m

points, blocks = generate_projective_plane_order_4()
for _ in range(2000):
    m = random_glue(points, blocks)
    check677(m)
    check677_alt(m)
    dump(m)
    print("--")
