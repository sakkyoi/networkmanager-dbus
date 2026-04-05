use std::marker::PhantomData;
use zvariant::{OwnedValue, Value};

use crate::proxy_access::HasProxy;

/// A generic read-only D-Bus property accessor.
///
/// This type provides a strongly-typed interface for retrieving
/// a property value from a D-Bus object, without requiring
/// a dedicated type per property.
///
/// # Design Notes
///
/// - This is a lightweight wrapper (zero-cost abstraction)
/// - Property names are provided as string literals
/// - Type safety is enforced at conversion time (`TryFrom<OwnedValue>`)
///
/// # Type Parameters
///
/// - `O`: Owner type that implements [`HasProxy`]
/// - `T`: Target Rust type of the property value
///
/// # Example
/// ```rust
/// let value: String = object.prop("SomeProperty").get().await?;
/// ```
pub struct Property<'a, O: ?Sized, T> {
    /// Reference to the owning object (provides the proxy)
    owner: &'a O,

    /// D-Bus property name (must match the interface definition)
    name: &'static str,

    /// Marker for the value type
    _marker: PhantomData<T>,
}

impl<'a, O: ?Sized, T> Property<'a, O, T> {
    /// Creates a new read-only property accessor.
    pub fn new(owner: &'a O, name: &'static str) -> Self {
        Self {
            owner,
            name,
            _marker: PhantomData,
        }
    }

    /// Returns the D-Bus property name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<'a, O, T> Property<'a, O, T>
where
    O: HasProxy + ?Sized,
    T: TryFrom<OwnedValue>,
    T::Error: Into<zbus::Error>,
{
    /// Retrieves the property value from D-Bus.
    ///
    /// Internally this performs a `Get` call on the
    /// `org.freedesktop.DBus.Properties` interface.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The D-Bus call fails
    /// - The property does not exist
    /// - The value cannot be converted into `T`
    pub async fn get(&self) -> zbus::Result<T> {
        self.owner.proxy().get_property(self.name).await
    }
}

/// A generic read-write D-Bus property accessor.
///
/// This extends [`Property`] by supporting write operations via
/// the `Set` method of the `org.freedesktop.DBus.Properties` interface.
///
/// # Design Notes
///
/// - Read and write types can differ if needed
/// - Defaults to the same type for both (`W = R`)
///
/// # Type Parameters
///
/// - `O`: Owner type implementing [`HasProxy`]
/// - `R`: Read type
/// - `W`: Write type (defaults to `R`)
///
/// # Example
///
/// ```rust
/// object.prop_rw::<bool>("Enabled").set(true).await?;
/// ```
pub struct WritableProperty<'a, O: ?Sized, R, W = R> {
    /// Reference to the owning object
    owner: &'a O,

    /// D-Bus property name
    name: &'static str,

    /// Marker for read/write types
    _marker: PhantomData<(R, W)>,
}

impl<'a, O: ?Sized, R, W> WritableProperty<'a, O, R, W> {
    /// Creates a new writable property accessor.
    pub fn new(owner: &'a O, name: &'static str) -> Self {
        Self {
            owner,
            name,
            _marker: PhantomData,
        }
    }

    /// Returns the D-Bus property name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<'a, O, R, W> WritableProperty<'a, O, R, W>
where
    O: HasProxy + ?Sized,
    R: TryFrom<OwnedValue>,
    R::Error: Into<zbus::Error>,
{
    /// Retrieves the property value from D-Bus.
    ///
    /// Equivalent to [`Property::get`].
    pub async fn get(&self) -> zbus::Result<R> {
        self.owner.proxy().get_property(self.name).await
    }
}

impl<'a, O, R, W> WritableProperty<'a, O, R, W>
where
    O: HasProxy + ?Sized,
{
    /// Set the property value via D-Bus.
    ///
    /// Internally this performs a `Set` call on the
    /// `org.freedesktop.DBus.Properties` interface.
    ///
    /// # Type Constraints
    ///
    /// The value must be convertible into a `zvariant::Value`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The property is not writable
    /// - The caller lacks permission (e.g. PolicyKit)
    /// - The D-Bus call fails
    pub async fn set<'t>(&self, value: W) -> zbus::fdo::Result<()>
    where
        W: 't + Into<Value<'t>>,
    {
        self.owner.proxy().set_property(self.name, value).await
    }
}

/// Extension trait providing ergonomic access to D-Bus properties.
///
/// This trait builds on top of [`HasProxy`] and offers convenient
/// helper methods for constructing generic property accessors
/// ([`Property`] and [`WritableProperty`]).
///
/// # Design Goals
///
/// - Avoid repetitive boilerplate when accessing D-Bus properties.
/// - Provide a uniform, strongly-typed interface
/// - Keep the abstraction lightweight and allocation-free
/// - Remain fully generic and reusable across different D-Bus services
///
/// # Usage
///
/// Any type that implements [`HasProxy`] automatically gains
/// access to this trait via a blanket implementation.
///
/// ```rust
/// let name: String = object.prop("Name").get().await?;
/// object.prop_rw::<bool>("Enabled").set(true).await?;
/// ```
///
/// # Notes
///
/// - Property names are provided as string literals and must match
///   the D-Bus interface definition exactly.
/// - Type safety is enforced at runtime via `TryFrom<OwnedValue>`.
/// - This trait does not perform caching; each call results in a
///   D-Bus roundtrip.
///
/// # See Also
///
/// - [`Property`] for read-only access
/// - [`WritableProperty`] for read-write access.
pub trait PropertyAccess: HasProxy {
    /// Creates a read-only property accessor.
    ///
    /// This is equivalent to constructing a [`Property`] manually,
    /// but provides a more ergonomic API.
    ///
    /// # Example
    ///
    /// ```rust
    /// let state: u32 = object.prop("State").get().await?;
    /// ```
    fn prop<T>(&self, name: &'static str) -> Property<'_, Self, T>
    where
        Self: Sized,
    {
        Property::new(self, name)
    }

    /// Creates a read-write property accessor.
    ///
    /// This allows both retrieving and updating the property value.
    ///
    /// # Example
    ///
    /// ```rust
    /// object.prop_rw::<bool>("Enabled").set(true).await?;
    /// ```
    fn prop_rw<T>(&self, name: &'static str) -> WritableProperty<'_, Self, T>
    where
        Self: Sized,
    {
        WritableProperty::new(self, name)
    }
}

/// Blanket implementation so that all types implementing [`HasProxy`]
/// automatically gain property access capabilities.
impl<T: HasProxy + Sized> PropertyAccess for T {}
