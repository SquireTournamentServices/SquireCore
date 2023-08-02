#[allow(unused_imports)]
use std::convert::Infallible;

#[cfg(feature = "server")]
use axum::{body::HttpBody, handler::Handler, routing::MethodRouter};
use serde::{de::DeserializeOwned, Serialize};

use super::url::Url;
#[cfg(feature = "server")]
use crate::server::state::ServerState;

/// An enum that is used by the `RestRequest` to "know" what `MethodRouter` contructor to call
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

// Only integers, chars, and bools can be used as const generics, so we define consts for each
// method variant.
pub const GET: u8 = Method::Get as u8;
pub const POST: u8 = Method::Post as u8;
pub const PATCH: u8 = Method::Patch as u8;
pub const DELETE: u8 = Method::Delete as u8;

/// A function used to turn const-generics into Methods at compile time.
///
/// NOTE: This panics only at compile time, but it will do so when you run `cargo build` not `cargo
/// check`.
const fn to_method<const M: u8>() -> Method {
    match M {
        GET => Method::Get,
        POST => Method::Post,
        PATCH => Method::Patch,
        DELETE => Method::Delete,
        _ => panic!("Invalid method value"),
    }
}

/// A trait that abstracts the connections between the data used in API definitions on the server's
/// side. This trait is not intended to be implmented directly. Instead, a type should implement
/// one of the method-specific traits since each of those has a blanket implementation for this
/// trait.
// TODO: Remove the generic `N` here. This trait does not need to have a full URL, only the string.
pub trait RestRequest<const N: usize, const M: u8>: Serialize + DeserializeOwned {
    const ROUTE: Url<N>;
    const METHOD: Method = to_method::<M>();
    type Response: DeserializeOwned;

    // TODO: Currently, this method can not ensure that the return value of the handler is the same
    // as the reponse type. We should support to have an even tighter link between client and
    // server. Moveover, we should also support the encoding method of the handler.
    #[cfg(feature = "server")]
    /// This method takes a handler, calls a `MethodRouter` constructor (based on the generic
    /// constant M), and returns the method router. This method is largely used by the
    /// `SquireRouter` to construct API routes by calling this method and used the associated URL.
    fn as_route<S, T, B, H>(handler: H) -> MethodRouter<S, B, Infallible>
    where
        S: ServerState,
        T: 'static,
        B: 'static + Send + HttpBody,
        H: Handler<T, S, B>,
    {
        use axum::routing::{delete, get, patch, post};

        match Self::METHOD {
            Method::Get => get(handler),
            Method::Post => post(handler),
            Method::Patch => patch(handler),
            Method::Delete => delete(handler),
        }
    }
}

/* Below are traits used by the client to abstract away what kind of API it is making.
 * Each of these traits then have provided implementations of the `RestRequest` trait, which is
 * used by the server to abstract over routing.
 *
 * TODO: Currently, we are not encoding what the returned data format should be (i.e. JSON,
 * Postcard binary, plain text, etc). This should be encoded in both the client and server traits.
 */

/* ------ GET Request ------ */
/// This trait abstracts the connections needed for calling and constructing GET APIs. It connects
/// a request type, a response type, and a URL.
pub trait GetRequest<const N: usize>: Serialize + DeserializeOwned {
    const ROUTE: Url<N>;
    type Response: DeserializeOwned;
}

impl<const N: usize, T> RestRequest<N, GET> for T
where
    T: GetRequest<N>,
{
    const ROUTE: Url<N> = T::ROUTE;
    type Response = T::Response;
}

/* ------ POST Request ------ */
/// This trait abstracts the connections needed for calling and constructing POST APIs. It connects
/// a request type, a response type, and a URL.
pub trait PostRequest<const N: usize>: Serialize + DeserializeOwned {
    const ROUTE: Url<N>;
    type Response: DeserializeOwned;
}

impl<const N: usize, T> RestRequest<N, POST> for T
where
    T: PostRequest<N>,
{
    const ROUTE: Url<N> = T::ROUTE;
    type Response = T::Response;
}

/* ------ PATCH Request ------ */
/// This trait abstracts the connections needed for calling and constructing PATCH APIs. It
/// connects a request type, a response type, and a URL.
pub trait PatchRequest<const N: usize>: Serialize + DeserializeOwned {
    const ROUTE: Url<N>;
    type Response: DeserializeOwned;
}

impl<const N: usize, T> RestRequest<N, PATCH> for T
where
    T: PatchRequest<N>,
{
    const ROUTE: Url<N> = T::ROUTE;
    type Response = T::Response;
}

/* ------ DELETE Request ------ */
/// This trait abstracts the connections needed for calling and constructing DELETE APIs. It
/// connects a request type, a response type, and a URL.
pub trait DeleteRequest<const N: usize>: Serialize + DeserializeOwned {
    const ROUTE: Url<N>;
    type Response: DeserializeOwned;
}

impl<const N: usize, T> RestRequest<N, DELETE> for T
where
    T: DeleteRequest<N>,
{
    const ROUTE: Url<N> = T::ROUTE;
    type Response = T::Response;
}
