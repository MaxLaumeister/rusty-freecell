const RANKS: i8 = 13;
const SUITS: i8 = 4;

const HEARTS: i8 = 0;
const DIAMONDS: i8 = 1;
const CLUBS: i8 = 2;
const SPADES: i8 = 3;

struct Card {
    rank: i8,
    suit: i8
}

struct Deck {
    cards: Vec<Card>
}

impl Deck {
    fn standard() -> Deck {
        let mut deck = Deck {cards: Vec::with_capacity(RANKS as usize * SUITS as usize)};
        for r in 1..RANKS+1 {
            for s in 0..SUITS {
                deck.cards.push(Card{rank: r, suit: s});
            }
        }
        deck
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
    let deck1 = Deck::standard();
    println!("Your Deck: {}", deck1);
}
