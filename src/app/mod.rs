//! # Application layer
//!
//! **Purpose:** own everything that is "the running editor" as opposed to "a
//! piece of text".
//!
//! **Responsibility:** holds the global state (open buffers, current mode,
//! pending command line, transient status message) and drives the event loop
//! that ties input, editing and rendering together.
//!
//! **Public API:** [`mode::Mode`].

pub mod mode;
