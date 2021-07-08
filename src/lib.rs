use ark_ff::Field;


pub mod bos_coster;

/// A vector addition chain
///
/// Encodes a procedure for computing a target vector
/// from basis vectors using binary additions
///
/// Each computed term is labelled.
///
/// Labels start at 0 and go up.
#[derive(Debug)]
pub struct VecAddChain {
    /// The dimension of the target vector (and number of basis vectors).
    ///
    /// Basis vectors have labels 0 through `dimension-1`, in order.
    pub dimension: usize,
    /// A list of additions to perform.
    ///
    /// Each addition contains two labels: the terms it adds
    ///
    /// Summand of index `i` has label `dimension + i`.
    pub adds: Vec<(usize, usize)>,
}

pub fn check_chain<F: Field>(chain: &VecAddChain, target: &[F]) {
    let mut vecs: Vec<Vec<F>> = Vec::new();
    assert_eq!(chain.dimension, target.len());
    let add_vecs = |a: &[F], b: &[F]| {
        assert_eq!(a.len(), b.len());
        a.iter().zip(b).map(|(a, b)| *a + b).collect::<Vec<F>>()
    };
    for i in 0..chain.dimension {
        vecs.push(vec![F::zero(); chain.dimension]);
        vecs.last_mut().unwrap()[i] = F::one();
    }
    for (a, b) in &chain.adds {
        let sum = add_vecs(&vecs[*a], &vecs[*b]);
        vecs.push(sum);
    }
    assert_eq!(vecs.last().unwrap(), target);
}

pub type ChainBuilder<F> = fn(target: Vec<F>) -> VecAddChain;

#[cfg(test)]
mod tests {
    use super::bos_coster;
    use super::{check_chain, ChainBuilder, VecAddChain};
    use ark_bls12_381::Fr;
    use ark_ff::PrimeField;

    fn test_on_target<F: PrimeField>(builder: ChainBuilder<F>, target: Vec<F>) -> VecAddChain {
        let chain = builder(target.clone());
        check_chain(&chain, &target);
        chain
    }

    fn test_ones<F: PrimeField>() {
        for n in 1..100 {
            let ch = test_on_target::<F>(
                bos_coster::build_chain::<F, bos_coster::UseShallow>,
                vec![F::one(); n],
            );
            assert_eq!(ch.adds.len(), n - 1);
        }
    }

    fn test_twos<F: PrimeField>() {
        for n in 1..100 {
            let _ch = test_on_target::<F>(
                bos_coster::build_chain::<F, bos_coster::UseShallow>,
                vec![F::from(2 as u32); n],
            );
            //assert_eq!(ch.adds.len(), n);
        }
    }

    fn test_incr<F: PrimeField>() {
        for n in 2..100 {
            let _ch = test_on_target::<F>(
                bos_coster::build_chain::<F, bos_coster::UseShallow>,
                (0..n).map(|i| F::from(i as u32)).collect(),
            );
            //assert_eq!(ch.adds.len(), n);
        }
    }

    fn test_rand<F: PrimeField>(size: usize, trials: usize) {
        let rng = &mut ark_std::test_rng();
        for _ in 0..trials {
            let _ch = test_on_target::<F>(
                bos_coster::build_chain::<F, bos_coster::UseShallow>,
                (0..size).map(|_| F::rand(rng)).collect(),
            );
            //assert_eq!(ch.adds.len(), n);
        }
    }


    #[test]
    fn test_ones_bls12_381() {
        test_ones::<Fr>();
    }

    #[test]
    fn test_twos_bls12_381() {
        test_twos::<Fr>();
    }

    #[test]
    fn test_incr_bls12_381() {
        test_incr::<Fr>();
    }

    #[test]
    fn test_rand_bls12_381() {
        test_rand::<Fr>(1000, 1);
    }
}
