extern crate dnsafe;

use std::net::UdpSocket;

use dnsafe::{recursive_lookup, PacketBuffer, BytePacketBuffer, DnsPacket, ResultCode};

fn main() {
    // Bind an UDP socket on port 2053
    let socket = UdpSocket::bind(("0.0.0.0", 2053)).unwrap();

    println!("Listening on 0.0.0.0:2053");

    // For now, queries are handled sequentially, so an infinite loop for servicing
    // requests is initiated.
    loop {
        // With a socket ready, we can go ahead and read a packet. This will
        // block until one is received.
        let mut req_buffer = BytePacketBuffer::new();
        let (_, src) = match socket.recv_from(&mut req_buffer.buf) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to read from UDP socket: {:?}", e);
                continue;
            }
        };

        // Here we use match to safely unwrap the `Result`. If everything's as expected,
        // the raw bytes are simply returned, and if not it'll abort by restarting the
        // loop and waiting for the next request. The `recv_from` function will write the
        // data into the provided buffer, and return the length of the data read as well
        // as the source adress. We're not interested in the length, but we need to keep
        // track of the source in order to send our reply later on.

        // Next, `DnsPacket::from_buffer` is used to parse the raw bytes into
        // a `DnsPacket`. It uses the same error handling idiom as the previous statement.

        let request = match DnsPacket::from_buffer(&mut req_buffer) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to parse UDP query packet: {:?}", e);
                continue;
            }
        };

        // Create and initialize the response packet
        let mut packet = DnsPacket::new();
        packet.header.id = request.header.id;
        packet.header.recursion_desired = true;
        packet.header.recursion_available = true;
        packet.header.response = true;

        // Being mindful of how unreliable input data from arbitrary senders can be, we
        // need make sure that a question is actually present. If not, we return `FORMERR`
        // to indicate that the sender made something wrong.
        if request.questions.is_empty() {
            packet.header.rescode = ResultCode::FORMERR;
        }
        // Usually a question will be present, though.
        else {
            let question = &request.questions[0];
            println!("Received query: {:?}", question);
            if let Ok(result) = recursive_lookup(&question.name, question.qtype) {
                packet.questions.push(question.clone());
                packet.header.rescode = result.header.rescode;
                packet.questions.push(question.clone());
                packet.header.rescode = result.header.rescode;

                for rec in result.answers {
                    println!("Answer: {:?}", rec);
                    packet.answers.push(rec);
                }
                for rec in result.authorities {
                    println!("Authority: {:?}", rec);
                    packet.authorities.push(rec);
                }
                for rec in result.resources {
                    println!("Resource: {:?}", rec);
                    packet.resources.push(rec);
                }
            } else {
                packet.header.rescode = ResultCode::SERVFAIL;
            }

            // The only thing remaining is to encode our response and send it off!

            let mut res_buffer = BytePacketBuffer::new();
            match packet.write(&mut res_buffer) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to encode UDP response packet: {:?}", e);
                    continue;
                }
            };

            let len = res_buffer.pos();
            let data = match res_buffer.get_range(0, len) {
                Ok(x) => x,
                Err(e) => {
                    println!("Failed to retrieve response buffer: {:?}", e);
                    continue;
                }
            };

            match socket.send_to(data, src) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to send response buffer: {:?}", e);
                    continue;
                }
            };
        }
    } // End of request loop
} // End of main
