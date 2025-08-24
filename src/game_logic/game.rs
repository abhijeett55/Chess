use super::{bot::Bot, coord::Coord, game_board::GameBoard, opponent::Opponent, ui::UI};
use crate::{
    pieces::{PieceColor, PieceMove, PieceType},
    utils::get_int_from_char,
};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum GameState {
    Checkmate,
    Draw,
    Playing,
    Promotion,
}

pub struct Game {
    pub game_board: GameBoard,
    pub ui: UI,
    pub bot: Option<Bot>,
    pub opponent: Option<Opponent>,
    pub player_turn: PieceColor,
    pub game_state: GameState,
}

impl Clone for Game {
    fn clone(&self) -> Self {
        let opponent_clone = self.opponent.as_ref().map(|p| Opponent {
            stream: p.stream.as_ref().and_then(|s| s.try_clone().ok()),
            opponent_will_move: p.opponent_will_move,
            color: p.color,
            game_started: p.game_started,
        });

        Game {
            game_board: self.game_board.clone(),
            ui: self.ui.clone(),
            bot: self.bot.clone(),
            opponent: opponent_clone,
            player_turn: self.player_turn,
            game_state: self.game_state,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            game_board: GameBoard::default(),
            ui: UI::default(),
            bot: None,
            opponent: None,
            player_turn: PieceColor::White,
            game_state: GameState::Playing,
        }
    }
}

impl Game {
    pub fn new(game_board: GameBoard, player_turn: PieceColor) -> Self {
        Self {
            game_board,
            ui: UI::default(),
            bot: None,
            opponent: None,
            player_turn,
            game_state: GameState::Playing,
        }
    }

    pub fn set_board(&mut self, game_board: GameBoard) {
        self.game_board = game_board;
    }

    pub fn set_player_turn(&mut self, player_turn: PieceColor) {
        self.player_turn = player_turn;
    }

    pub fn switch_player_turn(&mut self) {
        match self.player_turn {
            PieceColor::White => self.player_turn = PieceColor::Black,
            PieceColor::Black => self.player_turn = PieceColor::White,
        }
    }

    pub fn handle_cell_click(&mut self) {
        if self.game_state == GameState::Promotion {
            self.handle_promotion();
        } else if !(self.game_state == GameState::Checkmate)
            && !(self.game_state == GameState::Draw)
        {
            if self.ui.is_cell_selected() {
                self.already_selected_cell_action();
            } else {
                self.select_cell()
            }
        }
        self.update_game_state();
    }

    fn update_game_state(&mut self) {
        if self.game_board.is_checkmate(self.player_turn) {
            self.game_state = GameState::Checkmate;
        } else if self.game_board.is_draw(self.player_turn) {
            self.game_state = GameState::Draw;
        } else if self.game_board.is_latest_move_promotion() {
            self.game_state = GameState::Promotion;
        }
    }

    pub fn handle_promotion(&mut self) {
        self.promote_piece();

        if self.opponent.is_some() {
            self.handle_multiplayer_promotion();
        }

        if self.bot.is_some() {
            self.execute_bot_move();
        }
    }

    pub fn already_selected_cell_action(&mut self) {
        if self.ui.cursor_coordinates.is_valid() {
            let selected_coords_usize = &self.ui.selected_coordinates.clone();
            let cursor_coords_usize = &self.ui.cursor_coordinates.clone();
            self.execute_move(selected_coords_usize, cursor_coords_usize);
            self.ui.unselect_cell();
            self.switch_player_turn();

            if self.game_board.is_draw(self.player_turn) {
                self.game_state = GameState::Draw;
            }

            if (self.bot.is_none() || (self.bot.as_ref().is_some_and(|bot| bot.is_bot_starting)))
                && (self.opponent.is_none())
                && (!self.game_board.is_latest_move_promotion()
                || self.game_board.is_draw(self.player_turn)
                || self.game_board.is_checkmate(self.player_turn))
            {
                self.game_board.flip_the_board();
            }

            if self.bot.is_some() {
                if self.game_board.is_latest_move_promotion() {
                    self.game_state = GameState::Promotion;
                }

                if !(self.game_state == GameState::Promotion) {
                    if self.game_board.is_checkmate(self.player_turn) {
                        self.game_state = GameState::Checkmate;
                    }

                    if self.game_board.is_draw(self.player_turn) {
                        self.game_state = GameState::Draw;
                    }

                    if !(self.game_state == GameState::Checkmate) {
                        if let Some(bot) = self.bot.as_mut() {
                            bot.bot_will_move = true;
                        }
                    }
                }
            }

            if self.opponent.is_some() {
                if self.game_board.is_latest_move_promotion() {
                    self.game_state = GameState::Promotion;
                } else {
                    if self.game_board.is_checkmate(self.player_turn) {
                        self.game_state = GameState::Checkmate;
                    }

                    if self.game_board.is_draw(self.player_turn) {
                        self.game_state = GameState::Draw;
                    }

                    if !(self.game_state == GameState::Checkmate) {
                        if let Some(opponent) = self.opponent.as_mut() {
                            opponent.opponent_will_move = true;
                        }
                    }
                    self.opponent
                        .as_mut()
                        .unwrap()
                        .send_move_to_server(self.game_board.move_history.last().unwrap(), None);
                }
            }
        }
    }

    pub fn select_cell(&mut self) {
        let authorized_positions = self
            .game_board
            .get_authorized_positions(self.player_turn, self.ui.cursor_coordinates);

        if authorized_positions.is_empty() {
            return;
        }
        if let Some(piece_color) = self.game_board.get_piece_color(&self.ui.cursor_coordinates) {
            let authorized_positions = self
                .game_board
                .get_authorized_positions(self.player_turn, self.ui.cursor_coordinates);

            if piece_color == self.player_turn {
                self.ui.selected_coordinates = self.ui.cursor_coordinates;
                self.ui.old_cursor_position = self.ui.cursor_coordinates;
                self.ui
                    .move_selected_piece_cursor(true, 1, authorized_positions);
            }
        }
    }

    pub fn execute_bot_move(&mut self) {
        let is_bot_starting = if let Some(bot) = self.bot.as_ref() {
            bot.is_bot_starting
        } else {
            return;
        };

        let fen_position = self
            .game_board
            .fen_position(is_bot_starting, self.player_turn);

        let bot_move = if let Some(bot) = self.bot.as_mut() {
            bot.get_move(&fen_position)
        } else {
            return;
        };

        let from_y = get_int_from_char(bot_move.chars().next());
        let from_x = get_int_from_char(bot_move.chars().nth(1));
        let to_y = get_int_from_char(bot_move.chars().nth(2));
        let to_x = get_int_from_char(bot_move.chars().nth(3));

        let mut promotion_piece: Option<PieceType> = None;
        if bot_move.chars().count() == 5 {
            promotion_piece = match bot_move.chars().nth(4) {
                Some('q') => Some(PieceType::Queen),
                Some('r') => Some(PieceType::Rook),
                Some('b') => Some(PieceType::Bishop),
                Some('n') => Some(PieceType::Knight),
                _ => None,
            };
        }

        self.execute_move(&Coord::new(from_y, from_x), &Coord::new(to_y, to_x));

        if promotion_piece.is_some() {
            self.game_board.board[to_y as usize][to_x as usize] =
                Some((promotion_piece.unwrap(), self.player_turn));
        }
        if is_bot_starting {
            self.game_board.flip_the_board();
        }
    }

    pub fn promote_piece(&mut self) {
        if let Some(last_move) = self.game_board.move_history.last() {
            let new_piece = match self.ui.promotion_cursor {
                0 => PieceType::Queen,
                1 => PieceType::Rook,
                2 => PieceType::Bishop,
                3 => PieceType::Knight,
                _ => unreachable!("Promotion cursor out of boundaries"),
            };

            let current_piece_color = self
                .game_board
                .get_piece_color(&Coord::new(last_move.to.row, last_move.to.col));
            if let Some(piece_color) = current_piece_color {
                self.game_board.board[last_move.to.row as usize][last_move.to.col as usize] =
                    Some((new_piece, piece_color));
            }

            let latest_move = self.game_board.move_history.last_mut().unwrap();
            latest_move.piece_type = new_piece;
            self.game_board.board_history.pop();
            self.game_board.board_history.push(self.game_board.board);
        }
        self.game_state = GameState::Playing;
        self.ui.promotion_cursor = 0;
        if !self.game_board.is_draw(self.player_turn)
            && !self.game_board.is_checkmate(self.player_turn)
            && self.opponent.is_none()
            && self.bot.is_none()
        {
            self.game_board.flip_the_board();
        }
    }

    pub fn execute_move(&mut self, from: &Coord, to: &Coord) {
        if !from.is_valid() || !to.is_valid() {
            return;
        }

        let piece_type_from = self.game_board.get_piece_type(from);
        let piece_type_to = self.game_board.get_piece_type(to);

        let Some(piece_type_from) = piece_type_from else {
            return;
        };

        self.game_board
            .increment_consecutive_non_pawn_or_capture(piece_type_from, piece_type_to);

        self.game_board
            .add_piece_to_taken_pieces(from, to, self.player_turn);

        if self.game_board.is_latest_move_en_passant(from, to) {
            let row_index = to.row as i32 + 1;
            self.game_board.board[row_index as usize][to.col as usize] = None;
        }

        if self.game_board.is_latest_move_castling(*from, *to) {
            let from_x: i32 = from.col as i32;
            let mut new_to = to;
            let to_x: i32 = to.col as i32;

            let distance = from_x - to_x;
            let direction_x = if distance > 0 { -1 } else { 1 };

            let col_king = from_x + direction_x * 2;

            self.game_board.board[to.row as usize][col_king as usize] = self.game_board.board[from];

            if self.bot.is_some() && to_x == 6 && to.row == 0 {
                new_to = &Coord { row: 0, col: 7 };
            }
            if self.bot.is_some() && to_x == 2 && to.row == 0 {
                new_to = &Coord { row: 0, col: 0 };
            }

            let col_rook = if distance > 0 {
                col_king + 1
            } else {
                col_king - 1
            };

            self.game_board.board[new_to.row as usize][col_rook as usize] =
                Some((PieceType::Rook, self.player_turn));

            self.game_board.board[new_to] = None;
        } else {
            self.game_board.board[to] = self.game_board.board[from];
        }

        self.game_board.board[from] = None;

        self.game_board.move_history.push(PieceMove {
            piece_type: piece_type_from,
            piece_color: self.player_turn,
            from: *from,
            to: *to,
        });
        self.game_board.board_history.push(self.game_board.board);
    }

    pub fn execute_opponent_move(&mut self) {
        let opponent_move = self.opponent.as_mut().unwrap().read_stream();
        self.game_board.flip_the_board();
        self.opponent.as_mut().unwrap().opponent_will_move = false;

        if opponent_move.is_empty() {
            return;
        }

        let from_y = get_int_from_char(opponent_move.chars().next());
        let from_x = get_int_from_char(opponent_move.chars().nth(1));
        let to_y = get_int_from_char(opponent_move.chars().nth(2));
        let to_x = get_int_from_char(opponent_move.chars().nth(3));

        let mut promotion_piece: Option<PieceType> = None;
        if opponent_move.chars().count() == 5 {
            promotion_piece = match opponent_move.chars().nth(4) {
                Some('q') => Some(PieceType::Queen),
                Some('r') => Some(PieceType::Rook),
                Some('b') => Some(PieceType::Bishop),
                Some('n') => Some(PieceType::Knight),
                _ => None,
            };
        }

        let from = &Coord::new(from_y, from_x);
        let to = &Coord::new(to_y, to_x);

        self.execute_move(from, to);

        if promotion_piece.is_some() {
            self.game_board.board[to_y as usize][to_x as usize] =
                Some((promotion_piece.unwrap(), self.player_turn));
        }
        self.game_board.flip_the_board();
    }

    pub fn handle_multiplayer_promotion(&mut self) {
        let opponent = self.opponent.as_mut().unwrap();

        let last_move_promotion_type = self.game_board.get_last_move_piece_type_as_string();

        opponent.send_move_to_server(
            self.game_board.move_history.last().unwrap(),
            Some(last_move_promotion_type),
        );
        opponent.opponent_will_move = true;
    }
}
