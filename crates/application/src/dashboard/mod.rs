mod error;
mod model;
mod use_case;

pub use error::GetDashboardError;
pub use model::{Dashboard, DashboardAccount, DashboardActivity};
pub use use_case::{DashboardService, GetDashboardUseCase};
