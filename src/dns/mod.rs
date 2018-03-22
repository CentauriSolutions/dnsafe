mod dns_header;
mod result_code;
mod query_type;
mod dns_question;
mod dns_record;
mod dns_packet;
mod buffer;

// pub use self::byte_packet_buffer::BytePacketBuffer;
pub use self::buffer::{BytePacketBuffer, PacketBuffer};
pub use self::dns_header::DnsHeader;
pub use self::result_code::ResultCode;
pub use self::query_type::QueryType;
pub use self::dns_record::DnsRecord;
pub use self::dns_packet::DnsPacket;
pub use self::dns_question::DnsQuestion;

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};
    use super::*;

    #[test]
    fn it_parses_a_request() {
        let mut input = Cursor::new(include_bytes!("../../tests/query_packet.txt").to_vec());
        let mut buffer = BytePacketBuffer::new();
        input.read(&mut buffer.buf).unwrap();
        let record = DnsPacket::from_buffer(&mut buffer).unwrap();
        assert_eq!(record.questions[0].name, "centauri.solutions");
    }

    #[test]
    fn it_parses_a_response() {
        let mut input = Cursor::new(include_bytes!("../../tests/response_packet.txt").to_vec());
        let mut buffer = BytePacketBuffer::new();
        input.read(&mut buffer.buf).unwrap();
        let record = DnsPacket::from_buffer(&mut buffer).unwrap();
        assert_eq!(record.questions[0].name, "centauri.solutions");
        assert_eq!(
            record.answers[0],
            DnsRecord::A {
                domain: "centauri.solutions".into(),
                addr: "104.27.149.54".parse().unwrap(),
                ttl: 274,
            }
        );
    }
}
