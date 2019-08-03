use std::collections::HashMap;
use std::sync::mpsc;

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


pub trait RegretProvider {
    fn get_requester(&mut self) -> (mpsc::Sender<Request>, usize);
    fn get_receiver(&mut self, handler: usize) -> mpsc::Receiver<RegretResponse>;
    fn run(&mut self);
}

pub struct HashRegretProvider {
    //we keep
    request_receiver: mpsc::Receiver<Request>,
    //copy goes out to each agent
    request_sender: mpsc::Sender<Request>,

    //we keep, agents have receiver
    response_senders: Vec<Option<mpsc::Sender<RegretResponse>>>,

    p1_regrets: HashMap<u64, Vec<f64>>,
    p2_regrets: HashMap<u64, Vec<f64>>,
}

impl HashRegretProvider {
    pub fn new() -> HashRegretProvider {
        let (request_sender, request_receiver) = mpsc::channel();

        HashRegretProvider {
            request_sender,
            request_receiver,

            response_senders: vec![],

            p1_regrets: HashMap::new(),
            p2_regrets: HashMap::new(),
        }
    }

    fn handle_regret_request(&self, request: RegretRequest) {
        let regrets = match request.player {
            Player::P1 => &self.p1_regrets,
            Player::P2 => &self.p2_regrets,
        };
        let regret = regrets[&request.infoset_hash].clone();
        if let Some(Some(sender)) = self.response_senders.get(request.handler) {
            sender.send(RegretResponse {
                regret
            }).unwrap_or_else(|_| panic!("failed to send regret to for handler {}", request.handler));
        } else {
            panic!("failed to find regret sender for handler {}", request.handler);
        }
    }

    fn handle_regret_delta(&mut self, delta: RegretDelta) {
        let regrets = match delta.player {
            Player::P1 => &mut self.p1_regrets,
            Player::P2 => &mut self.p2_regrets,
        };
        let regret = regrets.entry(delta.infoset_hash)
            .or_insert(vec![0.0; delta.regret_delta.len()]);

        for (r, d) in regret.iter_mut().zip(delta.regret_delta.iter()) {
            *r += d
        }
    }
}

impl RegretProvider for HashRegretProvider {

    fn get_requester(&mut self) -> (mpsc::Sender<Request>, usize) {
        let sender = self.request_sender.clone();
        let handler = self.response_senders.len() + 1;
        self.response_senders.push(None);
        (sender, handler)
    }

    fn get_receiver(&mut self, handler: usize) -> mpsc::Receiver<RegretResponse> {
        let (sender, receiver) = mpsc::channel();
        if handler < self.response_senders.len() && self.response_senders[handler].is_none() {
            self.response_senders[handler] = Some(sender);
        } else {
            panic!("handler already taken");
        }
        receiver
    }

    fn run(&mut self) {
        let request = self.request_receiver.recv().expect("failed to receive request");

        match request {
            Request::Regret(request) => {
                self.handle_regret_request(request);
            },
            Request::Delta(delta) => {
                self.handle_regret_delta(delta);
            },
        };
    }


}