//! Sinks contains interfaces and implementations for reporting metric
//! data to an external system
use crate::{log::MetricContext, serialize::Serialize};
use hyper::Uri;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
};

pub trait Sink {
    fn accept(
        &mut self,
        context: MetricContext,
    );
}

pub(crate) struct Lambda(dyn Serialize);

impl Sink for Lambda {
    fn accept(
        &mut self,
        context: MetricContext,
    ) {
        println!("{}", self.0.serialize(context))
    }
}

pub(crate) struct Agent {
    log_group_name: String,
    log_stream_name: Option<String>,
    transport: Transport,
    serializer: Box<dyn Serialize + 'static>,
}

enum Transport {
    Tcp(TcpStream),
    Udp(UdpSocket),
}

impl Transport {
    async fn send(
        &mut self,
        bytes: &[u8],
    ) {
        // todo: communicate errs
        match self {
            Transport::Udp(stream) => {
                drop(stream.send(bytes).await);
            }
            Transport::Tcp(stream) => {
                drop(stream.write_all(bytes).await);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Endpoint {
    Tcp(String, u16),
    Udp(String, u16),
}

impl Agent {
    fn parse(endpoint: impl AsRef<str>) -> Option<Endpoint> {
        let uri = endpoint.as_ref().parse::<Uri>().ok()?;
        let (host, port) = (uri.host()?, uri.port()?.as_u16());
        match uri.scheme()?.as_str() {
            "tcp" => Some(Endpoint::Tcp(host.into(), port)),
            "udp" => Some(Endpoint::Udp(host.into(), port)),
            _ => None,
        }
    }

    pub async fn create(
        log_group_name: String,
        log_stream_name: Option<String>,
        config_endpoint: Option<String>,
        serializer: impl Serialize + 'static,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = match config_endpoint.and_then(Self::parse) {
            Some(Endpoint::Tcp(host, port)) => {
                Transport::Tcp(TcpStream::connect((host.as_str(), port)).await?)
            }
            Some(Endpoint::Udp(host, port)) => {
                Transport::Udp(UdpSocket::bind((host.as_str(), port)).await?)
            }
            _ => Transport::Tcp(TcpStream::connect("0.0.0.0:25888").await?),
        };
        Ok(Self {
            log_group_name,
            log_stream_name,
            transport,
            serializer: Box::new(serializer),
        })
    }
}

impl Sink for Agent {
    fn accept(
        &mut self,
        context: MetricContext,
    ) {
        let mut editable = context;
        editable
            .meta
            .insert("LogGroupName".into(), self.log_group_name.as_str().into());
        if let Some(stream) = &self.log_stream_name {
            editable
                .meta
                .insert("LogStreamName".into(), stream.as_str().into());
        }

        let payload = self.serializer.serialize(editable);
        self.transport.send(&payload.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_parses_udp_endpoint() {
        assert_eq!(
            Agent::parse("udp://0.0.0.0:7890"),
            Some(Endpoint::Udp("0.0.0.0".into(), 7890))
        )
    }

    #[test]
    fn agent_parses_tcp_endpoint() {
        assert_eq!(
            Agent::parse("tcp://0.0.0.0:7890"),
            Some(Endpoint::Tcp("0.0.0.0".into(), 7890))
        )
    }

    #[test]
    fn agent_ignores_other_endpoint() {
        assert_eq!(Agent::parse("other://0.0.0.0:7890"), None)
    }
}
