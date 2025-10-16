use prost::Message;
use std::error::Error;

pub fn to_binary<T: Message>(msg: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("Failed to encode protobuf");
    buf
}

pub fn from_binary<T: Message + Default>(bin: &[u8]) -> Result<T, Box<dyn Error>> {
    T::decode(bin).map_err(|e| e.into())
}
