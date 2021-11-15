import heapq

def random_prime_bits(bits):
    '''A random prime with this many bits'''
    return random_prime(2 ^ bits - 1, False, 2 ^ (bits - 1))

def test(bits, n, p_iters, scalar_iters, group_iters):
    good_ct = 0.0
    bad_ct = 0.0
    for _ in range(p_iters):
        p = random_prime_bits(bits)
        F = GF(p)
        for _ in range(scalar_iters):
            scalars = [int(random()*p) for _ in range(n)]
            chain = build_chain(scalars)
            check_chain(F, scalars, chain, 1)
            for _ in range(group_iters):
                pts = [F.random_element() for _ in range(n)]
                if check_complete(scalars, chain, pts):
                    good_ct += 1
                else:
                    bad_ct += 1
    bad_prob = bad_ct / (bad_ct + good_ct)
    print(f"{bits},{n},{bad_prob}")

def check_complete(scalars, chain, points):
    ps = [p for p in points]
    for a, b in chain:
        if a != b:
            if ps[a] in [ps[b], -ps[b]]:
                return False
        ps.append(ps[a] + ps[b])
    return True

def check_chain(F, scalars, chain, iters = 1):
    for _ in range(iters):
        n = len(scalars)
        x = F.random_element()
        ex = sum(x ** i * s for i, s in enumerate(scalars))
        wires = [x ** i for i in range(n)]
        for a, b in chain:
            wires.append(wires[a] + wires[b])
        result = wires.pop()
        if ex != result:
            print(chain)
            print(scalars)
            print(n)
            assert False

def build_chain(scalars):
    ''' Given a list of n scalars, builds an addition chain for an MSM under
    those scalars.

    A chain is a list of pairs of numbers of length m.

    Each number can be in [0, n+m) and corresponds to a wire in the chain's
    circuit.

    Wires in [0,n) are the inputs.

    Wires in [n, n+m) are outputs of the gates.

    Each pair in the chain is a gate. Each number is a wire number that goes
    into that gate.

    This implementation is adapted from https://cr.yp.to/badbatch/boscoster2.py
    '''
    n = len(scalars)
    x = [(-s, i) for i, s in enumerate(scalars) if s != 0]
    chain = []
    next_w = n
    heapq.heapify(x)
    while len(x) > 1 or x[0][0] != -1:
        nw1 = heapq.heappop(x)
        n1,w1 = -nw1[0],nw1[1]
        if len(x) == 0 or n1 > -2*x[0][0]:
            if n1 % 2 == 1:
                heapq.heappush(x,(-1,w1))
            heapq.heappush(x,(-(n1//2),next_w))
            chain.append((w1, w1))
        else:
            nw2 = heapq.heappop(x)
            n2,w2 = -nw2[0],nw2[1]
            if n1 > n2:
                heapq.heappush(x,(-(n1-n2),w1))
            heapq.heappush(x,(-n2,next_w))
            chain.append((w1, w2))
        next_w += 1
    assert heapq.heappop(x)[0] == -1
    return chain

print('bits,n,p')
for bits in range(5,31):
    for n in [10, 30, 100, 300, 1000]:
        test(bits, n, 500, 1, 1)
