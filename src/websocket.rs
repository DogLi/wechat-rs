pub mod message;

use anyhow::{bail, Error, Result};
use futures::stream::Stream;
use futures::task::{Context, Poll};
use message::Message;
use pin_project::pin_project;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message as WSMessage;

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
#[pin_project]
pub struct Websocket {
    #[pin]
    inner: WSStream,
}

impl Websocket {
    pub async fn new(ip: &str, port: u16) -> Result<Self> {
        let url = format!("ws://{}:{}", ip, port);
        let (stream, _) = connect_async(url).await?;
        let s = Self { inner: stream };
        Ok(s)
    }
}

impl Stream for Websocket {
    type Item = Result<Message, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = this.inner.poll_next(cx);
        match poll {
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(Some(Ok(m))) => match parse_message(m) {
                Ok(m) => Poll::Ready(Some(Ok(m))),
                Err(e) => Poll::Ready(Some(Err(e))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn parse_message(msg: WSMessage) -> Result<Message> {
    let m = match msg {
        WSMessage::Text(message) => match message.as_str() {
            "pong" => Message::Pong(vec![]),
            others => {
                // Message::Msg(others.into())
                let wechat_msg = match serde_json::from_str(others) {
                    Ok(r) => r,
                    Err(_e) => {
                        bail!("cant deserialize msg:\n{}", others);
                    }
                };
                Message::Msg(wechat_msg)
            }
        },
        WSMessage::Close(_) => Message::Closed,
        WSMessage::Ping(p) => Message::Ping(p),
        WSMessage::Pong(p) => Message::Pong(p),
        _ => {
            unreachable!()
        }
    };
    Ok(m)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::proto::DataType;
    use crate::websocket::message::WechatMsg;
    use futures::StreamExt;
    use std::env::var;

    #[tokio::test]
    async fn test_ws() {
        let ip = var("WX_IP").unwrap();
        let port = var("WX_PORT").unwrap().parse().unwrap();
        let mut ws = Websocket::new(&ip, port).await.unwrap();
        while let Some(Ok(Message::Msg(wechat_msg))) = ws.next().await {
            if let WechatMsg::TextMsg(msg) = wechat_msg {
                if msg.data_type == DataType::RecvTxtMsg {
                    println!("containt: {}", msg.content);
                    println!("wechat_id: {}", msg.wechat_id());
                    println!("room_id: {:?}", msg.room_id());
                }
            }
        }
    }
}
