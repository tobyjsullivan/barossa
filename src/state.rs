use crate::action::{Event, GameAction, Turn};
use crate::business::{Business, Job, JobApplication, Position};
use crate::location::Location;

pub struct GameState {
    pub day: u8,
    pub player_state: PlayerState,
    pub event_log: Vec<Event>,
}

impl GameState {
    pub fn new() -> Self {
        let player_state = PlayerState::new();
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
        }
    }

    pub fn apply_turn(mut self, turn: Turn) -> Self {
        self = self.hire_jobs();

        self.apply_player_action(turn.action)
    }

    fn hire_jobs(mut self) -> Self {
        match self.player_state.job_applications.first() {
            Some(&application) if application.application_day < self.day => {
                self.player_state.job_applications.retain(|&a| a != application);
                self.hire_for_job(application);
                self.hire_jobs()
            },
            _ => self,
        }
    }

    fn apply_player_action(mut self, action: GameAction) -> Self {
        match action {
            GameAction::ApplyForJob { employer, position } => {
                self.apply_for_job(employer, position);
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

    fn apply_for_job(&mut self, employer: Business, position: Position) {
        self.player_state.job_applications.push(JobApplication {
            application_day: self.day,
            business: employer,
            position,
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
            position: application.position,
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
}

pub struct PlayerState {
    pub balance: i64,
    pub location: Location,
    pub job: Option<Job>,
    pub job_applications: Vec<JobApplication>,
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
