use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use rand::Rng;
use vector_addition_chain::{
    bos_coster, bos_coster_fast, bos_coster_many, VecAddChain,
};

use clap::arg_enum;
use structopt::StructOpt;
arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Alg {
        Fast,
        Shallow,
        Deep,
        ManyShallow,
        ManyDeep,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bench", about = "Vector addition chain benchmarking")]
struct Opt {
    /// Algroithm
    #[structopt(short = "a", long = "alg", default_value = "fast")]
    alg: Alg,

    /// Size
    #[structopt()]
    size: usize,
}

fn test<F: PrimeField, R: Rng>(elems: usize, alg: Alg, rng: &mut R) {
    //let builder = bos_coster::build_chain::<Fr, bos_coster::UseShallow>;
    let target = (0..elems).map(|_| F::rand(rng)).collect::<Vec<F>>();
    let builder: Box<dyn Fn(Vec<F>) -> VecAddChain> = match alg {
        Alg::Shallow => Box::new(bos_coster::build_chain::<F, bos_coster::UseShallow>),
        Alg::Deep => Box::new(bos_coster::build_chain::<F, bos_coster::UseDeep>),
        Alg::ManyShallow => Box::new(bos_coster_many::build_chain::<F, bos_coster::UseShallow>),
        Alg::ManyDeep => Box::new(bos_coster_many::build_chain::<F, bos_coster::UseDeep>),
        Alg::Fast => Box::new(bos_coster_fast::build_chain::<F>),
    };
    let chain = builder(target.clone());
    //check_chain(&chain, &target);
    let adds = chain.adds.len();
    let ops_per_elem = adds as f64 / elems as f64;
    let add_cost = 6f64;
    let field_size = <F as PrimeField>::size_in_bits() as f64;
    let cs_per_elem = ops_per_elem * add_cost;
    let cs_per_bit = cs_per_elem / field_size;
    println!(
        "{:10?}, elems: {:>8}, Adds per elem: {:>8.1}, Cs per elem: {:>8.2}, Cs per bit: {:>8.3}",
        alg,
        elems,
        ops_per_elem,
        cs_per_elem,
        cs_per_bit,
    );
}

fn main() {
    let opt = Opt::from_args();
    let rng = &mut rand::thread_rng();
    //let rng = &mut ark_std::test_rng();
    test::<Fr, _>(opt.size, opt.alg, rng);
    //check_chain(&chain, &target);
}
