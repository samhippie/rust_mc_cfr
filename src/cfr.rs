use rand::distributions::Distribution;
use crate::game::{Game, Player, Infoset};
use crate::regret;
use crate::regret::regret_sharder;

pub struct CounterFactualRegret {
    //regret_handler: Option<regret::RegretHandler>,
    regret_handler: Option<regret_sharder::RegretSharder>,
    //strat_handler: regret::RegretHandler,
    strat_handler: regret_sharder::RegretSharder,

    on_player: Player,

    pub verbose: bool,
    iteration: i32,
}

impl CounterFactualRegret {

    pub fn new(regret_sharder: regret_sharder::RegretSharder, strategy_sharder: regret_sharder::RegretSharder) -> CounterFactualRegret {
        CounterFactualRegret {
            regret_handler: Some(regret_sharder),
            strat_handler: strategy_sharder,
            on_player: Player::P1,
            verbose: false,
            iteration: 0,
        }
    }

    pub fn new_strat_only(strategy_sharder: regret_sharder::RegretSharder) -> CounterFactualRegret {
        CounterFactualRegret {
            regret_handler: None,
            strat_handler: strategy_sharder,

            on_player: Player::P1,

            verbose: false,
            iteration: 0,
        }
    }

    pub fn set_iteration(&mut self, iteration: i32) {
        self.iteration = iteration;
        self.on_player = if iteration % 2 == 0 {
            Player::P1
        } else {
            Player::P2
        };
    }

    pub fn search<T>(&mut self, mut game: T) -> Option<f64>
        where T: Game + Clone
    {
        if self.verbose {
            println!("---------------");
            println!("{}", game);
        }

        if let Some(reward) = game.get_reward() {
            if self.verbose {
                println!("reward {:?}", reward);
            }

            return match self.on_player {
                Player::P1 => Some(reward),
                Player::P2 => Some(-1.0 * reward),
            }
        }

        let (player, actions) = game.get_turn();
        let infoset = game.get_infoset(player);
        if player == self.on_player {
            let probs = self.get_iter_strategy(player, &infoset, actions.len())?;

            let rewards: Option<Vec<f64>> = actions.iter().map(|action| {
                let mut subgame = game.clone();
                subgame.take_turn(player, action);
                self.search(subgame)
            }).collect();
            let rewards = rewards?;

            let expected_value: f64 = probs.iter().zip(rewards.iter())
                .map(|(p, r)| p * r)
                .sum();
            let regrets = rewards.iter().map(|r| r - expected_value).collect();

            if self.verbose {
                println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
                println!("{}", game);
                for ((a, p), r) in actions.iter().zip(probs.iter()).zip(rewards.iter()) {
                    println!("action {} prob {} reward {}", a, p, r);
                }
                println!("regrets {:?}", regrets);
                println!("expected value {:?}", expected_value);
            }

            self.regret_handler
                .as_mut()
                .expect("Tried to search in a stategy-only cfr instance")
                .send_delta(player, infoset.hash, regrets, self.iteration / 2 + 1)
                .expect("Failed to send regret delta");

            Some(expected_value)

        } else {
            let probs = self.get_iter_strategy(player, &infoset, actions.len())?;
            let sampler = rand::distributions::WeightedIndex::new(&probs).unwrap();

            self.strat_handler.send_delta(player, infoset.hash, probs, self.iteration)
                .expect("Failed to update average strategy");

            let action_index = sampler.sample(&mut rand::thread_rng());
            let action = &actions[action_index];
            game.take_turn(player, action);

            self.search(game)
        }
    }

    pub fn get_avg_strategy(&self, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f64>> {
        CounterFactualRegret::regret_match(&self.strat_handler, player, infoset, num_actions)
    }

    fn get_iter_strategy(&mut self, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f64>> {
        let regret_handler = self.regret_handler
            .as_mut()
            .expect("Tried to get iter stategy in a strategy-only cfr instance");
        CounterFactualRegret::regret_match(&regret_handler, player, infoset, num_actions)
    }

    fn regret_match(regret_handler: &regret_sharder::RegretSharder, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f64>>
    {
        let regret_response = regret_handler.get_regret(player, infoset.hash)
            .expect("Failed to get regret");

        let regrets = match regret_response {
            regret::Response::Regret(regret_response) => regret_response.regret,
            regret::Response::Closed => return None
        };

        let regrets = match regrets {
            Some(regret) => regret,
            None => vec![0.0; num_actions],
        };

        //println!("infoset hash {}", infoset.hash);
        //println!("actual regrets {:?}", regrets);

        let pos_regrets: Vec<f64> = regrets.iter().map(|&regret| {
            if regret > 0.0 {
                regret as f64
            } else {
                0.0
            }
        }).collect();

        let regret_sum: f64 = pos_regrets.iter().sum();

        if regret_sum > 0.0 {
            let probs = pos_regrets.into_iter().map(|regret| {
                regret / regret_sum
            }).collect();
            Some(probs)
        } else {
            let num = num_actions as f64;
            let probs = vec![1.0 / num; num_actions];
            Some(probs)
        }
    }

}