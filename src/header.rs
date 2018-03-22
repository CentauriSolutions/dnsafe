use nom::{IResult, be_u8, be_u16, le_u8, le_u16};

use ResultCode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_query_header() {
        let packet = include_bytes!("../tests/query_packet.txt");
        let header = DnsHeader::read(&packet[..]).unwrap().1;
        println!("Header: {:?}", header);
    }
}

#[derive(Clone, Debug)]
pub struct DnsHeader {
    pub id: u16, // 16 bits

    pub recursion_desired: bool,    // 1 bit
    pub truncated_message: bool,    // 1 bit
    pub authoritative_answer: bool, // 1 bit
    pub opcode: u8,                 // 4 bits
    pub response: bool,             // 1 bit

    pub rescode: ResultCode,       // 4 bits
    pub checking_disabled: bool,   // 1 bit
    pub authed_data: bool,         // 1 bit
    pub z: bool,                   // 1 bit
    pub recursion_available: bool, // 1 bit

    pub questions: u16,             // 16 bits
    pub answers: u16,               // 16 bits
    pub authoritative_entries: u16, // 16 bits
    pub resource_entries: u16,      // 16 bits
}

impl DnsHeader {
    pub fn new() -> DnsHeader {
        DnsHeader {
            id: 0,

            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,

            rescode: ResultCode::NOERROR,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,

            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }

    pub fn read(input: &[u8]) -> IResult<&[u8], DnsHeader> {
        dns_header(input)
    }
}

named!(dns_header <DnsHeader>,
    do_parse!(
        id: le_u16 >>
        recursion_desired: take_bool >>
        truncated_message: take_bool >>
        authoritative_answer: take_bool >>
        opcode: le_u8 >>
        response: take_bool >>
        rescode: rescode >>
        checking_disabled: take_bool >>
        authed_data: take_bool >>
        z: take_bool >>
        recursion_available: take_bool >>
        questions: le_u16 >>
        answers: be_u16 >>
        authoritative_entries:be_u16 >>
        resource_entries: be_u16 >> (
            DnsHeader {
                id,
                recursion_desired,
                truncated_message,
                authoritative_answer,
                opcode,
                response,
                rescode,
                checking_disabled,
                authed_data,
                z,
                recursion_available,
                questions,
                answers,
                authoritative_entries,
                resource_entries
            }
        )
));

named!( rescode<ResultCode>, do_parse!(
    num: be_u8 >> (
        ResultCode::from_num(num)
    )
));

named!( take_bool<bool>, do_parse!(
    b: take_bit >> (
        b == 1
    )
));

named!( take_bit<u8>, bits!( take_bits!( u8, 1 ) ) );
