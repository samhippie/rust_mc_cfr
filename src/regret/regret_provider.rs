use std::error;

use crate::game::Player;

pub struct RegretResponse {
    pub regret: Option<Vec<f32>>,
}

pub struct RegretRequest {
    pub player: Player,
    pub infoset_hash: u64,
    pub handler: usize,
}

pub struct RegretDelta {
    pub player: Player,
    pub infoset_hash: u64,
    pub regret_delta: Vec<f32>,
    pub iteration: i32,
}

pub enum Request {
    ///A request to get the request for a particular state
    Regret(RegretRequest),
    ///A request to add regret
    Delta(RegretDelta),
    ///Closes the provider
    Close,
}

pub enum Response {
    Regret(RegretResponse),
    Closed,
}

/// Interface for getting/setting regret in a regret provider
pub trait RegretHandler {
    fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>>;
    fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>>;
}

/// Controls a source of regret, giving access through handlers
/// 
/// May run in a thread through run()
pub trait RegretProvider {
    type Handler: RegretHandler;
    fn get_handler(&mut self) -> Self::Handler;
    fn run(&mut self);
}