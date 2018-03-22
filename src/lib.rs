extern crate rand;

use std::net::UdpSocket;
use std::io::Result;

mod dns;

pub use dns::{BytePacketBuffer, DnsHeader, DnsPacket, DnsQuestion, DnsRecord, PacketBuffer,
              QueryType, ResultCode};

pub fn lookup(qname: &str, qtype: QueryType, server: (&str, u16)) -> Result<DnsPacket> {
    let socket = try!(UdpSocket::bind(("0.0.0.0", 43210)));

    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(DnsQuestion::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer).unwrap();
    try!(socket.send_to(&req_buffer.buf[0..req_buffer.pos], server));

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf).unwrap();

    DnsPacket::from_buffer(&mut res_buffer)
}

pub fn recursive_lookup(qname: &str, qtype: QueryType) -> Result<DnsPacket> {
    // For now we're always starting with *a.root-servers.net*.
    let mut ns = "198.41.0.4".to_string();

    // Since it might take an arbitrary number of steps, we enter an unbounded loop.
    loop {
        println!("attempting lookup of {:?} {} with ns {}", qtype, qname, ns);

        // The next step is to send the query to the active server.
        let ns_copy = ns.clone();

        let server = (ns_copy.as_str(), 53);
        let response = try!(lookup(qname, qtype.clone(), server));

        // If there are entries in the answer section, and no errors, we are done!
        if !response.answers.is_empty() && response.header.rescode == ResultCode::NOERROR {
            return Ok(response.clone());
        }

        // We might also get a `NXDOMAIN` reply, which is the authoritative name servers
        // way of telling us that the name doesn't exist.
        if response.header.rescode == ResultCode::NXDOMAIN {
            return Ok(response.clone());
        }

        // Otherwise, we'll try to find a new nameserver based on NS and a corresponding A
        // record in the additional section. If this succeeds, we can switch name server
        // and retry the loop.
        if let Some(new_ns) = response.get_resolved_ns(qname) {
            ns = new_ns.clone();

            continue;
        }

        // If not, we'll have to resolve the ip of a NS record. If no NS records exist,
        // we'll go with what the last server told us.
        let new_ns_name = match response.get_unresolved_ns(qname) {
            Some(x) => x,
            None => return Ok(response.clone()),
        };

        // Here we go down the rabbit hole by starting _another_ lookup sequence in the
        // midst of our current one. Hopefully, this will give us the IP of an appropriate
        // name server.
        let recursive_response = try!(recursive_lookup(&new_ns_name, QueryType::A));

        // Finally, we pick a random ip from the result, and restart the loop. If no such
        // record is available, we again return the last result we got.
        if let Some(new_ns) = recursive_response.get_random_a() {
            ns = new_ns.clone();
        } else {
            return Ok(response.clone());
        }
    }
} // End of recursive_lookup
