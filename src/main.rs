// #TODO eliminate terminal flicker
// #TODO error handling
// #TODO moving multiple cards at once (shortcut)
// #TODO move count
// #TODO win screen
// #TODO decorate foundations with suits
// #TODO condense top row representation when terminal is small, expand when large
// #TODO refactor, ci, lint, publish

use std::{cmp, io::{stdout, Stdout, Write}};

use circular_buffer::CircularBuffer;
use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, execute, style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{size, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen, ScrollUp, SetSize}, ExecutableCommand, QueueableCommand
};

const RANKS: usize = 13;
const SUITS: usize = 4;
const DECK_SIZE: usize = RANKS* SUITS;

const HEARTS: i8 = 1;
const CLUBS: i8 = 2;
const DIAMONDS: i8 = 3;
const SPADES: i8 = 4;

const FREE_CELLS: usize = 4;
const TABLEAU_SIZE: usize = 8;

const TERM_WIDTH: usize = 80;
const TERM_HEIGHT: usize = 24;

const CARD_WIDTH: usize = 7;
const CARD_HEIGHT: usize = 5;
const TABLEAU_VERTICAL_OFFSET: usize = 2;

const UNDO_LEVELS: usize = 1000;

#[derive(Default, Copy, Clone)]
struct Card {
    rank: i8,
    suit: i8
}

struct Deck {
    cards: [Card; DECK_SIZE as usize]
}

#[derive(Default, Copy, Clone)]
struct Move {
    from: usize,
    to: usize
}

impl Deck {
    fn standard() -> Deck {
        let mut deck = Deck {cards: [Card {rank: 0, suit: 0}; DECK_SIZE as usize]};
        for r in 0..RANKS {
            for s in 0..SUITS {
                deck.cards[(s*RANKS+r) as usize] = Card{rank: (r+1) as i8, suit: (s+1) as i8};
            }
        }
        deck
    }
    fn shuffle(&mut self, rng: &mut rand::rngs::ThreadRng) {
        use rand::seq::SliceRandom;
        self.cards.shuffle(rng);
    }
}

impl std::fmt::Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for i in &self.cards {
            write!(f, "{} ", i)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let suit_str = match self.suit {
            0 => "h",
            1 => "d",
            2 => "c",
            3 => "s",
            _ => "x"
        };
        write!(f, "({}, {})", self.rank, suit_str)
    }
}

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
    undo_history: CircularBuffer::<UNDO_LEVELS, Move>
}

impl Game {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let game = Game {
            board: Board::new(rng),
            highlighted_card: SUITS + FREE_CELLS,
            selected_card_opt: None,
            undo_history: CircularBuffer::new()
        };
        game
    }
    fn print(&self, mut out: &Stdout) {
        let _ = out.queue(Clear(ClearType::All));

        for i in 0..(SUITS+FREE_CELLS+TABLEAU_SIZE) as usize {
            let stack = self.board.field[i];
            let stack_size = self.board.field_lengths[i];
            let top_card = self.board.field[i][if stack_size == 0 {0} else {stack_size - 1}];
            if i < SUITS as usize {
                // Print foundation
                Game::print_card_at_coord(out, i * CARD_WIDTH, 1, top_card, self.highlighted_card as usize == i, self.selected_card_opt == Some(i));
            } else if i < (SUITS + FREE_CELLS) as usize {
                // Print free cell
                Game::print_card_at_coord(out, i * CARD_WIDTH + 2, 1, top_card, self.highlighted_card as usize == i as usize, self.selected_card_opt == Some(i));
            } else if i < (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize {
                // Print tableau
                for (y, &card) in stack.iter().enumerate() {
                    // if no card there, continue
                    if card.rank == 0 {continue;};
                    Game::print_card_at_coord(out, (i - (SUITS + FREE_CELLS) as usize) * CARD_WIDTH + 1, y * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 1, card, (self.highlighted_card as usize == i as usize) && y == self.board.field_lengths[i as usize] - 1, (self.selected_card_opt == Some(i)) && y == self.board.field_lengths[i as usize] - 1);
                }
            }
        }
        
        // Print title bar
        let _ = out.queue(cursor::MoveTo(0, 0));
        print!("--- Rusty FreeCell ---------------------------------------");

        // Print bottom bar
        let _ = out.queue(cursor::MoveTo(0, TERM_HEIGHT as u16));
        print!("--- (New Game: ctrl-n) - (Undo: z) - (Quit: q) -----------");

        let _ = out.flush();
    }

    fn print_card_at_coord(mut out: &Stdout, x: usize, y: usize, card: Card, highlighted: bool, selected: bool) {
        let card_suit_rank_str =
            format!("{}{}", match card.rank {
                0 => " ",
                1 => "A",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                10 => "10",
                11 => "J",
                12 => "Q",
                13 => "K",
                _ => "e"
            }, match card.suit {
                0 => " ",
                HEARTS => "♥",
                CLUBS => "♣",
                DIAMONDS => "♦",
                SPADES => "♠",
                _ => "e"
            });

        let card_display_str;
        if selected {
            card_display_str= format!("\
                ╭─────╮\n\
                │ {: <3} │\n\
                │     │\n\
                │  △  │\n\
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
            if y+d >= TERM_HEIGHT {break};
            let _ = out.queue(cursor::MoveTo(x as u16, y as u16 + d as u16));
            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::Reverse));
            }
            match card.suit {
                HEARTS | DIAMONDS => {
                    // Print red card
                    let _ = out.queue(style::SetForegroundColor(style::Color::Red));
                    print!("{}", line);
                    let _ = out.queue(style::ResetColor);
                }
                _ => {
                    // Print black or placeholder card
                    print!("{}", line);
                }
            }
            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::NoReverse));
            }
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
            None => ()
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
            None => ()
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
                    return from_top_card.rank == to_top_card.rank + 1;
            } else {
                    return from_top_card.rank == 1;
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
    }
    fn player_try_execute_move(&mut self, from: usize, to: usize) {
        if self.move_is_valid(from, to) {
            // Execute move, add to undo history
            self.execute_move(from, to);
            self.undo_history.push_back(Move{from: from, to: to});

        }
    }
    fn perform_undo(&mut self) {
        let last_move_opt = self.undo_history.pop_back();
        match last_move_opt {
            Some(last_move) => {
                // Perform move in reverse, without checking if it follows the rules
                self.execute_move(last_move.to, last_move.from);
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
    game.print(&stdout);

    // Game loop
    loop {
        if let Event::Key(event) = event::read().expect("Failed to read line") {
            match event {
                KeyEvent {code: KeyCode::Char('q'), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    break
                },
                KeyEvent {code: KeyCode::Left, modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_cursor_left();
                },
                KeyEvent {code: KeyCode::Char('a'), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_cursor_left();
                },
                KeyEvent {code: KeyCode::Right, modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_cursor_right();
                },
                KeyEvent {code: KeyCode::Char('d'), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_cursor_right();
                },
                KeyEvent {code: KeyCode::Char(' '), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.handle_card_press();
                },
                KeyEvent {code: KeyCode::Enter, modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.handle_card_press();
                },
                KeyEvent {code: KeyCode::Char('z'), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.perform_undo();
                },
                KeyEvent {code: KeyCode::Char('n'), modifiers: event::KeyModifiers::CONTROL, kind: _, state: _} => {
                    game = Game::new(&mut rng);
                },
                _ => {
                    
                }
            }
            game.print(&stdout);
            //println!("{:?}", event);
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
