use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, execute, style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{size, Clear, ClearType, ScrollUp, SetSize}, ExecutableCommand
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
    foundations: [Option<Card>; SUITS as usize],
    free_cells: [Option<Card>; FREE_CELLS as usize],
    tableau: [[Option<Card>; DECK_SIZE as usize]; TABLEAU_SIZE as usize],
    tableau_lengths: [usize; TABLEAU_SIZE as usize]
}

impl Board {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Board {
        let mut board = Board {
            foundations: [None; SUITS as usize],
            free_cells: [None; FREE_CELLS as usize],
            tableau: [[None; DECK_SIZE as usize]; TABLEAU_SIZE as usize],
            tableau_lengths: [0; TABLEAU_SIZE as usize]
        };

        let mut deck = Deck::standard();
        deck.shuffle(rng);

        // Deal out onto the board
        let mut column = 0;
        for card in deck.cards {
            board.put_on_tableau(card, column as usize);
            column += 1;
            if column >= TABLEAU_SIZE {
                column = 0;
            }
        }

        board
    }

    fn put_on_tableau(&mut self, c: Card, column: usize) {
        //println!("putting card at {}", column);
        self.tableau[column][self.tableau_lengths[column]] = Some(c);
        self.tableau_lengths[column] += 1;
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
        // Write to buffer

        // Write foundations

        for (x, &card) in self.board.foundations.iter().enumerate() {
            Game::print_card_at_coord(&out, x * CARD_WIDTH, 1, card, false);
        }

        // Write freecells

        for (x, &card) in self.board.free_cells.iter().enumerate() {
            Game::print_card_at_coord(&out, SUITS as usize * CARD_WIDTH + x * CARD_WIDTH + 2, 1, card, false);
        }

        // Write tableau

        for (x, &column) in self.board.tableau.iter().enumerate() {
            for (y, &card) in column.iter().enumerate() {
                if y >= self.board.tableau_lengths[x] {
                    continue;
                }
                Game::print_card_at_coord(&out, x * CARD_WIDTH + 1, y * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 2, card, false);
            }
        }

        // Write currently highlighted card
        Game::print_top_card_at_index(self, out, self.highlighted_card as usize, true);

        // Write selected card, if any
        if self.selected_card > 0 {
            Game::print_top_card_at_index(self, out, self.selected_card as usize, true);
        }
    }
    fn print_top_card_at_index(&self, out: &Stdout, mut idx: usize, selected: bool) {
        if idx < SUITS as usize {
            Game::print_card_at_coord(&out, idx * CARD_WIDTH, 1, self.board.foundations[idx], selected);
        } else if idx < SUITS as usize + FREE_CELLS as usize {
            idx = idx % SUITS as usize;
            Game::print_card_at_coord(&out, SUITS as usize * CARD_WIDTH + idx * CARD_WIDTH + 2, 1, self.board.free_cells[idx], selected);
        } else if idx < SUITS as usize + FREE_CELLS as usize + TABLEAU_SIZE as usize {
            idx = idx % (SUITS as usize + FREE_CELLS as usize);
            Game::print_card_at_coord(&out, idx * CARD_WIDTH + 1, (self.board.tableau_lengths[idx] - 1) * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 2, self.board.tableau[idx][self.board.tableau_lengths[idx] - 1], selected);
        }
    }
    fn print_card_at_coord(out: &Stdout, x: usize, y: usize, card: Option<Card>, selected: bool) {

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
                HEARTS => "h",
                CLUBS => "c",
                DIAMONDS => "d",
                SPADES => "s",
                _ => "X"
            }),
            None => "--".to_string()
        };
        let pl_str;
        if selected {
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
            let _ = stdout().execute(cursor::MoveTo(x as u16, y as u16 + d as u16));
            print!("{}", line);

            // let _ = stdout().execute(cursor::MoveTo(0, 0));
            // print!("hello world");
            // let _ = stdout().execute(cursor::MoveTo(0, 1));
            // print!("hello two");
        }
        // let _ = stdout().execute(cursor::MoveTo(0, 0));
        // println!("hello world");
    }

    fn move_left(&mut self) {
        if self.highlighted_card == 0 {
            self.highlighted_card = SUITS + FREE_CELLS + TABLEAU_SIZE - 1;
        } else {
            self.highlighted_card -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.highlighted_card >= SUITS + FREE_CELLS + TABLEAU_SIZE - 1 {
            self.highlighted_card = 0;
        } else {
            self.highlighted_card += 1;
        }
    }
    fn select_current_card(&mut self) {
        self.selected_card = self.highlighted_card;
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    
    // Prepare terminal
    crossterm::terminal::enable_raw_mode()?;
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
                    game.move_left();
                },
                KeyEvent {code: KeyCode::Right, modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.move_right();
                },
                KeyEvent {code: KeyCode::Char(' '), modifiers: event::KeyModifiers::NONE, kind: _, state: _} => {
                    game.select_current_card();
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
    crossterm::terminal::disable_raw_mode().unwrap_or_else(|_| panic!());
    println!();
}

fn main() -> impl std::process::Termination {
    let _ = run();
    cleanup();
}
