use std::fmt::{Display, Formatter};

use crate::game;
use crate::game::{Player};

#[derive(Debug, Clone)]
pub struct MatrixGame {
    moves: (Option<Move>, Option<Move>),
    num_moves: usize,
    matrix: Vec<f32>,
}

pub type Move = usize;

impl MatrixGame {
    pub fn new(num_moves: usize, matrix: Vec<f32>) -> MatrixGame {
        if matrix.len() != num_moves * num_moves {
            panic!("Illegal matrix game");
        }
        MatrixGame {
            moves: (None, None),
            num_moves,
            matrix,
        }
    }

    pub fn new_rock_paper_scissors() -> MatrixGame {
        MatrixGame::new(3, 
            vec![
                0.0, -1.0, 1.0,
                1.0, 0.0, -1.0,
                -1.0, 1.0, 0.0,
            ]
        )
    }
}

impl game::Game for MatrixGame {
    type Action = Move;

    fn get_turn(&self) -> (Player, Vec<Move>) {
        let moves = (0 .. self.num_moves).collect();
        if self.moves.0.is_none() {
            (Player::P1, moves)
        } else {
            (Player::P2, moves)
        }
    }

    fn take_turn(&mut self, player: Player, action: &Move) {
        let player_move = player.lens_mut(&mut self.moves);
        if player_move.is_some() {
            panic!("Tried to issue second move for player");
        }
        *player_move = Some(*action);
    }

    fn get_reward(&self) -> Option<f32> {
        let p1_move = match self.moves.0 {
            Some(m) => m,
            None => return None,
        };
        let p2_move = match self.moves.1 {
            Some(m) => m,
            None => return None,
        };

        Some(self.matrix[self.num_moves * p1_move + p2_move])
    }

    fn get_infoset(&self, _player: Player) -> game::Infoset {
        //there is no visible state
        game::Infoset::new(0)
    }
}

impl Display for MatrixGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //nothing to see here
        writeln!(f, "Matrix:")?;
        for i in 0..self.num_moves {
            for j in 0..self.num_moves {
                write!(f, "{}\t", self.matrix[self.num_moves * i + j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    const ROCK: Move = 0;
    const PAPER: Move = 1;
    const SCISSORS: Move = 2;

    #[test]
    fn rps_p1_wins() {
        let mut game = MatrixGame::new_rock_paper_scissors();
        game.take_turn(Player::P1, &SCISSORS);
        game.take_turn(Player::P2, &PAPER);
        let reward = game.get_reward();
        assert_eq!(reward, Some(1.0));
    }

    #[test]
    fn rps_p2_wins() {
        let mut game = MatrixGame::new_rock_paper_scissors();
        game.take_turn(Player::P1, &SCISSORS);
        game.take_turn(Player::P2, &ROCK);
        let reward = game.get_reward();
        assert_eq!(reward, Some(-1.0));
    }

    #[test]
    fn rps_tie() {
        let mut game = MatrixGame::new_rock_paper_scissors();
        game.take_turn(Player::P1, &SCISSORS);
        game.take_turn(Player::P2, &SCISSORS);
        let reward = game.get_reward();
        assert_eq!(reward, Some(0.0));
    }

}