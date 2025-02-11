mod calculator;
mod cell;
mod drawer;
mod holder;

use crate::calculator::Calculator;
use crate::holder::Holder;
use clap::Parser;
use clap_derive::Parser;
use crossterm::event::{Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, event, terminal};
use std::io::{stdout, Result};
use std::process::exit;
use std::thread;
use tokio::select;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "0.12")]
    ratio: f64,
    #[clap(short, long, default_value = "100")]
    millis_per_frame: u64,
}

fn exit_on_q_pressed() -> Result<()> {
    terminal::enable_raw_mode()?;
    loop {
        let event = event::read()?;
        if let Event::Key(key_event) = event {
            if let KeyCode::Char('q') = key_event.code {
                break;
            }
        }
    }
    execute!(stdout(), cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    exit(0);
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        ratio,
        millis_per_frame,
    } = Args::parse();

    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        EnterAlternateScreen,
        Clear(ClearType::All)
    )?;

    thread::spawn(|| exit_on_q_pressed().expect("exit_on_q_input failed"));

    let (drawer_sender, drawer_receiver) = tokio::sync::mpsc::channel(100);
    let drawer = drawer::Drawer::new(drawer_receiver);
    let drawer_thread = tokio::spawn(async move {
        drawer.run().await;
    });

    let (holder_sender, holder_receiver) = tokio::sync::mpsc::channel(100);
    let holder = Holder::<100>::new(millis_per_frame, holder_receiver, drawer_sender);
    let holder_thread = tokio::spawn(async move {
        holder.run().await;
    });

    let (width, height) = terminal::size()?;
    let calculator = Calculator::new(ratio, width, height, holder_sender);
    let calculator_thread = tokio::spawn(async move {
        calculator.run().await;
    });

    select! {
        _ = calculator_thread => {
            panic!("[Main] calculator_thread finished");
        }
        _ = holder_thread => {
            panic!("[Main] holder_thread finished");
        }
        _ = drawer_thread => {
            panic!("[Main] drawer_thread finished");
        }
    }
}
