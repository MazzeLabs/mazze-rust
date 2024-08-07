pub mod handlers;
pub mod routes;
pub mod models;
pub mod error;

use warp::Filter;

#[tokio::main]
async fn main() {
    let api = filters::governance_routes();

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

mod filters {
    use super::handlers;
    use warp::Filter;

    pub fn governance_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("governance")
            .and(
                warp::get()
                    .and(warp::path("status"))
                    .and_then(handlers::get_status)
            )
            .or(
                warp::post()
                    .and(warp::path("vote"))
                    .and(warp::body::json())
                    .and_then(handlers::cast_vote)
            )
    }
}

