#[derive(Default, Copy, Clone)]
pub struct Card {
    pub rank: u8,
    pub suit: u8
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

pub fn new_standard_deck(ranks: u8, suits: u8) -> Vec<Card> {
        (0..ranks*suits).map(
            |i|
            Card {
                rank: (i % ranks + 1),
                suit: (i / ranks + 1)
            }
        ).collect()
}

// pub fn shuffle(&mut self, rng: &mut rand::rngs::ThreadRng) {
//     use rand::seq::SliceRandom;
//     self.cards.shuffle(rng);
// }

