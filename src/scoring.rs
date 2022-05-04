pub trait Score
where
    Self: ToString,
{
}

pub struct Standings<S> {
    scores: Vec<(String, S)>,
}

impl<S> Standings<S>
where
    S: Score,
{
    pub fn new(scores: Vec<(String, S)>) -> Self {
        Standings { scores }
    }
}
