use criterion::{criterion_group, criterion_main, Criterion};
use search::{filter_engine_configuration, SearchUserEnvironment};

const BASIC_CONFIG: &str = include_str!("./search-config-v2.json");

fn criterion_benchmark(c: &mut Criterion) {
    let config = BASIC_CONFIG.to_string();
    let env = SearchUserEnvironment {
        locale: "en-GB".into(),
        region: "GB".into(),
        channel: String::new(),
        distribution_id: String::new(),
        experiment: String::new(),
        app_name: String::new(),
        version: String::new(),
    };

    c.bench_function("filter_engine_configuration", |b| {
        b.iter(|| filter_engine_configuration(env.clone(), config.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
