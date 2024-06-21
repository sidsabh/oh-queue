mod queue;
mod server;

use server::*;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use std::path::PathBuf;
use log::info;
use tokio::sync::mpsc;
#[derive(StructOpt, Debug)]
#[structopt(name = "queue")]
pub struct Opt {
    /// Path to the queue file
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,
}

use queue::Queue;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel::<ServerControlMessage>(100);

    let opt = Opt::from_args();
    let queue = Queue::init(opt.path).expect("Failed to initialize queue");
    let queue_ref = Arc::new(Mutex::new(queue));

    tokio::spawn(http_server(queue_ref.clone(), rx));

    // Make sure that `run_app` is an async function that returns a `Result` and takes `Send` parameters.
    run_app(tx, queue_ref).await
}


use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, KeyCode, KeyEvent, read},
};
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    layout::{Constraint, Direction, Layout},
    text::{Span, Line},
    style::{Style, Color},
    Terminal,
    backend::CrosstermBackend,
    prelude::Modifier,
};

async fn run_app(
    tx: mpsc::Sender<ServerControlMessage>,
    queue_ref: Arc<Mutex<Queue>>,
) -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut server_running = false;
    let mut list_state = ListState::default();
    list_state.select(Some(0)); // Start with the first student selected

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Server Control").borders(Borders::ALL);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(5), Constraint::Percentage(95)].as_ref())
                .split(size);

            let control_items = if !server_running {
                vec![
                    ListItem::new(Span::styled("Press 's' to start the server", Style::default().fg(Color::Yellow))),
                    ListItem::new(Span::styled("Press 'q' to quit", Style::default().fg(Color::Green))),
                ]
            } else {
                vec![
                    ListItem::new(Span::styled("Press 'x' to stop the server", Style::default().fg(Color::LightRed))),
                    ListItem::new(Span::styled("Press 'q' to quit", Style::default().fg(Color::Green))),
                ]
            };

            let control_list = List::new(control_items).block(block);
            f.render_widget(control_list, chunks[0]);

            let queue = queue_ref.lock().unwrap();

            let block = Block::default().title("Queue").borders(Borders::ALL);
            let items: Vec<_> = queue.students.iter().enumerate().map(|(i, student)| {
                let content = if Some(i) == list_state.selected() {
                    vec![
                        Line::from(Span::styled(format!("Name: {}", student.info.name), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                        Line::from(Span::styled(format!("ID: {}", student.id), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                        Line::from(Span::styled(format!("CSID: {}", student.info.csid), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                        Line::from(Span::styled(format!("Purpose: {:?}", student.info.purpose), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                        Line::from(Span::styled(format!("Details: {}", student.info.details), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                        Line::from(Span::styled(format!("Steps: {}", student.info.steps), Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))),
                    ]
                } else {
                    vec![
                        Line::from(Span::styled(format!("Name: {}, ID: {}", student.info.name, student.id), Style::default().fg(Color::Gray))),
                    ]
                };
                ListItem::new(content)
            }).collect();

            let list = List::new(items).block(block).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            f.render_stateful_widget(list, chunks[1], &mut list_state);
        })?;

        if let Ok(true) = event::poll(std::time::Duration::from_millis(500)) {
            match read()? {
                event::Event::Key(KeyEvent { code: KeyCode::Char('s'), .. }) if !server_running => {
                    if tx.send(ServerControlMessage::Start).await.is_err() {
                        info!("Failed to send start command");
                        continue;
                    }
                    server_running = true;
                },
                event::Event::Key(KeyEvent { code: KeyCode::Char('x'), .. }) if server_running => {
                    if tx.send(ServerControlMessage::Stop).await.is_err() {
                        info!("Failed to send stop command");
                        continue;
                    }
                    server_running = false;
                },
                event::Event::Key(KeyEvent { code: KeyCode::Char('d'), .. }) => {
                    // Remove selected student
                    let mut queue = queue_ref.lock().unwrap();
                    if queue.size() == 0 {
                        continue;
                    }
                    if let Some(index) = list_state.selected() {
                        queue.students.remove(index);
                        queue.save().expect("Failed to save queue.");
                        list_state.select(Some(index.saturating_sub(1))); // Adjust selection
                    }
                },
                event::Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    // Move selection down
                    let queue = queue_ref.lock().unwrap();
                    let next_index = match list_state.selected() {
                        Some(i) => if i >= queue.students.len() - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    list_state.select(Some(next_index));
                },
                event::Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    // Move selection up
                    let queue = queue_ref.lock().unwrap();
                    let prev_index = match list_state.selected() {
                        Some(i) => if i == 0 { queue.students.len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    list_state.select(Some(prev_index));
                },
                event::Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    break;
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}