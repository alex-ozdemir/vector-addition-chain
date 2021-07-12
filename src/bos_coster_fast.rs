use super::VecAddChain;
use ark_ff::{BigInteger, PrimeField};
use std::cmp::{Ord, PartialOrd};
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct Entry<B> {
    pub x: B,
    pub g: usize,
}

struct State<B> {
    chain: VecAddChain,
    heap: BinaryHeap<Entry<B>>,
}

impl<F: BigInteger> State<F> {
    fn add(&mut self, a: usize, b: usize) -> usize {
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        self.chain.adds.push((a, b));
        self.chain.adds.len() + self.chain.dimension - 1
    }
    fn new(target: Vec<F>) -> Self {
        let mut this = State {
            chain: VecAddChain {
                adds: vec![],
                dimension: target.len(),
            },
            heap: BinaryHeap::new(),
        };
        for (i, f) in target.into_iter().enumerate() {
            if !f.is_zero() {
                this.heap.push(Entry { x: f, g: i })
            }
        }
        this
    }
    fn finalize(mut self) -> VecAddChain {
        assert_eq!(self.heap.len(), 1);
        let entry = self.heap.pop().unwrap();
        let mut p2 = entry.g;
        let mut acc = None;
        let mut scalar = entry.x;
        while !scalar.is_zero() {
            if scalar.is_odd() {
                acc = Some(match acc {
                    Some(acc) => self.add(acc, p2),
                    None => p2,
                });
            }
            scalar.div2();
            p2 = self.add(p2, p2);
        }
        // safe b/c non-zero things are eventually odd when divided by 2
        let acc = acc.unwrap();
        if acc < self.chain.dimension {
            self.chain.adds.clear();
        } else {
            self.chain.adds.truncate(acc - self.chain.dimension + 1);
        }
        self.chain
    }
}

pub fn build_chain<F: PrimeField>(target: Vec<F>) -> VecAddChain {
    let mut state = State::<F::BigInt>::new(target.into_iter().map(|f| f.into_repr()).collect());
    while state.heap.len() > 1 {
        let mut first = state.heap.pop().unwrap();
        let mut second = state.heap.pop().unwrap();
        let half_first = {
            let mut t = first.x;
            t.div2();
            t
        };
        if half_first > second.x {
            state.heap.push(second);
            if first.x.is_odd() {
                state.heap.push(Entry {
                    x: F::one().into_repr(),
                    g: first.g.clone(),
                });
            }
            first.x.div2();
            first.g = state.add(first.g.clone(), first.g);
            state.heap.push(first);
        } else {
            assert!(!first.x.sub_noborrow(&second.x));
            //first.x -= second.x;
            second.g = state.add(first.g.clone(), second.g);
            state.heap.push(second);
            if !first.x.is_zero() {
                state.heap.push(first);
            }
        }
    }
    state.finalize()
}
