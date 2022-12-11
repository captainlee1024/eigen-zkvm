use criterion::*;
use rayon::prelude::*;
use starky::merklehash_bn128::MerkleTree;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

fn run_merklehash(pols: Vec<BaseElement>) {
    let n = 1 << 24;
    let idx = 32;
    let n_pols = 20;

    let now = std::time::Instant::now();
    let tree = MerkleTree::merkelize(pols, n_pols, n).unwrap();
    println!("time cost: {}", now.elapsed().as_secs());
    let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
    let root = tree.root();
    assert_eq!(
        tree.verify_group_proof(&root, &mp, idx, &group_elements)
            .unwrap(),
        true
    );
}

fn merklehash_group_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("merklehash");

    let n = 1 << 24;
    let n_pols = 20;
    let mut pols: Vec<BaseElement> = vec![BaseElement::ZERO; n_pols * n];

    rayon::scope(|_s| {
        pols.par_chunks_mut(n).enumerate().for_each(|(i, bb)| {
            for j in 0..n {
                bb[j] = BaseElement::from((j + i * n) as u64)
            }
        });
    });
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    for data in [&pols].iter() {
        group.bench_with_input(
            BenchmarkId::new("merkelize", format!("height*n_pols={}", pols.len())),
            data,
            |b, s| {
                b.iter(|| run_merklehash(s.to_vec()));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, merklehash_group_bench);
criterion_main!(benches);