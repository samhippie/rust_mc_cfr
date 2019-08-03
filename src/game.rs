use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Player {
    P1,
    P2,
}

pub struct Infoset {
    pub infoset: Vec<u64>,
    pub hash: u64,
}

impl Infoset {
    pub fn new(infoset: Vec<u64>) -> Infoset {
        let mut hasher = DefaultHasher::new();
        infoset.hash(&mut hasher);
        let hash = hasher.finish();

        Infoset {
            infoset,
            hash,
        }
    }
}

/// 2 player zero sum game
///
/// Game is over when get_reward returns Some(reward) for player 1
pub trait Game: fmt::Display {

    type Action: fmt::Display;

    /// Returns player to move and all legal actions
    fn get_turn(&self) -> (Player, Vec<Self::Action>);

    /// The given player does the given action for their turn
    /// # Panics
    /// This may panic if the player cannot move or the action is invalid
    fn take_turn(&mut self, player: Player, action: &Self::Action);

    /// Returns None if the game is not over
    /// 
    /// Otherwise returns the reward for Player 1
    fn get_reward(&self) -> Option<f32>;

    /// Returns a player's infoset as a vector of hashes
    /// 
    /// Earlier parts of the infoset should come first, so an early infoset
    /// is a prefix of a later infoset
    fn get_infoset(&self, player: Player) -> Infoset;
}