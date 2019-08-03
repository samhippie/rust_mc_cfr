use rand::distributions::Distribution;
use crate::game::{Game, Player, Infoset};
use crate::regret;

pub struct CounterFactualRegret {
    regret_handler: regret::RegretHandler,

    on_player: Player,
    rng: rand::rngs::ThreadRng,
}

impl CounterFactualRegret {
    pub fn new(regret_provider: &mut regret::RegretProvider) -> CounterFactualRegret {
        let regret_handler = regret_provider.get_handler();

        CounterFactualRegret {
            regret_handler: regret_handler,

            on_player: Player::P1,
            rng: rand::thread_rng(),
        }
    }

    pub fn search<T>(&mut self, mut game: T) -> f64
        where T: Game + Clone
    {
        let (player, actions) = game.get_turn();
        let infoset = game.get_infoset(player);
        if player == self.on_player {
            //normally I like to get probs first, but recursing first seems interesting
            let rewards: Vec<f64> = actions.iter().map(|action| {
                let subgame = game.clone();
                game.take_turn(player, action);
                self.search(subgame)
            }).collect();

            let probs = self.regret_match(player, &infoset, actions.len());
            let expected_value: f64 = probs.iter().zip(rewards.iter())
                .map(|(p, r)| p * r)
                .sum();
            let regrets = rewards.iter().map(|r| r - expected_value).collect();

            self.regret_handler.send_delta(player, infoset.hash, regrets)
                .expect("Failed to send regret delta");

            expected_value

        } else {
            //TODO are we supposed to sample from here? or from the average strategy?
            //let probs = self.get_avg_strategy(player, &infoset, actions.len());
            let probs = self.regret_match(player, &infoset, actions.len());
            //TODO save probs to average strategy
            let sampler = rand::distributions::WeightedIndex::new(probs).unwrap();
            let action_index = sampler.sample(&mut self.rng);
            let action = &actions[action_index];
            game.take_turn(player, action);

            self.search(game)
        }
    }

    pub fn get_avg_strategy(&self, player: Player, infoset: &Infoset, num_actions: usize) -> Vec<f64> {
        //TODO
        unimplemented!();
    }

    fn regret_match(&self, player: Player, infoset: &Infoset, num_actions: usize) -> Vec<f64>
    {
        let regret_response = self.regret_handler.get_regret(player, infoset.hash)
            .expect("Failed to get regret");

        let regrets = regret_response.regret;

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
            probs
        } else {
            let num = num_actions as f64;
            let probs = vec![1.0 / num; num_actions];
            probs
        }
    }

}