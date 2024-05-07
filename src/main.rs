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
    selected_card: usize
}

impl Game {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let game = Game {
            board: Board::new(rng),
            selected_card: SUITS as usize + FREE_CELLS as usize
        };
        game
    }
    fn print(&self) {
        let mut display_buf: [String; TERM_HEIGHT] = core::array::from_fn(|_| " ".repeat(TERM_WIDTH).to_string());
        // Write to buffer

        // Write foundations

        for (x, &card) in self.board.foundations.iter().enumerate() {
            self.print_card(&mut display_buf, x * CARD_WIDTH, 1, card, false);
        }

        // Write freecells

        for (x, &card) in self.board.free_cells.iter().enumerate() {
            self.print_card(&mut display_buf, SUITS as usize * CARD_WIDTH + x * CARD_WIDTH + 2, 1, card, false);
        }

        // Write tableau

        for (x, &column) in self.board.tableau.iter().enumerate() {
            for (y, &card) in column.iter().enumerate() {
                if y >= self.board.tableau_lengths[x] {
                    continue;
                }
                self.print_card(&mut display_buf, x * CARD_WIDTH + 1, y * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 2, card, false);
            }
        }

        // Write currently selected card
        let mut idx = self.selected_card;
        if idx < SUITS as usize {
            self.print_card(&mut display_buf, idx * CARD_WIDTH, 1, self.board.foundations[idx], true);
        } else if idx < SUITS as usize + FREE_CELLS as usize {
            idx = idx % SUITS as usize;
            self.print_card(&mut display_buf, SUITS as usize * CARD_WIDTH + idx * CARD_WIDTH + 2, 1, self.board.free_cells[idx], true);
        } else if idx < SUITS as usize + FREE_CELLS as usize + TABLEAU_SIZE as usize {
            idx = idx % (SUITS as usize + FREE_CELLS as usize);
            self.print_card(&mut display_buf, idx * CARD_WIDTH + 1, (self.board.tableau_lengths[idx] - 1) * TABLEAU_VERTICAL_OFFSET + CARD_HEIGHT + 2, self.board.tableau[idx][self.board.tableau_lengths[idx] - 1], true);
        }

        // Print buffer

        for line in display_buf {
            println!("{}", line);
        }
    }
    fn print_card(&self, buffer: &mut [String; TERM_HEIGHT],x: usize, y: usize, card: Option<Card>, selected: bool) {
        // println!("Printing Card {} At Location: ({}, {})", match card {
        //     Some(card) => card.rank,
        //     None => 0
        // }, x, y);
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
        self.print_chars_at_location(buffer, x, y, pl_str.as_str());
    }
    fn print_chars_at_location (&self, buffer: &mut [String; TERM_HEIGHT], x: usize, y: usize, lines_to_write: &str) {
        for (i, line_to_write) in lines_to_write.lines().enumerate() {
            if y as usize + i >= buffer.len() {
                println!("break");
                break;
            }
            if x as usize + line_to_write.len() >= buffer[y as usize + i].len() {
                println!("continue");
                continue;
            }
            buffer[y as usize + i].replace_range(x as usize..(line_to_write.len() + (x as usize)), line_to_write);
        }
    }
}

fn main() {
    //println!("Welcome to Rust");
    //let card1 = Card {rank: 10, suit: CLUBS};
    //println!("Your Card: {}", card1);

    //let mut deck1 = Deck::standard();
    //println!("Your Deck: {}", deck1);
    // deck1.shuffle(&mut rng);
    //println!("Shuffled Deck: {}", deck1);

    // let array1 = [10; 5];
    // println!("el 0: {}", array1[0]);
    // println!("el 1: {}", array1[1]);

    // Create game
    let mut rng = rand::thread_rng();
    let game = Game::new(&mut rng);
    game.print();
}
