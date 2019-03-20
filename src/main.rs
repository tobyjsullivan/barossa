use std::io::{self, Write};

#[derive(Clone, Copy)]
struct PlayerState {
    day: u8,
    balance: u64,
    done: bool,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            day: 1,
            balance: 1000,
            done: false,
        }
    }

    fn apply_action(mut self, action: Action) -> Self {
        match action {
            Action::None => {
                self.day += 1;
                self
            },
            Action::Exit => {
                self.done = true;
                self
            }
        }
    }
}

enum Action {
    None,
    Exit,
}

fn summarise(state: &PlayerState) -> String {
    format!("It is Day {}\nYou have ${}", state.day, state.balance)
}

fn capture_input() -> Action {
    print!("Input: ");
    if let Err(_) = io::stdout().flush() {
        panic!("Unexpected error during flush.");
    }

    let mut buffer = String::new();
    let res = io::stdin().read_line(&mut buffer);

    if let Err(_) = res {
        panic!("error during read");
    }

    match buffer.trim() {
        "c" => {
            Action::None
        },
        "x" => {
            Action::Exit
        },
        _ => Action::None,
    }
}

fn print_controls() {
    println!("Available actions:");
    println!("   c: Continue without action.");
    println!("   x: Exit.");
}

fn print_summary(state: &PlayerState) {
    println!("{}", summarise(state));
}

fn run_turn(state: PlayerState) -> PlayerState {
    print_summary(&state);
    println!();
    print_controls();
    println!();
    let action = capture_input();
    let state = PlayerState::apply_action(state, action);
    state
}

fn main() {
    let mut state = PlayerState::new();

    loop {
        state = run_turn(state);

        if state.done {
            break;
        }
    }

    println!("Goodbye!");
}
