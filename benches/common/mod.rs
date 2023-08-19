#![allow(dead_code)]

use criterion::PlotConfiguration;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use rand::Rng;
use vers_vecs::{BitVec, RsVec, UDSTree, UDSTreeBuilder};

pub const SIZES: [usize; 12] = [
    1 << 8,
    1 << 10,
    1 << 12,
    1 << 14,
    1 << 16,
    1 << 18,
    1 << 20,
    1 << 22,
    1 << 24,
    1 << 26,
    1 << 28,
    1 << 30,
];

pub fn construct_vers_vec(rng: &mut ThreadRng, len: usize) -> RsVec {
    let sample = Uniform::new(0, u64::MAX);

    let mut bit_vec = BitVec::new();
    for _ in 0..len / 64 {
        bit_vec.append_word(sample.sample(rng));
    }

    RsVec::from_bit_vec(bit_vec)
}

pub fn fill_random_vec(rng: &mut ThreadRng, len: usize) -> Vec<u64> {
    let sample = Uniform::new(0, u64::MAX);

    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(sample.sample(rng));
    }

    vec
}

pub fn construct_random_tree(
    rng: &mut ThreadRng,
    tree_size: usize,
    max_children: usize,
) -> (UDSTree, Vec<usize>) {
    let mut builder = UDSTreeBuilder::with_capacity(tree_size);
    let mut nodes = Vec::with_capacity(tree_size);
    let mut children = 0;

    while children < tree_size - max_children {
        let n = rng.gen_range(1..=max_children);
        nodes.push(builder.visit_node(n).expect("failed to visit node"));
        children += n;
    }

    nodes.push(
        builder
            .visit_node(tree_size - children - 1)
            .expect("failed to visit node"),
    );
    builder.visit_remaining_nodes();

    (builder.build().expect("failed to build tree"), nodes)
}

pub fn plot_config() -> PlotConfiguration {
    PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic)
}
