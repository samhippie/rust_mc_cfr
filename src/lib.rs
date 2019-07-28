use std::fmt;
use rand::seq::SliceRandom;

mod game;
use game::Game;

mod tictactoe;

pub fn run() {
    let mut game = tictactoe::TicTacToe::new();
    play_game(&mut game);
}

pub fn play_game<A: fmt::Display>(game: &mut Game<Action=A>) {
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
