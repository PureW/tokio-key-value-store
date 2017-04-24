/// Very simple key-value store implemented on top of Tokio.
/// Implements a simple protocol similar to Redis' RESP.

extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use std::error::Error;
use std::sync::Arc;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use futures::{future, Future, BoxFuture};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::TcpServer;

extern crate key_value_store;
use key_value_store::keyvalue::keyvalue::KeyValueStore;

/// The `Command` codifies a request
#[derive(Debug)]
pub enum Command {
    Set(String, String),
    Get(String),
    Unknown,
}

impl Command {
    /// Build a `Command from a `String`
    pub fn from_line(line: &str) -> Result<Command, io::Error> {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some(cmd) => {
                let parts: Vec<_> = parts.map(|s| s.to_string()).collect();

                Command::new_from_str(cmd, parts)
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "no command")),
        }
    }

    /// Helper-func to verify input and generate error-message for faulty input
    fn verify_input_args(cmd: &str, num_args: usize, args: &Vec<String>) -> Result<(), io::Error> {
        if args.len() != num_args {
            Err(io::Error::new(io::ErrorKind::Other,
                               format!("wrong number of arguments to '{:?}'-command", cmd)))
        } else {
            Ok(())
        }
    }

    /// Logic for building a `Command`
    fn new_from_str(evt: &str, mut parts: Vec<String>) -> Result<Command, io::Error> {
        match evt {
            "set" => {
                try!(Command::verify_input_args(evt, 2, &parts));
                let mut p = parts.drain(0..);
                Ok(Command::Set(p.next().unwrap(), p.next().unwrap()))
            }
            "get" => {
                try!(Command::verify_input_args(evt, 1, &parts));
                let mut p = parts.drain(0..);
                Ok(Command::Get(p.next().unwrap()))
            }
            _ => Err(io::Error::new(io::ErrorKind::Other, format!("unknown command"))),
        }
    }
}

/// `LineCodec` converts from bytes into higher-level primitives
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    /// `decode` produces a `String` from bytes on the wire.
    ///
    /// Protocol is string-based utf-8 encoded separated by newlines (\n)
    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(i);

            // Also remove the '\n'
            buf.split_to(1);

            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }
}

/// Codifies the response from daemon
pub enum KVResponse {
    Ok,
    Nil,
    SimpleStr(String),
    Error(String, String), // Integer(i32),
}


impl Encoder for LineCodec {
    type Item = KVResponse;
    type Error = io::Error;

    /// `encode` translates from a `KVResponse` into the corresponding byte-stream.
    fn encode(&mut self, resp: KVResponse, buf: &mut BytesMut) -> io::Result<()> {
        println!("Encoding response...");
        let s: String = match resp {
            KVResponse::Ok => "+OK".to_string(),
            KVResponse::Nil => "$-1".to_string(),
            KVResponse::SimpleStr(s) => format!("+{}", s).to_string(),
            KVResponse::Error(error, msg) => format!("-{} {}", error, msg).to_string(),
        };
        buf.extend(s.as_bytes());
        buf.extend(b"\r\n");
        Ok(())
    }
}

pub struct LineProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    /// For this protocol style, `Request` matches the codec `In` type
    type Request = String;

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = KVResponse;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

/// Service exposing `KeyValueStore` to Tokio
pub struct KVService {
    map: Arc<KeyValueStore>,
}

impl KVService {
    pub fn new(map: Arc<KeyValueStore>) -> KVService {
        KVService { map: map }
    }

    /// Process a `Command`.
    ///
    /// This takes place after Tokio translates from bytes to `Command`.
    ///
    /// Returns a `KVResponse` which Tokio translates to bytes.
    fn handle_req(&self, req: &Command) -> KVResponse {
        match req {
            &Command::Set(ref k, ref v) => {
                self.map.set(k.clone(), v.clone());
                KVResponse::Ok
            }
            &Command::Get(ref k) => {
                match self.map.get(&k) {
                    Some(v) => KVResponse::SimpleStr((*v).clone()),
                    None => KVResponse::Nil,
                }
            }
            &Command::Unknown => KVResponse::Error("ERR".to_string(), format!("unknown command")),
        }
    }
}

/// Tokio details of gluing `KVService` into tokio-framework
impl Service for KVService {
    // These types must match the corresponding protocol types:
    type Request = String;
    type Response = KVResponse;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, line: Self::Request) -> Self::Future {
        let resp = match Command::from_line(&line) {
            Ok(req) => self.handle_req(&req),
            Err(e) => KVResponse::Error("ERR".to_string(), e.description().to_string()),
        };

        future::ok(resp).boxed()
    }
}

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    broken code

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    let map = Arc::new(KeyValueStore::new());
    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(move || Ok(KVService { map: map.clone() }));
}
