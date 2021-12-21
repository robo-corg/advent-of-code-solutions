use std::hash::Hash;

// Substantially faster
use rustc_hash::FxHashMap as HashMap;

use itertools::iproduct;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Game {
    cur_player: u8,
    player_score: [u8; 2],
    player_pos: [u8; 2],
}

impl Hash for Game {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash: u32 = self.cur_player as u32      // cur_player is 1-bit
            | (self.player_pos[0] as u32) << 1      // position is 4-bit since it is mod 10
            | (self.player_pos[1] as u32) << 5      // position is 4-bit since it is mod 10
            | (self.player_score[0] as u32) << 9    // score is 5-bit since it is < 21
            | (self.player_score[1] as u32) << 14;  // score is 5-bit since it is < 21
        state.write_u32(hash);
    }
}

fn flip_arr<T: Copy>(a: [T; 2]) -> [T; 2] {
    [a[1], a[0]]
}

impl Game {
    pub fn new(player_pos: [i32; 2]) -> Self {
        Game {
            cur_player: 0,
            player_pos: [player_pos[0] as u8, player_pos[1] as u8],
            player_score: [0, 0],
        }
    }

    fn flipped(&self) -> Self {
        Game {
            cur_player: 1 - self.cur_player,
            player_score: flip_arr(self.player_score),
            player_pos: flip_arr(self.player_pos),
        }
    }
}

fn play_part2_inner(
    game: Game,
    possible_rolls: &Vec<u8>,
    memoize: &mut HashMap<Game, [usize; 2]>,
) -> [usize; 2] {
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
        } else {
            game_fork.cur_player = 1 - game_fork.cur_player;
            let all_outcomes = play_part2_inner(game_fork, possible_rolls, memoize);

            for player_num in 0..2 {
                player_wins[player_num] += all_outcomes[player_num];
            }
        }
    }

    memoize.insert(game, player_wins);

    player_wins
}

pub fn play_part2(game: Game) -> [usize; 2] {
    let possible_rolls_part2: Vec<u8> = iproduct!(1..4, 1..4, 1..4)
        .map(|(a, b, c)| a + b + c)
        .collect();
    let mut memoized = HashMap::default();
    play_part2_inner(game, &possible_rolls_part2, &mut memoized)
}
