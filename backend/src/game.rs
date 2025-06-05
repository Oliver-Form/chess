use rand::{thread_rng, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PieceType { 
    Pawn,
    Knight, 
    Bishop, 
    Rook, 
    Queen, 
    King 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Color { 
    White, 
    Black 
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    color: Color,
}

pub type Square = Option<Piece>;
pub type Board = Vec<Square>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Move {
    from: u8,
    to: u8,
    promotion: Option<PieceType>,
    is_castle: bool,
    is_en_passant: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameState {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<u8>,
    halfmove_clock: u32, 
    fullmove_clock: u32,
    game_code: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool
}

// ----------- Checks -----------

fn opposite_color(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

fn is_square_attacked(state: &GameState, square: u8, attacker_color: Color) -> bool {
    let target_x = square % 8;
    let target_y = square / 8;
    for i in 0..64 {
        if let Some(attacker) = state.board[i] {
            if attacker.color == attacker_color {
                match attacker.piece_type {
                    PieceType::King => {
                        let x = i % 8;
                        let y = i / 8;
                        if (x as i8 - target_x as i8).abs() <= 1 && (y as i8 - target_y as i8).abs() <= 1 {
                            return true;
                        }
                    }
                    _ => {
                        let pseudo_moves = legal_moves_for_piece(state, i as u8);
                        if pseudo_moves.contains(&square) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn legal_moves_for_piece(state: &GameState, pos: u8) -> Vec<u8> {
    let Some(piece) = state.board[pos as usize] else { return vec![] };
    let mut moves = Vec::new();

    let (x, y) = (pos % 8, pos / 8);
    let color = piece.color;

    let mut push = |dx: i8, dy: i8, repeat: bool, captures_only: bool| {
        let mut step = 1;
        loop {
            let nx = x as i8 + dx * step;
            let ny = y as i8 + dy * step;
            if nx < 0 || ny < 0 || nx >= 8 || ny >= 8 {
                break;
            }
            let to = (ny * 8 + nx) as u8;
            match state.board[to as usize] {
                Some(p) if p.color == color => break,
                Some(p) => {
                    if !captures_only {
                        moves.push(to);
                    } else {
                        moves.push(to);
                    }
                    break;
                }
                None => {
                    if !captures_only {
                        moves.push(to);
                    }
                }
            }
            if !repeat { break; }
            step += 1;
        }
    };
 

    match piece.piece_type {
        PieceType::Pawn => {
            let dir = if color == Color::White { 1 } else { -1 };
            let start_row = if color == Color::White { 1 } else { 6 };

            let forward_one = pos as i8 + dir * 8;
            if (0..64).contains(&forward_one) && state.board[forward_one as usize].is_none() {
                moves.push(forward_one as u8);

                let forward_two = pos as i8 + dir * 16;
                if y == start_row && state.board[forward_two as usize].is_none() {
                    moves.push(forward_two as u8);
                }
            }

            for dx in [-1, 1] {
                let nx = x as i8 + dx;
                let ny = y as i8 + dir;
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    let to = (ny * 8 + nx) as u8;
                    if let Some(p) = state.board[to as usize] {
                        if p.color != color {
                            moves.push(to);
                        }
                    } else if Some(to) == state.en_passant_square {
                        moves.push(to);
                    }
                }
            }
        }

        PieceType::Knight => {
            let deltas = [
                (1, 2), (2, 1), (-1, 2), (-2, 1),
                (1, -2), (2, -1), (-1, -2), (-2, -1),
            ];
            for (dx, dy) in deltas {
                let nx = x as i8 + dx;
                let ny = y as i8 + dy;
                if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                    let to = (ny * 8 + nx) as u8;
                    if let Some(p) = state.board[to as usize] {
                        if p.color != color {
                            moves.push(to);
                        }
                    } else {
                        moves.push(to);
                    }
                }
            }
        }

        PieceType::Bishop => {
            for &(dx, dy) in &[(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                push(dx, dy, true, false);
            }
        }

        PieceType::Rook => {
            for &(dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                push(dx, dy, true, false);
            }
        }

        PieceType::Queen => {
            for &(dx, dy) in &[
                (1, 0), (-1, 0), (0, 1), (0, -1),
                (1, 1), (-1, 1), (1, -1), (-1, -1)
            ] {
                push(dx, dy, true, false);
            }
        }

        PieceType::King => {
            for &(dx, dy) in &[
                (1, 0), (-1, 0), (0, 1), (0, -1),
                (1, 1), (-1, 1), (1, -1), (-1, -1)
            ] {
                push(dx, dy, false, false);
            }

            // castling with check restrictions
            let opponent = opposite_color(color);
            if piece.color == Color::White && y == 0 && x == 4 {
                if state.castling_rights.white_kingside
                    && state.board[5].is_none()
                    && state.board[6].is_none()
                    && !is_square_attacked(state, 4, opponent)
                    && !is_square_attacked(state, 5, opponent)
                    && !is_square_attacked(state, 6, opponent)
                {
                    moves.push(6);
                }
                if state.castling_rights.white_queenside
                    && state.board[1].is_none()
                    && state.board[2].is_none()
                    && state.board[3].is_none()
                    && !is_square_attacked(state, 4, opponent)
                    && !is_square_attacked(state, 3, opponent)
                    && !is_square_attacked(state, 2, opponent)
                {
                    moves.push(2);
                }
            }

            if piece.color == Color::Black && y == 7 && x == 4 {
                if state.castling_rights.black_kingside
                    && state.board[61].is_none()
                    && state.board[62].is_none()
                    && !is_square_attacked(state, 60, opponent)
                    && !is_square_attacked(state, 61, opponent)
                    && !is_square_attacked(state, 62, opponent)
                {
                    moves.push(62);
                }
                if state.castling_rights.black_queenside
                    && state.board[57].is_none()
                    && state.board[58].is_none()
                    && state.board[59].is_none()
                    && !is_square_attacked(state, 60, opponent)
                    && !is_square_attacked(state, 59, opponent)
                    && !is_square_attacked(state, 58, opponent)
                {
                    moves.push(58);
                }
            }
        }
    }

   moves
}

pub fn legal_moves_for_piece_strict(state: &GameState, pos: u8) -> Vec<u8> {
    let Some(piece) = state.board[pos as usize] else { return vec![] };
    if piece.color != state.turn {
        return vec![];
    }

    let candidate_moves = legal_moves_for_piece(state, pos); 
    let mut legal_moves = Vec::new();

    for to in candidate_moves {
        let mut next_state = state.clone();

        next_state.board[to as usize] = next_state.board[pos as usize];
        next_state.board[pos as usize] = None;

        if piece.piece_type == PieceType::Pawn && state.en_passant_square == Some(to) {
            let offset = if piece.color == Color::White { -8 } else { 8 };
            next_state.board[(to as i8 + offset) as usize] = None;
        }

        let king_pos = next_state.board.iter().position(|&p| {
            matches!(p, Some(Piece { piece_type: PieceType::King, color }) if color == piece.color)
        });

        if let Some(king_idx) = king_pos {
            if !is_square_attacked(&next_state, king_idx as u8, opposite_color(piece.color)) {
                legal_moves.push(to);
            }
        }
    }

   legal_moves 
}

// ------- Initialisation -------

pub fn starting_board() -> Board {
    let mut board: Board = vec![None; 64];
    
    let back_rank = [
        PieceType::Rook,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Queen,
        PieceType::King,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Rook,
    ];

    for i in 0..8 {
        board[i] = Some(Piece { piece_type: back_rank[i], color: Color::White });
        board[i + 8] = Some(Piece { piece_type: PieceType::Pawn, color: Color::White });
    }

    for i in 0..8 {
        board[56 + i] = Some(Piece { piece_type: back_rank[i], color: Color::Black });
        board[48 + i] = Some(Piece { piece_type: PieceType::Pawn, color: Color::Black });
    }

   board 
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: starting_board(),
            turn: Color::White,
            castling_rights: CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            },
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_clock: 1,
            game_code: {
                let mut rng = thread_rng();
                format!("{:06}", rng.gen_range(0..1_000_000))
            },
        }
    }
    /// Returns the game code identifier
    pub fn game_code(&self) -> &str {
        &self.game_code
    }
    /// Returns the current player's turn color
    pub fn turn(&self) -> Color {
        self.turn
    }
    
    /// Moves a piece from one square to another, without validation, then switches turn
    pub fn move_piece(&mut self, from: u8, to: u8, promotion: Option<PieceType>) {
        let from_idx = from as usize;
        let to_idx = to as usize;
        // handle castling: king moves two squares
        if let Some(piece) = self.board[from_idx] {
            if piece.piece_type == PieceType::King {
                // kingside
                if to_idx == from_idx + 2 {
                    let (rook_from, rook_to) = match piece.color {
                        Color::White => (7, 5),
                        Color::Black => (63, 61),
                    };
                    self.board[rook_to] = self.board[rook_from];
                    self.board[rook_from] = None;
                }
                // queenside
                if to_idx + 2 == from_idx {
                    let (rook_from, rook_to) = match piece.color {
                        Color::White => (0, 3),
                        Color::Black => (56, 59),
                    };
                    self.board[rook_to] = self.board[rook_from];
                    self.board[rook_from] = None;
                }
                // disable both castling rights for this king
                match piece.color {
                    Color::White => {
                        self.castling_rights.white_kingside = false;
                        self.castling_rights.white_queenside = false;
                    }
                    Color::Black => {
                        self.castling_rights.black_kingside = false;
                        self.castling_rights.black_queenside = false;
                    }
                }
            }
            // if rook moves, disable its castling side
            if piece.piece_type == PieceType::Rook {
                match piece.color {
                    Color::White => {
                        if from_idx == 0 { self.castling_rights.white_queenside = false; }
                        if from_idx == 7 { self.castling_rights.white_kingside = false; }
                    }
                    Color::Black => {
                        if from_idx == 56 { self.castling_rights.black_queenside = false; }
                        if from_idx == 63 { self.castling_rights.black_kingside = false; }
                    }
                }
            }
        }
        // handle en passant target and capture
        if let Some(piece) = self.board[from_idx] {
            if piece.piece_type == PieceType::Pawn {
                let row_from = from_idx as i8 / 8;
                let row_to = to_idx as i8 / 8;
                // double-step opens en passant
                if (row_from - row_to).abs() == 2 {
                    // target square is the square passed over
                    let between = ((from + to) / 2) as u8;
                    self.en_passant_square = Some(between);
                } else {
                    // normal pawn move or capture
                    // if moving to en_passant_square, remove captured pawn
                    if Some(to as u8) == self.en_passant_square {
                        let cap_idx = if piece.color == Color::White {
                            to_idx - 8
                        } else {
                            to_idx + 8
                        };
                        self.board[cap_idx] = None;
                    }
                    self.en_passant_square = None;
                }
            } else {
                // any other piece clears en passant availability
                self.en_passant_square = None;
            }
        }
        // move the piece
        self.board[to_idx] = self.board[from_idx];
        self.board[from_idx] = None;
        // handle pawn promotion
        if let Some(mut piece) = self.board[to_idx] {
            if piece.piece_type == PieceType::Pawn {
                let row = (to_idx / 8) as u8;
                let last_rank = if piece.color == Color::White { 7 } else { 0 };
                if row == last_rank {
                    // promote to chosen piece or default Queen
                    let new_type = promotion.unwrap_or(PieceType::Queen);
                    self.board[to_idx] = Some(Piece { piece_type: new_type, color: piece.color });
                }
            }
        }
        // switch current player's turn
        self.turn = opposite_color(self.turn);
    }
    /// Returns the type of the piece at a given board index, or None if empty
    pub fn piece_type_at(&self, idx: usize) -> Option<PieceType> {
        self.board.get(idx).and_then(|&sq| sq.map(|piece| piece.piece_type))
    }
    /// Returns the color of the piece at a given board index, or None if empty
    pub fn piece_color_at(&self, idx: usize) -> Option<Color> {
        self.board.get(idx).and_then(|&sq| sq.map(|piece| piece.color))
    }

    /// Returns true if the current player is in check (their king is attacked)
    pub fn is_in_check(&self) -> bool {
        // find king position for current turn
        let king_pos = self.board.iter().position(|&sq| {
            matches!(sq, Some(ref p) if p.piece_type == PieceType::King && p.color == self.turn)
        });
        if let Some(idx) = king_pos {
            is_square_attacked(self, idx as u8, opposite_color(self.turn))
        } else {
            false
        }
    }

    /// Returns true if the current player is checkmated
    pub fn is_checkmate(&self) -> bool {
        // must be in check and have no legal moves
        if !self.is_in_check() {
            return false;
        }
        // search for any legal move for current player
        for idx in 0..64u8 {
            if let Some(piece) = self.board[idx as usize] {
                if piece.color == self.turn {
                    if !legal_moves_for_piece_strict(self, idx).is_empty() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns true if the current player has no legal moves but is not in check (stalemate)
    pub fn is_stalemate(&self) -> bool {
        if self.is_in_check() {
            return false;
        }
        for idx in 0..64u8 {
            if let Some(piece) = self.board[idx as usize] {
                if piece.color == self.turn && !legal_moves_for_piece_strict(self, idx).is_empty() {
                    return false;
                }
            }
        }
        true
    }
}

fn main() {
    let game = GameState::new();
}
