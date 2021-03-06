//! buffers for use when writing and reading dns packets

use std::io::Result;

mod byte_packet_buffer;
mod stream_packet_buffer;
mod vector_packet_buffer;

pub use self::byte_packet_buffer::BytePacketBuffer;
pub use self::stream_packet_buffer::StreamPacketBuffer;
pub use self::vector_packet_buffer::VectorPacketBuffer;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_qname() {
        let mut buffer = VectorPacketBuffer::new();

        let instr1 = "a.google.com".to_string();
        let instr2 = "b.google.com".to_string();

        // First write the standard string
        match buffer.write_qname(&instr1) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        // Then we set up a slight variation with relies on a jump back to the data of
        // the first name
        let crafted_data = [0x01, b'b' as u8, 0xC0, 0x02];
        for b in &crafted_data {
            match buffer.write_u8(*b) {
                Ok(_) => {}
                Err(_) => panic!(),
            }
        }

        // Reset the buffer position for reading
        buffer.pos = 0;

        // Read the standard name
        let mut outstr1 = String::new();
        match buffer.read_qname(&mut outstr1) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        assert_eq!(instr1, outstr1);

        // Read the name with a jump
        let mut outstr2 = String::new();
        match buffer.read_qname(&mut outstr2) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        assert_eq!(instr2, outstr2);

        // Make sure we're now at the end of the buffer
        assert_eq!(buffer.pos, buffer.buffer.len());
    }

    #[test]
    fn test_write_qname() {
        let mut buffer = VectorPacketBuffer::new();

        match buffer.write_qname(&"ns1.google.com".to_string()) {
            Ok(_) => {}
            Err(_) => panic!(),
        }
        match buffer.write_qname(&"ns2.google.com".to_string()) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        assert_eq!(22, buffer.pos());

        match buffer.seek(0) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        let mut str1 = String::new();
        match buffer.read_qname(&mut str1) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        assert_eq!("ns1.google.com", str1);

        let mut str2 = String::new();
        match buffer.read_qname(&mut str2) {
            Ok(_) => {}
            Err(_) => panic!(),
        }

        assert_eq!("ns2.google.com", str2);
    }
}

pub trait PacketBuffer {
    fn read(&mut self) -> Result<u8>;
    fn get(&mut self, pos: usize) -> Result<u8>;
    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]>;
    fn write(&mut self, val: u8) -> Result<()>;
    fn set(&mut self, pos: usize, val: u8) -> Result<()>;
    fn pos(&self) -> usize;
    fn seek(&mut self, pos: usize) -> Result<()>;
    fn step(&mut self, steps: usize) -> Result<()>;
    fn find_label(&self, label: &str) -> Option<usize>;
    fn save_label(&mut self, label: &str, pos: usize);

    fn write_u8(&mut self, val: u8) -> Result<()> {
        try!(self.write(val));

        Ok(())
    }

    fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        try!(self.set(pos, (val >> 8) as u8));
        try!(self.set(pos + 1, (val & 0xFF) as u8));

        Ok(())
    }

    fn write_u16(&mut self, val: u16) -> Result<()> {
        try!(self.write((val >> 8) as u8));
        try!(self.write((val & 0xFF) as u8));

        Ok(())
    }

    fn write_u32(&mut self, val: u32) -> Result<()> {
        try!(self.write(((val >> 24) & 0xFF) as u8));
        try!(self.write(((val >> 16) & 0xFF) as u8));
        try!(self.write(((val >> 8) & 0xFF) as u8));
        try!(self.write((val & 0xFF) as u8));

        Ok(())
    }

    fn write_qname(&mut self, qname: &str) -> Result<()> {
        let split_str = qname.split('.').collect::<Vec<&str>>();

        let mut jump_performed = false;
        for (i, label) in split_str.iter().enumerate() {
            let search_lbl = split_str[i..split_str.len()].join(".");
            if let Some(prev_pos) = self.find_label(&search_lbl) {
                let jump_inst = (prev_pos as u16) | 0xC000;
                try!(self.write_u16(jump_inst));
                jump_performed = true;

                break;
            }

            let pos = self.pos();
            self.save_label(&search_lbl, pos);

            let len = label.len();
            try!(self.write_u8(len as u8));
            for b in label.as_bytes() {
                try!(self.write_u8(*b));
            }
        }

        if !jump_performed {
            try!(self.write_u8(0));
        }

        Ok(())
    }

    fn read_u16(&mut self) -> Result<u16> {
        let res = (u16::from(try!(self.read())) << 8) | (u16::from(try!(self.read())));

        Ok(res)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let res = (u32::from(try!(self.read())) << 24) | (u32::from(try!(self.read())) << 16)
            | (u32::from(try!(self.read())) << 8)
            | (u32::from(try!(self.read())));

        Ok(res)
    }

    fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
        let mut pos = self.pos();
        let mut jumped = false;

        let mut delim = "";
        loop {
            let len = try!(self.get(pos));

            // A two byte sequence, where the two highest bits of the first byte is
            // set, represents a offset relative to the start of the buffer. We
            // handle this by jumping to the offset, setting a flag to indicate
            // that we shouldn't update the shared buffer position once done.
            if (len & 0xC0) > 0 {
                // When a jump is performed, we only modify the shared buffer
                // position once, and avoid making the change later on.
                if !jumped {
                    try!(self.seek(pos + 2));
                }

                let b2 = u16::from(try!(self.get(pos + 1)));
                let offset = ((u16::from(len) ^ 0xC0) << 8) | b2;
                pos = offset as usize;
                jumped = true;
                continue;
            }

            pos += 1;

            // Names are terminated by an empty label of length 0
            if len == 0 {
                break;
            }

            outstr.push_str(delim);

            let str_buffer = try!(self.get_range(pos, len as usize));
            outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

            delim = ".";

            pos += len as usize;
        }

        if !jumped {
            try!(self.seek(pos));
        }

        Ok(())
    }
}
