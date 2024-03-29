use super::{ClientOpLink, ServerOpLink, SyncForwardResp};
use crate::sync::{OpSync, SyncError};

/// A struct that tracks the messages passed between a client and server during the sync process.
#[derive(Debug)]
pub struct SyncChain {
    /// All of the messages used during the syncing process.
    links: Vec<(ClientOpLink, ServerOpLink)>,
}

impl SyncChain {
    /// Creates a new chain from a client message. If the message is not `ClientOpLink::Init`, an
    /// error is returned.
    pub fn new(op: &ClientOpLink) -> Result<Self, SyncError> {
        match op {
            ClientOpLink::Init(_) => Ok(Self { links: vec![] }),
            _ => Err(SyncError::NotInitialized),
        }
    }

    // TODO: We need to detect when a message will be the final messages in the chain.
    //  - When a completion message is sent
    //  - When an error is sent
    //  - When a "terminated seen" message is sent
    //
    //  The add message method should signal to the manager that the chain is concluded and those
    //  methods should clear away all by the last one or two messages and mark the chain as
    //  completed.
    //
    //  We should not remove everything, though. It is possible that the response is dropped in
    //  transit. If this occurs, the client will continue to send follow-up messages. If that case,
    //  we need to be able to auto-reply to incoming messages.
    //
    //  We need to detect if we have already received a message from the client. Storing the last
    //  message should suffice.

    pub fn add_link(
        &mut self,
        client: ClientOpLink,
        server: ServerOpLink,
    ) -> Option<(ClientOpLink, ServerOpLink)> {
        match &server {
            ServerOpLink::Conflict(_) => {
                self.links.push((client, server));
                None
            }
            ServerOpLink::Completed(_)
            | ServerOpLink::TerminatedSeen { .. }
            | ServerOpLink::Error(_) => Some((client, server)),
        }
    }

    /// Checks to see if an incoming message is valid and if we have already seen this message. If
    /// we have seen this message, we return `Err(Ok(ServerOpLink))`. This signals that the message
    /// should not be processed and instead, the returned message should be returned
    pub fn validate_client_message(&self, msg: &ClientOpLink) -> Result<(), ServerOpLink> {
        // TODO: Turn these panics into errors? This should only happen if a chain is created from
        // a Init message and then that message is validated.
        let last = self.links.last().unwrap();
        if msg == &last.0 {
            return Err(last.1.clone());
        }
        // TODO: Can we do better? We do checks for this elsewhere. Can we move them here?
        let ServerOpLink::Conflict(ref proc) = last.1 else {
            return Ok(());
        };
        match msg {
            ClientOpLink::Init(_) => Err(SyncError::AlreadyInitialized.into()),
            ClientOpLink::Terminated => Ok(()),
            ClientOpLink::Decision(dec) if proc.valid_decision(dec) => Ok(()),
            ClientOpLink::Decision(_) => Err(SyncError::EmptySync.into()),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ForwardChain {
    init: OpSync,
    resp: Option<SyncForwardResp>,
}
