use std::io::{self, Write};

use crossterm::{cursor, style::{self, Stylize}, terminal, QueueableCommand};

use crate::{cards::Card, game::Game, MIN_TERMINAL_WIDTH};

use super::{CLUBS, DIAMONDS, FREE_CELLS, HEARTS, RANKS, SPADES, SUITS, TABLEAU_SIZE};

const TYPICAL_BOARD_HEIGHT: u16 = 24;

const CARD_PRINT_WIDTH: u16 = 7;
const CARD_PRINT_HEIGHT: u16 = 5;
const TABLEAU_VERTICAL_OFFSET: u16 = 2;

const DEFAULT_TERMINAL_WIDTH: u16 = 80;
const DEFAULT_TERMINAL_HEIGHT: u16 = 24;

const SUIT_STRINGS: [&str; SUITS as usize + 1] = [" ", "♥", "♣", "♦", "♠"];
const RANK_STRINGS: [&str; RANKS as usize + 1] = [" ", "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];

impl Game {
    pub fn print(&self, out: &mut io::Stdout) -> Result<(), io::Error> {
        if self.is_won() {
            out.queue(style::SetAttribute(style::Attribute::Dim))?;
            self.print_board(out)?;
            out.queue(style::SetAttribute(style::Attribute::Reset))?;
            Game::print_chrome(out, self.move_count)?;
            Game::print_win(out)?;
        } else {
            self.print_board(out)?;
            Game::print_chrome(out, self.move_count)?;
        }
        out.flush()?;
        Ok(())
    }

    fn print_board(&self, out: &mut io::Stdout) -> Result<(), io::Error> {
        out.queue(terminal::Clear(terminal::ClearType::All))?;

        for (i, stack) in self.field.iter().enumerate() {
            let mut top_card = stack.last().copied().unwrap_or_default();
            let top_card_is_highlighted = self.highlighted_card == i && !self.won;
            if i < SUITS as usize {
                // Print foundation
                // If card is a placeholder, assign a suit for decoration
                #[allow(clippy::cast_possible_truncation)]
                if top_card == Card::default() {
                    top_card = Card{rank: 0, suit: i as u8 + 1};
                }
                #[allow(clippy::cast_possible_truncation)]
                Game::print_card_at_coord(
                    out,
                    i as u16 * CARD_PRINT_WIDTH + 1, 
                    1, 
                    top_card, 
                    top_card_is_highlighted, 
                    self.selected_card_opt == Some(i),
                    self.high_contrast
                )?;
            } else if i < (SUITS as usize + FREE_CELLS) {
                // Print free cell
                #[allow(clippy::cast_possible_truncation)]
                Game::print_card_at_coord(
                    out,
                    i as u16 * CARD_PRINT_WIDTH + 3,
                    1, top_card,
                    top_card_is_highlighted,
                    self.selected_card_opt == Some(i),
                    self.high_contrast
                )?;
            } else if i < (SUITS as usize + FREE_CELLS + TABLEAU_SIZE) {
                // Print tableau column card-by-card
                let mut card_stack_iter = stack.iter().enumerate().peekable();
                while let Some((y, &card)) = card_stack_iter.next() {
                    let is_top_card = card_stack_iter.peek().is_none(); // Check if we are currently printing the top card
                    #[allow(clippy::cast_possible_truncation)]
                    Game::print_card_at_coord(
                        out,
                        (i as u16 - (u16::from(SUITS) + FREE_CELLS as u16)) * CARD_PRINT_WIDTH + 2,
                        y as u16 * TABLEAU_VERTICAL_OFFSET + CARD_PRINT_HEIGHT + 1,
                        card,
                        top_card_is_highlighted && is_top_card,
                        self.selected_card_opt == Some(i) && is_top_card,
                        self.high_contrast,
                    )?;
                }
                // If tableau column is empty, print placeholder instead
                if stack.is_empty() {
                    #[allow(clippy::cast_possible_truncation)]
                    Game::print_card_at_coord(
                        out,
                        (i as u16 - (u16::from(SUITS) + FREE_CELLS as u16)) * CARD_PRINT_WIDTH + 2,
                        CARD_PRINT_HEIGHT + 1,
                        top_card,
                        top_card_is_highlighted,
                        self.selected_card_opt == Some(i),
                        self.high_contrast
                    )?;
                }
            }
        }

        Ok(())
    }

    fn print_chrome(out: &mut std::io::Stdout, move_count: u32) -> Result<(), io::Error> {
        let (_term_width, term_height) = terminal::size().unwrap_or((DEFAULT_TERMINAL_WIDTH, DEFAULT_TERMINAL_HEIGHT));
        
        // Print title bar
        out.queue(cursor::MoveTo(0, 0))?;
        print!("╭── Rusty FreeCell ────────────────────────────────────────╮");
        out.queue(cursor::MoveTo(40, 0))?;
        print!(" Moves: {move_count} ");

        // Print side bars

        for i in 1..term_height {
            out.queue(cursor::MoveTo(0, i))?;
            print!("│");
            out.queue(cursor::MoveTo(crate::MIN_TERMINAL_WIDTH - 1, i))?;
            print!("│");
        }

        // Print bottom bar
        out.queue(cursor::MoveTo(0, term_height))?;
        print!("╰── (New Game: ctrl-n) ─ (Undo: z) ─ (Quit: ctrl-q) ───────╯");

        Ok(())
    }

    fn print_card_at_coord(out: &mut io::Stdout, x: u16, y: u16, card: Card, highlighted: bool, selected: bool, high_contrast: bool)  -> Result<(), io::Error> {
        let card_suit_rank_str = RANK_STRINGS[card.rank as usize].to_owned() + SUIT_STRINGS[card.suit as usize];
        let card_display_str;
        if selected {
            card_display_str= format!("\
                ╭─────╮\n\
                │ {card_suit_rank_str: <3} │\n\
                │     │\n\
                │  △  │\n\
                ╰─────╯\n");
        } else if card.rank == 0 {
            // Print suit-decorated placeholder
            card_display_str= format!("\
            ╭─────╮\n\
            │     │\n\
            │ {card_suit_rank_str}  │\n\
            │     │\n\
            ╰─────╯\n");
        } else {
            card_display_str= format!("\
            ╭─────╮\n\
            │ {card_suit_rank_str: <3} │\n\
            │     │\n\
            │     │\n\
            ╰─────╯\n");
        }

        for (d, line) in card_display_str.lines().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            out.queue(cursor::MoveTo(x, y + d as u16))?;
            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::Reverse));
            } else if card.rank == 0 {
                // dim placeholder
                let _= out.queue(style::SetAttribute(style::Attribute::Dim));
            }

            if card.rank != 0 {
                if high_contrast {
                    match card.suit {
                        HEARTS => {
                            print!("{}", line.with(style::Color::DarkRed));
                        },
                        CLUBS => {
                            print!("{}", line.with(style::Color::White));
                        },
                        DIAMONDS => {
                            print!("{}", line.with(style::Color::Magenta));
                        },
                        SPADES => {
                            print!("{}", line.with(style::Color::Yellow));
                        },
                        _ => {
                            print!("{line}");
                        }
                    }
                } else {
                    match card.suit {
                        HEARTS | DIAMONDS  => {
                            print!("{}", line.with(style::Color::Red));
                        },
                        _ => {
                            print!("{line}");
                        }
                    }
                }
            } else {
                print!("{line}");
            }

            if highlighted {
                let _= out.queue(style::SetAttribute(style::Attribute::NoReverse));
            } else if card.rank == 0 {
                // undim placeholder
                let _= out.queue(style::SetAttribute(style::Attribute::NormalIntensity));
            }
        }
        Ok(())
    }

    fn print_win (out: &mut io::Stdout) -> Result<(), io::Error> {
        let win_message_width = 20;
        let win_message_height = 4;
        Game::print_string_at_coord(out,   
        "╭──────────────────╮\n\
                 │ You Win!         │\n\
                 │ New Game: ctrl-n │\n\
                 ╰──────────────────╯",
                MIN_TERMINAL_WIDTH / 2 - win_message_width / 2,
                TYPICAL_BOARD_HEIGHT / 2 - win_message_height / 2)?;
        Ok(())
    }

    fn print_string_at_coord(out: &mut io::Stdout, string: &str, x: u16, y: u16) -> Result<(), io::Error> {
        for (i, line) in string.lines().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            out.queue(cursor::MoveTo(x, y + i as u16))?;
            print!("{line}");
        }
        Ok(())
    }
}

