use std::fmt;
use rand::seq::SliceRandom;

mod game;
use game::Game;
mod tictactoe;
mod cfr;
mod regret;


pub fn run() {
    let game = tictactoe::TicTacToe::new();
    let mut regret_provider = regret::HashRegretProvider::new();
    //can each agent mutably borrow the same regret provider before I start the threads?
    let mut cfr = cfr::CounterFactualRegret::new(&mut regret_provider);
    cfr.search(game);
}

pub fn play_random_game<A: fmt::Display>(game: &mut Game<Action=A>) {
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
