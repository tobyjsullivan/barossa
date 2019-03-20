use std::io::{self, Write};

#[derive(Clone, Copy)]
struct PlayerState {
    day: u8,
    balance: u64,
    location: Location,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            day: 1,
            balance: 1000,
            location: Location::Home,
        }
    }

    fn apply_action(mut self, action: PlayerAction) -> Self {
        match action {
            PlayerAction::Sleep => {
                self.day += 1;
                self
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Location {
    Home,
}

struct MenuHelpText {
    command: &'static str,
    help_text: &'static str,
}

struct Menu {
    parser: fn(&str) -> Option<Command>,
    help_text: &'static [MenuHelpText],
}

impl Menu {
    fn parse_input(&self, input: &str) -> Option<Command> {
        match input {
            "h" | "help" => Some(Command::System {
                action: SystemAction::Help,
            }),
            "x" => Some(Command::System {
                action: SystemAction::Exit,
            }),
            _ => (self.parser)(input),
        }
    }
}

const MAIN_MENU: Menu = Menu {
    parser: |input: &str| match input {
        "s" => Some(Command::Player {
            action: PlayerAction::Sleep,
        }),
        _ => None,
    },
    help_text: &[MenuHelpText {
        command: "s",
        help_text: "Sleep.",
    }],
};

/// A list of all possible input commands.
/// Intended to decouple CLI inputs from actual command handling.
enum Command {
    System { action: SystemAction },
    Player { action: PlayerAction },
}

#[derive(PartialEq)]
enum SystemAction {
    Exit,
    Help,
}

#[derive(PartialEq)]
enum PlayerAction {
    Sleep,
}

fn summarise(state: &PlayerState) -> String {
    let bar = "****************";
    format!(
        "{}\nIt is Day {}\nYou have ${}",
        bar, state.day, state.balance
    )
}

fn print_commands(menu: &Menu) {
    println!("Available actions:");
    for i in 0..menu.help_text.len() {
        let opt = &menu.help_text[i];
        println!("   {}: {}", opt.command, opt.help_text);
    }
    println!();
    println!("   h: Print this help.");
    println!("   x: Exit.");
}

fn next_command(menu: &Menu) -> Command {
    loop {
        print!("Input: ");
        if let Err(_) = io::stdout().flush() {
            panic!("Unexpected error during flush.");
        }

        let input = read_line();
        let parsed = menu.parse_input(&input);

        match parsed {
            Some(command) => {
                return command;
            }
            None => {
                println!("Unknown command: {:?}", input);
                // Restart loop
            }
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

fn print_summary(state: &PlayerState) {
    println!("{}", summarise(state));
    println!();
}

fn get_menu(player_state: &PlayerState) -> Menu {
    match player_state.location {
        Location::Home => MAIN_MENU,
    }
}

fn main() {
    let mut player_state = PlayerState::new();

    let mut last_day = 0;
    loop {
        if player_state.day > last_day {
            print_summary(&player_state);
        }
        last_day = player_state.day;

        let menu = get_menu(&player_state);
        let command = next_command(&menu);
        match command {
            Command::System {
                action: SystemAction::Help,
            } => {
                print_commands(&menu);
                continue;
            }
            Command::System {
                action: SystemAction::Exit,
            } => {
                break;
            }
            Command::Player { action } => {
                player_state = player_state.apply_action(action);
            }
        }
    }

    println!("Goodbye!");
}
