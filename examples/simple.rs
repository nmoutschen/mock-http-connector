use hyper::{Body, Request};
use mock_http_connector::Connector;

#[tokio::main]
async fn main() {
    let mut builder = Connector::builder();
    builder.expect().times(1).returning(|_| async { "" });
    let connector = builder.build();

    let client = hyper::Client::builder().build::<_, Body>(connector.clone());
    let res = client
        .request(
            Request::builder()
                .uri("http://example.com/test")
                .body("".to_string().into())
                .unwrap(),
        )
        .await
        .unwrap();

    connector.checkpoint().unwrap();

    dbg!(res);
}
