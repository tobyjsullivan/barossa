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
            location: Location::TenundaHotel,
        }
    }

    fn apply_action(mut self, action: PlayerAction) -> Self {
        match action {
            PlayerAction::BuyDrink => {
                self.balance -= 10;
                print_finances(&self);
                println!("Cheers!");
                self
            }
            PlayerAction::Go { destination } => {
                self.location = destination;
                print_location(&self);
                self
            }
            PlayerAction::Sleep => {
                if let Some(cost) = self.location.sleep_cost() {
                    self.balance -= cost;
                    print_finances(&self);
                }
                println!("Zzzzzzz...");

                self.day += 1;
                self
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Location {
    TenundaHotel,
    TenundaStreets,
}

impl Location {
    fn sleep_cost(&self) -> Option<u64> {
        match self {
            Location::TenundaHotel => Some(150),
            Location::TenundaStreets => None,
        }
    }
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
            "help" => Some(Command::System {
                action: SystemAction::Help,
            }),
            "x" => Some(Command::System {
                action: SystemAction::Exit,
            }),
            _ => (self.parser)(input),
        }
    }
}

const TENUNDA_HOTEL_MENU: Menu = Menu {
    parser: |input: &str| match input {
        "b" => Some(Command::Player {
            action: PlayerAction::BuyDrink,
        }),
        "o" => Some(Command::Player {
            action: PlayerAction::Go {
                destination: Location::TenundaStreets,
            },
        }),
        "s" => Some(Command::Player {
            action: PlayerAction::Sleep,
        }),
        _ => None,
    },
    help_text: &[
        MenuHelpText {
            command: "b",
            help_text: "Buy a beer. ($10)",
        },
        MenuHelpText {
            command: "o",
            help_text: "Go outside.",
        },
        MenuHelpText {
            command: "s",
            help_text: "Sleep. ($150)",
        },
    ],
};

const TENUNDA_STREETS_MENU: Menu = Menu {
    parser: |input: &str| match input {
        "h" => Some(Command::Player {
            action: PlayerAction::Go {
                destination: Location::TenundaHotel,
            },
        }),
        _ => None,
    },
    help_text: &[MenuHelpText {
        command: "h",
        help_text: "Go to the hotel.",
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
    BuyDrink,
    Go { destination: Location },
    Sleep,
}

fn print_summary(state: &PlayerState) {
    println!("****************");
    print_day(state);
    print_finances(state);
    print_location(state);

    println!();
}

fn print_day(state: &PlayerState) {
    println!("It is Day {}", state.day);
}

fn print_finances(state: &PlayerState) {
    println!("You have ${}", state.balance);
}

fn print_location(state: &PlayerState) {
    let out = match state.location {
        Location::TenundaHotel => format!("You are at the Tenunda Hotel."),
        Location::TenundaStreets => format!("You are on the streets of Tenunda"),
    };
    println!("{}", out);
}

fn print_commands(menu: &Menu) {
    println!("Available actions:");
    for i in 0..menu.help_text.len() {
        let opt = &menu.help_text[i];
        println!("   {}: {}", opt.command, opt.help_text);
    }
    println!();
    println!("   help: Print this help.");
    println!("   x: Exit.");
}

fn next_command(menu: &Menu) -> Command {
    loop {
        print!("> ");
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

fn get_menu(player_state: &PlayerState) -> Menu {
    match player_state.location {
        Location::TenundaHotel => TENUNDA_HOTEL_MENU,
        Location::TenundaStreets => TENUNDA_STREETS_MENU,
    }
}

fn main() {
    let mut player_state = PlayerState::new();

    let mut last_day = 0;
    loop {
        let menu = get_menu(&player_state);
        if player_state.day > last_day {
            print_summary(&player_state);
            print_commands(&menu);
        }
        last_day = player_state.day;

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
