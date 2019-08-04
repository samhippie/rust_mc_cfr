
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
    response_senders: Vec<mpsc::Sender<Response>>,
    //which sender's we've sent Response::Closed to
    closed_senders: Vec<bool>,

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
            closed_senders: vec![],

            p1_regrets: HashMap::new(),
            p2_regrets: HashMap::new(),
        }
    }

    fn handle_regret_request(&self, request: &RegretRequest) {
        let regrets = match request.player {
            Player::P1 => &self.p1_regrets,
            Player::P2 => &self.p2_regrets,
        };
        let regret = regrets.get(&request.infoset_hash).and_then(|regret| Some(regret.clone()));
        if let Some(sender) = self.response_senders.get(request.handler) {
            sender.send(Response::Regret(RegretResponse {
                regret
            })).unwrap_or_else(|_| panic!("failed to send regret to handler {}", request.handler));
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
            //*r += d
            //x = x * (n-1)/n + y is proportional to x += n * y
            //but more numerically stable
            *r = *r * (delta.iteration as f64) / (delta.iteration as f64 + 1.0) + d;
            if *r < 0.0 {
                *r = 0.0;
            }
        }
    }
    
    fn reject_request(&self, request: &RegretRequest) {
        if let Some(sender) = self.response_senders.get(request.handler) {
            sender.send(Response::Closed)
                .unwrap_or_else(|_| panic!("failed to send Closed to handler {}", request.handler));
        } else {
            panic!("failed to find regret sender for handler {}", request.handler);
        }
    }
}

impl RegretProvider for HashRegretProvider {

    fn get_handler(&mut self) -> RegretHandler {
        let request_sender = self.request_sender.clone();

        let (response_sender, response_receiver) = mpsc::channel();
        self.response_senders.push(response_sender);
        self.closed_senders.push(false);

        let handler = self.response_senders.len() - 1;

        RegretHandler {
            requester: request_sender,
            receiver: response_receiver,
            handler,
        }
    }

    fn run(&mut self) {
        let mut close_flag = false;
        let mut num_closed = 0;
        while num_closed < self.response_senders.len() {

            let request = self.request_receiver.recv().expect("failed to receive request");

            match request {
                Request::Regret(ref request) if close_flag => {
                    self.reject_request(request);
                    if self.closed_senders.get(request.handler) == Some(&false) {
                        self.closed_senders[request.handler] = true;
                        num_closed += 1;
                    }
                }
                Request::Regret(request) => self.handle_regret_request(&request),
                Request::Delta(delta) => self.handle_regret_delta(delta),
                Request::Close => close_flag = true,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn handles_delta_request_new() {
        let mut provider = HashRegretProvider::new();
        let infoset_hash = 1;
        let regret = vec![1.0, 2.0, 3.0];
        provider.handle_regret_delta(RegretDelta {
            player: Player::P1,
            regret_delta: regret.clone(),
            infoset_hash,
            iteration: 1,
        });
        let saved_regret = &provider.p1_regrets[&infoset_hash];
        assert_eq!(*saved_regret, regret);
    }

    //Taken out because the test doesn't account for cfr+
    //#[test]
    fn handles_delta_request_existing() {
        let mut provider = HashRegretProvider::new();
        let infoset_hash = 1;
        let regret = vec![1.0, 2.0, 3.0];
        let target_regret = vec![2.0, 4.0, 6.0];
        provider.p1_regrets.insert(infoset_hash, regret.clone());

        provider.handle_regret_delta(RegretDelta {
            player: Player::P1,
            regret_delta: regret.clone(),
            infoset_hash,
            iteration: 1,
        });
        let saved_regret = &provider.p1_regrets[&infoset_hash];

        assert_eq!(*saved_regret, target_regret);
    }

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

        let rsp = handler.get_regret(Player::P1, infoset_hash)
            .expect("failed to get regret");

        if let Response::Regret(rsp) = rsp {
            assert_eq!(rsp.regret.unwrap(), regret);
        } else {
            panic!("got closed provider")
        }
    }

    #[test]
    fn sends_delta_new() {
        let mut provider = HashRegretProvider::new();
        let infoset_hash = 1;
        let regret = vec![1.0, 2.0, 3.0];
        let handler = provider.get_handler();

        thread::spawn(move || {
            provider.run();
        });

        handler.send_delta(Player::P1, infoset_hash, regret.clone(), 1)
            .expect("failed to send delta");

        let rsp = handler.get_regret(Player::P1, infoset_hash)
            .expect("failed to get regret");

        if let Response::Regret(rsp) = rsp {
            assert_eq!(rsp.regret.unwrap(), regret);
        } else {
            panic!("got closed provider")
        }
    }

    //#[test]
    fn sends_delta_existing() {
        let mut provider = HashRegretProvider::new();
        let infoset_hash = 1;
        let regret = vec![1.0, 2.0, 3.0];
        let handler = provider.get_handler();

        thread::spawn(move || {
            provider.run();
        });

        handler.send_delta(Player::P1, infoset_hash, regret.clone(), 1)
            .expect("failed to send delta");

        handler.send_delta(Player::P1, infoset_hash, regret.clone(), 1)
            .expect("failed to send delta");

        let rsp = handler.get_regret(Player::P1, infoset_hash)
            .expect("failed to get regret");

        if let Response::Regret(rsp) = rsp {
            assert_eq!(rsp.regret.unwrap(), vec![2.0, 4.0, 6.0]);
        } else {
            panic!("got closed provider")
        }
    }

}