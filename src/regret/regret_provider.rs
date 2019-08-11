//use std::sync::mpsc;
use std::error;
use crossbeam_channel;

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

pub struct RegretHandler {
    //pub requester: mpsc::Sender<Request>,
    pub requester: crossbeam_channel::Sender<Request>,
    //pub receiver: mpsc::Receiver<Response>,
    pub receiver: crossbeam_channel::Receiver<Response>,
    pub handler: usize,
}

impl RegretHandler {
    pub fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>> {
        self.requester.try_send(Request::Regret(RegretRequest {
            player, 
            infoset_hash,
            handler: self.handler,
        }))?;
        let rsp = self.receiver.recv()?;
        Ok(rsp)
    }

    pub fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>> {
        self.requester.try_send(Request::Delta(RegretDelta {
            player,
            infoset_hash,
            regret_delta,
            iteration,
        }))?;
        Ok(())
    }
}

pub trait RegretProvider {
    fn get_handler(&mut self) -> RegretHandler;
    fn run(&mut self);
}