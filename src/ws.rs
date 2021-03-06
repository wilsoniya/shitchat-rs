use std::io::{Read, Write};
use std::io::Result;
use bufstream::BufStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

//    0                   1                   2                   3
//    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//   +-+-+-+-+-------+-+-------------+-------------------------------+
//   |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
//   |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
//   |N|V|V|V|       |S|             |   (if payload len==126/127)   |
//   | |1|2|3|       |K|             |                               |
//   +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
//   |     Extended payload length continued, if payload len == 127  |
//   + - - - - - - - - - - - - - - - +-------------------------------+
//   |                               |Masking-key, if MASK set to 1  |
//   +-------------------------------+-------------------------------+
//   | Masking-key (continued)       |          Payload Data         |
//   +-------------------------------- - - - - - - - - - - - - - - - +
//   :                     Payload Data continued ...                :
//   + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
//   |                     Payload Data continued ...                |
//   +---------------------------------------------------------------+
//
//   source: https://tools.ietf.org/html/rfc6455#section-5.2

static FIN_MASK:         u16 = 0b1000000000000000;
static RSV_MASK:         u16 = 0b0111000000000000;
static OPCODE_MASK:      u16 = 0b0000111100000000;
static MASK_MASK:        u16 = 0b0000000010000000;
static PAYLOAD_LEN_MASK: u16 = 0b0000000001111111;

pub fn read_stream<T: Read + Write>(stream: &mut BufStream<T>) -> Result<Vec<u8>> {
    let mut mask_key = [0u8; 4];
    let header = try!(stream.read_u16::<BigEndian>());

    let fin: bool = ((header & FIN_MASK) >> 15) == 1;
    let rsv: u8 = ((header & RSV_MASK) >> 12) as u8;
    let opcode: u8 = ((header & OPCODE_MASK) >> 8) as u8;
    let mask: bool = ((header & MASK_MASK) >> 7) == 1;

    let payload_len: u64 = match header & PAYLOAD_LEN_MASK {
        126 => try!(stream.read_u16::<BigEndian>()) as u64,
        127 => try!(stream.read_u64::<BigEndian>()),
        x => x as u64,
    };

    if mask {
        let _ = try!(stream.read(&mut mask_key));
    }

    let mut data: Vec<u8> = Vec::new();
    stream.take(payload_len).read_to_end(&mut data);

//  let mut data: Vec<u8> = try!(stream.read_exact(payload_len as usize));

    if mask {
        for i in 0usize..(payload_len as usize) {
            data[i] = data[i] ^ mask_key[i % 4];
        }
    }

    Ok(data)
}

pub fn write_stream<T: Read + Write>(stream: &mut BufStream<T>, data: &Vec<u8>) {
    let mut header: u16 = 0b0;
    let fin = 0b1 << 15;        // FIN frame
    let opcode = 0b0001 << 8;   // text mode
    let mask = 0b0 << 7;        // no mask

    let payload_len = match data.len() {
        l if l <= 125 => l,
        l if l > (2 as usize).pow(16) => 127,
        _ => 126,
    } as u16;

    header = header | fin | opcode | mask | payload_len;

//  let _ = stream.write_be_u16(header);
    let _ = stream.write_u16::<BigEndian>(header);

    if data.len() > 125 && data.len() > (2 as usize).pow(16) {
        // case: 64-bit data length
        let data_len: u64 = data.len() as u64;
//      let _ = stream.write_be_u64(data_len);
        let _ = stream.write_u64::<BigEndian>(data_len);
    } else if data.len() > 125 {
        // case: 16-bit data length
        let data_len: u16 = data.len() as u16;
//      let _ = stream.write_be_u16(data_len);
        let _ = stream.write_u16::<BigEndian>(data_len);
    };

    let _ = stream.write_all(&data[..]);
    let _ = stream.flush();
}
