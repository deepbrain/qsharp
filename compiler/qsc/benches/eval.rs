// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::interpret::stateless;
use qsc_eval::output::GenericReceiver;
use qsc_frontend::compile::SourceMap;

const TELEPORT: &str = include_str!("../../../samples/algorithms/Teleportation.qs");
const DEUTSCHJOZSA: &str = include_str!("../../../samples/algorithms/DeutschJozsa.qs");
const LARGE: &str = include_str!("./large.qs");

pub fn teleport(c: &mut Criterion) {
    c.bench_function("Teleport evaluation", |b| {
        let sources = SourceMap::new([("Teleportation.qs".into(), TELEPORT.into())], None);
        let evaluator = stateless::Interpreter::new(true, sources).expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            let mut eval_ctx = evaluator.new_eval_context();
            assert!(eval_ctx.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn deutsch_jozsa(c: &mut Criterion) {
    c.bench_function("Deutsch-Jozsa evaluation", |b| {
        let sources = SourceMap::new([("DeutschJozsa.qs".into(), DEUTSCHJOZSA.into())], None);
        let evaluator = stateless::Interpreter::new(true, sources).expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            let mut eval_ctx = evaluator.new_eval_context();
            assert!(eval_ctx.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn large_file(c: &mut Criterion) {
    c.bench_function("Large file parity evaluation", |b| {
        let sources = SourceMap::new([("large.qs".into(), LARGE.into())], None);
        let evaluator = stateless::Interpreter::new(true, sources).expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            let mut eval_ctx = evaluator.new_eval_context();
            assert!(eval_ctx.eval_entry(&mut rec).is_ok());
        })
    });
}

criterion_group!(benches, teleport, deutsch_jozsa, large_file);
criterion_main!(benches);
