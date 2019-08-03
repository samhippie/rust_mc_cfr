
use std::collections::HashMap;
use std::sync::mpsc;
use crate::game::*;
use crate::regret::regret_provider::*;

pub struct HashRegretProvider {
    //we keep
    request_receiver: mpsc::Receiver<Request>,
    //copy goes out to each agent
    request_sender: mpsc::Sender<Request>,

    //we keep, agents have receiver
    response_senders: Vec<mpsc::Sender<RegretResponse>>,

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
        if let Some(sender) = self.response_senders.get(request.handler) {
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

    fn get_handler(&mut self) -> RegretHandler {
        let request_sender = self.request_sender.clone();

        let (response_sender, response_receiver) = mpsc::channel();
        self.response_senders.push(response_sender);

        let handler = self.response_senders.len();

        RegretHandler {
            requester: request_sender,
            receiver: response_receiver,
            handler,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game;
    use std::thread;

    #[test]
    fn gets_regret() {
        let mut provider = HashRegretProvider::new();
        let infoset_hash = 1;
        let regret = vec![1.0, 2.0, 3.0];
        provider.p1_regrets.insert(1, regret.clone());

        let handler = provider.get_handler();

        thread::spawn(move || {
            provider.run();
        });

        handler.requester.send(Request::Regret(RegretRequest {
            player: game::Player::P1,
            infoset_hash: infoset_hash,
            handler: handler.handler,
        })).expect("failed to send request");
        let rsp = handler.receiver.recv().expect("failed to receive response");
        assert_eq!(regret, rsp.regret);
    }
}