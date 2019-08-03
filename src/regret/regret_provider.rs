use std::sync::mpsc;
use std::error;

use crate::game::Player;

pub struct RegretResponse {
    pub regret: Vec<f64>
}

pub struct RegretRequest {
    pub player: Player,
    pub infoset_hash: u64,
    pub handler: usize,
}

pub struct RegretDelta {
    pub player: Player,
    pub infoset_hash: u64,
    pub regret_delta: Vec<f64>,
}

pub enum Request {
    Regret(RegretRequest),
    Delta(RegretDelta),
}

pub struct RegretHandler {
    pub requester: mpsc::Sender<Request>,
    pub receiver: mpsc::Receiver<RegretResponse>,
    pub handler: usize,
}

impl RegretHandler {
    pub fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<RegretResponse, Box<error::Error>> {
        self.requester.send(Request::Regret(RegretRequest {
            player, 
            infoset_hash,
            handler: self.handler,
        }))?;
        let rsp = self.receiver.recv()?;
        Ok(rsp)
    }

    pub fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f64>) -> Result<(), mpsc::SendError<Request>> {
        self.requester.send(Request::Delta(RegretDelta {
            player,
            infoset_hash,
            regret_delta,
        }))
    }

    pub fn recv(&self) -> Result<RegretResponse, mpsc::RecvError> {
        self.receiver.recv()
    }
}

pub trait RegretProvider {
    fn get_handler(&mut self) -> RegretHandler;
    fn run(&mut self);
}