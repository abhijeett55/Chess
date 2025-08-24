
use super::{Movable, PieceColor, PieceMove, PieceType, Position};
use crate::constants::DisplayMode;
use crate::game_logic::coord::Coord;
use crate::game_logic::game_board::GameBoard;
use crate::utils::{cleaned_positions, invert_position, is_cell_color_ally};


pub struct Pawn;

impl Movable for Pawn {
    fn piece_move(
        coordinates: &Coord,
        color: PieceColor,
        game_board: &GameBoard,
        allow_move_on_ally_positions : bool,
    ) -> Vec<Coord> {
        let direction : i8 = if allow_move_on_ally_positions {1} else {-1};
        let mut positions : Vec<Coord> = vec![];

        let (y, x) = (coordinates.row, coordinates.col);

        let new_x_front_one = x;
        let new_y_front_one = y as i8 + direction;
        let new_coordinates_front_one = Coord::new(new_y_front_one as u8, new_x_front_one);

        if new_coordinates_front_one.is_valid() && !allow_move_on_ally_positions &&
            game_board.get_piece_color(&new_coordinates_front_one)
                .is_none() {
            positions.push(new_coordinates_front_one);

            let new_x_front_two = x;
            let new_y_front_two = y as i8 + direction * 2;
            let new_coordinates_front_two = Coord::new(new_y_front_two as u8, new_x_front_two);

            if new_coordinates_front_two.is_valid() && game_board.get_piece_color(&new_coordinates_front_two).is_none() && ( y == 6) {
                positions.push(new_coordinates_front_two);
            }
        }


        let new_x_right = x + 1;
        let new_y_right = y as i8 + direction;
        let new_coordinates_right = if let Some(new_coord) = Coord::opt_new(new_y_right, new_x_right) {
            new_coord
        } else {
            Coord::undefined()
        };


        let new_x_left = x as i8 - 1;
        let new_y_left = y as i8 + direction;
        let new_coordinates_left = if let Some(new_coord) = Coord::opt_new(new_y_left, new_x_left) {
            new_coord
        } else {
            Coord::undefined()
        };

        if allow_move_on_ally_positions {
            if new_coordinates_right.is_valid() {
                positions.push(new_coordinates_right);
            };
            if new_coordinates_left.is_valid() {
                positions.push(new_coordinates_left);
            };
        } else {
            if new_coordinates_right.is_valid() && game_board.get_piece_color(&new_coordinates_right).is_some() &&
                !is_cell_color_ally(game_board, &new_coordinates_right, color)
            {
                positions.push(new_coordinates_right);
            }
            if new_coordinates_left.is_valid() && game_board.get_piece_color(&new_coordinates_left).is_some() &&
                !is_cell_color_ally(game_board, &new_coordinates_left, color) {
                positions.push(new_coordinates_left);
            }

        }

        if let Some(latest_move) = game_board.move_history.last() {
            let number_of_cells_move = latest_move.to.row as i8 - latest_move.from.row as i8;


            let last_coords = invert_position(&Coord::new(latest_move.to.row, latest_move.to.col));


            if latest_move.piece_type == PieceType::Pawn && number_of_cells_move == -2 && y == last_coords.row &&
                (x as i8 == (last_coords.col as i8) - 1|| x == last_coords.col + 1) {

                let new_y = y - 1;
                let new_x = last_coords.col;
                positions.push(Coord::new(new_y, new_x));
            }

        }

        cleaned_positions(&positions)
    }
}

impl Position for Pawn {
    fn authorized_positions(
        coordinates: &Coord,
        color: PieceColor,
        game_board: &GameBoard,
        _is_king_checked: bool,
    ) -> Vec<Coord> {
        game_board.impossible_positions_king_checked(
            coordinates,
            Self::piece_move(coordinates, color, game_board, false),
            color,
        )
    }

    fn protected_positions(
            coordinates: &Coord,
            color: PieceColor,
            game_board: &GameBoard,
        ) -> Vec<Coord> {
            Self::piece_move(coordinates, color, game_board, true)
    }

}

impl Pawn {
    pub fn to_string(display_mode : &DisplayMode) -> &'static str {
        match display_mode {
            DisplayMode::DEFAULT => {
                "\
        \n\
        \n\
      ▟█▙\n\
      ▜█▛\n\
     ▟███▙\n\
    "
            }
            DisplayMode::ASCII => "p",
        }
    }
    pub fn did_pawn_move_two_cells(last_move: Option<&PieceMove>) -> bool {
        match last_move {
            Some(last_move) => {
                let distance = (last_move.to.row as i8 - last_move.from.row as i8).abs();

                if last_move.piece_type == PieceType::Pawn && distance == 2 {
                    return true;
                }
                false
             }
            _ => false,
        }
    }
}