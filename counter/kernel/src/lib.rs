//! The kernel: every decision the application makes, written in the subset of
//! Rust that Charon and Aeneas translate to Lean (no `?`, no closures, no
//! `String`; loops over indices). `h5i app extract` turns it into
//! `proofs/generated/`, and the theorems in `proofs/` are about this code.

/// The whole state: a counter that must never pass its limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub count: u64,
    pub limit: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Add one, unless the counter is at its limit.
    Incr,
    /// Back to zero.
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    AtLimit,
}

/// One step: the next state, or why the command is refused.
pub fn transition(s: &State, cmd: &Command) -> Result<State, Error> {
    match cmd {
        Command::Incr => {
            if s.count >= s.limit {
                Err(Error::AtLimit)
            } else {
                Ok(State { count: s.count + 1, limit: s.limit })
            }
        }
        Command::Reset => Ok(State { count: 0, limit: s.limit }),
    }
}
