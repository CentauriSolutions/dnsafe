use std::net::UdpSocket;
use std::io::Result;

mod byte_packet_buffer;
mod dns_header;
mod result_code;
mod query_type;
mod dns_question;
mod dns_record;
mod dns_packet;

pub use byte_packet_buffer::BytePacketBuffer;
pub use dns_header::DnsHeader;
pub use result_code::ResultCode;
pub use query_type::QueryType;
pub use dns_record::DnsRecord;
pub use dns_packet::DnsPacket;
pub use dns_question::DnsQuestion;

pub fn lookup(qname: &str, qtype: QueryType, server: (&str, u16)) -> Result<DnsPacket> {
    let socket = try!(UdpSocket::bind(("0.0.0.0", 43210)));

    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet.questions.push(DnsQuestion::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer).unwrap();
    try!(socket.send_to(&req_buffer.buf[0..req_buffer.pos], server));

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf).unwrap();

    DnsPacket::from_buffer(&mut res_buffer)
}