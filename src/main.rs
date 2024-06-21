mod queue;
mod server;

use server::*;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use std::path::PathBuf;
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
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    layout::{Constraint, Direction, Layout},
    text::Span,
    style::{Style, Color},
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

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Server Control").borders(Borders::ALL);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
                .split(size);

            let items = if !server_running {
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

            let list = List::new(items).block(block);
            f.render_widget(list, chunks[0]);

            // Ensure you use async lock acquisition in actual application code
            let queue = queue_ref.lock().unwrap(); 

            let block = Block::default().title("Queue").borders(Borders::ALL);
            let items = queue.students.iter().map(|student| {
                ListItem::new(Span::styled(
                    format!("{}: {}", student.id, student.info.name),
                    Style::default(),
                ))
            }).collect::<Vec<_>>();

            let list = List::new(items).block(block);
            f.render_widget(list, chunks[1]);
        })?;

        if let Ok(true) = event::poll(std::time::Duration::from_millis(500)) {
            match read()? {
                event::Event::Key(KeyEvent { code: KeyCode::Char('s'), .. }) if !server_running => {
                    if let Err(_) = tx.send(ServerControlMessage::Start).await {
                        println!("Failed to send start command");
                        break;
                    }
                    server_running = true;
                },
                event::Event::Key(KeyEvent { code: KeyCode::Char('x'), .. }) if server_running => {
                    if let Err(_) = tx.send(ServerControlMessage::Stop).await {
                        println!("Failed to send stop command");
                        break;
                    }
                    server_running = false;
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
    std::process::exit(0);
}
