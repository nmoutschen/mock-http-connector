mod builder;
mod case;
mod case_old;
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
    use hyper::{Body, Request, Response};

    use super::*;

    #[tokio::test]
    async fn it_compiles() {
        let mut builder = Connector::builder();
        builder
            .expect()
            .times(1)
            .returning(|_| Response::builder().body(""));
        let connector = builder.build();

        let client = hyper::Client::builder().build::<_, Body>(connector.clone());
        let res = client
            .request(
                Request::builder()
                    .uri("http://example.com/")
                    .body("".to_string().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        connector.checkpoint().unwrap();

        dbg!(res);
    }
}
