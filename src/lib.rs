use rand::seq::SliceRandom;
use rand::Rng;
use std::thread;
use rand::distributions::Distribution;
use std::sync::{Barrier, Arc};
use std::io;
use std::io::prelude::*;

mod game;
mod cfr;
mod regret;
mod mcts_exploit;
mod tree_exploit;

use game::Game;
use regret::RegretProvider;

pub fn run() {
    /*
    for _ in 0 .. 10 {
        let mut game = game::MatrixGame::new(2, vec![1.0, 0.9, -0.7, 1.0]);
        play_random_game(&mut game);
        println!("-----------------");
    }
    */
    do_cfr();
}

enum RegretType {
    HashMap,
    RocksDb(String),
}

fn get_regret_providers(regret_type: RegretType, num: usize, config: &regret::RegretConfig) -> Vec<Box<dyn RegretProvider>> {
    (0..num).map(|i| {
        match &regret_type {
            RegretType::RocksDb(name) => {
                let mut provider = regret::RocksDbRegretProvider::new(&format!("{}-{}", name, i));
                provider.set_config(config);
                Box::new(provider) as Box<dyn RegretProvider>
            }
            RegretType::HashMap => {
                let mut provider = regret::HashRegretProvider::new();
                provider.set_config(config);
                Box::new(provider) as Box<dyn RegretProvider>
            }
        }
    }).collect()
}

fn do_cfr() {

    //let get_game = || game::TicTacToe::new();
    //let get_game = || game::OneCardPoker::new();
    let get_game = || game::Skulls::manual_new(game::Player::P1, 1, 3);
    //let get_game = || game::MatrixGame::new(2, vec![1.0, 0.9, -0.7, 1.0]);
    //let get_game = || game::DoubleMatrixGame::new(2, vec![1.0, 0.9, -0.7, 1.0]);
    //let get_game = || game::MatrixGame::new_rock_paper_scissors();

    //TODO have a better configuration method
    let num_threads = 16;
    let num_shards = 1;
    let num_mcts_shards = 8;
    let num_games = 20;
    let num_steps = 10;
    let step_size = 1000;
    let num_exploit_mcts_iterations = 100;//_000_000;

    let mut regret_config = regret::RegretConfig { 
        alpha: 1.5, 
        beta: 0.0, 
        gamma: 2.0, 
        is_strategy: false 
    };

    //TODO shouldn't the regret type go in the regret config?
    let regret_types = (RegretType::RocksDb(String::from("regret")), RegretType::RocksDb(String::from("strategy")));
    //let regret_types = (RegretType::HashMap, RegretType::HashMap);

    let mut regret_providers = get_regret_providers(regret_types.0, num_shards, &regret_config);
    regret_config.is_strategy = true;
    let mut strategy_providers = get_regret_providers(regret_types.1, num_shards, &regret_config);

    //each thread's agent
    //each agent gets its own regret handler, but the regret handlers share the providers
    //for loop + push instead of map because I don't feel like typing out cfrs's type signature
    let mut cfrs = vec![];
    for _ in 0..num_threads {
            let regret_handler = regret::RegretSharder::new(&mut regret_providers);
            let strategy_handler = regret::RegretSharder::new(&mut strategy_providers);
            let cfr = cfr::CounterFactualRegret::new(Box::new(regret_handler), Box::new(strategy_handler));
            cfrs.push(cfr);
    }

    //agent for after we've trained
    let strategy_handler = regret::RegretSharder::new(&mut strategy_providers);
    //let strategy_handler = strategy_provider.get_handler();
    let strat_cfr = cfr::CounterFactualRegret::new_strat_only(Box::new(strategy_handler));

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
    let barrier = Arc::new(Barrier::new(cfrs.len()));
    let providers: Arc<Vec<mcts_exploit::StrategyProvider>> = Arc::new(
        (0..num_mcts_shards).map(|_| mcts_exploit::StrategyProvider::new()).collect()
    );
    let children: Vec<thread::JoinHandle<_>> = cfrs.into_iter().enumerate().map(|(tid, mut cfr)| {
        let thread_barrier = barrier.clone();
        let providers = providers.clone();
        thread::spawn(move || {
            for step in 0..num_steps {
                /*
                //do this first to get a baseline over the default random strategy
                //all threads will do the mcts search, but thread 0 will manage everything
                let mut mcts = mcts_exploit::MonteCarloTreeSearch::new(Box::new(get_game), &cfr, providers.clone());
                for _ in 0..1 {
                    let (exp1, exp2) = mcts.run(num_exploit_mcts_iterations);
                    if tid == 0 {
                        println!("step, exp-p1, exp-p2, exploitability, {}, {}, {}, {}", step, exp1, exp2, exp1 + exp2);
                    }
                }
                panic!();
                */
                //thread_barrier.wait();
                /*
                if tid == 0 {
                    let mut tree = tree_exploit::TreeExploit::new(Box::new(get_game), &cfr);
                    let rsp_vals = tree.run();
                    println!("step, p1 best response, p2 best reponse, exploitability, {}, {}, {}, {}", step, rsp_vals.0, rsp_vals.1, rsp_vals.0 + rsp_vals.1);
                }
                */
                if tid == 0 {
                    for provider in providers.iter() {
                        provider.clear();
                    }
                    play_cfr_game(&mut get_game(), &cfr);
                }
                //thread_barrier.wait();

                for i in 0..step_size {
                    let iteration = step * step_size + i;
                    if tid == 0 {
                        println!("tid-iteration, {}, {}", tid, iteration);
                    }
                    let game = get_game();
                    cfr.set_iteration(iteration);
                    let exp_val = cfr.search(game, 0);
                    if tid == 0 {
                        if let Some(exp_val) = exp_val {
                            //println!("iteration-exp value {}, {}", iteration, exp_val);
                        }
                    }
                }

                //thread_barrier.wait();
                //do this first to get a baseline over the default random strategy
                //all threads will do the mcts search, but thread 0 will manage everything
                let mut mcts = mcts_exploit::MonteCarloTreeSearch::new(Box::new(get_game), &cfr, providers.clone());
                if tid == 0 {
                    mcts.set_verbose(true);
                }
                for _ in 0..1 {
                    let (exp1, exp2) = mcts.run(num_exploit_mcts_iterations);
                    if tid == 0 {
                        //println!("step, exp-p1, exp-p2, exploitability, {}, {}, {}, {}", step, exp1, exp2, exp1 + exp2);
                        panic!();
                    }
                }
                thread_barrier.wait();

            }
        })
    }).collect();

    for child in children.into_iter() {
        child.join().unwrap();
    }

    //playing games
    for _ in 0..num_games {
        println!("---------------------------");
        let mut game = get_game();
        play_cfr_game(&mut game, &strat_cfr);
    }
    //play_user_game(&mut get_game(), &strat_cfr);
    //print_ocp_table(&strat_cfr);
}

//generate table like http://www.cs.cmu.edu/~ggordon/poker/
pub fn print_ocp_table<R: regret::RegretHandler>(cfr : &cfr::CounterFactualRegret) {
    let num_cards = 13;

    print!("label,");
    for hand1 in 0..num_cards {
        print!("{},", hand1);
    }
    println!();

    print!("on pass,");
    for hand2 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((0, hand2), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Call);
        let probs = cfr.get_avg_strategy(game::Player::P2, &game.get_infoset(game::Player::P2), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!();

    print!("on bet,");
    for hand2 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((0, hand2), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Bet);
        let probs = cfr.get_avg_strategy(game::Player::P2, &game.get_infoset(game::Player::P2), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!();

    println!();

    print!("label,");
    for hand1 in 0..num_cards {
        print!("{},", hand1);
    }
    println!();

    print!("1st round,");
    for hand1 in 0..num_cards {
        let game = game::OneCardPoker::manual_new((hand1, 0), game::Player::P2);
        let probs = cfr.get_avg_strategy(game::Player::P1, &game.get_infoset(game::Player::P1), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!();
    print!("2nd round,");
    for hand1 in 0..num_cards {
        let mut game = game::OneCardPoker::manual_new((hand1, 0), game::Player::P2);
        game.take_turn(game::Player::P1, &game::OneCardPokerAction::Call);
        game.take_turn(game::Player::P2, &game::OneCardPokerAction::Bet);
        let probs = cfr.get_avg_strategy(game::Player::P1, &game.get_infoset(game::Player::P1), 2).unwrap();
        print!("{},", probs[1]);
    }
    println!();

}

pub fn play_cfr_game<G: Game>(game: &mut G, cfr: &cfr::CounterFactualRegret) {
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

                println!("***Taking action {}", action);
                game.take_turn(player, action);
            },
            Some(reward) => {
                println!("Player 1 Reward: {}", reward);
                break;
            }
        }
    }
}

pub fn play_user_game(game: &mut impl Game, cfr: &cfr::CounterFactualRegret) {
    let mut rng = rand::thread_rng();
    let user_player = if rng.gen::<bool>() {
        game::Player::P1
    } else {
        game::Player::P2
    };
    println!("You are {}", user_player);

    loop {
        println!();
        match game.get_reward() {
            None => {
                let (player, actions) = game.get_turn();
                if player == user_player {
                    println!("{}", game.get_summary_string(player));
                    println!();
                    println!("User Player {}", player);

                    for (i, action) in actions.iter().enumerate() {
                        println!("action {}: {}", i, action);
                    }

                    let action_index = loop {
                        print!("Your action:");
                        io::stdout().flush().ok().expect("Failed to flush stdout");
                        let mut action_index = String::new();
                        io::stdin().read_line(&mut action_index)
                            .expect("Failed to read line");

                        let action_index= action_index.trim().parse::<usize>();
                        if let Ok(i) = action_index {
                            if i < actions.len() {
                                break i;
                            }
                        }
                    };
                    let action = &actions[action_index];
                    println!("{} taking action {}", player, action);
                    game.take_turn(player, action);

                } else {
                    println!("CFR Player {}", player);
                    let infoset = game.get_infoset(player);
                    let probs = cfr.get_avg_strategy(player, &infoset, actions.len())
                        .expect("Failed to get strategy probabilities");

                    for (i, action) in actions.iter().enumerate() {
                        println!("action {}: {}", i, action);
                    }

                    let sampler = rand::distributions::WeightedIndex::new(&probs).unwrap();
                    let action_index = sampler.sample(&mut rng);
                    let action = &actions[action_index];

                    game.take_turn(player, action);
                }
            },
            Some(reward) => {
                let reward = if user_player == game::Player::P1 {
                    reward
                } else {
                    -1.0 * reward
                };
                println!("User Reward: {}", reward);
                break;
            }
        }
    }
}


pub fn play_random_game<G: Game>(game: &mut G) {
    let mut rng = rand::thread_rng();

    loop {
        println!("{}", game);
        match game.get_reward() {
            None => {
                let (player, actions) = game.get_turn();
                let action = actions.choose(&mut rng).expect("No actions in unfinished game");
                println!("***Taking action {}", action);
                game.take_turn(player, action);
            },
            Some(reward) => {
                println!("Player 1 Reward: {}", reward);
                break;
            }
        }
    }
}
