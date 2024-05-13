// #TODO n eliminate terminal flicker
// #TODO X error handling
// #TODO n moving multiple cards at once (shortcut)
// #TODO X move count
// #TODO X implement winning screen
// #TODO X test to make sure winning (and undo after winning) works
// #TODO X decorate foundations with suits
// #TODO n condense top row representation when terminal is small, expand when large
// #TODO   refactor, ci, lint, document, publish (lint: remove unnecessary "as" statements)
// #TODO ? fix windows terminal behavior
// #TODO X variable terminal size
// #TODO X member visibility (modules)
// #TODO X only allow card to be on matching foundation spot
// #TODO X get rid of memory allocations/heap (String usage) wherever possible
// #TODO X don't allow cursor to rest on empty space, when not in select mode
// #TODO X fix foundation decoration rendering when card is selected
// #TODO X fix tableau empty column decoration and cursor visibility
// #TODO X automatically stack cards onto foundation shortcut button
// #TODO X implement "symbol blind" mode - cyan and yellow suits
// #TODO X change array access to use iterators instead of indexing wherever possible, to prevent out of bounds errors
// #TODO   pet the coyote she has been so good

mod cards;
mod game;

use crate::cards::Card;
use crate::game::Game;

use std::io::{self, stdout, Stdout, Write};

use circular_buffer::CircularBuffer;
use crossterm::{
    cursor, style::{self, Stylize}, terminal, ExecutableCommand, QueueableCommand
};

const MIN_TERMINAL_WIDTH: u16 = 60;
const MIN_TERMINAL_HEIGHT: u16 = 24;

fn run() -> Result<(), io::Error> {
    // Prepare terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    // Create game
    let mut rng = rand::thread_rng();
    let mut game = Game::new(&mut rng);
    game.print(&mut stdout)?;

    // Game loop
    loop {
        let event = crossterm::event::read()?;
        match event {
            crossterm::event::Event::Key(key_event) => {
                use crossterm::event::{KeyModifiers as MOD, KeyCode::{Char, Left, Right, Enter}, KeyEventKind::{Press, Repeat}};
                if key_event.kind == Press || key_event.kind == Repeat {
                    match (key_event.code, key_event.modifiers) {
                        (Left | Char('a'), MOD::NONE) => {
                            if !game.is_won() {game.move_cursor_left();}
                        },
                        (Right | Char('d') , MOD::NONE) => {
                            if !game.is_won() {game.move_cursor_right();}
                        },
                        (Char(' ') | Enter, MOD::NONE) => {
                            if !game.is_won() {game.handle_card_press();}
                        },
                        (Char('z'), MOD::NONE) => {
                            game.perform_undo();
                        },
                        (Char('h'), MOD::NONE) => {
                            game.toggle_high_contrast();
                        },
                        (Char('f'), MOD::NONE) => {
                            game.quick_stack_to_foundations();
                        },
                        (Char('n'), MOD::CONTROL) => {
                            game = Game::new(&mut rng);
                        },
                        (Char('q'), MOD::CONTROL) => {
                            break
                        },
                        _ => {
                            
                        }
                    }
                }
            },
            crossterm::event::Event::Resize(_term_width, _term_height) => {
                // Resize event falls through and triggers game to print again
            }
            _ => {}
        }
        game.print(&mut stdout)?;
    }
    Ok(())
}

fn cleanup() {
    let mut stdout = stdout();
    // Do not catch errors here. By the time we cleanup, we want to execute as many of these as possible to reset the terminal.
    let _ = stdout.execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::All));
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //std::env::set_var("RUST_BACKTRACE", "1");
    let (term_width, term_height) = terminal::size().unwrap();
    if term_width < MIN_TERMINAL_WIDTH || term_height < MIN_TERMINAL_HEIGHT {
        println!("Your terminal window is too small for FreeCell! It's gotta be at least {} chars wide and {} chars tall.", MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT);
        return Err("terminal too small".into());
    }
    run()?;
    cleanup();
    Ok(())
}
