use std::{io::{self, BufRead}, collections::HashSet};

use ndarray::{Array2, Array1};

type Board = Array2<i32>;
type BoardMarkings = Array2<i32>;



fn parse_boards(mut reader: impl BufRead) -> (Vec<i32>, Vec<Board>){
    let mut drawings_line = String::new();
    reader.read_line(&mut drawings_line).unwrap();

    let drawings: Vec<i32> = drawings_line.split(",").map(|num| i32::from_str_radix(num.trim(), 10).unwrap()).collect();

    let mut boards = Vec::new();

    {
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
    }


    'read_boards: loop {
        let mut row_buf = String::new();

        let mut board_rows: Vec<Array1<i32>> = Vec::new();

        loop {
            if reader.read_line(&mut &mut row_buf).unwrap() > 0 {
                if row_buf.trim().len() == 0 {
                    dbg!("finished board");
                    break;
                }

                dbg!(&row_buf);

                let row_vec: Vec<i32> = row_buf.trim().split_whitespace().map(|num| i32::from_str_radix(num, 10).unwrap()).collect();

                board_rows.push(Array1::from_vec(row_vec));

                row_buf.clear();
            }
            else {
                if board_rows.is_empty() {
                    break 'read_boards;
                }
                else {
                    break;
                }
            }
        }

        if !board_rows.is_empty() {
            // push to boards
            //Board::from(board_rows);

            let width = board_rows[0].len();
            let height = board_rows.len();

            dbg!(width, height);

            let mut board = Board::zeros((0, width));

            for row in board_rows.into_iter() {
                dbg!(&row);
                board.push_row(row.view()).unwrap();
            }

            dbg!(&board);

            boards.push(board);
        }
    }

    (
        drawings,
        boards
    )
}

fn main() {
    let (drawings, boards) = {
        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        parse_boards(stdin_lock)
    };

    let mut board_markings: Vec<BoardMarkings> = boards.iter().map(|board| {
        let s = board.shape();
        BoardMarkings::zeros((s[0], s[1]))
    }).collect();

    dbg!(&board_markings);

    let mut winning_board_infos = Vec::new();

    let mut winners = HashSet::new();

    'find_winning_board: for drawing in drawings.iter().copied() {
        for (board_id, board) in boards.iter().enumerate() {
            if winners.contains(&board_id) {
                continue;
            }

            let mut markings = &mut board_markings[board_id];

            let s = board.shape();
            let row_count = s[0];
            let col_count = s[1];

            for row in 0..row_count {
                for col in 0..col_count {
                    if board[(row, col)] == drawing {
                        markings[(row, col)] = 1;
                    }
                }
            }

            let row_matches = markings.rows().into_iter().any(|row| row.iter().all(|cell| *cell > 0));
            let col_matches = markings.columns().into_iter().any(|row| row.iter().all(|cell| *cell > 0));

            let bingo = row_matches || col_matches;

            if bingo {
                winning_board_infos.push((board_id, drawing));
                winners.insert(board_id);
                //break 'find_winning_board;
            }
        }
    }

    //dbg!(winning_board_info);

    if let Some((winning_board_id, final_call)) = winning_board_infos.last() {
        let winning_board = &boards[*winning_board_id];
        let winning_markings = &board_markings[*winning_board_id];

        dbg!(&winning_markings);

        let marked_inv = winning_markings.mapv(|cell| if cell > 0 { 0 } else { 1 });

        dbg!(&marked_inv);

        let only_marked = winning_board * marked_inv;

        dbg!(&only_marked);

        let raw_score: i32 = only_marked.iter().sum();
        let score = raw_score * final_call;

        dbg!(score);
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use ndarray::array;

    use crate::{parse_boards, Board};

    fn get_test_input() -> (Vec<i32>, Vec<Board>) {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_boards(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let (drawings, boards) = get_test_input();

        assert_eq!(drawings, vec![7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1]);

        assert_eq!(boards.len(), 3);

        assert_eq!(
            boards[0],
            array![
                [22, 13, 17, 11, 0],
                [8, 2, 23, 4, 24],
                [21, 9, 14, 16, 7],
                [6, 10, 3, 18, 5],
                [1, 12, 20, 15, 19],
            ]
        );
    }
}