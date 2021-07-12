use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use rand::Rng;
use vector_addition_chain::{bos_coster, ChainBuilder, bos_coster_many, bos_coster_fast};
fn test<F: PrimeField, R: Rng>(
    l2elems: usize,
    builder: ChainBuilder<F>,
    builder_name: &str,
    rng: &mut R,
) {
    //let builder = bos_coster::build_chain::<Fr, bos_coster::UseShallow>;
    let elems = 1 << l2elems;
    let target = (0..elems).map(|_| F::rand(rng)).collect::<Vec<F>>();
    let chain = builder(target.clone());
    //check_chain(&chain, &target);
    let adds = chain.adds.len();
    let ops_per_elem = adds as f64 / elems as f64;
    let add_cost = 6f64;
    let constraints_per_elem = ops_per_elem * add_cost;
    println!(
        "{:10}, log2(elems): {:2}, Adds per elem: {:>8.1}, Cs per elem: {:>8.2}, Cs: {:>8}",
        builder_name,
        l2elems,
        ops_per_elem,
        constraints_per_elem,
        constraints_per_elem * elems as f64
    );
}

fn main() {
    let rng = &mut ark_std::test_rng();
    for l2elems in 0..14 {
        for _i in 0..1 {
            {
                test(
                    l2elems,
                    bos_coster::build_chain::<Fr, bos_coster::UseShallow>,
                    "shallow",
                    rng,
                );
                test(
                    l2elems,
                    bos_coster::build_chain::<Fr, bos_coster::UseDeep>,
                    "deep",
                    rng,
                );
                test(
                    l2elems,
                    bos_coster_many::build_chain::<Fr, bos_coster::UseShallow>,
                    "m-shallow",
                    rng,
                );
                test(
                    l2elems,
                    bos_coster_many::build_chain::<Fr, bos_coster::UseDeep>,
                    "m-deep",
                    rng,
                );
                test(
                    l2elems,
                    bos_coster_fast::build_chain::<Fr>,
                    "fast",
                    rng,
                );
            }
        }
    }
    //check_chain(&chain, &target);
}
