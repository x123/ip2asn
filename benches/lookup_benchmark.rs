use criterion::{Criterion, criterion_group, criterion_main};
use ip2asn::{Builder, IpAsnMap};
use std::hint::black_box;
use std::net::{IpAddr, Ipv4Addr};

// Path to the dataset provided by the user.
const DATASET_PATH: &str = "testdata/ip2asn-combined.tsv.gz";

// A function to create the map for the benchmarks.
// It will be used by the lookup benchmarks.
fn create_map() -> IpAsnMap {
    Builder::new()
        .from_path(DATASET_PATH)
        .expect("Failed to build map from testdata")
        .build()
        .expect("Failed to build map")
}

fn benchmark_build(c: &mut Criterion) {
    // The build benchmark group will measure the time to build the map from the large dataset.
    // This is a heavy operation, so we'll sample it fewer times.
    let mut group = c.benchmark_group("build");
    group.sample_size(10);
    group.bench_function("build_from_large_dataset", |b| {
        b.iter(|| {
            let builder = Builder::new()
                .from_path(black_box(DATASET_PATH))
                .expect("Failed to create builder");
            let _map = builder.build().expect("Failed to build map");
        })
    });
    group.finish();
}

fn benchmark_lookups(c: &mut Criterion) {
    let map = create_map();

    // IPs for testing hits. These are known to be in the dataset.
    let ipv4_hit: IpAddr = Ipv4Addr::new(1, 0, 0, 1).into(); // Part of 1.0.0.0/24
    let ipv6_hit: IpAddr = "2001:200::1".parse().unwrap(); // Part of 2001:200::/32

    // IPs for testing misses. These should not be in the dataset.
    let ipv4_miss: IpAddr = Ipv4Addr::new(10, 0, 0, 1).into(); // Private range
    let ipv6_miss: IpAddr = "::1".parse().unwrap(); // Loopback

    let mut group = c.benchmark_group("lookups");

    group.bench_function("lookup_ipv4_hit", |b| {
        b.iter(|| map.lookup(black_box(ipv4_hit)))
    });

    group.bench_function("lookup_ipv6_hit", |b| {
        b.iter(|| map.lookup(black_box(ipv6_hit)))
    });

    group.bench_function("lookup_ipv4_miss", |b| {
        b.iter(|| map.lookup(black_box(ipv4_miss)))
    });

    group.bench_function("lookup_ipv6_miss", |b| {
        b.iter(|| map.lookup(black_box(ipv6_miss)))
    });
    group.finish();
}

criterion_group!(benches, benchmark_build, benchmark_lookups);
criterion_main!(benches);
