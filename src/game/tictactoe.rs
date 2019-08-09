use std::fmt::{Display, Formatter};

use crate::game;
use crate::game::{Player};

#[derive(Debug, Clone)]
pub struct TicTacToe {
    current_player: Player,
    /// 3x3 grid as a 1d row-major array
    board: [Option<Player>; 9],
    /// Player actions in the order they were made
    history: Vec<(Player, usize)>
}

impl TicTacToe {
    pub fn new() -> TicTacToe {
        TicTacToe {
            current_player: Player::P1,
            board: [
                None, None, None, 
                None, None, None, 
                None, None, None
            ],
            history: vec![],
        }
    }
}

impl game::Game for TicTacToe {

    type Action = usize;

    fn get_turn(&self) -> (Player, Vec<usize>) {
        //empty spaces
        let spaces = self.board.iter().enumerate()
            .filter_map(|(ind, space)| {
                match space {
                    None => Some(ind as usize),
                    _ => None,
                }
            })
            .collect();
        
        (self.current_player, spaces)
    }

    fn take_turn(&mut self, player: Player, action: &usize) {
        if player != self.current_player {
            panic!("Given player doesn't match saved player");
        }

        if self.board[*action].is_some() {
            panic!("Tried to overwrite non-empty space");
        }
        
        self.board[*action] = Some(player);
        self.current_player = match player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        };
        self.history.push((player, *action));
    }

    fn get_reward(&self) -> Option<f64> {
        
        match check_rows(&self)
            .or_else(|| check_cols(&self))
            .or_else(|| check_diagonals(&self)) 
        {
            Some(Player::P1) => Some(1.0),
            Some(Player::P2) => Some(-1.0),
            None => {
                //empty space means game isn't over
                if self.board.iter().any(|space| space.is_none()) {
                    None
                } else {
                    //no empty space and no winner means tie
                    Some(0.0)
                }
            }
        }
    }

    fn get_infoset(&self, _player: Player) -> game::Infoset {
        game::Infoset::new(self.history.clone())
    }
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "To move: {}\n", space_to_string(Some(self.current_player)))?;
        for row in 0..3 {
            for col in 0..3 {
                write!(f, "|{}", space_to_string(self.board[3 * row + col]))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}



pub fn space_to_string(space: Option<Player>) -> String {
    match space {
        Some(Player::P1) => String::from("X"),
        Some(Player::P2) => String::from("O"),
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
            history: vec![],
            board: board,
            current_player: Player::P2,
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
            history: vec![],
            board: board,
            current_player: Player::P2,
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
            history: vec![],
            board: board,
            current_player: Player::P2,
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
            history: vec![],
            board: board,
            current_player: Player::P1,
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
            history: vec![],
            board: board,
            current_player: Player::P1,
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
            history: vec![],
            board: board,
            current_player: Player::P1,
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
        assert_eq!(game.history.len(), 5);
        assert_eq!(game.get_infoset(Player::P1).hash,
            game.get_infoset(Player::P2).hash);
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
        assert_eq!(game.history.len(), 9);
        assert_eq!(game.get_infoset(Player::P1).hash,
            game.get_infoset(Player::P2).hash);
    }
}