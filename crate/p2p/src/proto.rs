use std::{net::SocketAddr};

use bytes::{BytesMut, BufMut, Buf};
use num_enum::{TryFromPrimitive, IntoPrimitive};
use tokio_util::codec::{Encoder, Decoder};
use byteorder::{ReadBytesExt, BigEndian};


use crate::{err, event, peer::{PeerMetadata, DeviceType, PeerId}};


pub(crate) const SIGNATURE: [u8; 2] = hex_literal::hex!("4040");


// pub(crate) trait Length {
//     fn get_length(&self) -> u16;
// }
// https://developerlife.com/2022/03/30/rust-proc-macro/
// https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#customderivemacros
// rust custom derive macro

pub struct DiscoveryCodec;


impl Decoder for DiscoveryCodec {
    type Item = event::DiscoveryEvent;
    type Error = err::ParseError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(header) = HeaderCodec.decode(src)? else {
            return Ok(None);
        };

        if header.message_type != MessageType::Discovery {
            return Err(Self::Error::MsgType(header.message_type));
        }

        match src.get_u8() {
            0 => Ok(Some(event::DiscoveryEvent::PresenceRequest)),
            1 => {
                let device_type_raw = src.get_u16();
                let device_name_length = src.get_u16();
                let device_name_bytes = src.split_to(device_name_length.into());
                let device_name_raw = &device_name_bytes[..];
                let device_name = String::from_utf8(device_name_raw.to_vec()).unwrap();
                let device_id_raw = src.split_to(40);
                let device_id = String::from_utf8(device_id_raw.to_vec()).unwrap();
                let id = PeerId::from_string(device_id)?;
                let device_addr_length = src.get_u16();
                let device_addr_bytes = src.split_to(device_addr_length.into());
                let device_addr_str = String::from_utf8(device_addr_bytes.to_vec()).unwrap();
                let device_addr: SocketAddr = device_addr_str.parse()?;
                let device_type = DeviceType::try_from_primitive(device_type_raw)?;
                
                Ok(Some(event::DiscoveryEvent::PresenceResponse(PeerMetadata {
                    typ: device_type,
                    name: device_name,
                    id,
                    addr: device_addr
                })))
            },
            x => Err(Self::Error::Enum(x.into()))
        }
        
    }
}

impl Encoder<event::DiscoveryEvent> for DiscoveryCodec {
    type Error = err::ParseError;

    fn encode(&mut self, item: event::DiscoveryEvent, dst: &mut BytesMut) -> Result<(), Self::Error> {        
        
        HeaderCodec.encode(Header::new(MessageType::Discovery, &item), dst)?;
        match item {
            event::DiscoveryEvent::PresenceRequest => {
                dst.put_u8(0); // DiscoveryType
            },
            event::DiscoveryEvent::PresenceResponse(metadata) => {
                dst.put_u8(1); // DiscoveryType
                dst.put_u16(metadata.typ.into()); // DeviceType
                dst.put_u16(metadata.name.len().try_into().unwrap()); // DeviceNameLength
                dst.put(metadata.name.as_bytes()); // DeviceName
                dst.put(metadata.id.as_bytes()); // DeviceId
                let addr = metadata.addr.to_string(); // DeviceAddressLength
                dst.put_u16(u16::try_from(addr.len()).unwrap()); // DeviceAddress
                dst.put(addr.as_bytes());
            }
        }
        Ok(())
    }
}

pub struct ConnectionCodec;

pub enum Connection {
    Request{id: PeerId, tag: Vec<u8>}, // sent by client
    Response(Vec<u8>), // sent by host
    CompleteRequest, // sent by client
    CompleteResponse, // sent by host
    Failure(u32) // sent by either on error
}

impl Frame for Connection {
    fn len(&self) -> u16 {
        match self {
            Connection::Request { .. } => 1 + 40 + 32,
            Connection::Response(_) => 1 + 32,
            Connection::CompleteRequest => 1,
            Connection::CompleteResponse => 1,
            Connection::Failure(_) => 1 + 4
        }
    }
}



impl Decoder for ConnectionCodec {
    type Item = Connection;

    type Error = err::ParseError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(header) = HeaderCodec.decode(src)? else {
            return Ok(None);
        };

        if header.message_type != MessageType::Connect {
            return Err(Self::Error::MsgType(header.message_type));
        }

        match src.get_u8() {
            0 => {
                let peer_id_raw = src.split_to(40);
                let peer_id = PeerId::from_string(String::from_utf8(peer_id_raw.to_vec()).unwrap()).unwrap();
                let hmac = src.split_to(32).to_vec();
                Ok(Some(Connection::Request { id: peer_id, tag: hmac }))
            },
            1 => {
                let hmac = src.split_to(32).to_vec();
                Ok(Some(Connection::Response(hmac)))
            },
            2 => {
                Ok(Some(Connection::CompleteRequest))
            },
            3 => {
                Ok(Some(Connection::CompleteResponse))
            },
            4 => {
                Ok(Some(Connection::Failure(src.get_u32())))
            },
            x => {
                Err(Self::Error::Enum(x.into()))
            }
        }
    }
}

impl Encoder<Connection> for ConnectionCodec {
    type Error = err::ParseError;

    fn encode(&mut self, item: Connection, dst: &mut BytesMut) -> Result<(), Self::Error> {
        HeaderCodec.encode(Header::new(MessageType::Connect, &item), dst)?;
        match item {
            Connection::Request { id, tag } => {
                dst.put_u8(0);
                dst.put(id.as_bytes());
                dst.put(tag.as_ref());
            },
            Connection::Response(tag) => {
                dst.put_u8(1);
                dst.put(tag.as_ref());
            },
            Connection::CompleteRequest => {
                dst.put_u8(2);
            },
            Connection::CompleteResponse => {
                dst.put_u8(3);
            }
            Connection::Failure(code) => {
                dst.put_u8(4);
                dst.put_u32(code);
            }
        }
        Ok(())
    }
}





pub struct HeaderCodec;

impl Decoder for HeaderCodec {
    type Item = Header;

    type Error = err::ParseError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 5 {
            return Ok(None)
        }

        //let mut peek = Cursor::new(&src[..5]);


        // if !src.starts_with(&SIGNATURE) {
        //     return Err(HeaderError::NotAHeader);
        // }

        // let message_length = peek.get_u16();


        let Some(signature_raw) = src.get(0..2) else {
            return Ok(None)
        };
        if signature_raw != SIGNATURE {
            return Err(Self::Error::NotAPacket);
        }

        // if signature_raw != SIGNATURE {
        //     return Err(HeaderError::NotAHeader);
        // }

        // let message_length = src.get_u16();

        let Some(mut len_bytes) = src.get(2..4) else {
            return Ok(None);
        };
        let Ok(message_length) = len_bytes.read_u16::<BigEndian>() else {
            return Ok(None);
        };
        if src.len() < message_length.into() {
            return Ok(None);
        }
        src.advance(4);
        let message_type_raw = src.get_u8();
        let message_type = MessageType::try_from_primitive(message_type_raw)?;

        Ok(Some(Header { 
            length: message_length, 
            message_type, 
        }))
    
    }
}

impl Encoder<Header> for HeaderCodec {
    type Error = err::ParseError;

    fn encode(&mut self, item: Header, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(&SIGNATURE[..]); // signature
        dst.put_u16(item.length); // message len
        dst.put_u8(item.message_type.into()); // message type
        // dst.put_u64(item.request_id); // request id

        Ok(())
    }
}

pub struct Header {
    pub length: u16,
    pub message_type: MessageType,
}

impl Header {
    pub fn new(typ: MessageType, item: &impl Frame) -> Header {
        let mut header = Header {
            message_type: typ,
            length: item.len()
        };
        header.length += header.len();
        header
    }
}

impl Frame for Header {
    fn len(&self) -> u16 {
        2 + 2 + 1 // dont forget signature ;)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MessageType {
    // None = 0, 
    Discovery = 1, 
    Connect = 2,
    // Control = 3, 
    // Session = 4, 
    // Ack = 5 
}

/// Each frame needs to know it's length before sending
pub trait Frame {
    fn len(&self) -> u16;
}

#[cfg(test)]
mod tests {

    use std::{net::{SocketAddr, SocketAddrV4, Ipv4Addr}, fmt::Debug};
    use bytes::{BytesMut, BufMut};
    use tokio_util::codec::{Decoder, Encoder};
    use crate::{event::DiscoveryEvent, peer::{PeerMetadata, PeerId}, proto::{ConnectionCodec, Connection}};
    use super::{DiscoveryCodec, SIGNATURE};


    fn consume<D>(decoder: &mut D, src: &mut BytesMut) -> Vec<Option<D::Item>>
    where 
        D: Decoder,
        D::Error: Debug {
        let mut result = Vec::new();
        loop {
            match decoder.decode(src) {
                Ok(None) => { break; }
                output => result.push(output.unwrap())
            }
        }
        result
    }

    #[test]
    fn decode_discovery_presence_request() {
        let mut decoder = DiscoveryCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(6); // length
        src.put_u8(1);  // type
        src.put_u8(0);  // discovery type
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(DiscoveryEvent::PresenceRequest)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn decode_discovery_presence_response() {
        let mut decoder = DiscoveryCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(76); // length
        src.put_u8(1);  // type
        src.put_u8(1);  // discovery type 
        src.put_u16(6);  // device type
        src.put_u16(10);  // device name length
        src.put(&b"test phone"[..]); // device name
        src.put(&b"0123456789012345678901234567890123456789"[..]); // device id
        src.put_u16(14); // address length
        src.put(&b"127.0.0.1:5001"[..]); // address
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(DiscoveryEvent::PresenceResponse(meta))) = result.pop() else {
            panic!("invalid frame");
        };

        assert_eq!(PeerMetadata {
            name: "test phone".to_string(),
            typ: crate::peer::DeviceType::AppleiPhone,
            id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(),
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1),5001))
        }, meta);
    }

    #[test]
    fn encode_discovery_presence_request() {
        let mut encoder = DiscoveryCodec;
        let mut dst = BytesMut::new();

        let item = DiscoveryEvent::PresenceRequest;

        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(DiscoveryEvent::PresenceRequest)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn encode_discovery_presence_response() {
        let mut encoder = DiscoveryCodec;
        let mut dst = BytesMut::new();

        let item = DiscoveryEvent::PresenceResponse(PeerMetadata {
            name: "test phone".to_string(), 
            typ: crate::peer::DeviceType::AppleiPhone, 
            id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(), 
            addr:  SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1),5001))});

        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(DiscoveryEvent::PresenceResponse(meta))) = result.pop() else {
            panic!("invalid frame");
        };

        assert_eq!(PeerMetadata {
            name: "test phone".to_string(),
            typ: crate::peer::DeviceType::AppleiPhone,
            id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(),
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1),5001))
        }, meta);
    }

    #[test]
    fn decode_connect_request() {
        let mut decoder = ConnectionCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(73 + 5); // length
        src.put_u8(2);  // type
        src.put_u8(0);  // connect type 
        src.put(&b"0123456789012345678901234567890123456789"[..]); // peer id
        src.put(&b"0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT"[..]); // hmac
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Request { id, tag })) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!("0123456789012345678901234567890123456789", id.to_string());
        assert_eq!("0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT", String::from_utf8(tag).unwrap());
    }

    #[test]
    fn decode_connect_response() {
        let mut decoder = ConnectionCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(33 + 5); // length
        src.put_u8(2);  // type
        src.put_u8(1);  // connect type 
        src.put(&b"0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT"[..]); // hmac
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Response(tag))) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!("0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT", String::from_utf8(tag).unwrap());
    }

    #[test]
    fn decode_connect_complete_request() {
        let mut decoder = ConnectionCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(1 + 5); // length
        src.put_u8(2);  // type
        src.put_u8(2);  // connect type 
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::CompleteRequest)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn decode_connect_complete_response() {
        let mut decoder = ConnectionCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(1 + 5); // length
        src.put_u8(2);  // type
        src.put_u8(3);  // connect type 
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::CompleteResponse)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn decode_connect_failure() {
        let mut decoder = ConnectionCodec;
        let mut src = BytesMut::new();

        src.put(&SIGNATURE[..]);
        src.put_u16(5 + 5); // length
        src.put_u8(2);  // type
        src.put_u8(4);  // connect type 
        src.put_u32(2001); // result
        let mut result = consume(&mut decoder, &mut src);

        assert_eq!(0, src.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Failure(code))) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!(2001, code);
    }

    #[test]
    fn encode_connect_request() {
        let mut encoder = ConnectionCodec;
        let mut dst = BytesMut::new();

        let item = Connection::Request {
            id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(), 
            tag: Vec::from(&b"0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT"[..]) 
        };
        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Request { id, tag })) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!("0123456789012345678901234567890123456789", id.to_string());
        assert_eq!("0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT", String::from_utf8(tag).unwrap());
    }

    #[test]
    fn encode_connect_response() {
        let mut encoder = ConnectionCodec;
        let mut dst = BytesMut::new();

        let item = Connection::Response(Vec::from(&b"0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT"[..]));
        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Response(tag))) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!("0TQEnaM5YHPJ8LJ2KD32bTGdnfK23ScT", String::from_utf8(tag).unwrap());
    }

    #[test]
    fn encode_connect_completed_request() {
        let mut encoder = ConnectionCodec;
        let mut dst = BytesMut::new();

        let item = Connection::CompleteRequest;
        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::CompleteRequest)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn encode_connect_completed_response() {
        let mut encoder = ConnectionCodec;
        let mut dst = BytesMut::new();

        let item = Connection::CompleteResponse;
        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::CompleteResponse)) = result.pop() else {
            panic!("invalid frame");
        };
    }

    #[test]
    fn encode_connect_failure() {
        let mut encoder = ConnectionCodec;
        let mut dst = BytesMut::new();

        let item = Connection::Failure(2001);
        encoder.encode(item, &mut dst).expect("Error Encoding");
        // assert_eq!(dst, BytesMut::from(&hex!("")[..]))

        let mut result = consume(&mut encoder, &mut dst);
        assert_eq!(0, dst.len());
        assert_eq!(1, result.len());
        let Some(Some(Connection::Failure(code))) = result.pop() else {
            panic!("invalid frame");
        };
        assert_eq!(2001, code);
    }
}