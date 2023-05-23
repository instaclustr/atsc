
//! To configure Prometheus to send samples to this binary, add the following to your prometheus.yml:
//!
//! ```yml
//! remote_write:
//!   - url: "http://localhost:9201/api/write"
//!
//! remote_read:
//!   - url: "http://localhost:9201/api/read"
//! ```

use async_trait::async_trait;
use std::{convert::Infallible, sync::Arc};

use prom_remote_api::{
    types::{
        Error, Label, Query, QueryResult, RemoteStorage, Result, Sample, TimeSeries, WriteRequest,
    },
    web,
};
use warp::Filter;

#[derive(Clone, Copy)]
struct MockStorage;

fn generate_samples(start_ms: i64, end_ms: i64, step_ms: i64) -> Vec<Sample> {
    // instant query
    if step_ms == 0 {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }

    // range query
    (start_ms..end_ms)
        .step_by(step_ms as usize)
        .enumerate()
        .map(|(i, timestamp)| Sample {
            value: 1.0 + i as f64,
            timestamp,
        })
        .collect()
}
impl MockStorage {
    fn with_context() -> impl Filter<Extract = (u64,), Error = Infallible> + Clone {
        warp::any().map(|| 1)
    }
}

#[async_trait]
impl RemoteStorage for MockStorage {
    type Err = Error;
    type Context = u64;

    async fn write(&self, _ctx: Self::Context, req: WriteRequest) -> Result<()> {
        println!("mock write, req:{req:?}");
        Ok(())
    }

    async fn process_query(&self, _ctx: &Self::Context, query: Query) -> Result<QueryResult> {
        println!("mock read, req:{query:?}");

        Ok(QueryResult {
            timeseries: vec![TimeSeries {
                labels: vec![
                    Label {
                        name: "job".to_string(),
                        value: "mock-remote".to_string(),
                    },
                    Label {
                        name: "instance".to_string(),
                        value: "127.0.0.1:9201".to_string(),
                    },
                    Label {
                        name: "__name__".to_string(),
                        value: "up".to_string(),
                    },
                ],
                samples: generate_samples(
                    query.start_timestamp_ms,
                    query.end_timestamp_ms,
                    query
                        .hints
                        .as_ref()
                        .map(|hint| hint.step_ms)
                        .unwrap_or(1000),
                ),
                ..Default::default()
            }],
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let storage = Arc::new(MockStorage);
    let write_api = warp::path!("write")
        .and(web::warp::with_remote_storage(storage.clone()))
        .and(MockStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::write);
    let query_api = warp::path!("read")
        .and(web::warp::with_remote_storage(storage))
        .and(MockStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::read);

    let routes = warp::path("api").and(write_api.or(query_api));

    let port = 9201;
    println!("Listen on {port}...");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}