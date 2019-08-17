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
pub trait RegretHandler : Send {
    fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>>;
    fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>>;
}

///https://arxiv.org/pdf/1809.04040.pdf
#[derive(Clone)]
pub struct RegretConfig {
    alpha: f32,
    beta: f32,
    gamma: f32,

    is_strategy: bool,
}

impl Default for RegretConfig {
    fn default() -> Self {
        RegretConfig {
            alpha: 1.0,
            beta: 1.0,
            gamma: 1.0,
            is_strategy: false,
        }
    }
}

impl RegretConfig {
    pub fn apply_delta(&self, iteration: f32, regret: f32, delta: f32) -> f32 {
        let t = if self.is_strategy {
            iteration.powf(self.gamma)
        } else if regret < 0.0 {
            iteration.powf(self.beta)
        } else {
            iteration.powf(self.alpha)
        };
        regret * t / (t + 1.0) + delta
    }
}

/// Controls a source of regret, giving access through handlers
/// 
/// May run in a thread through run()
pub trait RegretProvider : Send {
    fn set_config(&mut self, config: &RegretConfig);
    fn get_handler(&mut self) -> Box<dyn RegretHandler>;
    fn run(&mut self);
}