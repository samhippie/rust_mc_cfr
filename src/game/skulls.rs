use std::fmt;
use rand::Rng;

use crate::game::{Game, Player, Infoset};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Card {
    Skull,
    Flower,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
    Stack { card: Card },
    Bid { amount : u8 },
    Pass,
    //flipping is automatic in 2 player games, so there's no need to encode
}

#[derive(Clone, PartialEq, Debug)]
enum GameState {
    Stack { player: Player },
    PreStack { player: Player },
    Bid { amount: u8, leader: Player, player: Player, has_passed: bool },
    End { winner: Player },
}


#[derive(Clone, Debug)]
enum HistoryEntry {
    PlayerAction(Player, Action),
    Flip(Player, Player, Card),
    LoseCard(Player, Card),
    GetPoint(Player),
}

#[derive(Clone, Debug)]
struct Hand {
    skulls: u8,
    flowers: u8,
}

#[derive(Clone, Debug)]
pub struct Skulls {
    hands: (Hand, Hand),
    game_state: GameState,
    stacks: (Vec<Card>, Vec<Card>),
    has_flipped: (bool, bool),
    history: Vec<HistoryEntry>,
}

impl Skulls {
    pub fn new() -> Skulls {
        let mut rng = rand::thread_rng();
        let player = if rng.gen::<bool>() {
            Player::P1
        } else {
            Player::P2
        };
        Skulls::manual_new(player)
    }

    pub fn manual_new(player: Player) -> Skulls {
        Skulls {
            hands: (Hand { skulls: 1, flowers: /*3*/2}, Hand { skulls: 1, flowers: /*3*/2}),
            game_state: GameState::PreStack { player },
            stacks: (vec![], vec![]),
            has_flipped: (false, false),
            history: vec![],
        }
    }
}

impl Game for Skulls {
    type Action = Action;

    fn get_turn(&self) -> (Player, Vec<Action>) {
        match self.game_state {
            GameState::PreStack { player } => (player, hand_to_stack_actions(player.lens(&self.hands))),
            GameState::Stack { player } => (player, [hand_to_stack_actions(player.lens(&self.hands)), board_to_bid_actions(0, &self.stacks)].concat()),
            //if the player is the leader, then just pass, as there is no point in out-bidding yourself
            GameState::Bid { amount, player, leader, .. } if leader != player => (player, board_to_bid_actions(amount, &self.stacks)),
            GameState::Bid { player, .. } => (player, vec![Action::Pass]),
            GameState::End { winner } => (winner, vec![]),
        }
    }

    fn take_turn(&mut self, player: Player, action: &Action) {
        self.history.push(HistoryEntry::PlayerAction(player, *action));

        let cur_player = player;
        let new_state = match (&self.game_state, action) {
            (GameState::PreStack { player }, Action::Stack { card }) if *player == cur_player => {
                play_card(*card, player.lens_mut(&mut self.stacks), player.lens_mut(&mut self.hands));

                if player.other().lens(&self.stacks).is_empty() {
                    GameState::PreStack { player: player.other() }
                } else {
                    GameState::Stack { player: player.other() }
                }
            },

            (GameState::Stack { player }, Action::Stack { card }) if *player == cur_player => {
                play_card(*card, player.lens_mut(&mut self.stacks), player.lens_mut(&mut self.hands));
                GameState::Stack { player: player.other() }
            },
            (GameState::Stack { player }, Action::Bid { amount }) if *player == cur_player => GameState::Bid { amount: *amount, leader: *player, player: player.other(), has_passed: false },

            //bidding is technically simultaneous, but for 2 player games bidding twice is basically the same as just bidding the higher amount
            //(unless you're just testing to see if the other person will bet, but then you have to get the timing exactly right and I've never seen that happen)
            //so I'm just going to enforce taking turns while bidding
            (GameState::Bid { player, .. }, Action::Bid { amount }) if *player == cur_player => GameState::Bid { amount: *amount, leader: *player, player: player.other(), has_passed: false},
            (GameState::Bid { amount, leader, player, has_passed: false, }, Action::Pass) if *player == cur_player => GameState::Bid { amount: *amount, leader: *leader, player: player.other(), has_passed: true},
            //end of round, both players passed
            (GameState::Bid { amount, leader, player, has_passed: true, .. }, Action::Pass) if *player == cur_player => {

                //restore hands based on stacks
                for p in [Player::P1, Player::P2].iter() {
                    for card in p.lens(&self.stacks) {
                        match card {
                            Card::Flower => p.lens_mut(&mut self.hands).flowers += 1,
                            Card::Skull => p.lens_mut(&mut self.hands).skulls += 1,
                        }
                    }
                }

                //get all cards to be flipped, leader first, then most recently played first
                let flipped_cards = leader.lens(&self.stacks).iter().rev()
                    .map(|c| { (leader, c) })
                    .chain(
                        leader.other().lens(&self.stacks).iter().rev()
                        .map(|c| { (leader, c) })
                    ).take(*amount as usize);
                
                //seach for skull
                let mut found_skull = false;
                for (player, card) in flipped_cards {
                    //normally we'd record all card flips
                    //but I'm trying out only recording skull flips to save memory
                    //because that lets us figure out the result of flipping sequences with minimal information
                    //actually, it might sufficient to make the game state unique, which means that our memory usage will be the same
                    //self.history.push(HistoryEntry::Flip(*leader, *player, *card));
                    if *card == Card::Skull {
                        self.history.push(HistoryEntry::Flip(*leader, *player, *card));
                        found_skull = true;
                        let mut hand = leader.lens_mut(&mut self.hands);
                        if *player == *leader {
                            //remove flowers then skulls
                            if hand.flowers > 0 {
                                hand.flowers -= 1;
                                self.history.push(HistoryEntry::LoseCard(*leader, Card::Flower));
                            } else {
                                hand.skulls -= 1;
                                self.history.push(HistoryEntry::LoseCard(*leader, Card::Skull));
                            }
                        } else {
                            //remove randomly
                            let num_cards = hand.flowers + hand.skulls;
                            let remove_index = rand::thread_rng().gen_range(0, num_cards);
                            if remove_index < hand.flowers {
                                hand.flowers -= 1;
                                self.history.push(HistoryEntry::LoseCard(*leader, Card::Flower));
                            } else {
                                hand.skulls -= 1;
                                self.history.push(HistoryEntry::LoseCard(*leader, Card::Skull));
                            }
                        }
                        break;
                    }
                }

                //reset stacks
                self.stacks.0.clear();
                self.stacks.1.clear();

                //new game state
                if found_skull {
                    let hand = leader.lens(&self.hands);
                    if hand.skulls == 0 && hand.flowers == 0 {
                        GameState::End { winner: leader.other() }
                    } else {
                        GameState::PreStack { player: *leader }
                    }
                } else if *leader.lens(&self.has_flipped) {
                        GameState::End { winner: *leader }
                } else {
                    *leader.lens_mut(&mut self.has_flipped) = true;
                    self.history.push(HistoryEntry::GetPoint(*leader));
                    GameState::PreStack { player: *leader }
                }
            },

            _ => panic!("Tried to take turn with illegal action"),
        };

        self.game_state = new_state;
    }

    fn get_reward(&self) -> Option<f32> {
        if let GameState::End { winner } = self.game_state {
            Some(*winner.lens(&(1.0, -1.0)))
        } else {
            None
        }
    }

    fn get_infoset(&self, player: Player) -> Infoset {
        let infoset : Vec<(u8, u8, u8, u8)> = self.history.iter().map(|entry| {
            match *entry {
                HistoryEntry::GetPoint(p) => (p, p, 0, 0),
                HistoryEntry::PlayerAction(p, action) => {
                    //player knows everything they did
                    if p == player {
                        match action {
                            Action::Bid { amount } => (p, p, 1, amount),
                            Action::Pass => (p, p, 2, 0),
                            //i'm reserving 0 for unknown
                            Action::Stack { card } => (p, p, 3, 1 + card as u8),
                        }
                    } else {
                        match action {
                            Action::Bid { amount } => (p, p, 1, amount),
                            Action::Pass => (p, p, 2, 0),
                            Action::Stack { .. } => (p, p, 3, 0),
                        }
                    }
                },
                HistoryEntry::Flip(flipper, target, card) => (flipper, target, 4, card as u8),
                //again, reserving 0 for unknown
                HistoryEntry::LoseCard(flipper, card) if flipper == player => (flipper, flipper, 5, 1 + card as u8),
                HistoryEntry::LoseCard(flipper, _) => (flipper, flipper, 5, 0),
            }
        })
        //change perspective so the player thinks they're P1
        .map(|(p1, p2, x, y)| (player.view(p1) as u8, player.view(p2) as u8, x, y))
        .collect();
        Infoset::new(infoset)
    }
}

fn hand_to_stack_actions(hand: &Hand) -> Vec<Action> {
    let mut actions = vec![];
    if hand.flowers > 0 {
        actions.push(Action::Stack { card: Card::Flower });
    }
    if hand.skulls > 0 {
        actions.push(Action::Stack { card: Card::Skull });
    }
    actions
}

fn board_to_bid_actions(current_bid: u8, stacks: &(Vec<Card>, Vec<Card>)) -> Vec<Action> {
    let (stack1, stack2) = stacks;
    let num_cards = (stack1.len() + stack2.len()) as u8;

    let maybe_pass = if current_bid == 0 {
        None
    } else {
        Some(Action::Pass)
    };
    let bids = ((current_bid+1) ..= num_cards).map(|amount| {
        Action::Bid { amount }
    });
    maybe_pass.into_iter().chain(bids).collect()
}

fn play_card(card: Card, stack: &mut Vec<Card>, hand: &mut Hand) {
    stack.push(card);
    if card == Card::Skull {
        hand.skulls -= 1;
    } else {
        hand.flowers -= 1;
    }

}

impl fmt::Display for Skulls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "hands {:?}", self.hands)?;
        writeln!(f, "stacks {:?}", self.stacks)?;
        writeln!(f, "state {:?}", self.game_state)?;
        writeln!(f, "has flipped {:?}", self.has_flipped)?;
        Ok(())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    fn wins_game_flipping(player: Player) -> Option<f32> {
        let mut game = Skulls::manual_new(player);

        //prestack
        game.take_turn(player, &Action::Stack { card: Card::Flower });
        game.take_turn(player.other(), &Action::Stack { card: Card::Flower });
        assert_eq!(game.game_state, GameState::Stack { player });

        //stack
        let (_, actions) = game.get_turn();
        assert!(actions.contains(&Action::Stack { card: Card::Flower }));
        assert!(actions.contains(&Action::Stack { card: Card::Skull }));
        for i in 1..=2 {
            assert!(actions.contains(&Action::Bid { amount: i }));
        }
        assert_eq!(actions.len(), 4);

        //bid
        game.take_turn(player, &Action::Bid { amount: 2 });
        let (p, actions) = game.get_turn();
        assert_eq!(p, player.other());
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Pass);
        game.take_turn(player.other(), &Action::Pass);

        let (p, actions) = game.get_turn();
        assert_eq!(p, player);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Pass);
        game.take_turn(player, &Action::Pass);

        //check result of bid
        assert_eq!(player.lens(&game.has_flipped), &true);
        assert_eq!(player.other().lens(&game.has_flipped), &false);

        //prestack
        game.take_turn(player, &Action::Stack { card: Card::Flower });
        game.take_turn(player.other(), &Action::Stack { card: Card::Flower });
        assert_eq!(game.game_state, GameState::Stack { player: player });

        //stack
        let (_, actions) = game.get_turn();
        assert!(actions.contains(&Action::Stack { card: Card::Flower }));
        assert!(actions.contains(&Action::Stack { card: Card::Skull }));
        for i in 1..=2 {
            assert!(actions.contains(&Action::Bid { amount: i }));
        }
        assert_eq!(actions.len(), 4);

        //bid
        game.take_turn(player, &Action::Bid { amount: 2 });
        let (p, actions) = game.get_turn();
        assert_eq!(p, player.other());
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Pass);
        game.take_turn(player.other(), &Action::Pass);

        let (p, actions) = game.get_turn();
        assert_eq!(p, player);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Pass);
        game.take_turn(player, &Action::Pass);

        game.get_reward()
    }

    //this is less assert-heavy, as the other function has a lot of calls to assert
    fn loses_game_self_elimination(player: Player) -> Option<f32> {
        let mut game = Skulls::manual_new(player);

        for i in 0..3 {
            //prestack
            game.take_turn(player, &Action::Stack { card: Card::Flower });
            game.take_turn(player.other(), &Action::Stack { card: Card::Flower });

            //stack flowers while we can
            while player.lens(&game.hands).flowers > 0 {
                game.take_turn(player, &Action::Stack { card: Card::Flower });
                game.take_turn(player.other(), &Action::Stack { card: Card::Flower });
            }
            //then play a skull
            game.take_turn(player, &Action::Stack { card: Card::Skull});
            game.take_turn(player.other(), &Action::Stack { card: Card::Skull });

            //bid
            game.take_turn(player, &Action::Bid { amount: 2 });
            game.take_turn(player.other(), &Action::Pass);
            game.take_turn(player, &Action::Pass);

            assert_eq!(player.lens(&game.hands).flowers, 3 - i - 1);
        }

        game.take_turn(player, &Action::Stack { card: Card::Skull });
        game.take_turn(player.other(), &Action::Stack { card: Card::Flower });
        game.take_turn(player, &Action::Bid { amount: 2 });
        game.take_turn(player.other(), &Action::Pass);
        game.take_turn(player, &Action::Pass);

        game.get_reward()
    }

    #[test]
    fn reward_flipping_p1() {
        let reward = wins_game_flipping(Player::P1);
        assert_eq!(Some(1.0), reward);
    }

    #[test]
    fn reward_flipping_p2() {
        let reward = wins_game_flipping(Player::P2);
        assert_eq!(Some(-1.0), reward);
    }

    #[test]
    fn reward_self_elimination_p1() {
        let reward = loses_game_self_elimination(Player::P1);
        assert_eq!(Some(-1.0), reward);
    }

    #[test]
    fn reward_self_elimination_p2() {
        let reward = loses_game_self_elimination(Player::P2);
        assert_eq!(Some(1.0), reward);
    }

    #[test]
    fn extended_bidding() {
        for player in [Player::P1, Player::P2].into_iter() {
            let player = *player;
            let mut game = Skulls::manual_new(player);
            //prestack
            game.take_turn(player, &Action::Stack { card: Card::Skull });
            game.take_turn(player.other(), &Action::Stack { card: Card::Skull });

            //stack flowers while we can
            for _ in 0..3 {
                game.take_turn(player, &Action::Stack { card: Card::Flower });
                game.take_turn(player.other(), &Action::Stack { card: Card::Flower });
            }

            game.take_turn(player, &Action::Bid { amount: 1 });
            game.take_turn(player.other(), &Action::Bid { amount: 2 });
            game.take_turn(player, &Action::Bid { amount: 3 });
            game.take_turn(player.other(), &Action::Bid { amount: 4 });
            game.take_turn(player, &Action::Pass);
            game.take_turn(player.other(), &Action::Pass);

            //other player just barely flipped over their own skull
            assert_eq!(player.other().lens(&game.hands).flowers, 2);
        }
    }

}