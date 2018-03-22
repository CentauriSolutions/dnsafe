use std::io::Result;
use std::io::{Error, ErrorKind};

pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize
}

impl BytePacketBuffer {

    // This gives us a fresh buffer for holding the packet contents, and a field for
    // keeping track of where we are.
    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0
        }
    }

    // When handling the reading of domain names, we'll need a way of
    // reading and manipulating our buffer position.

    fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    // A method for reading a single byte, and moving one step forward
    fn read(&mut self) -> Result<u8> {
        if self.pos >= 512 {
            return Err(Error::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    // Methods for fetching data at a specified position, without modifying
    // the internal position

    fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= 512 {
            return Err(Error::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        Ok(self.buf[pos])
    }

    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= 512 {
            return Err(Error::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        Ok(&self.buf[start..start+len as usize])
    }

    // Methods for reading a u16 and u32 from the buffer, while stepping
    // forward 2 or 4 bytes

    pub fn read_u16(&mut self) -> Result<u16>
    {
        let res = ((try!(self.read()) as u16) << 8) |
                  (try!(self.read()) as u16);

        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32>
    {
        let res = ((try!(self.read()) as u32) << 24) |
                  ((try!(self.read()) as u32) << 16) |
                  ((try!(self.read()) as u32) << 8) |
                  ((try!(self.read()) as u32) << 0);

        Ok(res)
    }

    // The tricky part: Reading domain names, taking labels into consideration.
    // Will take something like [3]www[6]google[3]com[0] and append
    // www.google.com to outstr.
    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()>
    {
        // Since we might encounter jumps, we'll keep track of our position
        // locally as opposed to using the position within the struct. This
        // allows us to move the shared position to a point past our current
        // qname, while keeping track of our progress on the current qname
        // using this variable.
        let mut pos = self.pos();

        // track whether or not we've jumped
        let mut jumped = false;

        // Our delimeter which we append for each label. Since we don't want a dot at the
        // beginning of the domain name we'll leave it empty for now and set it to "." at
        // the end of the first iteration.
        let mut delim = "";
        loop {
            // At this point, we're always at the beginning of a label. Recall
            // that labels start with a length byte.
            let len = try!(self.get(pos));

            // If len has the two most significant bit are set, it represents a jump to
            // some other offset in the packet:
            if (len & 0xC0) == 0xC0 {
                // Update the buffer position to a point past the current
                // label. We don't need to touch it any further.
                if !jumped {
                    try!(self.seek(pos+2));
                }

                // Read another byte, calculate offset and perform the jump by
                // updating our local position variable
                let b2 = try!(self.get(pos+1)) as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                // Indicate that a jump was performed.
                jumped = true;
            }

            // The base scenario, where we're reading a single label and
            // appending it to the output:
            else {
                // Move a single byte forward to move past the length byte.
                pos += 1;

                // Domain names are terminated by an empty label of length 0, so if the length is zero
                // we're done.
                if len == 0 {
                    break;
                }

                // Append the delimiter to our output buffer first.
                outstr.push_str(delim);

                // Extract the actual ASCII bytes for this label and append them to the output buffer.

                let str_buffer = try!(self.get_range(pos, len as usize));
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

                delim = ".";

                // Move forward the full length of the label.
                pos += len as usize;
            }
        }

        // If a jump has been performed, we've already modified the buffer position state and
        // shouldn't do so again.
        if !jumped {
            try!(self.seek(pos));
        }

        Ok(())
    } // End of read_qname

} // End of BytePacketBuffer