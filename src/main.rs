const RANKS: i8 = 13;
const SUITS: i8 = 4;
const DECK_SIZE: usize = RANKS as usize * SUITS as usize;

const HEARTS: i8 = 0;
const DIAMONDS: i8 = 1;
const CLUBS: i8 = 2;
const SPADES: i8 = 3;

#[derive(Copy, Clone)]
struct Card {
    rank: i8,
    suit: i8
}

struct Deck {
    cards: [Card; DECK_SIZE]
}

impl Deck {
    fn standard() -> Deck {
        let mut deck = Deck {cards: [Card {rank: 0, suit: 0}; DECK_SIZE]};
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
}
