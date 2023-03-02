use std::{
    cmp::min,
    future::Future,
    io,
    pin::Pin,
    task::{ready, Context, Poll, Waker},
};

use httparse::{Request, Status};
use hyper::{
    client::connect::{Connected, Connection},
    Response, Uri,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{response::ResponseFuture, Connector, Error};

pub struct MockStream {
    res: ResponseState,
    req_data: Vec<u8>,
    waker: Option<Waker>,

    uri: Uri,

    connector: Connector,
}

impl MockStream {
    pub(crate) fn new(connector: Connector, uri: Uri) -> Self {
        Self {
            res: ResponseState::New,
            req_data: Vec::new(),
            waker: None,
            uri,
            connector,
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
        let (data, mut pos) = match &mut self.res {
            ResponseState::New => {
                self.waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
            ResponseState::Fut(fut) => {
                let res = ready!(Pin::new(fut).poll(cx))
                    .map_err(|err| into_connect_error(Error::Runtime(err)))?;
                (into_data(res)?, 0)
            }
            ResponseState::Data(data, pos) => (data.clone(), *pos),
        };

        let size = min(buf.remaining(), data.len() - pos);
        buf.put_slice(&data[pos..pos + size]);
        pos += size;

        self.res = ResponseState::Data(data, pos);

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

        let status = req
            .parse(&self.req_data)
            .map_err(|err| into_connect_error(err.into()))?;

        let body = match status {
            Status::Complete(body_pos) => &self.req_data[body_pos..],
            Status::Partial => &[],
        };

        self.res = ResponseState::Fut(
            self.connector
                .matches(req, body, &self.uri)
                .map_err(into_connect_error)?,
        );

        if let Some(w) = self.waker.take() {
            w.wake()
        }

        Poll::Ready(Ok(buf.len()))
    }
}

#[derive(Default)]
enum ResponseState {
    #[default]
    New,
    Fut(ResponseFuture),
    Data(Vec<u8>, usize),
}

fn into_data(res: Response<String>) -> Result<Vec<u8>, io::Error> {
    let mut data = String::new();
    let status = res.status();
    data.push_str(&format!(
        "HTTP/1.1 {} {}\r\n",
        status.as_u16(),
        status.as_str()
    ));

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

    Ok(data.into_bytes())
}

fn into_connect_error(err: Error) -> io::Error {
    io::Error::new(io::ErrorKind::ConnectionRefused, err)
}
