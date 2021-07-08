use super::VecAddChain;
use ark_ff::{BigInteger, PrimeField};
use hashconsing::{
    coll::{HConMap, HConSet},
    HConsed, HConsign, HashConsign,
};
use std::cmp::{max, Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::marker::PhantomData;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct ChainData {
    //pub size: usize,
    pub depth: usize,
    pub form: Form,
}

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Form {
    Add(Chain, Chain),
    Basis(usize),
}

pub type Chain = HConsed<ChainData>;

pub trait ChainCmp {
    /// Greater chain is more likely to be subtracted.
    fn cmp(l: &Chain, r: &Chain) -> Ordering;
}

struct Entry<B, C> {
    x: B,
    g: Chain,
    comp: PhantomData<C>,
}

impl<F: PartialOrd, C: ChainCmp> PartialEq for Entry<F, C> {
    fn eq(&self, other: &Self) -> bool {
        self.x
            .partial_cmp(&other.x)
            .map(|o| o.then_with(|| C::cmp(&self.g, &other.g)) == Ordering::Equal)
            .unwrap_or(true)
    }
}
impl<F: PartialOrd, C: ChainCmp> Eq for Entry<F, C> {}
impl<F: PartialOrd, C: ChainCmp> PartialOrd for Entry<F, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.x
            .partial_cmp(&other.x)
            .map(|o| o.then_with(|| C::cmp(&self.g, &other.g)))
    }
}
impl<F: Ord, C: ChainCmp> Ord for Entry<F, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x.cmp(&other.x).then_with(|| C::cmp(&self.g, &other.g))
    }
}

struct State<B, C> {
    terms: HConsign<ChainData>,
    heap: BinaryHeap<Entry<B, C>>,
    /// A list of all terms, smallest first. Useful for avoiding issues dropping
    drop_list: Vec<Chain>,
    dups: usize,
    dimension: usize,
}

impl<F: BigInteger, C: ChainCmp> State<F, C> {
    fn add(&mut self, a: Chain, b: Chain) -> Chain {
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        let (c, new) = self.terms.mk_is_new(ChainData {
            //size: 1 + a.size + b.size,
            depth: 1 + max(a.depth, b.depth),
            form: Form::Add(a, b),
        });
        self.dups += 1 - new as usize;
        self.drop_list.push(c.clone());
        c
    }
    fn new_basis(&mut self, index: usize) -> Chain {
        let (c, new) = self.terms.mk_is_new(ChainData {
            //size: 1,
            depth: 1,
            form: Form::Basis(index),
        });
        self.dups += 1 - new as usize;
        self.drop_list.push(c.clone());
        c
    }
    fn mult(&mut self, scalar: F, chain: Chain) -> Chain {
        assert!(!scalar.is_zero());
        let mut scalar_bits = scalar;
        let mut p2 = chain.clone();
        let mut acc = None;
        while !scalar_bits.is_zero() {
            if scalar_bits.is_odd() {
                acc = Some(
                    acc.map(|a| self.add(a, p2.clone()))
                        .unwrap_or_else(|| p2.clone()),
                );
            }
            p2 = self.add(p2.clone(), p2);
            scalar_bits.div2();
        }
        acc.expect("zero chain requested")
    }
    fn new(target: Vec<F>) -> Self {
        let mut this = State {
            dups: 0,
            terms: HConsign::empty(),
            heap: BinaryHeap::new(),
            dimension: target.len(),
            drop_list: Vec::new(),
        };
        for (i, f) in target.into_iter().enumerate() {
            let basis = this.new_basis(i);
            if !f.is_zero() {
                this.heap.push(Entry {
                    x: f,
                    g: basis,
                    comp: Default::default(),
                })
            }
        }
        this
    }
    fn finalize(mut self) -> VecAddChain {
        assert_eq!(self.heap.len(), 1);
        let entry = self.heap.pop().unwrap();
        assert_eq!(self.heap.len(), 0);
        //println!("depth: {}", entry.g.depth);
        let mut labels = HConMap::<Chain, usize>::new();
        let mut children_added = HConSet::<Chain>::new();
        let mut stack = vec![self.mult(entry.x, entry.g.clone())];
        let mut adds = Vec::new();
        std::mem::drop(entry);
        while let Some(chain) = stack.pop() {
            match &chain.form {
                Form::Add(l, r) => {
                    if children_added.contains(&chain) {
                        let label = self.dimension + adds.len();
                        let l_label = labels.get(l).unwrap();
                        let r_label = labels.get(r).unwrap();
                        adds.push((*l_label, *r_label));
                        labels.insert(chain, label);
                    } else {
                        let l = l.clone();
                        let r = r.clone();
                        children_added.insert(chain.clone());
                        stack.push(chain);
                        stack.push(l);
                        stack.push(r);
                    }
                }
                Form::Basis(i) => {
                    let i = *i;
                    labels.insert(chain, i);
                }
            }
        }
        std::mem::drop(children_added);
        std::mem::drop(stack);
        std::mem::drop(labels);
        VecAddChain {
            dimension: self.dimension,
            adds,
        }
    }
}

impl<F, C> Drop for State<F, C> {
    fn drop(&mut self) {
        //println!("Duplicate report: {} dups in {} terms", self.dups, self.drop_list.len());
        // drain table first, to control drop order
        self.terms.table.drain();
        while let Some(_t) = self.drop_list.pop() {
            // Drop most complex first. Children still in list, so no recursive drop.
        }
    }
}

pub fn build_chain<F: PrimeField, C: ChainCmp>(target: Vec<F>) -> VecAddChain {
    let mut state = State::<F::BigInt, C>::new(target.into_iter().map(|f| f.into_repr()).collect());
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

pub struct UseShallow;

impl ChainCmp for UseShallow {
    fn cmp(l: &Chain, r: &Chain) -> Ordering {
        l.depth.cmp(&r.depth).reverse()
    }
}

pub struct UseDeep;

impl ChainCmp for UseDeep {
    fn cmp(l: &Chain, r: &Chain) -> Ordering {
        UseShallow::cmp(l, r).reverse()
    }
}
