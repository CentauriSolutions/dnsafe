use std::io::{Result};

use BytePacketBuffer;
use QueryType;
use DnsRecord;
use DnsHeader;
use DnsQuestion;

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};
    use super::*;

    #[test]
    fn it_parses_a_request() {
        let mut input = Cursor::new(include_bytes!("../tests/query_packet.txt").to_vec());
        let mut buffer = BytePacketBuffer::new();
        input.read(&mut buffer.buf).unwrap();
        let record = DnsPacket::from_buffer(&mut buffer).unwrap();
        assert_eq!(record.questions[0].name, "centauri.solutions");
    }

    #[test]
    fn it_parses_a_response() {
        let mut input = Cursor::new(include_bytes!("../tests/response_packet.txt").to_vec());
        let mut buffer = BytePacketBuffer::new();
        input.read(&mut buffer.buf).unwrap();
        let record = DnsPacket::from_buffer(&mut buffer).unwrap();
        assert_eq!(record.questions[0].name, "centauri.solutions");
        assert_eq!(record.answers[0], DnsRecord::A{domain: "centauri.solutions".into(), addr: "104.27.149.54".parse().unwrap(), ttl: 274});
    }
}

#[derive(Clone, Debug)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>
}

impl DnsPacket {
    pub fn new() -> DnsPacket {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new()
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<DnsPacket> {
        let mut result = DnsPacket::new();
        try!(result.header.read(buffer));

        for _ in 0..result.header.questions {
            let mut question = DnsQuestion::new("".to_string(),
                                                QueryType::UNKNOWN(0));
            try!(question.read(buffer));
            result.questions.push(question);
        }

        for _ in 0..result.header.answers {
            let rec = try!(DnsRecord::read(buffer));
            result.answers.push(rec);
        }
        for _ in 0..result.header.authoritative_entries {
            let rec = try!(DnsRecord::read(buffer));
            result.authorities.push(rec);
        }
        for _ in 0..result.header.resource_entries {
            let rec = try!(DnsRecord::read(buffer));
            result.resources.push(rec);
        }

        Ok(result)
    }
}