// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use criterion::*;
use fastcrypto::hash::MultisetHash;
use iota_types::{accumulator::Accumulator, base_types::ObjectDigest};

fn accumulator_benchmark(c: &mut Criterion) {
    {
        let digests: Vec<_> = (0..1_000).map(|_| ObjectDigest::random()).collect();
        let mut accumulator = Accumulator::default();

        let mut group = c.benchmark_group("accumulator");
        group.throughput(Throughput::Elements(digests.len() as u64));

        group.bench_function("accumulate_digests", |b| {
            b.iter(|| accumulator.insert_all(&digests))
        });
    }

    {
        let mut group = c.benchmark_group("accumulator");
        group.throughput(Throughput::Elements(1));

        let mut accumulator = Accumulator::default();
        let point = {
            let digest = ObjectDigest::random();
            let mut accumulator = Accumulator::default();
            accumulator.insert(digest);
            accumulator
        };
        group.bench_function("sum_accumulators", |b| b.iter(|| accumulator.union(&point)));
    }
}

criterion_group!(benches, accumulator_benchmark);
criterion_main!(benches);
