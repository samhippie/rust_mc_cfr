use rand::seq::SliceRandom;
use std::fmt;
use std::thread;
use rand::distributions::Distribution;

mod game;
mod tictactoe;
mod rps;
mod cfr;
mod regret;

use game::Game;
use regret::RegretProvider;


pub fn run() {
    //let mut game = rps::RockPaperScissors::new();
    //play_random_game(&mut game);
    do_cfr();
}

fn do_cfr() {

    //let get_game = || rps::RockPaperScissors::new();
    let get_game = || tictactoe::TicTacToe::new();

    let num_threads = 16;

    let mut regret_provider = regret::HashRegretProvider::new();
    let mut strat_provider = regret::HashRegretProvider::new();
    //can each agent mutably borrow the same regret provider before I start the threads?
    let cfrs: Vec<cfr::CounterFactualRegret> = (0..num_threads)
        .map(|_| cfr::CounterFactualRegret::new(&mut regret_provider, &mut strat_provider))
        .collect();

    //agent for after we've trained
    let strat_cfr = cfr::CounterFactualRegret::new_strat_only(&mut strat_provider);

    thread::spawn(move || {
        regret_provider.run();
    });
    thread::spawn(move || {
        strat_provider.run();
    });

    println!("num cfrs {}", cfrs.len());
    let children: Vec<thread::JoinHandle<_>> = cfrs.into_iter().enumerate().map(|(tid, mut cfr)| {
        println!("starting thread {}", tid);
        thread::spawn(move || {
            for i in 0..1000 {
                let game = get_game();
                cfr.set_iteration(i);
                cfr.search(game);
            }
        })
    }).collect();

    for child in children.into_iter() {
        child.join().unwrap();
    }

    for _ in 0..10 {
        println!("---------------------------");
        let mut game = get_game();
        play_cfr_game(&mut game, &strat_cfr);
    }


}

pub fn play_cfr_game<A: fmt::Display + fmt::Debug>(game: &mut Game<Action=A>, cfr: &cfr::CounterFactualRegret) {
    let mut rng = rand::thread_rng();

    loop {
        println!("{}", game);
        match game.get_reward() {
            None => {
                let (player, actions) = game.get_turn();
                let infoset = game.get_infoset(player);
                let probs = cfr.get_avg_strategy(player, &infoset, actions.len())
                    .expect("Failed to get strategy probabilities");

                println!("actions {:?}", actions);
                println!("probs {:?}", probs);

                let sampler = rand::distributions::WeightedIndex::new(&probs).unwrap();
                let action_index = sampler.sample(&mut rng);
                let action = &actions[action_index];

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
