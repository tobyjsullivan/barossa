use crate::action::GameAction;
use crate::business::{Position, TENUNDA_BREWING};
use crate::state::GameState;

#[derive(Clone, Copy, PartialEq)]
pub enum Location {
    TenundaBrewery,
    TenundaHotel,
    TenundaStreets,
}

impl Location {
    pub fn available_actions(&self, game_state: &GameState) -> Vec<GameAction> {
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

        let mut applied_here = false;
        for &job_app in &game_state.player_state.job_applications {
            if job_app.business.location == *self {
                applied_here = true;
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
                if !employed_here && !applied_here {
                    out.push(GameAction::ApplyForJob {
                        employer: TENUNDA_BREWING,
                        position: Position::Server,
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
