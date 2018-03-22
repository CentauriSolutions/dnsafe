use std::io::Result;

use PacketBuffer;
use BytePacketBuffer;
use QueryType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: QueryType,
}

impl DnsQuestion {
    pub fn new(name: String, qtype: QueryType) -> DnsQuestion {
        DnsQuestion {
            name: name,
            qtype: qtype,
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        try!(buffer.read_qname(&mut self.name));
        self.qtype = QueryType::from_num(try!(buffer.read_u16())); // qtype
        let _ = try!(buffer.read_u16()); // class

        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        try!(buffer.write_qname(&self.name));

        let typenum = self.qtype.to_num();
        try!(buffer.write_u16(typenum));
        try!(buffer.write_u16(1));

        Ok(())
    }
}
