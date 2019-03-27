use crate::location::Location;

#[derive(Clone, Copy, PartialEq)]
pub struct Business {
    pub name: &'static str,
    pub location: Location,
}

pub const TENUNDA_BREWING: Business = Business {
    name: "Tenunda Brewing",
    location: Location::TenundaBrewery,
};

#[derive(Clone, Copy, PartialEq)]
pub struct JobApplication {
    pub business: Business,
    pub application_day: u8,
    pub position: Position,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Job {
    pub business: Business,
    pub position: Position,
    pub next_work_day: u8,
    pub pay: u64,
}

impl Job {
    pub fn work(mut self) -> (Self, u64) {
        self.next_work_day += 1;
        (self, self.pay)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Position {
    Server,
}
