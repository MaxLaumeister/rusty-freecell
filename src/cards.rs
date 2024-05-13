//! Utilities for creating cards and decks

/// A struct representing a playing card with a rank and a suit.
#[derive(Default, Copy, Clone)]
pub struct Card {
    /// The rank of the card, from 1 (Ace) to 13 (King).
    pub rank: u8,

    /// The suit of the card, represented as an integer where:
    /// - 1: Hearts
    /// - 2: Clubs
    /// - 3: Diamonds
    /// - 4: Spades
    pub suit: u8
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

/// Creates a standard deck of playing cards.
///
/// # Arguments
///
/// * `ranks` - The number of ranks per suit in the deck.
/// * `suits` - The number of suits in the deck.
///
/// # Returns
///
/// A vector containing a standard deck of cards with ranks ranging from 1 to `ranks` and suits ranging from 1 to `suits`.
pub fn new_standard_deck(ranks: u8, suits: u8) -> Vec<Card> {
        (0..ranks*suits).map(
            |i|
            Card {
                rank: (i % ranks + 1),
                suit: (i / ranks + 1)
            }
        ).collect()
}
