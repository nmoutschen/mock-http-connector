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

use crate::{handler::DefaultHandler, response::ResponseFuture, Connector};

pub struct MockStream<FE, FM> {
    res: ResponseState,
    req_data: Vec<u8>,
    waker: Option<Waker>,

    uri: Uri,

    connector: Connector<FE, FM>,
}

impl<FE, FM> MockStream<FE, FM> {
    pub(crate) fn new(connector: Connector<FE, FM>, uri: Uri) -> Self {
        Self {
            res: ResponseState::New,
            req_data: Vec::new(),
            waker: None,
            uri,
            connector,
        }
    }
}

impl<FE, FM> Connection for MockStream<FE, FM> {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl<FE, FM> AsyncRead for MockStream<FE, FM>
where
    FE: DefaultHandler,
    FM: DefaultHandler,
{
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
                let res = ready!(Pin::new(fut).poll(cx)).unwrap_or_else(|_| self.connector.error());
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

impl<FE, FM> AsyncWrite for MockStream<FE, FM>
where
    FE: DefaultHandler,
    FM: DefaultHandler,
{
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

        let status = match req.parse(&self.req_data) {
            Ok(status) => status,
            Err(_err) => {
                self.res = ResponseState::Data(into_data(self.connector.error())?, 0);
                if let Some(w) = self.waker.take() {
                    w.wake()
                }
                return Poll::Ready(Ok(buf.len()));
            }
        };

        let body = match status {
            Status::Complete(body_pos) => &self.req_data[body_pos..],
            Status::Partial => &[],
        };

        self.res = match self.connector.matches(req, body, &self.uri) {
            Ok(Some(res)) => ResponseState::Fut(res),
            Ok(None) => ResponseState::Data(into_data(self.connector.missing())?, 0),
            Err(_err) => ResponseState::Data(into_data(self.connector.error())?, 0),
        };

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
