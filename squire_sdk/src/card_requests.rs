use mtgjson::mtgjson::{atomics::Atomics, meta::Meta};

use crate::response::SquireResponse;

/// The response type used by the `cards/meta` SC API. Contains the MTGJSON meta data from the
/// latest card collection.
pub type MetaResponse = SquireResponse<Meta>;

/// The response type used by the `cards/atomics` SC API. Contains the latest entire atomic card
/// collection and meta data.
pub type AtomicCardsResponse = SquireResponse<(Meta, Atomics)>;

///// The response type used by the `cards/atomics` SC API. Contains the latest minimal card
///// collection in the requested language (or English if not found) and meta data.
//pub type MinimalCardsResponse = SquireResponse<(Meta, MinimalCardCollection)>;
