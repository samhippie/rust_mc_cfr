//http://www.cs.cmu.edu/~ggordon/poker/

use rand::Rng;
use std::fmt::{Display, Formatter};

use crate::game::{Game, Player, Infoset};

const NUM_CARDS: u32 = 13;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Fold,
    Call,
    Bet,
}

//these comments assume the dealer is p2
//but it doesn't really matter
#[derive(Clone, Copy)]
enum PokerState {
    //just dealt, p1 can check or bet
    P1Deal,
    //p1 checked, p2 can check or bet
    P2Check,
    //p2 bet, p1 can call or fold
    P1Raise,

    //p1 bet on the first round, p2 can call or fold
    P2Bet,

    FoldEnd,
    ShowdownEnd,
}

fn state_to_actions(state: PokerState) -> Vec<Action> {
    match state {
        PokerState::P1Deal => vec![Action::Call, Action::Bet],
        PokerState::P2Check => vec![Action::Call, Action::Bet],
        PokerState::P1Raise => vec![Action::Fold, Action::Call],
        PokerState::P2Bet => vec![Action::Fold, Action::Call],
        _ => vec![],
    }
}

#[derive(Clone)]
pub struct OneCardPoker {
    dealer: Player,
    pot: (u32, u32),
    hands: (u32, u32),
    history: Vec<(Player, Action)>,
    state: PokerState,
    current_player: Player,
    current_actions: Vec<Action>,
}

impl OneCardPoker {
    pub fn new() -> OneCardPoker {
        let mut rng = rand::thread_rng();
        let hand1: u32 = rng.gen_range(0, NUM_CARDS);
        let mut hand2: u32 = rng.gen_range(0, NUM_CARDS);
        if hand2 >= hand1 {
            hand2 += 1;
        }

        let dealer = match rng.gen() {
            true => Player::P1,
            false => Player::P2,
        };

        OneCardPoker::manual_new((hand1, hand2), dealer)
    }

    pub fn manual_new(hands: (u32, u32), dealer: Player) -> OneCardPoker {
        OneCardPoker {
            dealer,
            hands: hands,
            pot: (1, 1),
            history: vec![],
            state: PokerState::P1Deal,
            current_player: dealer.other(),
            current_actions: state_to_actions(PokerState::P1Deal),
        }

    }
}

impl Game for OneCardPoker {
    type Action = Action;

    fn get_turn(&self) -> (Player, Vec<Action>) {
        (self.current_player, self.current_actions.clone())
    }

    fn take_turn(&mut self, player: Player, action: &Action) {
        if player != self.current_player {
            panic!("Tried to take turn with wrong player");
        }
        if !self.current_actions.contains(action) {
            panic!("Tried to take turn with illegal action");
        }

        let p2 = self.dealer;
        let p1 = p2.other();

        let (current_player, state) = match self.state {
            PokerState::P1Deal if *action == Action::Call => (p2, PokerState::P2Check),
            PokerState::P1Deal if *action == Action::Bet => {
                *p1.lens_mut(&mut self.pot) += 1;
                (p2, PokerState::P2Bet)
            },
            PokerState::P2Check if *action == Action::Call => (p1, PokerState::ShowdownEnd),
            PokerState::P2Check if *action == Action::Bet => {
                *p2.lens_mut(&mut self.pot) += 1;
                (p1, PokerState::P1Raise)
            }
            PokerState::P2Bet if *action == Action::Fold => (p1, PokerState::FoldEnd),
            PokerState::P2Bet if *action == Action::Call => {
                *p2.lens_mut(&mut self.pot) += 1;
                (p1, PokerState::ShowdownEnd)
            },
            PokerState::P1Raise if *action == Action::Fold => (p2, PokerState::FoldEnd),
            PokerState::P1Raise if *action == Action::Call => {
                *p1.lens_mut(&mut self.pot) += 1;
                (p2, PokerState::ShowdownEnd)
            },
            _ => panic!("Illegal action for the current state"),
        };
        self.current_player = current_player;
        self.state = state;
        self.current_actions = state_to_actions(state);
        self.history.push((player, *action));
    }

    fn get_reward(&self) -> Option<f64> {
        //the reward is the other player's contribution to the pot
        //divide by 2 to put the rewards between -1 and 1
        match self.state {
            PokerState::FoldEnd if self.current_player == Player::P1 => Some(self.pot.1 as f64 / 2.0),
            PokerState::FoldEnd if self.current_player == Player::P2 => Some(-1.0 * self.pot.0 as f64 / 2.0),
            PokerState::ShowdownEnd if self.hands.0 > self.hands.1 => Some(self.pot.1 as f64 / 2.0),
            PokerState::ShowdownEnd => Some(-1.0 * self.pot.0 as f64 / 2.0),
            _ => None,
        }
    }

    fn get_infoset(&self, player: Player) -> Infoset {
        //the player's hand is only known to them
        let mut infoset = vec![*player.lens(&self.hands) as u64];
        //the bet history is public
        for (p, a) in self.history.iter() {
            let a = *a as u64;
            let p = *p as u64;
            infoset.push((a << 1) | p);
        }
        Infoset::new(infoset)
    }
}

impl Display for OneCardPoker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "P1 hand: {}, P2 hand: {}; ", self.hands.0, self.hands.1)?;
        for (player, action) in self.history.iter() {
            write!(f, "{} does {}; ", player, action)?;
        }
        Ok(())
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    #[test]
    fn early_showdown() {
        let mut game = OneCardPoker::manual_new((3,5), Player::P1);

        game.take_turn(Player::P2, &Action::Call);

        let infoset = game.get_infoset(Player::P2);
        assert_eq!(infoset.infoset.len(), 2);
        let infoset = game.get_infoset(Player::P1);
        assert_eq!(infoset.infoset.len(), 2);

        game.take_turn(Player::P1, &Action::Call);
        let reward = game.get_reward();
        assert_eq!(reward, Some(-0.5));
    }

    #[test]
    fn early_p2_fold() {
        let mut game = OneCardPoker::manual_new((3,5), Player::P2);

        game.take_turn(Player::P1, &Action::Bet);

        let infoset = game.get_infoset(Player::P2);
        assert_eq!(infoset.infoset.len(), 2);
        let infoset = game.get_infoset(Player::P1);
        assert_eq!(infoset.infoset.len(), 2);

        game.take_turn(Player::P2, &Action::Fold);
        let reward = game.get_reward();
        assert_eq!(reward, Some(0.5));
    }

    #[test]
    fn late_p2_fold() {
        let mut game = OneCardPoker::manual_new((3,5), Player::P1);

        game.take_turn(Player::P2, &Action::Call);
        game.take_turn(Player::P1, &Action::Bet);

        let infoset = game.get_infoset(Player::P2);
        assert_eq!(infoset.infoset.len(), 3);
        let infoset = game.get_infoset(Player::P1);
        assert_eq!(infoset.infoset.len(), 3);

        game.take_turn(Player::P2, &Action::Fold);
        let reward = game.get_reward();
        assert_eq!(reward, Some(0.5));
    }

    #[test]
    fn late_p1_fold() {
        let mut game = OneCardPoker::manual_new((3,5), Player::P2);

        game.take_turn(Player::P1, &Action::Call);
        game.take_turn(Player::P2, &Action::Bet);

        let infoset = game.get_infoset(Player::P1);
        assert_eq!(infoset.infoset.len(), 3);
        let infoset = game.get_infoset(Player::P2);
        assert_eq!(infoset.infoset.len(), 3);

        game.take_turn(Player::P1, &Action::Fold);
        let reward = game.get_reward();
        assert_eq!(reward, Some(-0.5));
    }

    #[test]
    fn late_showdown() {
        let mut game = OneCardPoker::manual_new((5,3), Player::P1);

        game.take_turn(Player::P2, &Action::Call);
        game.take_turn(Player::P1, &Action::Bet);

        let infoset = game.get_infoset(Player::P2);
        assert_eq!(infoset.infoset.len(), 3);
        let infoset = game.get_infoset(Player::P1);
        assert_eq!(infoset.infoset.len(), 3);

        game.take_turn(Player::P2, &Action::Call);
        let reward = game.get_reward();
        assert_eq!(reward, Some(1.0));
    }
}