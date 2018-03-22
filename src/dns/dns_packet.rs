use std::io::Result;

use rand::random;

use PacketBuffer;
use BytePacketBuffer;
use QueryType;
use DnsRecord;
use DnsHeader;
use DnsQuestion;

#[derive(Clone, Debug)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl DnsPacket {
    pub fn new() -> DnsPacket {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<DnsPacket> {
        let mut result = DnsPacket::new();
        try!(result.header.read(buffer));

        for _ in 0..result.header.questions {
            let mut question = DnsQuestion::new("".to_string(), QueryType::UNKNOWN(0));
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

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        try!(self.header.write(buffer));

        for question in &self.questions {
            try!(question.write(buffer));
        }
        for rec in &self.answers {
            try!(rec.write(buffer));
        }
        for rec in &self.authorities {
            try!(rec.write(buffer));
        }
        for rec in &self.resources {
            try!(rec.write(buffer));
        }

        Ok(())
    }

    // It's useful to be able to pick a random A record from a packet. When we
    // get multiple IP's for a single name, it doesn't matter which one we
    // choose, so in those cases we can now pick one at random.
    pub fn get_random_a(&self) -> Option<String> {
        if !self.answers.is_empty() {
            let idx = random::<usize>() % self.answers.len();
            let a_record = &self.answers[idx];
            if let DnsRecord::A { ref addr, .. } = *a_record {
                return Some(addr.to_string());
            }
        }

        None
    }

    // We'll use the fact that name servers often bundle the corresponding
    // A records when replying to an NS query to implement a function that returns
    // the actual IP for an NS record if possible.
    pub fn get_resolved_ns(&self, qname: &str) -> Option<String> {
        // First, we scan the list of NS records in the authorities section:
        let mut new_authorities = Vec::new();
        for auth in &self.authorities {
            if let DnsRecord::NS {
                ref domain,
                ref host,
                ..
            } = *auth
            {
                if !qname.ends_with(domain) {
                    continue;
                }

                // Once we've found an NS record, we scan the resources record for a matching
                // A record...
                for rsrc in &self.resources {
                    if let DnsRecord::A {
                        ref domain,
                        ref addr,
                        ttl,
                    } = *rsrc
                    {
                        if domain != host {
                            continue;
                        }

                        let rec = DnsRecord::A {
                            domain: host.clone(),
                            addr: *addr,
                            ttl: ttl,
                        };

                        // ...and push any matches to a list.
                        new_authorities.push(rec);
                    }
                }
            }
        }

        // If there are any matches, we pick the first one.
        if !new_authorities.is_empty() {
            if let DnsRecord::A { addr, .. } = new_authorities[0] {
                return Some(addr.to_string());
            }
        }

        None
    } // End of get_resolved_ns

    // However, not all name servers are as that nice. In certain cases there won't
    // be any A records in the additional section, and we'll have to perform *another*
    // lookup in the midst. For this, we introduce a method for returning the host
    // name of an appropriate name server.
    pub fn get_unresolved_ns(&self, qname: &str) -> Option<String> {
        let mut new_authorities = Vec::new();
        for auth in &self.authorities {
            if let DnsRecord::NS {
                ref domain,
                ref host,
                ..
            } = *auth
            {
                if !qname.ends_with(domain) {
                    continue;
                }

                new_authorities.push(host);
            }
        }

        if !new_authorities.is_empty() {
            let idx = random::<usize>() % new_authorities.len();
            return Some(new_authorities[idx].clone());
        }

        None
    } // End of get_unresolved_ns
}
