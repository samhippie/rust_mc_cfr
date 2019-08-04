use rand::seq::SliceRandom;
use std::fmt;
use std::thread;
use rand::distributions::Distribution;

mod game;
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

    //let get_game = || game::RockPaperScissors::new();
    //let get_game = || game::TicTacToe::new();
    let get_game = || game::OneCardPoker::new();

    //TODO have a better configuration method
    let num_threads = 24;
    let num_shards = 6;
    let num_iterations = 10_000_000;
    let num_games = 20;
    //println!("agent threads: {}", num_threads);
    //println!("regret provider threads: {}", num_shards);
    //println!("strategy provider threads: {}", num_shards);
    //println!("iterations: {}", num_iterations);


    //each provider will hold part of the regret table
    let mut regret_providers: Vec<regret::HashRegretProvider> = (0..num_shards)
        .map(|_| {
            regret::HashRegretProvider::new()
        })
        .collect();
    let mut strategy_providers: Vec<regret::HashRegretProvider> = (0..num_shards)
        .map(|_| {
            regret::HashRegretProvider::new()
        })
        .collect();

    //each thread's agent
    //each agent gets its own regret sharder, but the regret shards share the providers
    let cfrs: Vec<cfr::CounterFactualRegret> = (0..num_threads)
        .map(|_| {
            let regret_sharder = regret::RegretSharder::new(&mut regret_providers);
            let strategy_sharder = regret::RegretSharder::new(&mut strategy_providers);
            cfr::CounterFactualRegret::new(regret_sharder, strategy_sharder)
        })
        .collect();

    //agent for after we've trained
    let strategy_sharder = regret::RegretSharder::new(&mut strategy_providers);
    let strat_cfr = cfr::CounterFactualRegret::new_strat_only(strategy_sharder);

    //let the providers run
    //if we want to stop them, we'll have to send a Request::Close
    for mut provider in regret_providers.into_iter() {
        thread::spawn(move || {
            provider.run();
        });
    }
    for mut provider in strategy_providers.into_iter() {
        thread::spawn(move || {
            provider.run();
        });
    }

    //training
    let children: Vec<thread::JoinHandle<_>> = cfrs.into_iter().enumerate().map(|(tid, mut cfr)| {
        thread::spawn(move || {
            for i in 0..num_iterations {
                let game = get_game();
                cfr.set_iteration(i);
                cfr.search(game);
            }
        })
    }).collect();

    for child in children.into_iter() {
        child.join().unwrap();
    }

    //playing games
    /*
    for _ in 0..num_games {
        println!("---------------------------");
        let mut game = get_game();
        play_cfr_game(&mut game, &strat_cfr);
    }
    */
    print_ocp_table(&strat_cfr);


}

//generate table like http://www.cs.cmu.edu/~ggordon/poker/
pub fn print_ocp_table(cfr : &cfr::CounterFactualRegret) {
    let num_cards = 13;

    print!("label,");
    for hand1 in 0..num_cards {
        print!("{},", hand1);
    }
    println!("");

    print!("on pass,");
    for hand2 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((0, hand2), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Call);
        let probs = cfr.get_avg_strategy(game::Player::P2, &game.get_infoset(game::Player::P2), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!("");

    print!("on bet,");
    for hand2 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((0, hand2), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Bet);
        let probs = cfr.get_avg_strategy(game::Player::P2, &game.get_infoset(game::Player::P2), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!("");

    println!("");

    print!("label,");
    for hand1 in 0..num_cards {
        print!("{},", hand1);
    }
    println!("");

    print!("1st round,");
    for hand1 in 0..num_cards {
        let game = game::OneCardPoker::manual_new((hand1, 0), game::Player::P2);
        let probs = cfr.get_avg_strategy(game::Player::P1, &game.get_infoset(game::Player::P1), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!("");
    print!("2nd round,");
    for hand1 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((hand1, 0), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Call);
        game.take_turn(game::Player::P2, &game::OneCardPokerAction::Bet);
        let probs = cfr.get_avg_strategy(game::Player::P1, &game.get_infoset(game::Player::P1), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!("");

}

pub fn play_cfr_game<A: fmt::Display + fmt::Debug>(game: &mut Game<Action=A>, cfr: &cfr::CounterFactualRegret) {
    let mut rng = rand::thread_rng();

    loop {
        println!();
        println!("{}", game);
        match game.get_reward() {
            None => {
                let (player, actions) = game.get_turn();
                let infoset = game.get_infoset(player);
                let probs = cfr.get_avg_strategy(player, &infoset, actions.len())
                    .expect("Failed to get strategy probabilities");

                println!("player {}", player);
                for (action, prob) in actions.iter().zip(probs.iter()) {
                    println!("action {}\tprob {}", action, prob);
                }

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
