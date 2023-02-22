mod builder;
mod case;
mod connector;
mod error;
mod handler;
mod stream;

pub use builder::Builder;
use case::Case;
pub use connector::Connector;
pub use error::Error;

#[cfg(test)]
mod tests {
    use hyper::{Body, Request};

    use super::*;

    #[tokio::test]
    async fn simple_get() {
        let mut builder = Connector::builder();
        builder
            .expect()
            .times(1)
            .with_uri("http://example.com/test")
            .unwrap()
            .returning(|_| "OK");
        let connector = builder.build();

        let client = hyper::Client::builder().build::<_, Body>(connector.clone());
        let _res = client
            .request(
                Request::builder()
                    .uri("http://example.com/test")
                    .body("".to_string().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        connector.checkpoint().unwrap();
    }
}
