use juniper::{EmptyMutation, EmptySubscription, RootNode};

#[derive(Debug, Clone)]
enum MatchStatus {
    Open,
    Uncertified,
    Certified,
    Deleted,
}

#[derive(Debug, Clone)]
struct Match {
    id: i32,
    number: u32,
    status: MatchStatus,
}

#[juniper::graphql_object(description = "A match in a tournament")]
impl Match {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn status(&self) -> MatchStatus {
        self.status
    }
}

#[derive(Debug, Clone)]
struct Player {
    id: i32,
    name: String,
    matches: Vec<Match>,
}

#[juniper::graphql_object(description = "A player in a tournament")]
impl Player {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn matches(&self) -> Vec<Match> {
        self.matches.iter().map(|m| m.clone()).collect()
    }
}

struct Tournament {
    id: i32,
    name: String,
    players: Vec<Player>,
    matches: Vec<Match>,
}

#[juniper::graphql_object(description = "A tournament")]
impl Tournament {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn players(&self) -> Vec<Player> {
        self.players.iter().map(|p| p.clone()).collect()
    }

    pub fn matches(&self) -> Vec<Match> {
        self.matches.iter().map(|m| m.clone()).collect()
    }
}

pub struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    fn tournaments() -> Vec<Tournament> {
        vec![
            Tournament {
                id: 1,
                name: "Link".to_owned(),
                players: Vec::new(),
                matches: Vec::new(),
            },
            Tournament {
                id: 2,
                name: "Mario".to_owned(),
                players: Vec::new(),
                matches: Vec::new(),
            },
        ]
    }
}

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<()>, EmptySubscription<()>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new())
}
