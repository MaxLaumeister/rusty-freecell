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

#[derive(Copy, Clone)]
pub struct CardStack {
    cards: [Card; crate::DECK_SIZE as usize],
    length: usize
}

impl Default for CardStack {
    fn default() -> Self {
        CardStack {
            cards: [Card::default(); crate::DECK_SIZE],
            length: 0,
        }
    }
}

impl CardStack {
    pub fn push(&mut self, card: Card) {
        self.cards[self.length] = card;
        self.length += 1;
    }
    pub fn peek(self) -> Option<Card> {
        if self.length == 0 {
            None
        } else {
            Some(self.cards[self.length - 1])
        }
    }
    pub fn pop(&mut self) -> Option<Card> {
        let card_opt = self.peek();
        if self.length != 0 {
            self.length -= 1;
        }
        card_opt
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub fn size(&self) -> usize {
        self.length
    }
    pub fn new_standard_deck() -> CardStack {
        CardStack {
            cards: core::array::from_fn(|i| Card {
                rank: (i % crate::RANKS + 1) as u8,
                suit: (i / crate::RANKS + 1) as u8
            }),
            length: crate::DECK_SIZE
        }
    }
    pub fn shuffle(&mut self, rng: &mut rand::rngs::ThreadRng) {
        use rand::seq::SliceRandom;
        self.cards.shuffle(rng);
    }
}

impl IntoIterator for CardStack {
    type Item = Card;
    type IntoIter = CardStackIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        CardStackIntoIterator {
            card_stack: self,
            index: 0,
        }
    }
}

pub struct CardStackIntoIterator {
    card_stack: CardStack,
    index: usize,
}

impl Iterator for CardStackIntoIterator {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        if self.index < self.card_stack.length {
            let card = self.card_stack.cards[self.index];
            self.index += 1;
            Some(card)
        } else {
            None
        }
    }
}
