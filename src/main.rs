use rand::seq::SliceRandom;
use std::fmt;

mod tictactoe;
mod game;

use crate::game::Game;

fn main() {
    let mut game = tictactoe::TicTacToe::new();
    play_game(&mut game);
}

fn play_game<A: fmt::Display>(game: &mut Game<Action=A>) {
    let mut rng = rand::thread_rng();

    loop {
        println!("{}", game);
        match game.get_reward() {
            None => {
                let (player, actions) = game.get_turn();
                let action = actions.choose(&mut rng).expect("No actions in unfinished game");
                println!("Taking action {}", action);
                game.take_turn(player, action);
            },
            Some(reward) => {
                println!("Player 1 Reward: {}", reward);
                break;
            }
        }
    }
}
