use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(PartialEq, Copy, Clone, Debug, Hash)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    /// Gets the opposite player
    pub fn other(&self) -> Player {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }

    /// Returns a mutable reference to either member of a 2-tuple
    /// 
    /// This allows you to use tuples for P1 and P2 instead of p1_val, p2_val everywhere
    pub fn lens_mut<'a, T>(&self, tuple: &'a mut (T,T)) -> &'a mut T {
        match self {
            Player::P1 => &mut tuple.0,
            Player::P2 => &mut tuple.1,
        }
    }
    /// Returns a reference to either member of a 2-tuple
    /// 
    /// This allows you to use tuples for P1 and P2 instead of p1_val, p2_val everywhere
    pub fn lens<'a, T>(&self, tuple: &'a (T,T)) -> &'a T {
        match self {
            Player::P1 => &tuple.0,
            Player::P2 => &tuple.1,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Player::P1 => "P1",
            Player::P2 => "P2",
        };
        write!(f, "{}", s)
    }
}

pub struct Infoset {
    pub hash: u64,
}

impl Infoset {
    pub fn new<T: Hash>(infoset: T) -> Infoset {
        let mut hasher = DefaultHasher::new();
        infoset.hash(&mut hasher);
        let hash = hasher.finish();

        Infoset {
            hash,
        }
    }
}

/// 2 player zero sum game
///
/// Game is over when get_reward returns Some(reward) for player 1
pub trait Game: fmt::Display {

    type Action: fmt::Display + fmt::Debug;

    /// Returns player to move and all legal actions
    fn get_turn(&self) -> (Player, Vec<Self::Action>);

    /// The given player does the given action for their turn
    /// # Panics
    /// This may panic if the player cannot move or the action is invalid
    fn take_turn(&mut self, player: Player, action: &Self::Action);

    /// Returns None if the game is not over
    /// 
    /// Otherwise returns the reward for Player 1
    fn get_reward(&self) -> Option<f64>;

    /// Returns a player's infoset as a vector of hashes
    /// 
    /// Earlier parts of the infoset should come first, so an early infoset
    /// is a prefix of a later infoset
    fn get_infoset(&self, player: Player) -> Infoset;
}