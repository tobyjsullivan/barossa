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
                self.player_state
                    .job_applications
                    .retain(|&a| a != application);
                self.hire_for_job(application).hire_jobs() // Recurse
            }
            // Base case
            _ => self,
        }
    }

    fn apply_player_action(mut self, action: GameAction) -> Self {
        match action {
            GameAction::ApplyForJob { employer, position } => {
                self.apply_for_job(employer, position)
            }
            GameAction::BuyBeer { cost } => self.drink_beer().change_balance(0 - cost),
            GameAction::Go { destination } => self.change_location(destination),
            GameAction::Sleep { cost } => {
                self = self.sleep();
                if let Some(cost) = cost {
                    self = self.change_balance(0 - cost);
                }

                self
            }
            GameAction::Work { job } => self.work(job),
        }
    }

    fn apply_for_job(mut self, employer: Business, position: Position) -> Self {
        self.player_state.job_applications.push(JobApplication {
            application_day: self.day,
            business: employer,
            position,
        });
        self.push_event(Event::AppliedForJob { employer })
    }

    fn change_balance(mut self, delta: i64) -> Self {
        let to = self.player_state.balance + delta;
        self.player_state.balance = to;
        self.push_event(Event::BalanceChanged { to })
    }

    fn change_day(mut self, delta: u8) -> Self {
        let to = self.day + delta;
        self.day = to;
        self.push_event(Event::DayChanged { to })
    }

    fn change_location(mut self, to: Location) -> Self {
        self.player_state.location = to;
        self.push_event(Event::LocationChanged { to })
    }

    fn drink_beer(self) -> Self {
        self.push_event(Event::DrankBeer)
    }

    fn hire_for_job(self, application: JobApplication) -> Self {
        let job = Job {
            business: application.business,
            next_work_day: self.day + 1,
            pay: 200,
            position: application.position,
        };
        self.update_job(Some(job)).push_event(Event::Hired { job })
    }

    fn push_event(mut self, event: Event) -> Self {
        self.event_log.push(event);
        self
    }

    fn sleep(self) -> Self {
        self.push_event(Event::Slept).change_day(1)
    }

    fn update_job(mut self, job: Option<Job>) -> Self {
        self.player_state.job = job;
        self
    }

    fn work(self, job: Job) -> Self {
        let (job, pay) = job.work();
        self.update_job(Some(job))
            .push_event(Event::Worked { job: job })
            .change_balance(pay as i64)
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
