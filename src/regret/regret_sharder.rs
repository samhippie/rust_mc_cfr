use std::error;

use crate::regret::regret_provider::{RegretProvider, RegretHandler, Response};
use crate::game::Player;

/// Combines several regret providers into a single regret handler
/// 
/// This is useful for using spreading regrets across multiple regret providers
/// on separate threads without the providers needing to coordinate
pub struct RegretSharder {
    regret_handlers: Vec<Box<dyn RegretHandler>>,
}

impl RegretSharder {
    pub fn new(regret_providers: &mut Vec<Box<dyn RegretProvider>>) -> RegretSharder {
        let regret_handlers = regret_providers.iter_mut()
            .map(|provider| provider.get_handler())
            .collect();

        RegretSharder {
            regret_handlers,
        }
    }
}

impl RegretHandler for RegretSharder {
    fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>> {
        let handler_index = infoset_hash as usize % self.regret_handlers.len();
        let handler = &self.regret_handlers[handler_index];
        handler.get_regret(player, infoset_hash)
    }

    fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>> {
        let handler_index = infoset_hash as usize % self.regret_handlers.len();
        let handler = &self.regret_handlers[handler_index];
        handler.send_delta(player, infoset_hash, regret_delta, iteration)
    }

}