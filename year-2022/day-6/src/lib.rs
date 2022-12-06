use std::collections::VecDeque;


pub fn scan_for_start(packet: &str, sz: usize) -> usize {
    // queue is most recently seen first so we can easily truncate off
    // the start of message seq
    let mut start_seq = VecDeque::with_capacity(sz);

    for (cur_pos, ch) in packet.chars().enumerate() {
        let maybe_duplicate_index = start_seq.iter().copied().position(|seq_ch| seq_ch == ch);

        // Remove the oldest part of the sequence up to and including the duplicating character
        if let Some(duplicate_index) = maybe_duplicate_index {
            start_seq.truncate(duplicate_index);
        }

        start_seq.push_front(ch);

        if start_seq.len() == sz {
            return cur_pos + 1;
        }
    }

    panic!("Did not find start");
}


/// Assumes packet is all ascii lower case letters
/// This is about 7x faster than scan_for_start on my
/// AMD Ryzen 9 5950X 16-Core Processor
pub fn scan_for_start_2(packet: &str, sz: usize) -> usize {
    // Track the last seen location of each letter
    let mut positions = [0usize;26];

    // potential start of the marker
    let mut marker_start = 0usize;

    for (cur_pos, ch) in (packet.bytes().enumerate()) {
        let ch_code = ch - b'a';

        // This will explode if the string is not ascii letters with a panic
        let existing_pos = positions[ch_code as usize];

        // If the existing pos is inside the marker, move the marker start past
        // this duplicate.
        if existing_pos >= marker_start {
            marker_start = existing_pos + 1;
            // Because we check if existing_pos >= marker_start we can leave any positions
            // that might be before the marker as they are with out having to worry
        }

        positions[ch_code as usize] = cur_pos;

        // If we have a marker of size sz we are done!
        if cur_pos >= sz && (cur_pos + 1 - marker_start) == sz {
            return cur_pos + 1;
        }
    }

    panic!("Did not find start");
}

pub fn scan_for_start_3(packet: &str, sz: usize) -> usize {
    let mut positions = [0usize;26];

    // same as scan_for_start_2 but marker_start is always -1 from the true start of the marker
    // which simplifies some of the math but is confusing
    let mut marker_start = 0usize;

    for (cur_pos, ch) in (packet.bytes().enumerate()) {
        let ch_code = ch - b'a';

        let code_position = positions.get_mut(ch_code as usize).unwrap();

        let existing_pos = *code_position;
        *code_position = cur_pos;

        if existing_pos > marker_start {
            marker_start = existing_pos;
        }

        if (cur_pos - marker_start) == sz {
            return cur_pos + 1;
        }
    }

    panic!("Did not find start");
}



pub fn old_scan_for_start(packet: &str, sz: usize) -> usize {
    // queue is most recently seen first so we can easily truncate off
    // the start of message seq
    let mut start_seq = VecDeque::with_capacity(sz);

    for (cur_pos, ch) in packet.chars().enumerate() {
        let maybe_duplicate_index = start_seq.iter().copied().position(|seq_ch| seq_ch == ch);

        // Remove the oldest part of the sequence up to and including the duplicating character
        if let Some(duplicate_index) = maybe_duplicate_index {
            start_seq.truncate(duplicate_index);
        }

        start_seq.push_front(ch);

        if start_seq.len() == sz {
            return cur_pos + 1;
        }
    }

    panic!("Did not find start");
}