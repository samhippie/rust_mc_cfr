//TODO it would be nice to make the fact that an action is a u32 an implementation detail
//and the same for board being an array of player options

use std::fmt::{Display, Formatter};

use crate::game;
use crate::game::{Player};

#[derive(Debug)]
pub struct TicTacToe {
    turn: Player,
    board: [Option<Player>; 9],
}

impl TicTacToe {
    pub fn new() -> TicTacToe {
        TicTacToe {
            turn: Player::P1,
            board: [
                None, None, None, 
                None, None, None, 
                None, None, None
            ],
        }
    }
}

impl game::Game for TicTacToe {

    type Action = u32;

    fn get_turn(&self) -> (Player, Vec<u32>) {
        let spaces = self.board.iter().enumerate()
            .filter_map(|(ind, space)| {
                match space {
                    None => Some(ind as u32),
                    _ => None,
                }
            })
            .collect();
        
        (self.turn, spaces)
    }

    fn take_turn(&mut self, player: Player, action: &u32) {
        if player != self.turn {
            panic!("Given player doesn't match saved player");
        }

        if self.board[*action as usize].is_some() {
            panic!("Tried to overwrite non-empty space");
        }
        
        self.board[*action as usize] = Some(player);
        self.turn = match player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }

    fn get_reward(&self) -> Option<f32> {
        
        match check_rows(&self)
            .or_else(|| check_cols(&self))
            .or_else(|| check_diagonals(&self)) {
                Some(Player::P1) => Some(1.0),
                Some(Player::P2) => Some(-1.0),
                None => {
                    if self.board.iter().any(|space| space.is_none()) {
                        None
                    } else {
                        Some(0.0)
                    }
                }
            }
    }
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "To move: {}\n", space_to_string(Some(self.turn)))?;
        for row in 0..3 {
            for col in 0..3 {
                write!(f, "|{}", space_to_string(self.board[3 * row + col]))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Player::P1 => "X",
            Player::P2 => "O",
        };
        write!(f, "{}", s)
    }
}

pub fn space_to_string(space: Option<Player>) -> String {
    match space {
        Some(player) => player.to_string(),
        None => String::from("_"),
    }
}

fn check_rows(game: &TicTacToe) -> Option<Player> {
    for row in 0..3 {
        if check_three(&game.board, 3 * row, 3 * row + 1, 3 * row + 2) {
            return game.board[3 * row]
        }
    }
    None
}

fn check_cols(game: &TicTacToe) -> Option<Player> {
    for col in 0..3 {
        if check_three(&game.board, col, col + 3, col + 6) {
            return game.board[col]
        }
    }
    None
}

fn check_diagonals(game: &TicTacToe) -> Option<Player> {
    if check_three(&game.board, 0, 4, 8) {
        game.board[0]
    } else if check_three(&game.board, 2, 4, 6) {
        game.board[2]
    } else {
        None
    }
}

fn check_three(board: &[Option<Player>; 9], a: usize, b: usize, c: usize) -> bool {
    board[a] != None && board[a] == board[b] && board[b] == board[c]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    #[test]
    fn has_col() {
        let board = [
            Some(Player::P1), Some(Player::P2), None,
            Some(Player::P1), Some(Player::P1), None,
            Some(Player::P1), Some(Player::P2), Some(Player::P2),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P2,
        };
        assert_eq!(check_cols(&game), Some(Player::P1));
    }

    #[test]
    fn has_no_col() {
        let board = [
            Some(Player::P1), Some(Player::P2), None,
            Some(Player::P2), Some(Player::P1), None,
            Some(Player::P1), Some(Player::P2), Some(Player::P2),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P2,
        };
        assert_eq!(check_cols(&game), None);
    }

    #[test]
    fn has_row() {
        let board = [
            None, Some(Player::P2), None,
            Some(Player::P2), Some(Player::P2), Some(Player::P1),
            Some(Player::P1), Some(Player::P1), Some(Player::P1),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P2,
        };
        assert_eq!(check_rows(&game), Some(Player::P1));
    }

    #[test]
    fn has_no_row() {
        let board = [
            None, Some(Player::P2), None,
            Some(Player::P1), None, Some(Player::P1),
            Some(Player::P2), Some(Player::P2), Some(Player::P1),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P1,
        };
        assert_eq!(check_rows(&game), None);
    }

    #[test]
    fn has_diagonals() {
        let board = [
            None, Some(Player::P2), Some(Player::P2),
            Some(Player::P1), Some(Player::P2), Some(Player::P1),
            Some(Player::P2), Some(Player::P1), Some(Player::P1),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P1,
        };
        assert_eq!(check_diagonals(&game), Some(Player::P2));
    }
 
    #[test]
    fn has_no_diagonals() {
        let board = [
            None, Some(Player::P2), Some(Player::P2),
            Some(Player::P1), Some(Player::P1), Some(Player::P2),
            Some(Player::P2), Some(Player::P1), Some(Player::P1),
        ];
        let game = TicTacToe {
            board: board,
            turn: Player::P1,
        };
        assert_eq!(check_diagonals(&game), None);
    }

    #[test]
    fn plays_game() {
        let mut game = TicTacToe::new();
        game.take_turn(Player::P1, &0);
        game.take_turn(Player::P2, &1);
        game.take_turn(Player::P1, &4);
        game.take_turn(Player::P2, &3);
        game.take_turn(Player::P1, &8);
        assert_eq!(game.get_reward(), Some(1.0));
    }

    #[test]
    fn plays_game_tie() {
        let mut game = TicTacToe::new();
        game.take_turn(Player::P1, &0);
        game.take_turn(Player::P2, &4);
        game.take_turn(Player::P1, &1);
        game.take_turn(Player::P2, &2);
        game.take_turn(Player::P1, &6);
        game.take_turn(Player::P2, &3);
        game.take_turn(Player::P1, &5);
        game.take_turn(Player::P2, &7);
        game.take_turn(Player::P1, &8);
        assert_eq!(game.get_reward(), Some(0.0));
    }
}