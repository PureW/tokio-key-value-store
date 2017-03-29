extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use std::fmt;
use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use futures::{future, Future, BoxFuture};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::TcpServer;

extern crate key_value_store;
use key_value_store::keyvalue::keyvalue::KeyValueStore;

pub enum InputEventType {
    Set,
    Get,
    Unknown,
}

pub struct InputEvent {
    event: InputEventType,
    key: String,
    value: Option<String>,
}

impl InputEvent {
    pub fn new(evt: InputEventType,
               key: &str,
               value: Option<&str>)
               -> Result<InputEvent, io::Error> {
        // TODO: Use Into-trait to accept String as well as &str
        let key = key.to_string();
        let sval = value.map(|s| s.to_string());

        Ok(InputEvent {
            event: evt,
            key: key,
            value: sval,
        })
    }
    pub fn new_from_str(evt: &str,
                        key: &str,
                        value: Option<&str>)
                        -> Result<InputEvent, io::Error> {
        let mut etype = match evt {
            "set" => InputEventType::Set,
            "get" => InputEventType::Get,
            _     => InputEventType::Unknown,

        };
        InputEvent::new(etype, key, value)
    }
}

pub struct LineCodec;

impl LineCodec {
    fn decode_line(&self, line: &str) -> Result<InputEvent, io::Error> {
        let parts: Vec<&str> = line.split("\t").collect();
        println!("Got {} parts", parts.len());
        match parts.len() {
            2 | 3 => {
                let (etype, key, val) = (parts[0], parts[1], parts.get(2));
                let val = val.map(|s| *s);
                println!("{} {} {:?}", etype, key, val);
                InputEvent::new_from_str(etype, key, val)
            }
            _ => Err(io::Error::new(io::ErrorKind::Other, "bad data")),
        }
    }
}

impl Decoder for LineCodec {
    type Item = InputEvent;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<InputEvent>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(i);

            // Also remove the '\n'
            buf.split_to(1);

            // Turn this data into a UTF string and return it in a Frame.
            match str::from_utf8(&line) {
                Ok(s) => {
                    println!("Decoding {}...", s);
                    match self.decode_line(s) {
                        Ok(evt) => Ok(Some(evt)),
                        Err(err) => Err(err),
                    }
                }
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }
}

pub enum KVResponse {
    Ok,
    Nil,
    SimpleStr(String),
    Error(String, String), // Integer(i32),
}


impl Encoder for LineCodec {
    type Item = KVResponse;
    type Error = io::Error;

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
    type Request = InputEvent;

    /// For this protocol style, `Response` matches the coded `Out` type
    type Response = KVResponse;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

pub struct KVService {
    map: KeyValueStore,
}

impl KVService {
    pub fn new() -> KVService {
        KVService { map: KeyValueStore::new() }
    }
}

impl Service for KVService {
    // These types must match the corresponding protocol types:
    type Request = InputEvent;
    type Response = KVResponse;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        let resp = match req.event {
            InputEventType::Set => {
                match req.value {
                    Some(val) => {
                        // TODO: Wont let me mutably borrow to modify map...
                        //self.map.set(req.key, val.clone());
                        KVResponse::Ok
                    },
                    None => KVResponse::Error("ERR".to_string(), "no value to set".to_string()),
                }
            }
            InputEventType::Get => {
                match self.map.get(&req.key) {
                    Some(v) => KVResponse::SimpleStr(v.clone()),
                    None => KVResponse::Nil,
                }
            },
            InputEventType::Unknown => {
                KVResponse::Error("ERR".to_string(), format!("unknown command"))
            }
        };

        future::ok(resp).boxed()
    }
}

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(|| Ok(KVService::new()));
}
