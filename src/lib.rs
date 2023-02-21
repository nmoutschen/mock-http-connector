mod case;
mod error;
mod stream;

pub use case::Case;
use case::Cases;
pub use error::Error;
use stream::MockStream;

use std::{
    future::{ready, Ready},
    io,
    task::{Context, Poll},
};

use hyper::{service::Service, Uri};

#[derive(Clone, Default)]
pub struct Connector {
    cases: Cases,
}

impl Connector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect(&self, case: Case) -> Result<&Self, Error> {
        self.cases.add(case)?;
        Ok(self)
    }

    pub fn checkpoint(&self) -> Result<(), Error> {
        self.cases.checkpoint()
    }
}

impl Service<Uri> for Connector {
    type Response = MockStream;
    type Error = io::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Uri) -> Self::Future {
        ready(Ok(MockStream::new(self.cases.clone())))
    }
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Request};

    use super::*;

    #[tokio::test]
    async fn it_compiles() {
        let connector = Connector::new();
        connector
            .expect(Case::new().expect_path("/".to_string()))
            .unwrap();

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
