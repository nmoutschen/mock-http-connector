use mock_http_connector::Connector;

#[cfg(feature = "hyper_0_14")]
use hyper_0_14::Request;

#[cfg(feature = "hyper_1")]
use hyper_1::Request;

#[cfg(feature = "hyper_0_14")]
pub fn client<C>(connector: C) -> hyper_0_14::Client<C>
where
    C: hyper_0_14::client::connect::Connect + Clone,
{
    hyper_0_14::Client::builder().build(connector)
}

#[cfg(feature = "hyper_1")]
pub fn client<C>(
    connector: C,
) -> hyper_util::client::legacy::Client<C, http_body_util::Full<hyper_1::body::Bytes>>
where
    C: hyper_util::client::legacy::connect::Connect + Clone,
{
    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector)
}

#[tokio::main]
async fn main() {
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .returning(|_| async { "" })
        .unwrap();
    let connector = builder.build();

    let client = client(connector.clone());
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
