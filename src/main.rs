mod queue;
mod server;
mod utils;

use crate::queue::*;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use dirs;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use server::*;
use std::io::{stdout, Result};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tokio;

fn main() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_app())
}

async fn run_app() -> Result<()> {
    let opt = Opt::from_args();
    let queue_path = opt.path;
    let queue = Queue::init(queue_path).expect("Failed to initialize queue");

    let queue_ref = Arc::new(Mutex::new(queue));

    start_server(queue_ref).await?;

    // stdout().execute(EnterAlternateScreen)?;
    // enable_raw_mode()?;
    // let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    // terminal.clear()?;

    // loop {
    //     terminal.draw(|frame| {
    //         let area = frame.size();
    //         frame.render_widget(
    //             Paragraph::new("Hello, Teaching Assistant! Press 'q' to quit or 's' to start the server.")
    //                 .white(),
    //             area,
    //         );
    //     })?;
    //     if event::poll(std::time::Duration::from_millis(16))? {
    //         if let event::Event::Key(key) = event::read()? {
    //             if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
    //                 break;
    //             }
    //         }
    //     }
    //     // start server with s
    //     if event::poll(std::time::Duration::from_millis(16))? {
    //         if let event::Event::Key(key) = event::read()? {
    //             if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('s') {
    //                 start_server(queue_ref).await?;
    //                 break;
    //             }
    //         }
    //     }
    // }
    // stdout().execute(LeaveAlternateScreen)?;
    // disable_raw_mode()?;
    Ok(())
}
