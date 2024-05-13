
use circular_buffer::CircularBuffer;

use rand::seq::SliceRandom;

use crate::cards::{new_standard_deck, Card};

const RANKS: usize = 13;
const SUITS: usize = 4;
const DECK_SIZE: usize = RANKS* SUITS;

const HEARTS: u8 = 1;
const CLUBS: u8 = 2;
const DIAMONDS: u8 = 3;
const SPADES: u8 = 4;

const FREE_CELLS: usize = 4;
const TABLEAU_SIZE: usize = 8;
const FIELD_SIZE: usize = SUITS + FREE_CELLS + TABLEAU_SIZE;

const UNDO_LEVELS: usize = 1000;

#[derive(Default, Copy, Clone)]
struct Move {
    from: u8,
    to: u8
}

pub struct Game {
    field: [Vec<Card>; FIELD_SIZE],
    highlighted_card: usize,
    selected_card_opt: Option<usize>,
    undo_history: CircularBuffer::<UNDO_LEVELS, Move>,
    move_count: u32,
    high_contrast: bool,
    won: bool
}

impl Game {
    pub fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let mut game = Game {
            field: core::array::from_fn(|_| Vec::with_capacity(DECK_SIZE)),
            highlighted_card: SUITS + FREE_CELLS,
            selected_card_opt: None,
            undo_history: CircularBuffer::new(),
            move_count: 0,
            high_contrast: false,
            won: false
        };

        // Deal deck onto the board
        let mut deck = new_standard_deck(RANKS, SUITS);
        deck.shuffle(rng);
        for (i, card) in deck.into_iter().enumerate() {
            let field_column = SUITS + FREE_CELLS + (i % TABLEAU_SIZE);
            game.field[field_column].push(card);
        }

        game
    }
    pub fn is_won(&self) -> bool {
        self.won
    }
    
    pub fn toggle_high_contrast(&mut self) {
        self.high_contrast = !self.high_contrast;
    }

    pub fn move_cursor_left(&mut self) {
        // this modulo trick avoids negative numbers on the unsigned int
        self.highlighted_card = (self.highlighted_card + FIELD_SIZE - 1) % FIELD_SIZE;

        match self.selected_card_opt {
            Some(selected_card) => {
                while !self.move_is_valid(selected_card, self.highlighted_card) && selected_card != self.highlighted_card {
                    self.move_cursor_left();
                }
            }
            None => {
                while self.field[self.highlighted_card].last() == None {
                    self.move_cursor_left();
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        self.highlighted_card = (self.highlighted_card + 1) % FIELD_SIZE;

        match self.selected_card_opt {
            Some(selected_card) => {
                while !self.move_is_valid(selected_card, self.highlighted_card) && selected_card != self.highlighted_card {
                    self.move_cursor_right();
                }
            }
            None => {
                while self.field[self.highlighted_card].last() == None {
                    self.move_cursor_right();
                }
            }
        }
    }

    pub fn quick_stack_to_foundations(&mut self) {
        let mut made_move = false;

        'outer: for source_column in 0..self.field.len() {
            for target_column in 0..SUITS {
                if self.move_is_valid(source_column, target_column) {
                    self.player_try_execute_move(source_column, target_column);
                    made_move = true;
                    break 'outer;
                }
            }
        }
        // If we made a move, check the new board state for more opportunities
        if made_move {self.quick_stack_to_foundations()};
    }

    pub fn handle_card_press(&mut self) {
        if self.selected_card_opt == None {
            // Select a card
            self.selected_card_opt = Some(self.highlighted_card);
        } else if Some(self.highlighted_card) == self.selected_card_opt {
            // Deselect a card
            self.selected_card_opt = None;
        } else {
            // Execute a move
            match self.selected_card_opt {
                Some(selected_card) => {
                    self.player_try_execute_move(selected_card, self.highlighted_card);
                }
                None => {

                }
            }
        }
    }
    pub fn player_try_execute_move(&mut self, from: usize, to: usize) {
        if self.move_is_valid(from, to) {
            // Execute move, add to undo history
            self.execute_move(from, to);
            self.move_count += 1;
            self.undo_history.push_back(Move{from: from as u8, to: to as u8});
        }
    }
    pub fn perform_undo(&mut self) {
        let last_move_opt = self.undo_history.pop_back();
        match last_move_opt {
            Some(last_move) => {
                // Perform move in reverse, without checking if it follows the rules
                self.execute_move(last_move.to as usize, last_move.from as usize);
                self.move_count -= 1;
            }
            None => {
                // History empty
            }
        }
    }

    fn are_opposite_colors(card1: Card, card2: Card) -> bool {
        if card1.suit == HEARTS || card1.suit == DIAMONDS {return card2.suit == SPADES || card2.suit == CLUBS};
        if card1.suit == SPADES || card1.suit == CLUBS {return card2.suit == HEARTS || card2.suit == DIAMONDS};
        false
    }

    fn move_is_valid(&self, from: usize, to: usize) -> bool {
        if from == to {return false;};
        let from_top_card = self.field[from].last().cloned().unwrap_or_default();
        let to_top_card = self.field[to].last().cloned().unwrap_or_default();
        if to < SUITS {
            // Foundation case
            if to_top_card.rank != 0 {
                    return from_top_card.rank == to_top_card.rank + 1 && from_top_card.suit == to_top_card.suit;
            } else {
                    return from_top_card.rank == 1 && to as u8 == from_top_card.suit - 1;
            }
        } else if to < SUITS + FREE_CELLS {
            // Free cell case
            return to_top_card.rank == 0;
        } else if to < SUITS + FREE_CELLS + TABLEAU_SIZE {
            // Tableau case
            if to_top_card.rank != 0 {
                return from_top_card.rank == to_top_card.rank - 1 && Game::are_opposite_colors(from_top_card, to_top_card);
            } else {
                return true;
            }
        }
        false
    }

    fn execute_move (&mut self, from: usize, to: usize) {
        // Execute the move
        // Move "from" card to "to" column
        let from_card_opt = self.field[from].last();
        match from_card_opt {
            Some(&from_card) => {
                self.field[from].pop();
                self.field[to].push(from_card);
            },
            None => {}
        }
        self.selected_card_opt = None;
        // Check to see if player won or unwon (due to undo)
        self.won = self.field.iter().all(|stack| stack.len() == RANKS);
    }
}

mod print;
