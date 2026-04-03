use zbus::Proxy;

/// Trait representing the minimal capability required to access D-Bus properties.
///
/// This is the shared foundation for all generic abstractions built on top of
/// `zbus::Proxy`, including:
///
/// - property access (`PropertyAccess`)
/// - signal subscription (`SignalAccess`)
///
/// # Design Notes
///
/// - This trait intentionally exposes only a read-only reference to the proxy.
/// - It allows higher-level abstractions to remain generic and reusable.
/// - Any wrapper type (e.g. domain object, facades) can implement this trait.
///
/// # Example
/// ```rust
/// impl HasProxy for MyObject<'_> {
///     fn proxy(&self) -> &Proxy<'_> {
///         &self.proxy
///     }
/// }
///```
pub trait HasProxy {
    /// Returns a reference to the underlying D-Bus proxy.
    fn proxy(&self) -> &Proxy<'_>;
}
