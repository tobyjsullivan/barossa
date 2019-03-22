use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};

extern crate colored;

use colored::*;

struct GameState {
    day: u8,
    player_state: PlayerState,
    event_log: Vec<Event>,
    show_help: bool,
    done: bool,
}

impl GameState {
    fn new(player_state: PlayerState) -> Self {
        let initial_log = vec![
            Event::DayChanged { to: 1 },
            Event::BalanceChanged {
                to: player_state.balance,
            },
            Event::LocationChanged {
                to: player_state.location,
            },
        ];

        Self {
            day: 1,
            event_log: initial_log,
            player_state,
            show_help: true,
            done: false,
        }
    }

    fn apply_turn(mut self, turn: Turn) -> Self {
        let mut hire = Vec::new();
        for i in 0..self.player_state.job_applications.len() {
            if self.day > self.player_state.job_applications[i].application_day {
                hire.push(self.player_state.job_applications[i]);
            }
        }
        for &job_app in &hire {
            self.player_state.job_applications.retain(|a| *a != job_app);
            self.hire_for_job(job_app);
        }

        match turn.command {
            Command::System { action } => self.apply_system_action(action),
            Command::Game { action } => self.apply_player_action(action),
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

    fn apply_player_action(mut self, action: GameAction) -> Self {
        match action {
            GameAction::ApplyForJob { employer } => {
                self.apply_for_job(employer);
                self
            }
            GameAction::BuyBeer { cost } => {
                self.drink_beer();
                self.change_balance(0 - cost);
                self
            }
            GameAction::Go { destination } => {
                self.change_location(destination);
                self
            }
            GameAction::Sleep { cost } => {
                self.sleep();
                self.change_day(1);
                if let Some(cost) = cost {
                    self.change_balance(0 - cost);
                }

                self
            }
            GameAction::Work => {
                self.work();
                self
            }
        }
    }

    fn apply_for_job(&mut self, employer: Business) {
        self.player_state.job_applications.push(JobApplication {
            application_day: self.day,
            business: employer,
        });
        self.event_log.push(Event::AppliedForJob { employer });
    }

    fn change_balance(&mut self, delta: i64) {
        let to = self.player_state.balance + delta;
        self.player_state.balance = to;
        self.event_log.push(Event::BalanceChanged { to });
    }

    fn change_day(&mut self, delta: u8) {
        let to = self.day + delta;
        self.day = to;
        self.event_log.push(Event::DayChanged { to });
    }

    fn change_location(&mut self, to: Location) {
        self.player_state.location = to;
        self.event_log.push(Event::LocationChanged { to });
    }

    fn drink_beer(&mut self) {
        self.event_log.push(Event::DrankBeer);
    }

    fn hire_for_job(&mut self, application: JobApplication) {
        let job = Job {
            business: application.business,
            next_work_day: self.day + 1,
            pay: 200,
        };
        self.player_state.job = Some(job);
        self.event_log.push(Event::Hired { job });
    }

    fn sleep(&mut self) {
        self.event_log.push(Event::Slept);
    }

    fn work(&mut self) {
        let mut pay = None;
        if let Some(mut job) = self.player_state.job.as_mut() {
            pay = Some(job.pay);
            job.next_work_day = self.day + 1;
            self.event_log.push(Event::Worked { job: *job });
        }
        if let Some(pay) = pay {
            self.change_balance(pay as i64);
        }
    }

    fn available_commands(&self) -> Vec<Command> {
        let mut out = Vec::new();
        let sys_actions = self.available_system_actions();
        for i in 0..sys_actions.len() {
            out.push(Command::System {
                action: sys_actions[i],
            });
        }

        let game_actions = self.player_state.location.available_actions(&self);
        for i in 0..game_actions.len() {
            out.push(Command::Game {
                action: game_actions[i],
            });
        }

        out
    }

    fn available_system_actions(&self) -> Vec<SystemAction> {
        vec![SystemAction::Exit, SystemAction::Help]
    }
}

struct PlayerState {
    balance: i64,
    location: Location,
    job: Option<Job>,
    job_applications: Vec<JobApplication>,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            balance: 1000,
            location: Location::TenundaHotel,
            job: None,
            job_applications: Vec::new(),
        }
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
    AppliedForJob { employer: Business },
    BalanceChanged { to: i64 },
    DayChanged { to: u8 },
    DrankBeer,
    Hired { job: Job },
    LocationChanged { to: Location },
    Slept,
    Worked { job: Job },
}

#[derive(Clone, Copy, PartialEq)]
enum Location {
    TenundaBrewery,
    TenundaHotel,
    TenundaStreets,
}

impl Location {
    fn available_actions(&self, game_state: &GameState) -> Vec<GameAction> {
        let mut out = Vec::new();

        let mut employed_here = false;
        if let Some(job) = game_state.player_state.job {
            if job.business.location == *self {
                employed_here = true;
                if job.next_work_day == game_state.day {
                    out.push(GameAction::Work);
                }
            }
        }

        match self {
            Location::TenundaBrewery => {
                out.append(&mut vec![
                    GameAction::BuyBeer { cost: 6 },
                    GameAction::Go {
                        destination: Location::TenundaStreets,
                    },
                ]);
                if !employed_here {
                    out.push(GameAction::ApplyForJob {
                        employer: TENUNDA_BREWING,
                    });
                }
            }
            Location::TenundaHotel => out.append(&mut vec![
                GameAction::BuyBeer { cost: 10 },
                GameAction::Go {
                    destination: Location::TenundaStreets,
                },
                GameAction::Sleep { cost: Some(120) },
            ]),
            Location::TenundaStreets => out.append(&mut vec![
                GameAction::Go {
                    destination: Location::TenundaBrewery,
                },
                GameAction::Go {
                    destination: Location::TenundaHotel,
                },
            ]),
        }

        out
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Business {
    name: &'static str,
    location: Location,
}

const TENUNDA_BREWING: Business = Business {
    name: "Tenunda Brewing",
    location: Location::TenundaBrewery,
};

/// A list of all possible input commands.
/// Intended to decouple CLI inputs from actual command handling.
#[derive(Clone, Copy)]
enum Command {
    System { action: SystemAction },
    Game { action: GameAction },
}

#[derive(Clone, Copy, PartialEq)]
enum SystemAction {
    Exit,
    Help,
}

#[derive(Clone, Copy, PartialEq)]
enum GameAction {
    ApplyForJob { employer: Business },
    BuyBeer { cost: i64 },
    Go { destination: Location },
    Sleep { cost: Option<i64> },
    Work,
}

#[derive(Clone, Copy, PartialEq)]
struct JobApplication {
    business: Business,
    application_day: u8,
}

#[derive(Clone, Copy)]
struct Job {
    business: Business,
    next_work_day: u8,
    pay: u64,
}

fn print_location(location: Location) -> String {
    match location {
        Location::TenundaBrewery => format!("You are at the Tenunda Brewery."),
        Location::TenundaHotel => format!("You are at the Tenunda Hotel."),
        Location::TenundaStreets => format!("You are on the streets of Tenunda"),
    }
}

fn get_command_input(command: Command) -> &'static str {
    match command {
        Command::Game {
            action: GameAction::ApplyForJob { employer: _ },
        } => "j",
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
            action: GameAction::Work,
        } => "w",
        Command::System {
            action: SystemAction::Exit,
        } => "q",
        Command::System {
            action: SystemAction::Help,
        } => "?",
    }
}

fn get_command_description(command: Command) -> String {
    match command {
        Command::Game {
            action: GameAction::ApplyForJob { employer: _ },
        } => "Apply for a job.".to_owned(),
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
            action: GameAction::Work,
        } => "Work.".to_owned(),

        Command::System {
            action: SystemAction::Exit,
        } => "Exit.".to_owned(),
        Command::System {
            action: SystemAction::Help,
        } => "Show/hide this help.".to_owned(),
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
        Event::LocationChanged { to: location } => print_location(location),
        Event::Slept => "Zzzzzzz...".to_owned(),
        Event::Worked { job } => format!(
            "You worked a shift at {}. You're next shift is Day {}.",
            job.business.name, job.next_work_day
        ),
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
