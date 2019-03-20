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
            Action::Sleep => {
                self.day += 1;
                self
            },
            Action::Exit => {
                self.done = true;
                self
            },
            Action::Help => {
                print_controls();
                self
            },
            Action::None => {
                self
            }
        }
    }
}

struct MainMenu {

}

impl MainMenu {
    fn parse(cmd: &str) -> Option<Action> {
        match cmd {
            "h" | "help" => Some(Action::Help),
            "s" => Some(Action::Sleep),
            "x" => Some(Action::Exit),
            _ => {
                None
            },
        }
    }
}

#[derive(PartialEq)]
enum Action {
    Exit,
    Help,
    Sleep,
}

fn summarise(state: &PlayerState) -> String {
    let bar = "****************";
    format!("{}\nIt is Day {}\nYou have ${}", bar, state.day, state.balance)
}

fn capture_input() -> Action {
    loop {
        print!("Input: ");
        if let Err(_) = io::stdout().flush() {
            panic!("Unexpected error during flush.");
        }

        let command = read_line();
        let parsed = MainMenu::parse(&command);

        match parsed {
            Some(Action::Help) => {
                print_controls();
                // Restart loop
            },
            Some(action) => {
                return action;
            },
            None => {
                println!("Unknown command: {:?}", command);
                // Restart loop
            },
        }
    }
}

fn read_line() -> String {
    let mut buffer = String::new();
    let res = io::stdin().read_line(&mut buffer);

    if let Err(_) = res {
        panic!("error during read");
    }
    let command = buffer.trim();
    String::from(command)
}

fn print_controls() {
    println!("Available actions:");
    println!("   s: Sleep.");
    println!("   x: Exit.");
}

fn print_summary(state: &PlayerState) {
    println!("{}", summarise(state));
    println!();
}

fn run_turn(state: PlayerState) -> PlayerState {
    let action = capture_input();
    PlayerState::apply_action(state, action)
}

fn main() {
    let mut state = PlayerState::new();

    let mut last_day = 0;
    loop {
        if state.day > last_day {
            print_summary(&state);
        }
        last_day = state.day;
        state = run_turn(state);

        if state.done {
            break;
        }
    }

    println!("Goodbye!");
}
