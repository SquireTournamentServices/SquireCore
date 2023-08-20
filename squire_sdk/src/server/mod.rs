use axum::{
    body::{Body, HttpBody},
    extract::State,
    handler::Handler,
    Router,
};

use self::{gathering::init_gathering_hall, state::ServerState};
use crate::api::*;

pub mod gathering;
pub mod session;
pub mod state;
pub mod tournaments;

pub fn create_router<S: ServerState>(state: S) -> SquireRouter<S, Body> {
    init_gathering_hall(state);
    get_routes::<S>().merge(tournaments::get_routes::<S>())
}

fn get_routes<S: ServerState>() -> SquireRouter<S> {
    SquireRouter::new().add_route::<0, GET, GetVersion, _, _>(get_version::<S>)
}

#[derive(Debug)]
pub struct SquireRouter<S, B = Body> {
    router: Router<S, B>,
}

impl<S, B> SquireRouter<S, B>
where
    S: ServerState,
    B: HttpBody + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            router: Default::default(),
        }
    }

    pub fn add_route<const N: usize, const M: u8, R, T, H>(self, handler: H) -> Self
    where
        R: RestRequest<N, M>,
        T: 'static,
        H: Handler<T, S, B>,
    {
        println!("Adding route: {} {}", R::METHOD, R::ROUTE);
        Self {
            router: self.router.route(R::ROUTE.as_str(), R::as_route(handler)),
        }
    }

    pub fn merge(self, Self { router }: Self) -> Self {
        Self {
            router: self.router.merge(router),
        }
    }

    pub fn into_router(self) -> Router<S, B> {
        let Self { router } = self;
        router
    }
}

pub async fn get_version<S: ServerState>(State(state): State<S>) -> ServerVersionResponse {
    ServerVersionResponse::new(state.get_version())
}

impl<S> Default for SquireRouter<S, Body>
where
    S: ServerState,
{
    fn default() -> Self {
        Self::new()
    }
}
