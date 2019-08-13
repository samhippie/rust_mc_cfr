use std::error;

use crate::regret::regret_provider::{RegretProvider, RegretHandler, Response};
use crate::game::Player;

/// Combines several regret providers into a single regret handler
/// 
/// This is useful for using spreading regrets across multiple regret providers
/// on separate threads without the providers needing to coordinate
pub struct RegretSharder<R: RegretHandler> {
    regret_handlers: Vec<R>,
}

impl<T: RegretHandler> RegretSharder<T> {
    pub fn new<P: RegretProvider<Handler=T>>(regret_providers: &mut Vec<P>) -> RegretSharder<T> {
        let regret_handlers = regret_providers.iter_mut()
            .map(|provider| provider.get_handler())
            .collect();

        RegretSharder {
            regret_handlers,
        }
    }
}

impl<T: RegretHandler> RegretHandler for RegretSharder<T> {
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