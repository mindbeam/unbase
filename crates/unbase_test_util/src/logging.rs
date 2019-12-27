
pub fn init_test_logger(name: &'static str) {
    #[cfg(not(target_arch = "wasm32"))]
    native::init(name);

    #[cfg(target_arch = "wasm32")]
    wasm::init(name);
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub (crate) fn init(_name: &'static str) {
        #[cfg(feature="trace_basic")]
        basic::init(_name);

        #[cfg(feature="trace_jaeger")]
        jaeger::init(_name);
    }
    #[cfg(all(feature="trace_basic"))]
    mod basic {
        pub fn init(_name: &'static str) {
            tracing_subscriber::fmt::init();
        }
    }
    #[cfg(feature = "trace_jaeger")]
    mod jaeger {
        pub fn init(name: &'static str) {
            use opentelemetry::{api::{Provider, Sampler}, exporter::trace::jaeger, global, sdk};
            use tracing_opentelemetry::OpentelemetryLayer;
            use tracing_subscriber::{Layer, Registry};

            let exporter = jaeger::Exporter::builder()
                .with_collector_endpoint("127.0.0.1:6831".parse().unwrap())
                .with_process(jaeger::Process {
                    service_name: name,
                    tags: Vec::new(),
                })
                .init();
            let provider = sdk::Provider::builder()
                .with_exporter(exporter)
                .with_config(sdk::Config {
                    default_sampler: Sampler::Always,
                    ..Default::default()
                })
                .build();
            global::set_provider(provider);

            let tracer = global::trace_provider().get_tracer("tracing");
            let opentelemetry = OpentelemetryLayer::with_tracer(tracer);
            let subscriber = opentelemetry.with_subscriber(Registry::default());

            tracing::subscriber::set_global_default(subscriber).unwrap();
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    pub fn init(_name: &'static str) {
        log::set_logger(&wasm_bindgen_console_logger::DEFAULT_LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
        LogTracer::init().unwrap();
    }
}