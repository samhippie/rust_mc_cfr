use std::fmt;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Player {
    P1,
    P2,
}

/// 2 player zero sum game
///
/// Game is over when get_reward returns Some(reward) for player 1
pub trait Game: fmt::Display {

    type Action: fmt::Display;

    /// Returns player to move and all legal actions
    fn get_turn(&self) -> (Player, Vec<Self::Action>);

    /// The given player does the given action for their turn
    fn take_turn(&mut self, player: Player, action: &Self::Action);

    /// Returns None if the game is not over
    /// 
    /// Otherwise returns the reward for Player 1
    fn get_reward(&self) -> Option<f32>;

}