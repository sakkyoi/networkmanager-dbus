use futures_util::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use zbus::{message::Message};
use zvariant::Type;

use crate::proxy_access::HasProxy;

/// Extension trait providing ergonomic access to D-Bus signal streams.
///
/// This trait builds on top of [`HasProxy`] and offers helper methods
/// for subscribing to D-Bus signals.
///
/// # Design Philosophy
///
/// Unlike properties (which are request/response based), signals are:
///
/// - asynchronous
/// - push-based
/// - potentially unbounded objects.
///
/// # Features
///
/// This trait provides two levels of access:
///
/// - **Raw access** -> [`receive_signal_raw`]
///   - Returns raw [`zbus::Message`] objects
///   - Useful for low-level inspection or custom decoding
///
/// - **Typed access** -> [`receive_signal_typed`]
///   - Automatically deserializes the message body into `T`
///   - Provides a strongly-typed, ergonomic API
///
/// # Type Requirements
///
/// For typed signals:
///
/// - `T: DeserializeOwned` -> required for deserialization
/// - `T: Type` -> required to match D-Bus signature at runtime
///
/// # Typical Usage
///
/// ```rust
/// let mut stream = object
///     .receive_signal_typed::<(u32, u32, u32)>("StateChanged")
///     .await?;
/// ```
///
/// # Lifetime Notes
///
/// The returned stream borrows from the underlying proxy. Therefore:
///
/// - The stream must not outlive `self`
/// - The owning object must remain alive while consuming the stream
///
/// # See Also
///
/// - [`receive_signal_raw`](SignalAccess::receive_signal_raw)
/// - [`receive_signal_typed`](SignalAccess::receive_signal_typed)
pub trait SignalAccess: HasProxy {
    /// Subscribe to a raw D-Bus signal stream.
    ///
    /// Each item in the stream is a [`zbus::Message`] representing a signal.
    ///
    /// # Use Cases
    ///
    /// - Inspecting message metadata (headers, sender, path, etc.)
    /// - Handling complex or dynamic payloads
    /// - Debugging / tracing
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be established.
    async fn receive_signal_raw(
        &self,
        name: &'static str,
    ) -> zbus::Result<impl Stream<Item = Message> + '_>
    where
        Self: Sized,
    {
        self.proxy().receive_signal(name).await
    }

    /// Subscribe to a typed D-Bus signal stream.
    ///
    /// Each incoming signal message is deserialized into `T`.
    ///
    /// # Type Requirements
    ///
    /// - `T: DeserializeOwned` -> for serde-based decoding
    /// - `T: Type` -> for D-Bus signature matching
    ///
    /// # Supported Types
    ///
    /// This works well with:
    ///
    /// - primitives (`u32`, `bool`, `String`, ...)
    /// - tuples (`(u32, u32)`, `(String, bool)`, ...)
    /// - structs that match the D-Bus body layout
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut stream = object
    ///     .receive_signal_typed::<(u32, u32, u32)>("StateChanged")
    ///     .await?;
    /// ```
    ///
    /// # Errors
    ///
    /// - Subscription failure -> returned by outer `Result`
    /// - Deserialization failure -> returned per item (`Stream<Item = Result<T>>`)
    async fn receive_signal_typed<T>(
        &self,
        name: &'static str,
    ) -> zbus::Result<impl Stream<Item = zbus::Result<T>> + '_>
    where
        Self: Sized,
        T: DeserializeOwned + Type,
    {
        let stream = self.proxy().receive_signal(name).await?;
        Ok(stream.map(|msg| {
            msg.body()
                .deserialize::<T>()
                .map_err(Into::into)
        }))
    }
}

/// Blanket implementation so that all types implementing [`HasProxy`]
/// automatically gain signal access capabilities.
impl<T: HasProxy + Sized> SignalAccess for T {}
