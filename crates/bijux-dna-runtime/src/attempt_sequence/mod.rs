//! Immutable attempt identity reservation and sequence recovery.

mod reservation;

pub use reservation::{
    reserve_attempt_sequence, supersede_attempt_sequence, AttemptSequenceIdentity,
    AttemptSequenceReservation, ATTEMPT_SEQUENCE_RESERVATION_SCHEMA_V1,
};
