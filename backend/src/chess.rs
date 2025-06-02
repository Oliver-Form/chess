#[derive(Copy, Clone, Debug)]
enum PieceType { 
    Pawn,
    Knight, 
    Bishop, 
    Rook, 
    Queen, 
    King 
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Color { 
    White, 
    Black 
}

#[derive(Copy, Clone, Debug)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

type Square = Option<Piece>;
type Board = [Square; 64];

#[derive(Debug)]
struct Move {
    from: u8,
    to: u8,
    promotion: Option<PieceType>,
    is_castle: bool,
    is_en_passant: bool,
}

#[derive(Debug)]
struct GameState {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<u8>,
    halfmove_clock: u32, 
    fullmove_clock: u32,
}

#[derive(Debug)]
struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool
}

// ------- Initialisation -------

fn starting_board() -> Board {
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
        board[i + 8] = Some(Piece {piece_type: PieceType::Pawn, color: Color::White })
    }

    for i in 0..8 {
        board[56 + i] = Some(Piece { piece_type: back_rank[i], color: Color::Black });
        board[48 + i] = Some(Piece {piece_type: PieceType::Pawn, color: Color::Black })
    }

    board
}

fn initial_game_state() -> GameState {
    GameState {
        board: starting_board(),
        turn: Color::White,
        castling_rights: CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside:true,
            black_queenside:true,
        },
        en_passant_square: None,
        halfmove_clock: 0,
        fullmove_clock: 1,
    }
}

fn main() {
    let game = initial_game_state();
}

// 