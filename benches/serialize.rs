use aws_embedded_metrics::{
    dimensions,
    log::MetricContext,
    serialize::{Log, Serialize},
    Unit,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn serialize(c: &mut Criterion) {
    c.bench_function("serialize", |b| {
        b.iter(|| {
            let mut ctx = MetricContext::default();
            ctx.put_metric("foo", 1, Unit::Seconds);
            ctx.put_metric("bar", 2, Unit::Bytes);
            ctx.put_dimensions(dimensions! {
                "foo" => "1"
            });
            ctx.put_dimensions(dimensions! {
                "bar" => "2",
                "baz" => "3"
            });
            Log.serialize(ctx);
        })
    });
}

criterion_group!(benches, serialize);
criterion_main!(benches);
