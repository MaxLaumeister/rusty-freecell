// #TODO n eliminate terminal flicker
// #TODO   error handling
// #TODO n moving multiple cards at once (shortcut)
// #TODO X move count
// #TODO X implement winning screen
// #TODO   test to make sure winning (and undo after winning) works
// #TODO X decorate foundations with suits
// #TODO   condense top row representation when terminal is small, expand when large
// #TODO   refactor, ci, lint, publish
// #TODO ? fix windows terminal behavior
// #TODO   variable terminal size
// #TODO   member visibility (modules)
// #TODO X only allow card to be on matching foundation spot
// #TODO   get rid of memory allocations/heap (String usage) wherever possible
// #TODO X don't allow cursor to rest on empty space, when not in select mode
// #TODO X fix foundation decoration rendering when card is selected
// #TODO X fix tableau empty column decoration and cursor visibility
// #TODO   automatically stack cards onto foundation shortcut button
// #TODO   pet the coyote she has been so good

use std::{cmp, io::{stdout, Stdout, Write}};

use circular_buffer::CircularBuffer;
use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, execute, style::{self, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor, SetStyle, Stylize}, terminal::{size, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen, ScrollUp, SetSize}, ExecutableCommand, QueueableCommand
};

const RANKS: usize = 13;
const SUITS: usize = 4;
const DECK_SIZE: usize = RANKS* SUITS;

const HEARTS: u8 = 1;
const CLUBS: u8 = 2;
const DIAMONDS: u8 = 3;
const SPADES: u8 = 4;

const FREE_CELLS: usize = 4;
const TABLEAU_SIZE: usize = 8;

const TYPICAL_BOARD_HEIGHT: usize = 24;

const CARD_PRINT_WIDTH: usize = 7;
const CARD_PRINT_HEIGHT: usize = 5;
const TABLEAU_VERTICAL_OFFSET: usize = 2;

const UNDO_LEVELS: usize = 1000;

const SUIT_STRINGS: [&str;SUITS+1] = [" ", "♥", "♣", "♦", "♠"];
const RANK_STRINGS: [&str;RANKS+1] = [" ", "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];

#[derive(Default, Copy, Clone)]
struct Card {
    rank: u8,
    suit: u8
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

struct Deck {
    cards: [Card; DECK_SIZE as usize]
}

#[derive(Default, Copy, Clone)]
struct Move {
    from: u8,
    to: u8
}

impl Deck {
    fn standard() -> Deck {
        let mut deck = Deck {cards: [Card {rank: 0, suit: 0}; DECK_SIZE as usize]};
        for r in 0..RANKS {
            for s in 0..SUITS {
                deck.cards[(s*RANKS+r) as usize] = Card{rank: (r+1) as u8, suit: (s+1) as u8};
            }
        }
        deck
    }
    fn shuffle(&mut self, rng: &mut rand::rngs::ThreadRng) {
        use rand::seq::SliceRandom;
        self.cards.shuffle(rng);
    }
}

// For debugging only

// impl std::fmt::Display for Deck {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{{ ")?;
//         for i in &self.cards {
//             write!(f, "{} ", i)?;
//         }
//         write!(f, "}}")?;
//         Ok(())
//     }
// }

// impl std::fmt::Display for Card {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         let suit_str = match self.suit {
//             0 => "h",
//             1 => "d",
//             2 => "c",
//             3 => "s",
//             _ => "x"
//         };
//         write!(f, "({}, {})", self.rank, suit_str)
//     }
// }

struct Board {
    field: [[Card; DECK_SIZE as usize]; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize],
    field_lengths: [usize; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize]
}

impl Board {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Board {
        let mut board = Board {
            field: [[Card::default(); DECK_SIZE as usize]; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize],
            field_lengths: [0; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize]
        };

        let mut deck = Deck::standard();
        deck.shuffle(rng);

        // Deal out onto the board
        let mut tableau_column = 0;
        for card in deck.cards {
            board.put_on_tableau(card, tableau_column as usize);
            tableau_column += 1;
            if tableau_column >= TABLEAU_SIZE {
                tableau_column = 0;
            }
        }

        board
    }

    fn put_on_tableau(&mut self, c: Card, tableau_column: usize) {
        let field_column = tableau_column + (SUITS + FREE_CELLS) as usize;
        self.field[field_column][self.field_lengths[field_column]] = c;
        self.field_lengths[field_column] += 1;
    }
}

struct Game {
    board: Board,
    highlighted_card: usize,
    selected_card_opt: Option<usize>,
    undo_history: CircularBuffer::<UNDO_LEVELS, Move>,
    move_count: u32,
    won: bool
}

impl Game {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let game = Game {
            board: Board::new(rng),
            highlighted_card: SUITS + FREE_CELLS,
            selected_card_opt: None,
            undo_history: CircularBuffer::new(),
            move_count: 0,
            won: false
        };
        game
    }
    fn print_board(&self, out: &mut Stdout) {
        let _ = out.queue(Clear(ClearType::All));

        for i in 0..(SUITS+FREE_CELLS+TABLEAU_SIZE) as usize {
            let stack = self.board.field[i];
            let stack_size = self.board.field_lengths[i];
            let mut top_card = self.board.field[i][if stack_size == 0 {0} else {stack_size - 1}];
            let top_card_is_highlighted = self.highlighted_card as usize == i && self.won != true;
            if i < SUITS as usize {
                // Print foundation
                // If card is a placeholder, assign a suit for decoration
                if top_card == Card::default() {
                    top_card.suit = i as u8 + 1;
                }
                Game::print_card_at_coord(
                    out,
                    i * CARD_PRINT_WIDTH, 
                    1, 
                    top_card, 
                    top_card_is_highlighted, 
                    self.selected_card_opt == Some(i)
                );
            } else if i < (SUITS + FREE_CELLS) as usize {
                // Print free cell
                Game::print_card_at_coord(
                    out,
                    i * CARD_PRINT_WIDTH + 2,
                    1, top_card,
                    top_card_is_highlighted,
                    self.selected_card_opt == Some(i)
                );
            } else if i < (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize {
                // Print tableau (at least print placeholders if no card in stack)
                for y in 0..cmp::max(1, stack_size) {
                    let card = stack[y];
                    Game::print_card_at_coord(
                        out,
                        (i - (SUITS + FREE_CELLS) as usize) * CARD_PRINT_WIDTH + 1,
                        y * TABLEAU_VERTICAL_OFFSET + CARD_PRINT_HEIGHT + 1, card,
                        top_card_is_highlighted && (y + 1 == stack_size || 0 == stack_size), // if we are currently printing top card (or placeholder)
                        self.selected_card_opt == Some(i) && y + 1 == stack_size
                    );
                }
            }
        }
    }

    fn print_status_bars(&self, out: &mut Stdout) {
        // Print title bar
        let _ = out.queue(cursor::MoveTo(0, 0));
        print!("--- Rusty FreeCell ---------------------------------------");
        let _ = out.queue(cursor::MoveTo(40, 0));
        print!(" Moves: {} ", self.move_count);

        // Print bottom bar
        let _ = out.queue(cursor::MoveTo(0, TYPICAL_BOARD_HEIGHT as u16));
        print!("--- (New Game: ctrl-n) - (Undo: z) - (Quit: q) -----------");
    }

    fn print_card_at_coord(out: &mut Stdout, x: usize, y: usize, card: Card, highlighted: bool, selected: bool) {
        let card_suit_rank_str = RANK_STRINGS[card.rank as usize].to_owned() + SUIT_STRINGS[card.suit as usize];
        let card_display_str;
        if selected {
            card_display_str= format!("\
                ╭─────╮\n\
                │ {: <3} │\n\
                │     │\n\
                │  △  │\n\
                ╰─────╯\n",
                card_suit_rank_str);
        } else if card.rank == 0 {
            // Print suit-decorated placeholder
            card_display_str= format!("\
            ╭─────╮\n\
            │     │\n\
            │ {}  │\n\
            │     │\n\
            ╰─────╯\n",
            card_suit_rank_str);
        } else {
            card_display_str= format!("\
            ╭─────╮\n\
            │ {: <3} │\n\
            │     │\n\
            │     │\n\
            ╰─────╯\n",
            card_suit_rank_str);
        }

        for (d, line) in card_display_str.lines().enumerate() {
            if y+d >= TYPICAL_BOARD_HEIGHT {break};
            let _ = out.queue(cursor::MoveTo(x as u16, y as u16 + d as u16));
            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::Reverse));
            } else if card.rank == 0 {
                // dim placeholder
                let _= out.queue(style::SetAttribute(style::Attribute::Dim));
            }

            if (card.suit == HEARTS || card.suit == DIAMONDS) && card.rank != 0 {
                // If it's a red card (and not a placeholder) print in red
                print!("{}", line.with(Color::Red));
            } else {
                print!("{}", line);
            }

            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::NoReverse));
            } else if card.rank == 0 {
                // undim placeholder
                let _= out.queue(style::SetAttribute(style::Attribute::NormalIntensity));
            }
        }
    }

    fn print_win (&self, out: &mut Stdout) {
        let win_message_width = 20;
        let win_message_height = 4;
        Game::print_string_at_coord(out,   
        "╭──────────────────╮\n\
                 │ You Win!         │\n\
                 │ New Game: ctrl-n │\n\
                 ╰──────────────────╯",
                (/* magic */ 58 / 2 - win_message_width / 2) as u16,
                (TYPICAL_BOARD_HEIGHT / 2 - win_message_height / 2) as u16);
    }

    fn print(&self, out: &mut Stdout) {
        if !self.won {
            self.print_board(out);
            self.print_status_bars(out);
        } else {
            // won
            let _ = out.queue(SetAttribute(style::Attribute::Dim));
            self.print_board(out);
            let _ = out.queue(SetAttribute(style::Attribute::Reset));
            self.print_status_bars(out);
            self.print_win(out);
        }
        let _ = out.flush();
    }

    fn print_string_at_coord(out: &mut Stdout, string: &str, x: u16, y: u16) {
        for (i, line) in string.lines().enumerate() {
            let _ = out.queue(cursor::MoveTo(x, y + i as u16));
            print!("{}", line);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.highlighted_card == 0 {
            self.highlighted_card = SUITS + FREE_CELLS + TABLEAU_SIZE - 1;
        } else {
            self.highlighted_card -= 1;
        }

        match self.selected_card_opt {
            Some(selected_card) => {
                while !self.move_is_valid(selected_card, self.highlighted_card) {
                    self.move_cursor_left();
                }
            }
            None => {
                while self.board.field_lengths[self.highlighted_card] == 0 {
                    self.move_cursor_left();
                }
            }
        }
    }

    fn move_cursor_right(&mut self) {
        if self.highlighted_card >= SUITS + FREE_CELLS + TABLEAU_SIZE - 1 {
            self.highlighted_card = 0;
        } else {
            self.highlighted_card += 1;
        }

        match self.selected_card_opt {
            Some(selected_card) => {
                while !self.move_is_valid(selected_card, self.highlighted_card) {
                    self.move_cursor_right();
                }
            }
            None => {
                while self.board.field_lengths[self.highlighted_card] == 0 {
                    self.move_cursor_right();
                }
            }
        }
    }
    fn handle_card_press(&mut self) {
        if self.selected_card_opt == None {
            // Select a card
            self.selected_card_opt = Some(self.highlighted_card);
        } else if Some(self.highlighted_card) == self.selected_card_opt {
            // Deselect a card
            self.selected_card_opt = None;
        } else {
            // Execute a move
            match self.selected_card_opt {
                Some(selected_card) => {
                    self.player_try_execute_move(selected_card, self.highlighted_card);
                }
                None => {

                }
            }
        }
    }
    fn are_opposite_colors(card1: Card, card2: Card) -> bool {
        if card1.suit == HEARTS || card1.suit == DIAMONDS {return card2.suit == SPADES || card2.suit == CLUBS};
        if card1.suit == SPADES || card1.suit == CLUBS {return card2.suit == HEARTS || card2.suit == DIAMONDS};
        return false;
    }
    fn move_is_valid(&self, from: usize, to: usize) -> bool {
        let from_top_card = if self.board.field_lengths[from as usize] > 0 {self.board.field[from as usize][self.board.field_lengths[from as usize] - 1]} else {Card::default()};
        let to_top_card = if self.board.field_lengths[to as usize] > 0 {self.board.field[to as usize][self.board.field_lengths[to as usize] - 1]} else {Card::default()};
        if from == to {return true;};
        if to < SUITS {
            // Foundation case
            if to_top_card.rank != 0 {
                    return from_top_card.rank == to_top_card.rank + 1 && from_top_card.suit == to_top_card.suit;
            } else {
                    return from_top_card.rank == 1 && to as u8 == from_top_card.suit - 1;
            }
        } else if to < SUITS + FREE_CELLS {
            // Free cell case
            return to_top_card.rank == 0;
        } else if to < SUITS + FREE_CELLS + TABLEAU_SIZE {
            // Tableau case
            if to_top_card.rank != 0 {
                return from_top_card.rank == to_top_card.rank - 1 && Game::are_opposite_colors(from_top_card, to_top_card);
            } else {
                return true;
            }
        }
        return false;
    }
    fn execute_move (&mut self, from: usize, to: usize) {
        // Execute the move
        self.board.field[to as usize][self.board.field_lengths[to as usize]] = self.board.field[from as usize][self.board.field_lengths[from as usize] - 1];
        self.board.field[from as usize][self.board.field_lengths[from as usize] - 1] = Default::default();
        self.board.field_lengths[from as usize] -= 1;
        self.board.field_lengths[to as usize] += 1;
        self.selected_card_opt = None;
        // Check to see if player won or unwon (due to undo)
        self.won = self.board.field_lengths[0..SUITS].iter().all(|&x| x == RANKS);
    }
    fn player_try_execute_move(&mut self, from: usize, to: usize) {
        if self.move_is_valid(from, to) {
            // Execute move, add to undo history
            self.execute_move(from, to);
            self.move_count += 1;
            self.undo_history.push_back(Move{from: from as u8, to: to as u8});
        }
    }
    fn perform_undo(&mut self) {
        let last_move_opt = self.undo_history.pop_back();
        match last_move_opt {
            Some(last_move) => {
                // Perform move in reverse, without checking if it follows the rules
                self.execute_move(last_move.to as usize, last_move.from as usize);
                self.move_count -= 1;
            }
            None => {
                // History empty
            }
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    
    // Prepare terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    // Create game
    let mut rng = rand::thread_rng();
    let mut game = Game::new(&mut rng);
    game.print(&mut stdout);

    // Game loop
    loop {
        if let Event::Key(event) = event::read().expect("Failed to read line") {
            match event {
                KeyEvent {code: KeyCode::Char('q'), modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    break
                },
                KeyEvent {code: KeyCode::Left, modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.move_cursor_left();}
                },
                KeyEvent {code: KeyCode::Char('a'), modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.move_cursor_left();}
                },
                KeyEvent {code: KeyCode::Right, modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.move_cursor_right();}
                },
                KeyEvent {code: KeyCode::Char('d'), modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.move_cursor_right();}
                },
                KeyEvent {code: KeyCode::Char(' '), modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.handle_card_press();}
                },
                KeyEvent {code: KeyCode::Enter, modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    if !game.won {game.handle_card_press();}
                },
                KeyEvent {code: KeyCode::Char('z'), modifiers: event::KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    game.perform_undo();
                },
                KeyEvent {code: KeyCode::Char('n'), modifiers: event::KeyModifiers::CONTROL, kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat, state: _} => {
                    game = Game::new(&mut rng);
                },
                _ => {
                    
                }
            }
            game.print(&mut stdout);
        };
    }

    Ok(())
}

fn cleanup() {
    let mut stdout = stdout();
    let _ = stdout.execute(cursor::Show);
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = stdout.execute(Clear(ClearType::All));
    let _ = stdout.execute(LeaveAlternateScreen);
    println!();
}

fn main() -> impl std::process::Termination {
    std::env::set_var("RUST_BACKTRACE", "1");
    let _ = run();
    cleanup();
}
