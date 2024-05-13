//! A `FreeCell` game written in Rust

#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::missing_docs_in_private_items
)]

mod game;
mod cards;

use crate::game::Game;

use std::io::{self, stdout};

use crossterm::{
    cursor, terminal, ExecutableCommand
};

/// Minimum width of the terminal window supported by the game.
const MIN_TERMINAL_WIDTH: u16 = 60;
/// Minimum height of the terminal window supported by the game.
const MIN_TERMINAL_HEIGHT: u16 = 24;

/// Runs the game loop.
///
/// # Errors
///
/// Returns an `io::Error` if there is an issue with terminal I/O.
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

/// Cleans up the terminal after the game finishes or is interrupted.
/// This function restores the terminal to its normal state, showing the cursor and disabling raw mode.
fn cleanup() {
    let mut stdout = stdout();
    // Do not catch errors here. By the time we cleanup, we want to execute as many of these as possible to reset the terminal.
    let _ = stdout.execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::All));
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    println!();
}

/// The main function of the `FreeCell` game.
///
/// # Errors
///
/// Returns an `Err` if the terminal window is too small to play the game.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //std::env::set_var("RUST_BACKTRACE", "1");
    let (term_width, term_height) = terminal::size()?;
    if term_width < MIN_TERMINAL_WIDTH || term_height < MIN_TERMINAL_HEIGHT {
        println!("Your terminal window is too small for FreeCell! It's gotta be at least {MIN_TERMINAL_WIDTH} chars wide and {MIN_TERMINAL_HEIGHT} chars tall.");
        return Err("terminal too small".into());
    }
    run()?;
    cleanup();
    Ok(())
}
