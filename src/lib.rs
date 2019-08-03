use rand::seq::SliceRandom;
use std::fmt;
use std::thread;

mod game;
mod tictactoe;
mod cfr;
mod regret;

use game::Game;
use regret::RegretProvider;


pub fn run() {
    let mut regret_provider = regret::HashRegretProvider::new();
    let mut strat_provider = regret::HashRegretProvider::new();
    //can each agent mutably borrow the same regret provider before I start the threads?
    let mut cfr = cfr::CounterFactualRegret::new(&mut regret_provider, &mut strat_provider);
    thread::spawn(move || {
        regret_provider.run();
    });
    thread::spawn(move || {
        strat_provider.run();
    });
    for i in 0..10001 {
        let game = tictactoe::TicTacToe::new();
        cfr.verbose = i == 1000;
        cfr.set_iteration(i);
        let expected_value = cfr.search(game);
        println!("iteration {} expected value {:?}", i, expected_value);
    }
}

pub fn play_random_game<A: fmt::Display + fmt::Debug>(game: &mut Game<Action=A>) {
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
