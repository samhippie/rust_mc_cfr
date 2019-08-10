use std::fmt::{Display, Formatter};

use crate::game;
use crate::game::{Player};

#[derive(Debug, Clone)]
pub struct RockPaperScissors {
    p1_move: Option<Move>,
    p2_move: Option<Move>,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Move {
    Rock,
    Paper,
    Scissors
}

impl RockPaperScissors {
    pub fn new() -> RockPaperScissors {
        RockPaperScissors {
            p1_move: None,
            p2_move: None,
        }
    }
}

impl game::Game for RockPaperScissors {
    type Action = Move;

    fn get_turn(&self) -> (Player, Vec<Move>) {
        let moves = vec![Move::Rock, Move::Paper, Move::Scissors];
        if self.p1_move.is_none() {
            (Player::P1, moves)
        } else {
            (Player::P2, moves)
        }
    }

    fn take_turn(&mut self, player: Player, action: &Move) {
        let player_move = match player {
            Player::P1 => &mut self.p1_move,
            Player::P2 => &mut self.p2_move,
        };
        if player_move.is_some() {
            panic!("Tried to issue second move for player");
        }
        *player_move = Some(*action);
    }

    fn get_reward(&self) -> Option<f32> {
        let p1_move = match self.p1_move {
            Some(m) => m,
            None => return None,
        };
        let p2_move = match self.p2_move {
            Some(m) => m,
            None => return None,
        };

        //there's probably some clever way to do this
        Some(if p1_move == p2_move {
            0.0
        } else if p1_move == Move::Paper && p2_move == Move::Rock {
            1.0
        } else if p1_move == Move::Paper && p2_move == Move::Scissors {
            -1.0
        } else if p1_move == Move::Rock && p2_move == Move::Scissors {
            1.0
        } else if p1_move == Move::Rock && p2_move == Move::Paper {
            -1.0
        } else if p1_move == Move::Scissors && p2_move == Move::Paper {
            1.0
        } else if p1_move == Move::Scissors && p2_move == Move::Rock {
            -1.0
        } else {
            0.0//should never happen
        })
    }

    fn get_infoset(&self, _player: Player) -> game::Infoset {
        //there is no visible state
        game::Infoset::new(0)
    }
}

impl Display for RockPaperScissors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //nothing to see here
        write!(f, "RPS")?;
        Ok(())
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //why do we not have proper emojis for this in 2019?
        let s = match self {
            Move::Rock => "Rock 🗿",
            Move::Paper => "Paper 📝",
            Move::Scissors => "Scissors ✂️",
        };
        write!(f, "{}", s)?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    #[test]
    fn rps_p1_wins() {
        let mut game = RockPaperScissors::new();
        game.take_turn(Player::P1, &Move::Scissors);
        game.take_turn(Player::P2, &Move::Paper);
        let reward = game.get_reward();
        assert_eq!(reward, Some(1.0));
    }

    #[test]
    fn rps_p2_wins() {
        let mut game = RockPaperScissors::new();
        game.take_turn(Player::P1, &Move::Scissors);
        game.take_turn(Player::P2, &Move::Rock);
        let reward = game.get_reward();
        assert_eq!(reward, Some(-1.0));
    }

    #[test]
    fn rps_tie() {
        let mut game = RockPaperScissors::new();
        game.take_turn(Player::P1, &Move::Scissors);
        game.take_turn(Player::P2, &Move::Scissors);
        let reward = game.get_reward();
        assert_eq!(reward, Some(0.0));
    }

}