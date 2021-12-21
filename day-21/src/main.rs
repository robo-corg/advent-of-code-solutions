use std::{io::{self, BufRead}, collections::HashMap};

use itertools::iproduct;

type Input = Vec<i32>;

fn parse_input(mut reader: impl BufRead) -> Input {
    unimplemented!()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Game {
    cur_player: u8,
    player_score: [u8; 2],
    player_pos: [u8; 2]
}

fn flip_arr<T: Copy>(a: [T;2]) -> [T;2] {
    [
        a[1],
        a[0]
    ]
}

impl Game {
    fn new(player_pos: [i32; 2]) -> Self {
        Game {
            cur_player: 0,
            player_pos: [player_pos[0] as u8, player_pos[1] as u8],
            player_score: [0, 0]
        }
    }

    fn flipped(&self) -> Self {
        Game {
            cur_player: 1 - self.cur_player,
            player_score: flip_arr(self.player_score),
            player_pos: flip_arr(self.player_pos)
        }
    }
}

fn play_part2(game: Game, possible_rolls: &Vec<u8>, memoize: &mut HashMap<Game, [usize; 2]>) -> [usize; 2] {
    assert!(game.player_score[0] < 21);
    assert!(game.player_score[1] < 21);

    if let Some(cached) = memoize.get(&game) {
        return *cached;
    }

    if let Some(cached) = memoize.get(&game.flipped()) {
        return flip_arr(*cached);
    }

    let mut player_wins = [0; 2];

    let player_num = game.cur_player as usize;

    for roll in possible_rolls.iter().copied() {
        //dbg!(roll_player_1, roll_player_2);
        let mut game_fork = game.clone();


        game_fork.player_pos[player_num] = (game_fork.player_pos[player_num] + roll) % 10;
        game_fork.player_score[player_num] += game_fork.player_pos[player_num] + 1;


        if game_fork.player_score[player_num] >= 21 {
            player_wins[player_num] += 1;
        }
        else {
            game_fork.cur_player = 1 - game_fork.cur_player;
            let all_outcomes = play_part2(game_fork, possible_rolls, memoize);

            for player_num in 0..2 {
                player_wins[player_num] += all_outcomes[player_num];
            }
        }
    }

    memoize.insert(game, player_wins);

    player_wins
}

fn main() {
    let start_positions = [
        6,
        2
    ];

    let mut player_score: [i32; 2] = [0; 2];
    let mut player_pos: [i32; 2] = start_positions;

    let mut rolls = 0;


    let mut maybe_winning_player = None;

    {
        let mut dice = (0..).map(|roll| {
            rolls += 1;
            (roll % 100) + 1
        });

        'game: loop {
            for player_num in 0..2 {
                let total:i32  = dice.by_ref().take(3).sum();
                //dbg!(total);
                player_pos[player_num] = (player_pos[player_num] + total) % 10;
                player_score[player_num] += player_pos[player_num] + 1;
                //dbg!(player_score[player_num]);

                if player_score[player_num] >= 1000 {
                    maybe_winning_player = Some(player_num);
                    break 'game;
                }
            }
        }
    }

    dbg!(player_score, rolls);

    println!("{}", rolls*player_score[1-maybe_winning_player.unwrap()]);

    let possible_rolls_part2: Vec<u8> = iproduct!(1..4, 1..4, 1..4).map(|(a, b, c)| { a + b + c}).collect();

    //let possible_rolls_part2: Vec<i32> = iproduct!(1..4).map(|a| { a }).collect();

    dbg!(&possible_rolls_part2);

    let mut memoized = HashMap::new();

    dbg!(start_positions);

    let win_counts = play_part2(
        Game::new(start_positions),
        &possible_rolls_part2,
        &mut memoized
    );

    dbg!(win_counts);
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{parse_input, Input};

    fn get_test_input() -> Input {
        let test_data_str = include_str!("../test_input.txt");

        let test_data_reader = Cursor::new(test_data_str.to_owned());

        parse_input(test_data_reader)
    }

    #[test]
    fn test_parse() {
        let test_data = get_test_input();
    }
}
