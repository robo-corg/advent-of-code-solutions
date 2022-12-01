use std::collections::HashMap;

use itertools::iproduct;

use day_21::{Game, play_part2};

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

    dbg!(start_positions);
    let win_counts = play_part2(
        Game::new(start_positions)
    );

    dbg!(win_counts);
}

#[cfg(test)]
mod test {
    use day_21::{play_part2, Game};

    #[test]
    fn test_part2() {
        let win_counts = play_part2(
            Game::new([3, 7])
        );

        assert_eq!(win_counts, [444356092776315, 341960390180808]);
    }
}
