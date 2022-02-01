pub use crate::player::Player;
pub use crate::player_registry::PlayerRegistry;
pub use crate::round_registry::RoundRegistry;

pub use std::collections::HashMap;
pub use std::any::Any;
pub use std::cmp::Ordering;

pub trait Score
where
    Self: ToString + UpcastAny + DynScorePartialEq + DynScorePartialOrd,
{
}

pub struct Standings {
    scores: Vec<(String, Box<dyn Score>)>,
}

pub trait ScoringSystem {
    fn new() -> Self
    where
        Self: Sized;
    
    fn update_settings(&mut self, settings: HashMap<String, String>) -> Result<(), ()>;

    fn get_standings(&self, player_reg: &PlayerRegistry, match_reg: &RoundRegistry) -> Standings;
}

impl Standings {
    pub fn new(scores: Vec<(String, Box<dyn Score>)>) -> Self {
        Standings { scores }
    }
}

// Below is a bunch of trait object up/down casting to allow Score to be both PartialOrd and able
// to be a trait object.

pub trait UpcastAny {
    fn upcast_any_ref(&self) -> &dyn Any;
}

pub trait DynScorePartialEq {
    fn dyn_eq(&self, other: &dyn Score) -> bool;
}

pub trait DynScorePartialOrd {
    fn dyn_partial_cmp(&self, other: &dyn Score) -> Option<Ordering>;
}

impl<T> UpcastAny for T
where
    T: Any,
{
    fn upcast_any_ref(&self) -> &dyn Any {
        self
    }
}

impl<T> DynScorePartialEq for T
where
    T: Score + PartialEq + Any,
{
    fn dyn_eq(&self, other: &dyn Score) -> bool {
        if let Some(comparable_other) = other.upcast_any_ref().downcast_ref() {
            self == comparable_other
        } else {
            false
        }
    }
}

impl<T> DynScorePartialOrd for T
where
    T: Score + PartialOrd + Any,
{
    fn dyn_partial_cmp(&self, other: &dyn Score) -> Option<Ordering> {
        if let Some(comparable_other) = other.upcast_any_ref().downcast_ref() {
            self.partial_cmp(comparable_other)
        } else {
            None
        }
    }
}

impl PartialEq for dyn Score {
    fn eq(&self, other: &dyn Score) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialOrd for dyn Score {
    fn partial_cmp(&self, other: &dyn Score) -> Option<Ordering> {
        self.dyn_partial_cmp(other)
    }
}
