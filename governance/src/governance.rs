
pub mod handlers;
pub mod routes;
pub mod models;
pub mod error;

use warp::Filter;

#[tokio::main]
async fn main() {
    let api = routes::governance_routes();

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
