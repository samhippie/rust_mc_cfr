//TODO it would be nice to make the fact that an action is a usize an implementation detail
//and the same for board being an array of player options

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

    fn get_reward(&self) -> Option<f32> {
        
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
        let infoset = self.history.iter().map(|(p, action)| {
            let player_bit = match p {
                Player::P1 => 0,
                Player::P2 => 1,
            };
            //highest action is 8 = 2^3, so player is set in the next bit
            (action | (player_bit << 4)) as u64
        }).collect();
        game::Infoset::new(infoset)
    }
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "To move: {}\n", self.current_player)?;
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
        assert_eq!(game.get_infoset(Player::P1).infoset.len(), 5);
        assert_eq!(game.get_infoset(Player::P2).infoset.len(), 5);
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
        assert_eq!(game.get_infoset(Player::P1).infoset.len(), 9);
        assert_eq!(game.get_infoset(Player::P2).infoset.len(), 9);
    }
}