use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};

extern crate colored;

use colored::*;

mod action;
mod business;
mod location;
mod state;

use action::{Event, GameAction, Turn};
use business::Position;
use location::Location;
use state::GameState;

fn format_position(position: Position) -> &'static str {
    match position {
        Position::Server => "Server",
    }
}

fn available_commands(game_state: &GameState) -> Vec<Command> {
    let mut out = Vec::new();
    let sys_actions = available_system_actions();
    for i in 0..sys_actions.len() {
        out.push(Command::System {
            action: sys_actions[i],
        });
    }

    let game_actions = game_state
        .player_state
        .location
        .available_actions(&game_state);
    for i in 0..game_actions.len() {
        out.push(Command::Game {
            action: game_actions[i],
        });
    }

    out
}

fn available_system_actions() -> Vec<SystemAction> {
    vec![SystemAction::Exit]
}

fn get_command_input(command: Command) -> &'static str {
    match command {
        Command::Game {
            action:
                GameAction::ApplyForJob {
                    employer: _,
                    position: _,
                },
        } => "a",
        Command::Game {
            action: GameAction::BuyBeer { cost: _ },
        } => "b",
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaBrewery,
                },
        } => "b",
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaHotel,
                },
        } => "h",
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaStreets,
                },
        } => "o",
        Command::Game {
            action: GameAction::Sleep { cost: _ },
        } => "s",
        Command::Game {
            action: GameAction::Work { job: _ },
        } => "w",
        Command::System {
            action: SystemAction::Exit,
        } => "q",
    }
}

fn get_command_description(command: Command) -> String {
    match command {
        Command::Game {
            action:
                GameAction::ApplyForJob {
                    employer: _,
                    position,
                },
        } => format!("Apply for a job as {}.", format_position(position)),
        Command::Game {
            action: GameAction::BuyBeer { cost },
        } => format!("Buy a beer. (${})", cost),
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaBrewery,
                },
        } => "Go to the brewery.".to_owned(),
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaHotel,
                },
        } => "Go to the hotel.".to_owned(),
        Command::Game {
            action:
                GameAction::Go {
                    destination: Location::TenundaStreets,
                },
        } => "Go outside.".to_owned(),
        Command::Game {
            action: GameAction::Sleep { cost: Some(cost) },
        } => format!("Sleep. (${})", cost),
        Command::Game {
            action: GameAction::Sleep { cost: None },
        } => "Sleep.".to_owned(),
        Command::Game {
            action: GameAction::Work { job },
        } => format!("Work. (+${})", job.pay),

        Command::System {
            action: SystemAction::Exit,
        } => "Exit.".to_owned(),
    }
}

fn print_commands(game_state: &GameState) {
    let mut commands = available_commands(game_state);
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
        (Command::Game { action: _ }, Command::System { action: _ }) => Ordering::Less,
        (Command::System { action: _ }, Command::Game { action: _ }) => Ordering::Greater,
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
        Event::AppliedForJob { employer: _ } => format!("\"We'll call you.\""),
        Event::BalanceChanged { to: balance } => format!("You have ${}", balance),
        Event::DayChanged { to: day } => format!("It is Day {}", day),
        Event::DrankBeer => "Cheers!".to_owned(),
        Event::Hired { job } => format!(
            "Congrats! You got the job at {}. You start on Day {}.",
            job.business.name, job.next_work_day
        ),
        Event::LocationChanged { to: location } => match location {
            Location::TenundaBrewery => "You are at the Tenunda Brewery.".to_owned(),
            Location::TenundaHotel => "You are at the Tenunda Hotel.".to_owned(),
            Location::TenundaStreets => "You are on the streets of Tenunda.".to_owned(),
        },
        Event::Slept => "Zzzzzzz...".to_owned(),
        Event::Worked { job } => format!("$$$ You're next shift is Day {}.", job.next_work_day),
    }
}

fn render(game_state: &GameState, log_start: usize) -> usize {
    let log_len = game_state.event_log.len();
    for i in log_start..log_len {
        let e = game_state.event_log[i];
        let styled = print_event(e).bold().green();
        println!("{}", styled);
    }

    print_commands(&game_state);

    log_len
}

#[derive(Clone, Copy)]
enum Command {
    System { action: SystemAction },
    Game { action: GameAction },
}

#[derive(Clone, Copy, PartialEq)]
enum SystemAction {
    Exit,
}

fn capture(game_state: &GameState) -> Command {
    let commands = available_commands(game_state);
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
    let mut game_state = GameState::new();

    let mut log_pos = 0;
    loop {
        log_pos = render(&game_state, log_pos);
        let cmd = capture(&game_state);
        match cmd {
            Command::Game { action } => {
                let turn = Turn { action };
                game_state = game_state.apply_turn(turn);
            }
            Command::System {
                action: SystemAction::Exit,
            } => break,
        }
    }

    println!("Goodbye!");
}
