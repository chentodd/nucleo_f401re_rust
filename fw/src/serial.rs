use core::u32;
use defmt::debug;
use heapless::Vec;
use micropb::{MessageDecode, PbDecoder};

use utils::*;

#[derive(Default)]
pub struct PacketDecoder {
    len: u32,
    message_id: MessageId,
}

impl PacketDecoder {
    // The packet is formatted as follows, unit: u8
    // [MESSAGE_ID:1] [LEN:4] [PROTO_PACKET:X] [CRC:1]
    //
    // MESSAGE_ID:
    // should be the same with enum `MessageId`
    //
    // LEN:
    // the length of current packet which is:
    // - size_of(MESSAGE_ID), 1 bytes +
    // - size_of(LEN), 4 bytes +
    // - size_of(PROTO_PACKET), X bytes +
    // - size_of(CRC), 1 bytes
    // = X + 6 bytes
    //
    // PROTO_PACKET:
    // the serialized bytes sent by sender
    //
    // CRC:
    // the check code of the packect
    pub fn new() -> Self {
        Self {
            len: u32::MAX,
            message_id: MessageId::NoId,
        }
    }

    pub fn is_packet_valid(&mut self, stream: &[u8]) -> bool {
        if stream.len() >= size_of::<MessageId>() + LENGTH_TYPE_IN_BYTES {
            // Get [MESSAGE_ID:1]
            if let Some(first_ch) = stream.first() {
                match first_ch {
                    0x00 => self.message_id = MessageId::NoId,
                    0x10 => self.message_id = MessageId::CommandRx,
                    _ => (),
                }
            }

            // Get [LEN:4]
            #[cfg(feature = "debug-rx")]
            debug!("test: {:?}", &stream[1..=4]);

            self.len = u32::from_le_bytes(stream[1..=4].try_into().unwrap());
            if stream.len() >= (self.len as usize) {
                let n = self.len as usize;
                let actual_crc = stream[n - 1];
                let expected_crc = calculate_crc(&stream[0..=n - 2]);
                return actual_crc == expected_crc;
            }
        }

        false
    }

    pub fn parse_proto_message(&mut self, stream: &[u8], packet: &mut impl MessageDecode) -> bool {
        let n = self.len as usize;
        let header_len = size_of::<MessageId>() + LENGTH_TYPE_IN_BYTES;
        let stream = &stream[header_len..=n - 2];

        let mut decoder = PbDecoder::new(stream);
        match packet.decode(&mut decoder, stream.len()) {
            Ok(_) => {
                return true;
            }
            Err(_e) => {
                #[cfg(feature = "debug-rx")]
                debug!("proto packet debug error");
            }
        }

        false
    }
}

// TODO, this function is nearly identical to the one in `serial_tool`, maybe we can
// move all the code in this file to `serial_tool` since it doesn't need hardware
// support
pub fn encode_packet(message_id: MessageId, proto_message: &[u8]) -> Vec<u8, 128> {
    let mut packet = Vec::<u8, 128>::new();

    let _ = packet.push(message_id as u8);
    let _ = packet.extend_from_slice(&[0; LENGTH_TYPE_IN_BYTES]);
    let _ = packet.extend_from_slice(proto_message);
    let _ = packet.push(0);

    let length = packet.len() as u32;
    packet[1..=LENGTH_TYPE_IN_BYTES].copy_from_slice(&length.to_le_bytes());

    let length = length as usize;
    packet[length - 1] = calculate_crc(&packet[0..=length - 2]);

    packet
}
