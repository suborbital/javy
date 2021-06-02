use criterion::{criterion_group, criterion_main, Criterion};
use lucet_runtime::{DlModule, Limits, MmapRegion, Region};
use lucetc::{Lucetc, LucetcOpts};
use lucet_wasi::{WasiCtxBuilder, bindings};
use std::sync::Arc;

fn dlmodule(name: &str, bytes: impl AsRef<[u8]>) -> Arc<DlModule> {
    let bindings = bindings();
    let path = format!("./{}.so", name);
    let output_path = std::path::Path::new(&path);
    let compiler = Lucetc::try_from_bytes(bytes)
        .unwrap()
        .with_opt_level(lucetc::OptLevel::SpeedAndSize)
        .with_bindings(bindings);

    compiler.shared_object_file(&output_path).unwrap();
    lucet_runtime::lucet_internal_ensure_linked();
    lucet_wasi::export_wasi_funcs();
    DlModule::load(&output_path).unwrap()
}

fn region() -> Arc<MmapRegion> {
    MmapRegion::create(
        1,
        &Limits {
            heap_memory_size: 8 * 1024 * 1024,
            stack_size: 128 * 1024,
            ..Limits::default()
        },
    ).unwrap()
}

fn exec_async(region: Arc<MmapRegion>, module: Arc<DlModule>, tokio_runtime: &tokio::runtime::Runtime) {
    let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build()
            .unwrap();


    let mut instance = region.new_instance(module).unwrap();
    instance.insert_embed_ctx(wasi_ctx);

    let result = tokio_runtime
        .block_on(instance.run_async("run", &[], None))
        .unwrap();

    assert_eq!(result.as_i32(), 8);
}

fn exec(region: Arc<MmapRegion>, module: Arc<DlModule>) {
    let mut instance = region.new_instance(module).unwrap();
    let result = instance.run("run", &[]).unwrap().returned().unwrap().as_i32();
    assert_eq!(result, 8);
}

fn lucet(c: &mut Criterion) {
    let mut group = c.benchmark_group("qjs lucet");
    group.bench_function("control", |b| {
        let dl = dlmodule("javy.control.wasm", &include_bytes!("javy.control.wasm"));
        let region = region();
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        b.iter(|| exec_async(region.clone(), dl.clone(), &tokio_runtime));
    });

    group.bench_function("wizer", |b| {
        let dl = dlmodule("javy.control.wasm", &include_bytes!("javy.opt.wizer.wasm"));
        let region = region();
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        b.iter(|| exec_async(region.clone(), dl.clone(), &tokio_runtime));
    });

    group.bench_function("assemblyscript.optimized", |b| {
        let dl = dlmodule("javy.control.wasm", &include_bytes!("as.optimized.wasm"));
        let region = region();

        b.iter(|| exec(region.clone(), dl.clone()));
    });

    group.finish();
}

criterion_group!(benches, lucet);
criterion_main!(benches);

