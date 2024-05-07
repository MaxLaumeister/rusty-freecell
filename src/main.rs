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
    freecells: [Option<Card>; FREE_CELLS as usize],
    tableau: [[Option<Card>; DECK_SIZE as usize]; TABLEAU_SIZE as usize],
    tableau_lengths: [usize; TABLEAU_SIZE as usize]
}

impl Board {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Board {
        let mut board = Board {
            foundations: [None; SUITS as usize],
            freecells: [None; FREE_CELLS as usize],
            tableau: [[None; DECK_SIZE as usize]; TABLEAU_SIZE as usize],
            tableau_lengths: [0; TABLEAU_SIZE as usize]
        };

        let mut deck = Deck::standard();
        deck.shuffle(rng);

        // Deal out onto the board
        let mut column: i8 = 0;
        for card in deck.cards {
            board.put_on_tableau(card, column as usize);
            column += 1;
            if column >= TABLEAU_SIZE {
                column = 0;
            }
        }

        board
    }

    fn put_on_tableau(&mut self, c: Card, n: usize) {
        self.tableau[n][self.tableau_lengths[n]] = Some(c);
        self.tableau_lengths[n] += 1;
    }
}

struct Game {
    board: Board,
    display_buf: [[char; TERM_WIDTH]; TERM_HEIGHT]
}

impl Game {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let mut game = Game {
            board: Board::new(rng),
            display_buf: [['X'; TERM_WIDTH]; TERM_HEIGHT]
        };
        game
    }
    fn print(&self) {
        for line in self.display_buf {
            for character in line {
                print!("{}", character);
            }
            println!();
        }
    }
    fn print_placeholder (&mut self, x: i8, y: i8) {
        let pl_str = " ------ ";
    }
    fn print_chars_at_location (&mut self, x: i8, y: i8, chars: &[char]) {
        for c in chars {
            let mut count: usize = 0;
            self.display_buf[y as usize][x as usize + count] = *c;
            count += 1;
        }
    }
}

fn main() {
    println!("Welcome to Rust");
    let card1 = Card {rank: 10, suit: CLUBS};
    println!("Your Card: {}", card1);

    let mut deck1 = Deck::standard();
    println!("Your Deck: {}", deck1);
    let mut rng = rand::thread_rng();
    deck1.shuffle(&mut rng);
    println!("Shuffled Deck: {}", deck1);

    let array1 = [10; 5];
    println!("el 0: {}", array1[0]);
    println!("el 1: {}", array1[1]);

    // Create game
    let game = Game::new(&mut rng);
    game.print();
}
