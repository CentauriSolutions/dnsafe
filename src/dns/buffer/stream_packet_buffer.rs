use std::io::{Read, Result};

use super::PacketBuffer;

pub struct StreamPacketBuffer<'a, T>
where
    T: Read + 'a,
{
    pub stream: &'a mut T,
    pub buffer: Vec<u8>,
    pub pos: usize,
}

impl<'a, T> StreamPacketBuffer<'a, T>
where
    T: Read + 'a,
{
    pub fn new(stream: &'a mut T) -> StreamPacketBuffer<T> {
        StreamPacketBuffer {
            stream,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl<'a, T> PacketBuffer for StreamPacketBuffer<'a, T>
where
    T: Read + 'a,
{
    fn find_label(&self, _: &str) -> Option<usize> {
        None
    }

    fn save_label(&mut self, _: &str, _: usize) {
        unimplemented!();
    }

    fn read(&mut self) -> Result<u8> {
        while self.pos >= self.buffer.len() {
            let mut local_buffer = [0; 1];
            try!(self.stream.read_exact(&mut local_buffer));
            self.buffer.push(local_buffer[0]);
        }

        let res = self.buffer[self.pos];
        self.pos += 1;

        Ok(res)
    }

    fn get(&mut self, pos: usize) -> Result<u8> {
        while pos >= self.buffer.len() {
            let mut local_buffer = [0; 1];
            try!(self.stream.read_exact(&mut local_buffer));
            self.buffer.push(local_buffer[0]);
        }

        Ok(self.buffer[pos])
    }

    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        while start + len > self.buffer.len() {
            let mut local_buffer = [0; 1];
            try!(self.stream.read_exact(&mut local_buffer));
            self.buffer.push(local_buffer[0]);
        }

        Ok(&self.buffer[start..start + len as usize])
    }

    fn write(&mut self, _: u8) -> Result<()> {
        unimplemented!();
    }

    fn set(&mut self, _: usize, _: u8) -> Result<()> {
        unimplemented!();
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;
        Ok(())
    }

    fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;
        Ok(())
    }
}
