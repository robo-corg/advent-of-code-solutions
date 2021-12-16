use std::{
    fmt,
    io::{self, BufRead},
};

struct BitSet {
    offset: i64,
    size: usize,
    data: Vec<u8>,
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bit in self.iter() {
            if bit {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }

        Ok(())
    }
}

impl BitSet {
    fn iter<'a>(&'a self) -> impl Iterator<Item = bool> + 'a {
        self.data
            .iter()
            .copied()
            .flat_map(|b| (0..8).map(move |bit| b & (1 << bit) != 0))
            .take(self.size)
    }

    /// Push up to 8 bits to stream
    fn push_bits(&mut self, bits: u8, count: u8) {
        if count == 0 {
            return;
        }

        let used_bits = (self.size % 8) as i32;

        let mask = 0xFF >> (8 - count);

        let masked_value = bits & mask;

        if used_bits == 0 {
            self.data.push(masked_value);
            self.size += count as usize;
        } else {
            // if we have a partially filled byte grab the last one
            // and append bits to it
            *self.data.last_mut().unwrap() |= masked_value << used_bits;

            let new_used_bits = i32::min(used_bits + count as i32, 8);

            let bits_written = new_used_bits - used_bits;
            self.size += bits_written as usize;

            let remaining_bits = i32::max(count as i32 - bits_written, 0);

            if remaining_bits > 0 {
                self.push_bits(masked_value >> bits_written, remaining_bits as u8);
            }
        }
    }

    pub fn new() -> Self {
        BitSet {
            offset: 0,
            size: 0,
            data: Vec::new(),
        }
    }

    pub fn get(&self, mut pos: usize, amount: u32) -> u32 {
        pos += self.offset as usize;

        let mut written: usize = 0;
        let mut output: u32 = 0;
        let mut shift = pos % 8;

        while written < amount as usize {
            let idx = pos / 8;
            let read = 8 - shift as usize;
            let fetched = (self.data[idx] >> shift) as u32;

            output |= fetched << written as u32;
            written += read;

            pos += written;
            // After the read the rest will be aligned to the start
            // of the byte being examined
            shift = 0;
        }

        let mask: u32 = (1 << amount) - 1;

        // Packet decoded numbers have opposite bit order
        output = output.reverse_bits() >> (u32::BITS - amount);
        // We may have fetched extra bits so mask them off
        output as u32 & mask
    }

    /// Split off a bitset that only extends to amount
    fn read_bitset(&mut self, amount: u32) -> BitSet {
        let ret = BitSet {
            offset: self.offset,
            size: usize::min(self.offset as usize + amount as usize, self.size),
            data: self.data.clone()
        };

        self.offset += amount as i64;

        ret
    }

    /// Read amount bits as a number and advance offset by amount
    fn read(&mut self, amount: u32) -> u32 {
        let ret = self.get(0, amount);
        self.offset += amount as i64;
        ret
    }

    fn len(&self) -> usize {
        self.size - self.offset as usize
    }
}

type Input = BitSet;

fn parse_str(s: &str) -> BitSet {
    let mut bitset = BitSet::new();

    for chunk in s
        .trim()
        .chars()
        .map(|ch| u8::from_str_radix(&ch.to_string(), 16).unwrap())
    {
        // hex bit patterns order is opposite how they should be projected
        // onto the bitstream so we have to reverse them
        bitset.push_bits(chunk.reverse_bits() >> 4, 4);
    }

    bitset
}

fn parse_input(mut reader: impl BufRead) -> Input {
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();

    parse_str(buf.as_str())
}

#[derive(Debug)]
struct Operator {
    ty: u32,
    version: u32,
    packets: Vec<Packet>
}

#[derive(Debug)]
enum Packet {
    Literal(u32, Vec<u8>),
    Operator(Operator)
}

impl Packet {
    fn version_sum(&self) -> u32 {
        match self {
            Packet::Literal(v, _) => *v,
            Packet::Operator(Operator { version, packets, ..} ) => {
                let total: u32 = packets.iter().map(Packet::version_sum).sum();
                *version + total
            }
        }
    }

    fn eval(&self) -> u64 {
        match self {
            Packet::Literal(_v, nums) => {
                let mut output: u64 = 0;

                dbg!(nums.len()*4);

                for (n, num) in nums.iter().rev().enumerate() {
                    let shift: u64 = 4 * n as u64;
                    output |= (*num as u64) << shift;
                }

                dbg!(output);

                output
            },
            Packet::Operator(op) => {
                let mut subpackets = op.packets.iter().map(Packet::eval);
                match op.ty {
                    // sum
                    0 => subpackets.sum(),
                    1 => subpackets.product(),
                    2 => subpackets.min().unwrap(),
                    3 => subpackets.max().unwrap(),
                    // 4 is reserverd for literals
                    5 => {
                        let lhs = subpackets.next().unwrap();
                        let rhs = subpackets.next().unwrap();

                        if lhs > rhs { 1 } else { 0 }
                    },
                    6 => {
                        let lhs = subpackets.next().unwrap();
                        let rhs = subpackets.next().unwrap();

                        if lhs < rhs { 1 } else { 0 }
                    }
                    7 => {
                        let lhs = subpackets.next().unwrap();
                        let rhs = subpackets.next().unwrap();

                        if lhs == rhs { 1 } else { 0 }
                    }
                    other => panic!("Invalid operator type: {}", other)
                }
            }
        }
    }
}

fn parse_packet(msg: &mut BitSet) -> Packet {
    dbg!(msg.len());
    let version = msg.read(3);
    let ty = msg.read(3);

    dbg!(version, ty);

    if ty == 4 {
        let mut nums = Vec::new();
        loop {
            let last = msg.read(1);
            let num = msg.read(4);

            dbg!(last);
            dbg!(num);

            nums.push(num as u8);

            if last == 0 {
                return Packet::Literal(version, nums);
            }
        }
    } else {
        let length_type = msg.read(1);

        dbg!(length_type);

        // length_type 0 = 15 bit length
        if length_type == 0 {
            let subpacket_length_bits = msg.read(15);

            dbg!(subpacket_length_bits);

            let mut subpacket_msg = msg.read_bitset(subpacket_length_bits);

            let mut packets = Vec::new();

            while subpacket_msg.len() > 0 {
                packets.push(parse_packet(&mut subpacket_msg));
            }

            return Packet::Operator(Operator { ty, version, packets });
        }
        else {
            let num_subpackets = msg.read(11) as usize;

            dbg!(num_subpackets);

            let mut packets = Vec::new();

            while packets.len() < num_subpackets {
                packets.push(parse_packet(msg));
            }

            return Packet::Operator(Operator { ty, version, packets });
        }
    }
}

fn main() {
    let mut msg = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_input(stdin_lock)
    };

    dbg!(&msg);

    let packet = parse_packet(&mut msg);
    dbg!(&packet);

    println!("version sum: {}", packet.version_sum());

    let evaled = packet.eval();

    println!("evaled: {}", evaled);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, parse_str, BitSet, Input, parse_packet, Packet, Operator};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();

        // assert_eq!(
        //     test_data.data,
        //     vec![0x8A, 0x00, 0x4A, 0x80, 0x1A, 0x80, 0x02, 0xF4, 0x78]
        // );
    }

    #[test]
    fn test_push_bits() {
        let mut bits = BitSet::new();

        bits.push_bits(0xFF, 8);
        assert_eq!(bits.data, vec![0xFF]);

        bits.push_bits(0x00, 3);
        assert_eq!(bits.data, vec![0xFF, 0x00]);

        bits.push_bits(0xFF, 8);
        // assert_eq!(bits.data, vec![0xFF, 0b11111000, 0b111]);
    }

    #[test]
    fn test_get_bits() {
        let mut bits = BitSet::new();

        bits.push_bits(0xFF, 8);
        assert_eq!(bits.data, vec![0xFF]);

        bits.push_bits(0x00, 3);
        assert_eq!(bits.data, vec![0xFF, 0x00]);

        assert_eq!(bits.get(5, 6), 0b111000);
    }

    #[test]
    fn test_iter() {
        let mut bits = BitSet::new();

        bits.push_bits(0x0D, 4);
        assert_eq!(bits.data, vec![0x0D]);

        let bitvec: Vec<bool> = bits.iter().collect();

        assert_eq!(bitvec, vec![true, false, true, true])
    }

    #[test]
    fn test_pase_str() {
        let bits = parse_str("D2FE28");

        assert_eq!(bits.get(0, 1), 1);
        assert_eq!(bits.get(1, 1), 1);
        assert_eq!(bits.get(2, 1), 0);

        assert_eq!(bits.get(0, 3), 6);
    }

    #[test]
    fn test_debug_fmt() {
        let bits = parse_str("D2FE28");

        let debug_str = format!("{:?}", bits);

        assert_eq!(debug_str, "110100101111111000101000");
    }

    #[test]
    fn test_pase_operatorlength_type_0() {
        let mut bits = parse_str("38006F45291200");

        assert_eq!(bits.get(7, 15), 27);

        let packet = parse_packet(&mut bits);

        match packet {
            Packet::Operator(Operator { packets, .. }) => {
                assert_eq!(packets.len(), 2);
            },
            _other => {
                panic!("Expected operator packet");
            }
        }
    }
    #[test]
    fn test_pase_operator_length_type_1() {
        let mut bits = parse_str("EE00D40C823060");

        let packet = parse_packet(&mut bits);

        match packet {
            Packet::Operator(Operator { packets, .. }) => {
                assert_eq!(packets.len(), 3);
            },
            _other => {
                panic!("Expected operator packet");
            }
        }
    }

    #[test]
    fn test_version_sum_1() {
        let mut bits = parse_str("8A004A801A8002F478");

        let packet = parse_packet(&mut bits);

        assert_eq!(packet.version_sum(), 16);
    }

    #[test]
    fn test_version_sum_2() {
        let mut bits = parse_str("620080001611562C8802118E34");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.version_sum(), 12);
    }

    #[test]
    fn test_version_sum_3() {
        let mut bits = parse_str("C0015000016115A2E0802F182340");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.version_sum(), 23);
    }

    #[test]
    fn test_version_sum_4() {
        let mut bits = parse_str("A0016C880162017C3686B18A3D4780");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.version_sum(), 31);
    }

    #[test]
    fn test_eval_1() {
        let mut bits = parse_str("C200B40A82");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 3);
    }

    #[test]
    fn test_eval_2() {
        let mut bits = parse_str("04005AC33890");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 54);
    }

    #[test]
    fn test_eval_3() {
        let mut bits = parse_str("880086C3E88112");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 7);
    }

    #[test]
    fn test_eval_4() {
        let mut bits = parse_str("CE00C43D881120");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 9);
    }

    #[test]
    fn test_eval_5() {
        let mut bits = parse_str("D8005AC2A8F0");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 1);
    }

    #[test]
    fn test_eval_6() {
        let mut bits = parse_str("F600BC2D8F");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 0);
    }

    #[test]
    fn test_eval_7() {
        let mut bits = parse_str("9C005AC2F8F0");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 0);
    }

    #[test]
    fn test_eval_8() {
        let mut bits = parse_str("9C0141080250320F1802104A08");

        let packet = parse_packet(&mut bits);

        dbg!(&packet);

        assert_eq!(packet.eval(), 1);
    }
}
