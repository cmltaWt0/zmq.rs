use async_trait::async_trait;
use dashmap::DashMap;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::codec::*;
use crate::error::*;
use crate::message::*;
use crate::util;
use crate::util::*;
use crate::{Socket, SocketType, ZmqResult};

pub struct RouterSocket {
    pub(crate) peers: Arc<DashMap<PeerIdentity, Peer>>,
    _accept_close_handle: futures::channel::oneshot::Sender<bool>,
}

impl Drop for RouterSocket {
    fn drop(&mut self) {
        self.peers.clear()
    }
}

impl RouterSocket {
    pub async fn bind(endpoint: &str) -> ZmqResult<Self> {
        let peers = Arc::new(DashMap::new());
        let router_socket = Self {
            peers: peers.clone(),
            _accept_close_handle: util::start_accepting_connections(
                endpoint,
                peers,
                SocketType::ROUTER,
            )
            .await?,
        };
        Ok(router_socket)
    }

    pub async fn recv_multipart(&mut self) -> ZmqResult<Vec<ZmqMessage>> {
        for mut peer in self.peers.iter_mut() {
            match peer.value_mut().recv_queue.try_next() {
                Ok(Some(Message::MultipartMessage(messages))) => return Ok(messages),
                Err(_) => continue,
                _ => todo!(),
            }
        }
        Err(ZmqError::NoMessage)
    }

    pub async fn send_multipart(&mut self, messages: Vec<ZmqMessage>) -> ZmqResult<()> {
        assert!(messages.len() > 2);
        let peer_id: PeerIdentity = messages[0].data.to_vec().try_into()?;
        match self.peers.get_mut(&peer_id) {
            Some(mut peer) => {
                peer.send_queue
                    .try_send(Message::MultipartMessage(messages[1..].to_vec()))?;
                Ok(())
            }
            None => return Err(ZmqError::Other("Destination client not found by identity")),
        }
    }
}

#[async_trait]
impl Socket for RouterSocket {
    async fn send(&mut self, _m: ZmqMessage) -> ZmqResult<()> {
        Err(ZmqError::Other(
            "This socket doesn't support sending individual messages",
        ))
    }

    async fn recv(&mut self) -> ZmqResult<ZmqMessage> {
        Err(ZmqError::Other(
            "This socket doesn't support receiving individual messages",
        ))
    }
}

pub struct DealerSocket {
    pub(crate) _inner: Framed<TcpStream, ZmqCodec>,
}

impl DealerSocket {
    pub async fn bind(_endpoint: &str) -> ZmqResult<Self> {
        todo!()
    }
}

#[async_trait]
impl Socket for DealerSocket {
    async fn send(&mut self, _m: ZmqMessage) -> ZmqResult<()> {
        unimplemented!()
    }

    async fn recv(&mut self) -> ZmqResult<ZmqMessage> {
        unimplemented!()
    }
}
