use serde::{Deserialize, Serialize};

/// What to do if a service fails to Poke often enough.
#[derive(Debug, Serialize, Deserialize)]
pub enum PokeAction {
    /// Used for services that don't send pokes, e.g. UI.
    Ignore,

    /// Restart just the service that failed to poke in time.
    Restart,

    // Shutdown all services.
    Shutdown,
}

/// Messages that the app receives.
#[derive(Debug, Serialize, Deserialize)]
pub enum AppMessages {
    /// Changes the maximum amount of seconds between pokes. Should only be used by unit
    /// tests. TODO: remove this?
    Duration(u64), // nicer to use chrono::Duration but that doesn't support serialization

    /// Most services should regularly Poke the app with their pid so that it can
    /// detect any services that are hung (or taking too long to process something).
    Poke(u32),

    // All services should register their pids with the app.
    Register(u32, PokeAction),
}
