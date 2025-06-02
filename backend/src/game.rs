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
pub type Board = [Square; 64];

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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool
}

// ------- Initialisation -------

pub fn starting_board() -> Board {
    let mut board: Board = [None; 64];
    
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
        }
    }
}

fn main() {
    let game = GameState::new();
}
