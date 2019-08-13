use std::error;

use crate::regret::regret_provider::{Response, Request, RegretHandler, RegretRequest, RegretDelta};
use crate::game::{Player};

/// Regret handler for using channels to communicate with a provider
pub struct ChannelRegretHandler {
    pub requester: crossbeam_channel::Sender<Request>,
    pub receiver: crossbeam_channel::Receiver<Response>,
    pub handler: usize,
}

impl RegretHandler for ChannelRegretHandler {
    fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>> {
        self.requester.try_send(Request::Regret(RegretRequest {
            player, 
            infoset_hash,
            handler: self.handler,
        }))?;
        let rsp = self.receiver.recv()?;
        Ok(rsp)
    }

    fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>> {
        self.requester.try_send(Request::Delta(RegretDelta {
            player,
            infoset_hash,
            regret_delta,
            iteration,
        }))?;
        Ok(())
    }
}