use std::{
    cmp::min,
    io,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use httparse::{Request, Status};
use hyper::{
    client::connect::{Connected, Connection},
    Response, Uri,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{handler::DefaultHandler, Connector};

pub struct MockStream<FE, FM> {
    res_data: Vec<u8>,
    res_pos: usize,
    req_data: Vec<u8>,
    waker: Option<Waker>,

    uri: Uri,

    connector: Connector<FE, FM>,
}

impl<FE, FM> MockStream<FE, FM> {
    pub(crate) fn new(connector: Connector<FE, FM>, uri: Uri) -> Self {
        Self {
            res_data: Vec::new(),
            res_pos: 0,
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

impl<FE, FM> AsyncRead for MockStream<FE, FM> {
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
                self.res_data = into_data(self.connector.error())?;
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

        self.res_data = match self.connector.matches(req, body, &self.uri) {
            Ok(Some(res)) => into_data(res)?,
            Ok(None) => into_data(self.connector.missing())?,
            Err(_err) => into_data(self.connector.error())?,
        };

        if let Some(w) = self.waker.take() {
            w.wake()
        }

        Poll::Ready(Ok(buf.len()))
    }
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
    data.push_str("\r\n\r\n");

    Ok(data.into_bytes())
}
