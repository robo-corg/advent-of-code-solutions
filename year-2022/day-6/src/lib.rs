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