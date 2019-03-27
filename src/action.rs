use crate::business::{Business, Job, Position};
use crate::location::Location;

#[derive(Clone, Copy, PartialEq)]
pub enum GameAction {
    ApplyForJob {
        employer: Business,
        position: Position,
    },
    BuyBeer {
        cost: i64,
    },
    Go {
        destination: Location,
    },
    Sleep {
        cost: Option<i64>,
    },
    Work { job: Job },
}

pub struct Turn {
    pub action: GameAction,
}

#[derive(Clone, Copy)]
pub enum Event {
    AppliedForJob { employer: Business },
    BalanceChanged { to: i64 },
    DayChanged { to: u8 },
    DrankBeer,
    Hired { job: Job },
    LocationChanged { to: Location },
    Slept,
    Worked { job: Job },
}
