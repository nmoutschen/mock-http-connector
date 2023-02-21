use std::{
    cmp::min,
    io,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use httparse::{Request, Status};
use hyper::{
    client::connect::{Connected, Connection},
    Response,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{Cases, Error};

pub struct MockStream {
    res_data: Vec<u8>,
    res_pos: usize,
    req_data: Vec<u8>,
    waker: Option<Waker>,
    cases: Cases,
}

impl MockStream {
    pub(crate) fn new(cases: Cases) -> Self {
        Self {
            res_data: Vec::new(),
            res_pos: 0,
            req_data: Vec::new(),
            waker: None,
            cases,
        }
    }
}

impl Connection for MockStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.res_data.is_empty() {
            self.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        let size = min(buf.remaining(), self.res_data.len() - self.res_pos);
        buf.put_slice(&self.res_data[self.res_pos..self.res_pos + size]);
        self.res_pos += size;

        self.waker = Some(cx.waker().clone());

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockStream {
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut headers);
        self.req_data.extend(buf);
        let body = match req
            .parse(&self.res_data)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?
        {
            Status::Complete(body_pos) => &self.req_data[body_pos..],
            Status::Partial => &[],
        };

        match self.cases.matches(req, body) {
            Ok(Some(res)) => {
                self.res_data = into_data(res)?;
                self.waker.take().map(|w| w.wake());

                Poll::Ready(Ok(buf.len()))
            }
            Ok(None) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                Error::ResponseNotFound,
            ))),
            Err(err) => Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, err))),
        }
    }
}

fn into_data(res: Response<String>) -> Result<Vec<u8>, io::Error> {
    let mut data = String::new();
    data.push_str("HTTP/1.1 200 OK\r\n");

    for (name, value) in res.headers() {
        data.push_str(&format!(
            "{name}: {}\r\n",
            value
                .to_str()
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err))?
        ));
    }

    data.push_str("\r\n");
    data.push_str(res.body());
    data.push_str("\r\n\r\n");

    Ok(data.into_bytes())
}
