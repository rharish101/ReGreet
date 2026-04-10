use zbus::{proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_path = "/org/freedesktop/Accounts",
    default_service = "org.freedesktop.Accounts",
    interface = "org.freedesktop.Accounts"
)]
trait AccountsService {
    /// Returns an array of [`User`] paths.
    fn list_cached_users(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts",
    interface = "org.freedesktop.Accounts.User"
)]
trait User {
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn shell(&self) -> zbus::Result<String>;
}
