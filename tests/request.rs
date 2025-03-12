use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::error::Error as StdError;
use tower::{Service as _, ServiceExt};
mod helpers;
use helpers::*;

#[rstest]
#[case(Method::POST)] // Remark: hyper defaults to GET
#[case(Method::OPTIONS)]
#[tokio::test]
async fn test_request(#[case] method: Method) -> Result<(), Box<dyn StdError>> {
    // GIVEN
    // * a connector that expects a payload
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_method(method.clone())
        .returning((202, "OK"))?;

    let mut connector = builder.build();

    // WHEN making a request with the right Method
    let res = <Connector as ServiceExt<Request<String>>>::ready(&mut connector)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(method)
                .body("".to_string())
                .unwrap(),
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    Ok(())
}
