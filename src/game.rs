//! Manages the state of the `FreeCell` game

use circular_buffer::CircularBuffer;

use rand::seq::SliceRandom;

use crate::cards::{new_standard_deck, Card};

/// The total number of ranks in a standard deck of cards.
const RANKS: u8 = 13;
/// The total number of suits in a standard deck of cards.
const SUITS: u8 = 4;
/// The total number of cards in a standard deck.
const DECK_SIZE: usize = RANKS as usize * SUITS as usize;

/// Constant representing the suit of Hearts.
const HEARTS: u8 = 1;
/// Constant representing the suit of Clubs.
const CLUBS: u8 = 2;
/// Constant representing the suit of Diamonds.
const DIAMONDS: u8 = 3;
/// Constant representing the suit of Spades.
const SPADES: u8 = 4;

/// The number of foundation piles in the game, one for each suit.
const FOUNDATIONS: usize = SUITS as usize;
/// The number of free cells available for storing cards temporarily.
const FREE_CELLS: usize = 4;
/// The number of tableau piles in the game.
const TABLEAU_SIZE: usize = 8;
/// The total size of the game field, including foundations, suits, free cells and tableau piles.
const FIELD_SIZE: usize = FOUNDATIONS + FREE_CELLS + TABLEAU_SIZE;

/// The maximum number of undo levels the game supports.
const UNDO_LEVELS: usize = 1000;

/// Represents a move in the game, indicating the source and destination stack indices on the game field.
#[derive(Default, Copy, Clone)]
struct Move {
    /// The index of the stack the card is moved from.
    from: usize,
    /// The index of the stack the card is moved to.
    to: usize
}

/// Represents the state of a `FreeCell` game.
pub struct Game {
    /// The playing field, consisting of stacks of cards.
    field: [Vec<Card>; FIELD_SIZE],

    /// The index of the card the player currently has highlighted
    highlighted_card: usize,

    /// The index of the card the player has marked to be moved, if any.
    selected_card_opt: Option<usize>,

    /// The circular buffer storing the game's undo history.
    undo_history: CircularBuffer<UNDO_LEVELS, Move>,

    /// The number of moves made so far in the game.
    move_count: u32,

    /// Indicates whether the game is in high contrast mode, where each suit is printed in a different color.
    high_contrast: bool,
}

impl Game {
    /// Creates a new instance of the `FreeCell` game.
    ///
    /// # Arguments
    ///
    /// * `rng` - A mutable reference to a random number generator.
    ///
    /// # Returns
    ///
    /// A new `Game` instance.
    pub fn new(rng: &mut rand::rngs::ThreadRng) -> Game {
        let mut game = Game {
            field: core::array::from_fn(|_| Vec::with_capacity(DECK_SIZE)),
            highlighted_card: FOUNDATIONS + FREE_CELLS,
            selected_card_opt: None,
            undo_history: CircularBuffer::new(),
            move_count: 0,
            high_contrast: false
        };

        // Deal deck onto the board
        let mut deck = new_standard_deck(RANKS, SUITS);
        deck.shuffle(rng);
        //deck.sort_by_key(|card| card.rank); // for testing
        //deck.reverse(); // for testing
        for (i, card) in deck.into_iter().enumerate() {
            let field_column = FOUNDATIONS + FREE_CELLS + (i % TABLEAU_SIZE);
            game.field[field_column].push(card);
        }

        game
    }

    /// Checks if the game has been won.
    ///
    /// # Returns
    ///
    /// `true` if the game has been won, otherwise `false`.
    pub fn is_won(&self) -> bool {
        // Check if all foundation piles are full
        self.field.iter().take(FOUNDATIONS).all(|stack| stack.len() == RANKS as usize)
    }
    
    /// Toggles high contrast mode, making diamonds magenta and spades yellow.
    pub fn toggle_high_contrast(&mut self) {
        self.high_contrast = !self.high_contrast;
    }

    /// Moves the cursor to the left on the game field, skipping invalid spots.
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
                while self.field[self.highlighted_card].last().is_none() {
                    self.move_cursor_left();
                }
            }
        }
    }

    /// Moves the cursor to the right on the game field, skipping invalid spots.
    pub fn move_cursor_right(&mut self) {
        self.highlighted_card = (self.highlighted_card + 1) % FIELD_SIZE;

        match self.selected_card_opt {
            Some(selected_card) => {
                while !self.move_is_valid(selected_card, self.highlighted_card) && selected_card != self.highlighted_card {
                    self.move_cursor_right();
                }
            }
            None => {
                while self.field[self.highlighted_card].last().is_none() {
                    self.move_cursor_right();
                }
            }
        }
    }

    /// Quick stacks all visible cards to the foundation piles, recursively.
    pub fn quick_stack_to_foundations(&mut self) {
        let mut made_move = false;

        'outer: for source_column in 0..self.field.len() {
            for target_column in 0..FOUNDATIONS {
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

    /// Handles the event where a player clicks space/enter on a card.
    pub fn handle_card_press(&mut self) {
        if self.selected_card_opt.is_none() {
            // Select a card
            self.selected_card_opt = Some(self.highlighted_card);
        } else if Some(self.highlighted_card) == self.selected_card_opt {
            // Deselect a card
            self.selected_card_opt = None;
        } else {
            // Execute a move
            if let Some(selected_card) = self.selected_card_opt {
                self.player_try_execute_move(selected_card, self.highlighted_card);
            }
        }
    }

    /// Executes a player move if it is valid.
    pub fn player_try_execute_move(&mut self, from: usize, to: usize) {
        if self.move_is_valid(from, to) {
            // Execute move, add to undo history
            self.execute_move(from, to);
            self.move_count += 1;
            self.undo_history.push_back(Move{from, to});
        }
    }

    /// Undoes the last move made by the player.
    /// Can be used multiple times to travel back in the game's history.
    pub fn perform_undo(&mut self) {
        let last_move_opt = self.undo_history.pop_back();
        if let Some(last_move) = last_move_opt {
            self.execute_move(last_move.to, last_move.from);
            self.move_count -= 1;
        } // Else history is empty
    }

    /// Checks if two cards are of opposite colors.
    fn are_opposite_colors(card1: Card, card2: Card) -> bool {
        if card1.suit == HEARTS || card1.suit == DIAMONDS {return card2.suit == SPADES || card2.suit == CLUBS};
        if card1.suit == SPADES || card1.suit == CLUBS {return card2.suit == HEARTS || card2.suit == DIAMONDS};
        false
    }

    /// Checks if a move from one position to another is valid.
    fn move_is_valid(&self, from: usize, to: usize) -> bool {
        if from == to {return false;};
        let from_top_card = self.field[from].last().copied().unwrap_or_default();
        let to_top_card = self.field[to].last().copied().unwrap_or_default();
        if to < FOUNDATIONS {
            // Foundation case
            if to_top_card.rank != 0 {
                    return from_top_card.rank == to_top_card.rank + 1 && from_top_card.suit == to_top_card.suit;
            }
            return from_top_card.rank == 1 && to == (from_top_card.suit - 1) as usize;
        } else if to < FOUNDATIONS + FREE_CELLS {
            // Free cell case
            return to_top_card.rank == 0;
        } else if to < FOUNDATIONS + FREE_CELLS + TABLEAU_SIZE {
            // Tableau case
            if to_top_card.rank != 0 {
                return from_top_card.rank == to_top_card.rank - 1 && Game::are_opposite_colors(from_top_card, to_top_card);
            }
            return true;
        }
        false
    }

    /// Executes a move from one position to another, not checking if it follows the rules.
    /// To try executing a move in a way that fails if the move does not follow the rules, use `player_try_execute_move`.
    fn execute_move (&mut self, from: usize, to: usize) {
        // Execute the move
        // Move "from" card to "to" column
        let from_card_opt = self.field[from].last();
        if let Some(&from_card) = from_card_opt {
            self.field[from].pop();
            self.field[to].push(from_card);
        }
        self.selected_card_opt = None;
    }
}

mod print;
