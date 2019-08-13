use rand::distributions::Distribution;
use crate::game::{Game, Player, Infoset};
use crate::regret;

pub struct CounterFactualRegret<R: regret::RegretHandler> {
    regret_handler: Option<R>,
    strat_handler: R,

    on_player: Player,

    pub verbose: bool,
    iteration: i32,
}

impl<R: regret::RegretHandler> CounterFactualRegret<R> {

    pub fn new(regret_handler: R, strategy_handler: R) -> CounterFactualRegret<R> {
        CounterFactualRegret {
            regret_handler: Some(regret_handler),
            strat_handler: strategy_handler,
            on_player: Player::P1,
            verbose: false,
            iteration: 0,
        }
    }

    pub fn new_strat_only(strategy_sharder: R) -> CounterFactualRegret<R> {
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

    pub fn search<T>(&mut self, mut game: T) -> Option<f32>
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

            let rewards: Option<Vec<f32>> = actions.into_iter().map(|action| {
                let mut subgame = game.clone();
                subgame.take_turn(player, &action);
                self.search(subgame)
            }).collect();
            let rewards = rewards?;

            let expected_value: f32 = probs.iter().zip(rewards.iter())
                .map(|(p, r)| p * r)
                .sum();
            let regrets = rewards.into_iter().map(|r| r - expected_value).collect();

            /*
            if self.verbose {
                println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
                println!("{}", game);
                for ((a, p), r) in actions.iter().zip(probs.iter()).zip(rewards.iter()) {
                    println!("action {} prob {} reward {}", a, p, r);
                }
                println!("regrets {:?}", regrets);
                println!("expected value {:?}", expected_value);
            }
            */

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

    pub fn get_avg_strategy(&self, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f32>> {
        CounterFactualRegret::regret_match(&self.strat_handler, player, infoset, num_actions)
    }

    fn get_iter_strategy(&mut self, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f32>> {
        let regret_handler = self.regret_handler
            .as_mut()
            .expect("Tried to get iter stategy in a strategy-only cfr instance");
        CounterFactualRegret::<R>::regret_match(&regret_handler, player, infoset, num_actions)
    }

    fn regret_match(regret_handler: &R, player: Player, infoset: &Infoset, num_actions: usize) -> Option<Vec<f32>>
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

        let pos_regrets: Vec<f32> = regrets.iter().map(|&regret| {
            if regret > 0.0 {
                regret as f32
            } else {
                0.0
            }
        }).collect();

        let regret_sum: f32 = pos_regrets.iter().sum();

        if regret_sum > 0.0 {
            let probs = pos_regrets.into_iter().map(|regret| {
                regret / regret_sum
            }).collect();
            Some(probs)
        } else {
            let num = num_actions as f32;
            let probs = vec![1.0 / num; num_actions];
            Some(probs)
        }
    }

}