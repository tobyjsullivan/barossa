use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};

extern crate colored;

use colored::*;

#[derive(Clone)]
struct GameState {
    day: u8,
    player_state: PlayerState,
    event_log: Vec<Event>,
    show_help: bool,
    done: bool,
}

impl GameState {
    fn new(player_state: PlayerState) -> Self {
        Self {
            day: 1,
            player_state,
            event_log: vec![
                Event::DayChanged { to: 1 },
                Event::BalanceChanged {
                    to: player_state.balance,
                },
                Event::LocationChanged {
                    to: player_state.location,
                },
            ],
            show_help: true,
            done: false,
        }
    }

    fn apply_turn(self, turn: Turn) -> Self {
        match turn.command {
            Command::System { action } => self.apply_system_action(action),
            Command::Player { action } => self.apply_player_action(action),
        }
    }

    fn apply_system_action(mut self, action: SystemAction) -> Self {
        match action {
            SystemAction::Exit => {
                self.done = true;
                self
            }
            SystemAction::Help => {
                self.show_help = !self.show_help;
                self
            }
        }
    }

    fn apply_player_action(mut self, action: PlayerAction) -> Self {
        match action {
            PlayerAction::BuyDrink { cost } => {
                self.drink_beer();
                self.change_balance(0 - cost);
                self
            }
            PlayerAction::Go { destination } => {
                self.change_location(destination);
                self
            }
            PlayerAction::Sleep { cost } => {
                self.sleep();
                self.change_day(1);
                if let Some(cost) = cost {
                    self.change_balance(0 - cost);
                }

                self
            }
        }
    }

    fn change_balance(&mut self, delta: i64) {
        let to = self.player_state.balance + delta;
        self.player_state.balance = to;
        self.event_log.push(Event::BalanceChanged { to });
    }

    fn change_location(&mut self, to: Location) {
        self.player_state.location = to;
        self.event_log.push(Event::LocationChanged { to });
    }

    fn drink_beer(&mut self) {
        self.event_log.push(Event::DrankBeer);
    }

    fn sleep(&mut self) {
        self.event_log.push(Event::Slept);
    }

    fn change_day(&mut self, delta: u8) {
        let to = self.day + delta;
        self.day = to;
        self.event_log.push(Event::DayChanged { to });
    }

    fn available_commands(&self) -> Vec<Command> {
        let mut out = Vec::new();
        let sys_actions = self.available_system_actions();
        for i in 0..sys_actions.len() {
            out.push(Command::System {
                action: sys_actions[i],
            });
        }

        let player_actions = self.player_state.available_actions();
        for i in 0..player_actions.len() {
            out.push(Command::Player {
                action: player_actions[i],
            });
        }

        out
    }

    fn available_system_actions(&self) -> Vec<SystemAction> {
        vec![SystemAction::Exit, SystemAction::Help]
    }
}

#[derive(Clone, Copy)]
struct PlayerState {
    balance: i64,
    location: Location,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            balance: 1000,
            location: Location::TenundaHotel,
        }
    }

    fn available_actions(&self) -> Vec<PlayerAction> {
        self.location.available_actions()
    }
}

struct Turn {
    command: Command,
}

impl Turn {
    fn new(command: Command) -> Self {
        Self { command }
    }
}

#[derive(Clone, Copy)]
enum Event {
    Slept,
    DayChanged { to: u8 },
    DrankBeer,
    LocationChanged { to: Location },
    BalanceChanged { to: i64 },
}

#[derive(Clone, Copy, PartialEq)]
enum Location {
    TenundaHotel,
    TenundaStreets,
}

impl Location {
    fn available_actions(&self) -> Vec<PlayerAction> {
        match self {
            Location::TenundaHotel => vec![
                PlayerAction::Go {
                    destination: Location::TenundaStreets,
                },
                PlayerAction::Sleep { cost: Some(150) },
                PlayerAction::BuyDrink { cost: 10 },
            ],
            Location::TenundaStreets => vec![PlayerAction::Go {
                destination: Location::TenundaHotel,
            }],
        }
    }
}

/// A list of all possible input commands.
/// Intended to decouple CLI inputs from actual command handling.
#[derive(Clone, Copy)]
enum Command {
    System { action: SystemAction },
    Player { action: PlayerAction },
}

#[derive(Clone, Copy, PartialEq)]
enum SystemAction {
    Exit,
    Help,
}

#[derive(Clone, Copy, PartialEq)]
enum PlayerAction {
    BuyDrink { cost: i64 },
    Go { destination: Location },
    Sleep { cost: Option<i64> },
}

fn print_location(location: Location) -> String {
    match location {
        Location::TenundaHotel => format!("You are at the Tenunda Hotel."),
        Location::TenundaStreets => format!("You are on the streets of Tenunda"),
    }
}

fn get_command_input(command: Command) -> &'static str {
    match command {
        Command::System {
            action: SystemAction::Exit,
        } => "q",
        Command::System {
            action: SystemAction::Help,
        } => "?",
        Command::Player {
            action: PlayerAction::BuyDrink { cost: _ },
        } => "b",
        Command::Player {
            action:
                PlayerAction::Go {
                    destination: Location::TenundaStreets,
                },
        } => "o",
        Command::Player {
            action:
                PlayerAction::Go {
                    destination: Location::TenundaHotel,
                },
        } => "h",
        Command::Player {
            action: PlayerAction::Sleep { cost: _ },
        } => "s",
    }
}

fn get_command_description(command: Command) -> String {
    match command {
        Command::System {
            action: SystemAction::Exit,
        } => "Exit.".to_owned(),
        Command::System {
            action: SystemAction::Help,
        } => "Show/hide this help.".to_owned(),

        Command::Player {
            action: PlayerAction::BuyDrink { cost },
        } => format!("Buy a beer. (${})", cost),
        Command::Player {
            action:
                PlayerAction::Go {
                    destination: Location::TenundaStreets,
                },
        } => "Go outside.".to_owned(),
        Command::Player {
            action:
                PlayerAction::Go {
                    destination: Location::TenundaHotel,
                },
        } => "Go into the hotel.".to_owned(),
        Command::Player {
            action: PlayerAction::Sleep { cost: Some(cost) },
        } => format!("Sleep. (${})", cost),
        Command::Player {
            action: PlayerAction::Sleep { cost: None },
        } => "Sleep.".to_owned(),
    }
}

fn print_commands(game_state: &GameState) {
    let mut commands = game_state.available_commands();
    if commands.len() == 0 {
        println!("No actions currently available.");
        return;
    }

    commands.sort_by(|&a, &b| {
        let a_input = get_command_input(a);
        let b_input = get_command_input(b);
        a_input.cmp(b_input)
    });

    commands.sort_by(|a, b| match (a, b) {
        (Command::Player { action: _ }, Command::System { action: _ }) => Ordering::Less,
        (Command::System { action: _ }, Command::Player { action: _ }) => Ordering::Greater,
        (_, _) => Ordering::Equal,
    });

    let mut sys_cmds = Vec::new();
    for &command in &commands {
        // Push system commands to a special section at the end.
        if let sys_cmd @ Command::System { action: _ } = command {
            sys_cmds.push(sys_cmd);
            continue;
        }
        let input = get_command_input(command);
        let desc = get_command_description(command);
        println!("   {}: {}", input, desc);
    }
    // println!();
    // for &command in &sys_cmds {
    //     let input = get_command_input(command);
    //     let desc = get_command_description(command);
    //     println!("   {}: {}", input, desc);
    // }
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

fn print_event(event: Event) -> String {
    match event {
        Event::BalanceChanged { to: balance } => format!("You have ${}", balance),
        Event::DayChanged { to: day } => format!("It is Day {}", day),
        Event::DrankBeer => "Cheers!".to_owned(),
        Event::LocationChanged { to: location } => print_location(location),
        Event::Slept => "Zzzzzzz...".to_owned(),
    }
}

fn render(game_state: &GameState, log_start: usize) -> usize {
    let log_len = game_state.event_log.len();
    for i in log_start..log_len {
        let e = game_state.event_log[i];
        let styled = print_event(e).bold().green();
        println!("{}", styled);
    }

    if game_state.show_help {
        println!();
        print_commands(&game_state);
    }

    log_len
}

fn capture(game_state: &GameState) -> Command {
    let commands = game_state.available_commands();
    let mut cmd_map = HashMap::new();
    for &command in &commands {
        let command_input = get_command_input(command);
        cmd_map.insert(command_input.to_owned(), command);
    }

    loop {
        print!("> ");
        if let Err(_) = io::stdout().flush() {
            panic!("Unexpected error during flush.");
        }

        let input = read_line();
        if let Some(command) = cmd_map.get(&input) {
            return *command;
        }

        println!("Unknown command: {:?}", input);
        // Restart loop.
    }
}

fn main() {
    let player_state = PlayerState::new();
    let mut game_state = GameState::new(player_state);

    let mut log_pos = 0;
    loop {
        log_pos = render(&game_state, log_pos);
        let cmd = capture(&game_state);
        let turn = Turn::new(cmd);
        game_state = game_state.apply_turn(turn);

        if game_state.done {
            break;
        }
    }

    println!("Goodbye!");
}
