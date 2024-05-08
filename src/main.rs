use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, execute, style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{size, Clear, ClearType, ScrollUp, SetSize}, ExecutableCommand, QueueableCommand
};

const RANKS: i8 = 13;
const SUITS: i8 = 4;
const DECK_SIZE: i8 = RANKS* SUITS;

const HEARTS: i8 = 0;
const CLUBS: i8 = 1;
const DIAMONDS: i8 = 2;
const SPADES: i8 = 3;

const FREE_CELLS: i8 = 4;
const TABLEAU_SIZE: i8 = 8;

const TERM_WIDTH: usize = 80;
const TERM_HEIGHT: usize = 24;

const CARD_WIDTH: usize = 7;
const CARD_HEIGHT: usize = 5;
const TABLEAU_VERTICAL_OFFSET: usize = 2;

#[derive(Copy, Clone)]
struct Card {
    rank: i8,
    suit: i8
}

struct Deck {
    cards: [Card; DECK_SIZE as usize]
}

impl Deck {
    fn standard() -> Deck {
        let mut deck = Deck {cards: [Card {rank: 0, suit: 0}; DECK_SIZE as usize]};
        for r in 0..RANKS {
            for s in 0..SUITS {
                deck.cards[(s*RANKS+r) as usize] = Card{rank: r+1, suit: s};
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
    field: [[Option<Card>; DECK_SIZE as usize]; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize],
    field_lengths: [usize; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize]
}

impl Board {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Board {
        let mut board = Board {
            field: [[None; DECK_SIZE as usize]; (SUITS + FREE_CELLS + TABLEAU_SIZE) as usize],
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
        self.field[field_column][self.field_lengths[field_column]] = Some(c);
        self.field_lengths[field_column] += 1;
    }
}

struct Game {
    board: Board,
    highlighted_card: i8,
    selected_card: i8
}

impl Game {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let game = Game {
            board: Board::new(rng),
            highlighted_card: SUITS + FREE_CELLS,
            selected_card: -1
        };
        game
    }
    fn print(&self, mut out: &Stdout) {
        // Clear

        let _ = out.queue(Clear(ClearType::All));
        
        // Print foundations

        for (x, &stack) in self.board.field[0..SUITS as usize].iter().enumerate() {
            Game::print_card_at_coord(out, x * CARD_WIDTH, 1, stack[self.board.field_lengths[x]], self.highlighted_card as usize == x, self.selected_card as usize == x);
        }

        // Print freecells

        for (x, &stack) in self.board.field[SUITS as usize .. (SUITS + FREE_CELLS) as usize].iter().enumerate() {
            Game::print_card_at_coord(out, SUITS as usize * CARD_WIDTH + x * CARD_WIDTH + 2, 1, stack[self.board.field_lengths[x + SUITS as usize]], self.highlighted_card as usize == x + SUITS as usize, self.selected_card as usize == x + SUITS as usize);
        }

        // Print tableau

        for (x, &stack) in self.board.field[(SUITS + FREE_CELLS) as usize ..].iter().enumerate() {
            for (y, &card) in stack.iter().enumerate() {
                match card {
                    Some(_) => {
                        Game::print_card_at_coord(out, x * CARD_WIDTH + 1, y * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 1, card, (self.highlighted_card as usize == x + (SUITS + FREE_CELLS) as usize) && y == self.board.field_lengths[x + (SUITS + FREE_CELLS) as usize] - 1, (self.selected_card as usize == x + (SUITS + FREE_CELLS) as usize) && y == self.board.field_lengths[x + (SUITS + FREE_CELLS) as usize] - 1);
                    }
                    None => {

                    }
                }
            }
        }

        // Print title bar
        let _ = out.queue(cursor::MoveTo(0, 0));
        print!("--- Rusty FreeCell ---------------------------------------");

        // Print bottom bar
        let _ = out.queue(cursor::MoveTo(0, TERM_HEIGHT as u16));
        print!("--- (q)uit -----------------------------------------------");

        let _ = out.flush();
    }

    fn print_card_at_coord(mut out: &Stdout, x: usize, y: usize, card: Option<Card>, highlighted: bool, selected: bool) {
        let card_suit_rank_str = match card {
            Some(card) => format!("{}{}", match card.rank {
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
                _ => "X"
            }, match card.suit {
                HEARTS => "♠",
                CLUBS => "♥",
                DIAMONDS => "♦",
                SPADES => "♣",
                _ => "X"
            }),
            None => "  ".to_string()
        };

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
            match card {
                Some(c) => match c.suit {
                    HEARTS | DIAMONDS => {
                        // Print red card
                        let _ = out.queue(style::SetForegroundColor(style::Color::Red));
                        print!("{}", line);
                        let _ = out.queue(style::ResetColor);
                    }
                    _ => {
                        // Print black card
                        print!("{}", line);
                    }
                }
                None => {
                    // Print "placeholder" card
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
    }

    fn move_cursor_right(&mut self) {
        if self.highlighted_card >= SUITS + FREE_CELLS + TABLEAU_SIZE - 1 {
            self.highlighted_card = 0;
        } else {
            self.highlighted_card += 1;
        }
    }
    fn handle_card_press(&mut self) {
        if self.selected_card == -1 {
            // Select a card
            self.selected_card = self.highlighted_card;
        } else if self.highlighted_card == self.selected_card {
            // Deselect a card
            self.selected_card = -1;
        } else {
            // Execute a move
            self.try_execute_move(self.selected_card, self.highlighted_card);
        }
    }
    fn are_opposite_colors(card1: Card, card2: Card) -> bool {
        if (card1.suit == HEARTS || card1.suit == DIAMONDS) {return card2.suit == SPADES || card2.suit == CLUBS};
        if (card1.suit == SPADES || card1.suit == CLUBS) {return card2.suit == HEARTS || card2.suit == DIAMONDS};
        return false;
    }
    fn move_is_valid(&self, from: i8, to: i8) -> bool {
        let from_top_card_op = if self.board.field_lengths[from as usize] > 0 {self.board.field[from as usize][self.board.field_lengths[from as usize] - 1]} else {None};
        let to_top_card_op = if self.board.field_lengths[to as usize] > 0 {self.board.field[to as usize][self.board.field_lengths[to as usize] - 1]} else {None};
        match from_top_card_op {
            Some(from_top_card) => {
                if from < SUITS {
                    // Foundation case
                    match to_top_card_op {
                        Some(to_top_card) => {
                            return from_top_card.rank == to_top_card.rank + 1;
                        }
                        None => {
                            return from_top_card.rank == 1;
                        }
                    }
                } else if SUITS < from && from < SUITS + FREE_CELLS {
                    // Free cell case
                    match to_top_card_op {
                        Some(to_top_card) => {
                            return false;
                        }
                        None => {
                            return true;
                        }
                    }
                } else if SUITS + FREE_CELLS < from && from < SUITS + FREE_CELLS + TABLEAU_SIZE {
                    // Tableau case
                    match to_top_card_op {
                        Some(to_top_card) => {
                            return from_top_card.rank == to_top_card.rank - 1 && Game::are_opposite_colors(from_top_card, to_top_card);
                        }
                        None => {
                            return true;
                        }
                    }
                }
            }
            None => {
                return false;
            }
        }
        return false;
    }
    fn execute_move (&mut self, from: i8, to: i8) {
        self.board.field[to as usize][self.board.field_lengths[to as usize]] = self.board.field[from as usize][self.board.field_lengths[from as usize] - 1];
        self.board.field[from as usize][self.board.field_lengths[from as usize] - 1] = Default::default();
        self.board.field_lengths[from as usize] -= 1;
        self.board.field_lengths[to as usize] += 1;
        self.selected_card = -1;
    }
    fn try_execute_move(&mut self, from: i8, to: i8) {
        if self.move_is_valid(from, to) {
            self.execute_move(from, to);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    
    // Prepare terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    let _ = stdout.execute(cursor::Hide);
    stdout.execute(Clear(ClearType::All))?;

    // stdout.execute(SetForegroundColor(Color::Blue))?;
    // stdout.execute(SetBackgroundColor(Color::Red))?;
    // stdout.execute(Print("Styled text here."))?;
    // stdout.execute(ResetColor)?;

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
                KeyEvent {code: KeyCode::Right, modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_cursor_right();
                },
                KeyEvent {code: KeyCode::Char(' '), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.handle_card_press();
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
    println!();
}

fn main() -> impl std::process::Termination {
    std::env::set_var("RUST_BACKTRACE", "1");
    let _ = run();
    cleanup();
}
