// #TODO n eliminate terminal flicker
// #TODO X error handling
// #TODO n moving multiple cards at once (shortcut)
// #TODO X move count
// #TODO X implement winning screen
// #TODO X test to make sure winning (and undo after winning) works
// #TODO X decorate foundations with suits
// #TODO n condense top row representation when terminal is small, expand when large
// #TODO   refactor, ci, lint, publish (lint: remove unnecessary "as" statements)
// #TODO ? fix windows terminal behavior
// #TODO X variable terminal size
// #TODO   member visibility (modules)
// #TODO X only allow card to be on matching foundation spot
// #TODO X get rid of memory allocations/heap (String usage) wherever possible
// #TODO X don't allow cursor to rest on empty space, when not in select mode
// #TODO X fix foundation decoration rendering when card is selected
// #TODO X fix tableau empty column decoration and cursor visibility
// #TODO X automatically stack cards onto foundation shortcut button
// #TODO X implement "symbol blind" mode - cyan and yellow suits
// #TODO   change array access to use iterators instead of indexing wherever possible, to prevent out of bounds errors
// #TODO   pet the coyote she has been so good

use std::{cmp, io::{self, stdout, Stdout, Write}};

use circular_buffer::CircularBuffer;
use crossterm::{
    cursor, event, style::{self, Stylize}, terminal, ExecutableCommand, QueueableCommand
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
const FIELD_SIZE: usize = SUITS + FREE_CELLS + TABLEAU_SIZE;

const DEFAULT_TERMINAL_WIDTH: u16 = 80;
const DEFAULT_TERMINAL_HEIGHT: u16 = 24;

const MIN_TERMINAL_WIDTH: u16 = 60;
const MIN_TERMINAL_HEIGHT: u16 = 24;

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
        deck.cards.iter_mut().enumerate().for_each(|(i, card)| {
            *card = Card { rank: (i % RANKS + 1) as u8, suit: (i / RANKS + 1) as u8 };
        });
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
    field: [CardStack; FIELD_SIZE]
}

#[derive(Copy, Clone)]
struct CardStack {
    cards: [Card; DECK_SIZE as usize],
    length: usize
}

impl Default for CardStack {
    fn default() -> Self {
        CardStack {
            cards: [Card::default(); DECK_SIZE],
            length: 0,
        }
    }
}

impl CardStack {
    fn push(&mut self, card: Card) {
        self.cards[self.length] = card;
        self.length += 1;
    }
    fn peek(&mut self) -> Option<Card> {
        if self.length == 0 {
            None
        } else {
            Some(self.cards[self.length - 1])
        }
    }
    fn pop(&mut self) -> Option<Card> {
        let card_opt = self.peek();
        if self.length != 0 {
            self.length -= 1;
        }
        card_opt
    }
}

impl Board {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Board {
        let mut board = Board {
            field: [CardStack::default(); FIELD_SIZE]
        };

        let mut deck = Deck::standard();
        deck.shuffle(rng);

        // Deal deck onto the board
        for (i, card) in deck.cards.iter().enumerate() {
            let field_column = (SUITS + FREE_CELLS) as usize + (i % TABLEAU_SIZE);
            board.field[field_column][board.field_lengths[field_column]] = *card;
            board.field[field_column].length += 1;
        }

        board
    }
}

struct Game {
    board: Board,
    highlighted_card: usize,
    selected_card_opt: Option<usize>,
    undo_history: CircularBuffer::<UNDO_LEVELS, Move>,
    move_count: u32,
    high_contrast: bool,
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
            high_contrast: false,
            won: false
        };
        game
    }
    fn print_board(&self, out: &mut Stdout) -> Result<(), io::Error> {
        out.queue(terminal::Clear(terminal::ClearType::All))?;

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
                    i * CARD_PRINT_WIDTH + 1, 
                    1, 
                    top_card, 
                    top_card_is_highlighted, 
                    self.selected_card_opt == Some(i),
                    self.high_contrast
                )?;
            } else if i < (SUITS + FREE_CELLS) as usize {
                // Print free cell
                Game::print_card_at_coord(
                    out,
                    i * CARD_PRINT_WIDTH + 3,
                    1, top_card,
                    top_card_is_highlighted,
                    self.selected_card_opt == Some(i),
                    self.high_contrast
                )?;
            } else if i < (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize {
                // Print tableau (at least print placeholders if no card in stack)
                for y in 0..cmp::max(1, stack_size) {
                    let card = stack[y];
                    Game::print_card_at_coord(
                        out,
                        (i - (SUITS + FREE_CELLS) as usize) * CARD_PRINT_WIDTH + 2,
                        y * TABLEAU_VERTICAL_OFFSET + CARD_PRINT_HEIGHT + 1, card,
                        top_card_is_highlighted && (y + 1 == stack_size || 0 == stack_size), // if we are currently printing top card (or placeholder)
                        self.selected_card_opt == Some(i) && y + 1 == stack_size,
                        self.high_contrast
                    )?;
                }
            }
        }

        Ok(())
    }

    fn toggle_high_contrast(&mut self) {
        self.high_contrast = !self.high_contrast;
    }

    fn print_chrome(&self, out: &mut Stdout) -> Result<(), io::Error> {
        let (_term_width, term_height) = terminal::size().unwrap_or_else(|_| (DEFAULT_TERMINAL_WIDTH, DEFAULT_TERMINAL_HEIGHT));
        
        // Print title bar
        out.queue(cursor::MoveTo(0, 0))?;
        print!("╭── Rusty FreeCell ────────────────────────────────────────╮");
        out.queue(cursor::MoveTo(40, 0))?;
        print!(" Moves: {} ", self.move_count);

        // Print side bars

        for i in 1..term_height {
            out.queue(cursor::MoveTo(0, i))?;
            print!("│");
            out.queue(cursor::MoveTo(MIN_TERMINAL_WIDTH - 1, i))?;
            print!("│");
        }

        // Print bottom bar
        out.queue(cursor::MoveTo(0, term_height))?;
        print!("╰── (New Game: ctrl-n) ─ (Undo: z) ─ (Quit: ctrl-q) ───────╯");

        Ok(())
    }

    fn print_card_at_coord(out: &mut Stdout, x: usize, y: usize, card: Card, highlighted: bool, selected: bool, high_contrast: bool)  -> Result<(), io::Error> {
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
            out.queue(cursor::MoveTo(x as u16, y as u16 + d as u16))?;
            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::Reverse));
            } else if card.rank == 0 {
                // dim placeholder
                let _= out.queue(style::SetAttribute(style::Attribute::Dim));
            }

            if card.rank != 0 {
                if high_contrast {
                    match card.suit {
                        HEARTS => {
                            print!("{}", line.with(style::Color::Red));
                        },
                        CLUBS => {
                            print!("{}", line.with(style::Color::Magenta));
                        },
                        DIAMONDS => {
                            print!("{}", line.with(style::Color::DarkCyan));
                        },
                        SPADES => {
                            print!("{}", line.with(style::Color::DarkYellow));
                        },
                        _ => {
                            print!("{}", line);
                        }
                    }
                } else {
                    match card.suit {
                        HEARTS | DIAMONDS  => {
                            print!("{}", line.with(style::Color::Red));
                        },
                        _ => {
                            print!("{}", line);
                        }
                    }
                }
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
        Ok(())
    }

    fn print_win (&self, out: &mut Stdout) -> Result<(), io::Error> {
        let win_message_width = 20;
        let win_message_height = 4;
        Game::print_string_at_coord(out,   
        "╭──────────────────╮\n\
                 │ You Win!         │\n\
                 │ New Game: ctrl-n │\n\
                 ╰──────────────────╯",
                (/* magic */ 58 / 2 - win_message_width / 2) as u16,
                (TYPICAL_BOARD_HEIGHT / 2 - win_message_height / 2) as u16)?;
        Ok(())
    }

    fn print(&self, out: &mut Stdout) -> Result<(), io::Error> {
        if !self.won {
            self.print_board(out)?;
            self.print_chrome(out)?;
        } else {
            // won
            out.queue(style::SetAttribute(style::Attribute::Dim))?;
            self.print_board(out)?;
            out.queue(style::SetAttribute(style::Attribute::Reset))?;
            self.print_chrome(out)?;
            self.print_win(out)?;
        }
        out.flush()?;
        Ok(())
    }

    fn print_string_at_coord(out: &mut Stdout, string: &str, x: u16, y: u16) -> Result<(), io::Error> {
        for (i, line) in string.lines().enumerate() {
            out.queue(cursor::MoveTo(x, y + i as u16))?;
            print!("{}", line);
        }
        Ok(())
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

    fn quick_stack_to_foundations(&mut self) {
        let mut source_column = 0;
        let mut target_column = 0;

        'outer: while source_column < self.board.field.len() {
            while target_column < SUITS {
                if self.move_is_valid(source_column, target_column) {
                    self.player_try_execute_move(source_column, target_column);
                    // Reset the loop to check the new board state for more opportunities
                    source_column = 0;
                    target_column = 0;
                    continue 'outer;
                }
                target_column += 1;
            }
            target_column = 0;
            source_column += 1;
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
        if from == to {return false;};
        let from_top_card = if self.board.field_lengths[from as usize] > 0 {self.board.field[from as usize][self.board.field_lengths[from as usize] - 1]} else {Card::default()};
        let to_top_card = if self.board.field_lengths[to as usize] > 0 {self.board.field[to as usize][self.board.field_lengths[to as usize] - 1]} else {Card::default()};
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

fn run() -> Result<(), io::Error> {
    // Prepare terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    // Create game
    let mut rng = rand::thread_rng();
    let mut game = Game::new(&mut rng);
    game.print(&mut stdout)?;

    // Game loop
    loop {
        let event = event::read()?;
        match event {
            event::Event::Key(key_event) => {
                match key_event {
                    event::KeyEvent {code: event::KeyCode::Char('q'), modifiers: event::KeyModifiers::CONTROL, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        break
                    },
                    event::KeyEvent {code: event::KeyCode::Left, modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.move_cursor_left();}
                    },
                    event::KeyEvent {code: event::KeyCode::Char('a'), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.move_cursor_left();}
                    },
                    event::KeyEvent {code: event::KeyCode::Right, modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.move_cursor_right();}
                    },
                    event::KeyEvent {code: event::KeyCode::Char('d'), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.move_cursor_right();}
                    },
                    event::KeyEvent {code: event::KeyCode::Char(' '), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.handle_card_press();}
                    },
                    event::KeyEvent {code: event::KeyCode::Enter, modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        if !game.won {game.handle_card_press();}
                    },
                    event::KeyEvent {code: event::KeyCode::Char('z'), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        game.perform_undo();
                    },
                    event::KeyEvent {code: event::KeyCode::Char('h'), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        game.toggle_high_contrast();
                    },
                    event::KeyEvent {code: event::KeyCode::Char('f'), modifiers: event::KeyModifiers::NONE, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        game.quick_stack_to_foundations();
                    },
                    event::KeyEvent {code: event::KeyCode::Char('n'), modifiers: event::KeyModifiers::CONTROL, kind: event::KeyEventKind::Press | event::KeyEventKind::Repeat, state: _} => {
                        game = Game::new(&mut rng);
                    },
                    _ => {
                        
                    }
                }
            },
            event::Event::Resize(_term_width, _term_height) => {
                // Resize event falls through and triggers game to print again
            }
            _ => {}
        }
        game.print(&mut stdout)?;
    }
    Ok(())
}

fn cleanup() {
    let mut stdout = stdout();
    // Do not catch errors here. By the time we cleanup, we want to execute as many of these as possible to reset the terminal.
    let _ = stdout.execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::All));
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //std::env::set_var("RUST_BACKTRACE", "1");
    let (term_width, term_height) = terminal::size().unwrap();
    if term_width < MIN_TERMINAL_WIDTH || term_height < MIN_TERMINAL_HEIGHT {
        println!("Your terminal window is too small for FreeCell! It's gotta be at least {} chars wide and {} chars tall.", MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT);
        return Err("terminal too small".into());
    }
    run()?;
    cleanup();
    Ok(())
}
