use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, execute, style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{size, Clear, ClearType, ScrollUp, SetSize}, ExecutableCommand
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
    fn print(&self, out: &Stdout) {
        // Print foundations

        for (x, &card) in self.board.field[0..SUITS as usize].iter().enumerate() {
            Game::print_card_at_coord(&out, x * CARD_WIDTH, 1, card[self.board.field_lengths[x]], self.highlighted_card as usize == x);
        }

        // Print freecells

        for (x, &card) in self.board.field[SUITS as usize .. (SUITS + FREE_CELLS) as usize].iter().enumerate() {
            Game::print_card_at_coord(&out, SUITS as usize * CARD_WIDTH + x * CARD_WIDTH + 2, 1, card[self.board.field_lengths[x + SUITS as usize]], self.highlighted_card as usize == x + SUITS as usize);
        }

        // Print tableau

        for (x, &column) in self.board.field[(SUITS + FREE_CELLS) as usize ..].iter().enumerate() {
            for (y, &card) in column.iter().enumerate() {
                match card {
                    Some(_) => {
                        Game::print_card_at_coord(&out, x * CARD_WIDTH + 1, y * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 2, card, (self.highlighted_card as usize == x + (SUITS + FREE_CELLS) as usize) && y == self.board.field_lengths[x + (SUITS + FREE_CELLS) as usize] - 1);
                    }
                    None => {

                    }
                }
            }
        }

        // Print title bar
        let _ = stdout().execute(cursor::MoveTo(0, 0));
        println!("--- Rusty FreeCell ---------------------------------------");

        // Print bottom bar
        let _ = stdout().execute(cursor::MoveTo(0, TERM_HEIGHT as u16));
        println!("--- (q)uit -----------------------------------------------");
    }

    fn print_card_at_coord(out: &Stdout, x: usize, y: usize, card: Option<Card>, highlighted: bool) {
        let mut stdout = stdout();
        let card_str = match card {
            Some(card) => format!("{}{}", match card.rank {
                1 => "1",
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
        let pl_str;
        if highlighted {
            pl_str = format!("\
            \x20OOOOO \n\
               O {: <3} O\n\
               O     O\n\
               O     O\n\
            \x20OOOOO \n", card_str);
        } else {
            pl_str = format!("\
            \x20----- \n\
               | {: <3} |\n\
               |     |\n\
               |     |\n\
            \x20----- \n", card_str);
        }
        for (d, line) in pl_str.lines().enumerate() {
            let _ = stdout.execute(cursor::MoveTo(x as u16, y as u16 + d as u16));
            match card {
                Some(c) => match c.suit {
                    HEARTS | DIAMONDS => {
                        // Print red card
                        let _ = stdout.execute(style::SetForegroundColor(style::Color::Red));
                        println!("{}", line);
                        let _ = stdout.execute(style::ResetColor);
                    }
                    _ => {
                        // Print black card
                        println!("{}", line);
                    }
                }
                None => {
                    // Print "placeholder" card
                    println!("{}", line);
                }
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
        if self.highlighted_card == -1 {
            self.selected_card = self.highlighted_card;
        } else {
            self.execute_move(self.selected_card, self.highlighted_card);
        }
    }
    fn execute_move(&mut self, from: i8, to: i8) {
        // let &from_ptr;
        // if from < SUITS + FREE_CELLS {
        //     from_ptr = self.
        // }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    
    // Prepare terminal
    crossterm::terminal::enable_raw_mode()?;
    let _ = stdout().execute(cursor::Hide);
    let mut stdout = stdout();
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
