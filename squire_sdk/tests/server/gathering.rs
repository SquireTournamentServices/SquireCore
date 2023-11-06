use std::collections::HashMap;

use async_trait::async_trait;
use squire_lib::{
    accounts::SquireAccount,
    tournament::{TournamentId, TournamentSeed},
};
use squire_sdk::{
    actor::{ActorBuilder, ActorClient, ActorState, Scheduler},
    server::gathering::{
        Gathering, GatheringHall, GatheringHallMessage, GatheringMessage, PersistMessage,
    },
    sync::TournamentManager,
};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::{sleep, Duration},
};
use uuid::Uuid;

use crate::utils::{get_seed, spoof_account};

#[tokio::test]
async fn gathering_successfully_terminates_when_expected() {
    let (hall, mut recv) = MockGatheringHall::new();
    let hall_client = ActorBuilder::new(hall).launch();

    hall_client.send(GatheringHallMessage::NewGathering(TournamentId::new(
        Uuid::new_v4(),
    )));

    const CHECK_FOR_TERMINATION_AFTER: Duration = Duration::from_secs(13);
    let timer = sleep(CHECK_FOR_TERMINATION_AFTER);

    let mut gathering_created = false;
    tokio::select! {
        _ = timer => {
            panic!("The gathering should have been dropped after 10 seconds")
        },
        res = recv.recv() => match res.unwrap() {
            MockGatheringHallResponse::GatheringCreated(gathering_client) => {
                if gathering_created {
                    panic!("The gathering should have been created only once")
                }
                gathering_created = true;
            },
            MockGatheringHallResponse::GatheringDestroyed => {
                if !gathering_created {
                    panic!("The gathering should have been created before being destroyed")
                }
                return;
            },
            _ => panic!("The gathering should have been created or destroyed")
        }
    }
}

#[tokio::test]
async fn gathering_does_not_terminate_if_not_necessary() {
    let (hall, mut recv) = MockGatheringHall::new();
    let hall_client = ActorBuilder::new(hall).launch();

    hall_client.send(GatheringHallMessage::NewGathering(TournamentId::new(
        Uuid::new_v4(),
    )));

    const CHECK_FOR_TERMINATION_AFTER: Duration = Duration::from_secs(13);
    const SEND_MESSAGE_TO_GATHERING_AFTER: Duration = Duration::from_secs(7);
    let timer = sleep(CHECK_FOR_TERMINATION_AFTER);

    let mut gathering_created = false;
    tokio::select! {
        _ = timer => {
            return;
        },
        res = recv.recv() => match res.unwrap() {
            MockGatheringHallResponse::GatheringCreated(gathering_client) => {
                if gathering_created {
                    panic!("The gathering should have been created only once")
                }
                gathering_created = true;

                sleep(SEND_MESSAGE_TO_GATHERING_AFTER).await;
                let (sender, _) = oneshot::channel();
                gathering_client.send(GatheringMessage::GetTournament(sender));
            },
            MockGatheringHallResponse::GatheringDestroyed => {
                panic!("The gathering should not have been destroyed before the timer ends");
            },
            _ => panic!("The gathering should have been created or destroyed")
        }
    }
}

#[derive(Debug)]
enum MockGatheringHallResponse {
    GatheringDestroyed,
    GatheringCreated(ActorClient<GatheringMessage>),
    Other,
}

struct MockGatheringHall {
    gathering: Option<ActorClient<GatheringMessage>>,
    sender: UnboundedSender<MockGatheringHallResponse>,
}

#[async_trait]
impl ActorState for MockGatheringHall {
    type Message = GatheringHallMessage;

    async fn process(&mut self, scheduler: &mut Scheduler<Self>, msg: Self::Message) {
        match (msg) {
            GatheringHallMessage::NewGathering(id) => {
                let gathering_builder = ActorBuilder::new(Gathering::new(
                    TournamentManager::new(spoof_account(), get_seed()),
                    scheduler.client(),
                ));

                let gathering_client = gathering_builder.client();
                self.gathering = Some(gathering_builder.launch());
                self.sender
                    .send(MockGatheringHallResponse::GatheringCreated(
                        gathering_client,
                    ))
            }
            GatheringHallMessage::DestroyGathering(_, _) => {
                self.gathering = None;
                self.sender
                    .send(MockGatheringHallResponse::GatheringDestroyed)
            }
            msg => self.sender.send(MockGatheringHallResponse::Other),
        };
    }
}

impl MockGatheringHall {
    fn new() -> (Self, UnboundedReceiver<MockGatheringHallResponse>) {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        (
            Self {
                gathering: None,
                sender: send,
            },
            recv,
        )
    }

    fn send_message_to_gathering(&mut self, msg: GatheringMessage) {
        let (sender, _) = oneshot::channel();
        self.gathering
            .as_mut()
            .unwrap()
            .send(GatheringMessage::GetTournament(sender));
    }
}
