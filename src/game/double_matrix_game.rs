use std::fmt::{Display, Formatter};

use crate::game;
use crate::game::{Player};
use crate::game::matrix_game::{Move, MatrixGame};

#[derive(Debug, Clone)]
pub struct DoubleMatrixGame {
    games: (MatrixGame, MatrixGame),
    game1_moves: (Option<usize>, Option<usize>),
}

fn transpose<T: Copy>(size: usize, mut matrix: Vec<T>) -> Vec<T>
{
    for r in 0..size {
        for c in r..size {
            let a = matrix[r * size + c];
            matrix[r * size + c] = matrix[c * size + r];
            matrix[c * size + r] = a;
        }
    }

    matrix
}

impl DoubleMatrixGame {
    pub fn new(num_moves: usize, matrix: Vec<f32>) -> DoubleMatrixGame {
        if matrix.len() != num_moves * num_moves {
            panic!("Illegal matrix game");
        }
        DoubleMatrixGame {
            games: (
                MatrixGame::new(num_moves, matrix.clone()),
                MatrixGame::new(num_moves, transpose(num_moves, matrix)),
            ),
            game1_moves: (None, None),
        }
    }

    pub fn new_rock_paper_scissors() -> DoubleMatrixGame {
        DoubleMatrixGame {
            games: (
                MatrixGame::new_rock_paper_scissors(),
                MatrixGame::new_rock_paper_scissors(),
            ),
            game1_moves: (None, None),
        }
    }
}

impl game::Game for DoubleMatrixGame {
    type Action = Move;

    fn get_turn(&self) -> (Player, Vec<Move>) {
        if self.games.0.get_reward().is_none() {
            self.games.0.get_turn()
        } else {
            self.games.1.get_turn()
        }
    }

    fn take_turn(&mut self, player: Player, action: &Move) {
        if self.games.0.get_reward().is_none() {
            self.games.0.take_turn(player, action);
            *player.lens_mut(&mut self.game1_moves) = Some(*action);
        } else {
            self.games.1.take_turn(player, action);
        }
    }

    fn get_reward(&self) -> Option<f32> {
        match (self.games.0.get_reward(), self.games.1.get_reward()) {
            (Some(r1), Some(r2)) => Some((r1 + r2) / 2.0),
            _ => None,
        }
    }

    fn get_infoset(&self, player: Player) -> game::Infoset {
        //only state is whether we've made the first move and what that first move was
        game::Infoset::new(*player.lens(&self.game1_moves))
    }
}

impl Display for DoubleMatrixGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Matrix 1:")?;
        writeln!(f, "{}", self.games.0)?;
        writeln!(f, "Matrix 2:")?;
        writeln!(f, "{}", self.games.1)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn transpose_test() {
        let m = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];

        let m_t = transpose(3, m.clone());

        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(m[r* 3 + c], m_t[c * 3 + r]);
            }
        }
    }
}