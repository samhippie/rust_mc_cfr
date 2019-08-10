use std::error;
use std::sync::mpsc;

use crate::regret::regret_provider::{RegretProvider, RegretHandler, Response, Request};
use crate::regret::HashRegretProvider;
use crate::game::Player;

pub struct RegretSharder {
    regret_handlers: Vec<RegretHandler>,
}

//It'd be really nice if this just used RegretProvider instead of HashRegretProvider
impl RegretSharder {
    pub fn new(regret_providers: &mut Vec<HashRegretProvider>) -> RegretSharder {
        let regret_handlers = regret_providers.iter_mut()
            .map(|provider| provider.get_handler())
            .collect();

        RegretSharder {
            regret_handlers,
        }
    }

    pub fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<error::Error>> {
        let handler_index = infoset_hash as usize % self.regret_handlers.len();
        let handler = &self.regret_handlers[handler_index];
        handler.get_regret(player, infoset_hash)
    }

    pub fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), mpsc::SendError<Request>> {
        let handler_index = infoset_hash as usize % self.regret_handlers.len();
        let handler = &self.regret_handlers[handler_index];
        handler.send_delta(player, infoset_hash, regret_delta, iteration)
    }

}