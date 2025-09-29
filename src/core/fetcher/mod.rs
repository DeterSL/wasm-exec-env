mod component_fetcher;
mod fs_fetcher;
mod http_fetcher;

pub use component_fetcher::{get_component_fetcher_for_source, get_one_time_component_fetcher_for_source, OneTimeFetcherFn, ComponentFetcher};
