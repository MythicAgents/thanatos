use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, BlockingSender};
use errors::ThanatosError;

const DBUS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Gets the system's joined domains
pub fn domains() -> Result<Vec<String>, ThanatosError> {
    let conn = dbus::blocking::Connection::new_system().map_err(ThanatosError::DbusError)?;

    if check_domain_joined(&conn)? == false {
        return Err(ThanatosError::NotDomainJoined);
    }

    let realm_paths = get_realm_paths(&conn)?;

    Ok(realm_paths
        .into_iter()
        .map_while(|realm_path| {
            let proxy = dbus::blocking::Proxy::new(
                "org.freedesktop.realmd",
                realm_path,
                DBUS_TIMEOUT,
                &conn,
            );

            let configured: String = proxy
                .get("org.freedesktop.realmd.Realm", "Configured")
                .ok()?;

            (!configured.is_empty())
                .then(|| proxy.get("org.freedesktop.realmd.Realm", "Name").ok())
                .flatten()
        })
        .collect::<Vec<String>>())
}

fn get_realm_paths(
    conn: &dbus::blocking::Connection,
) -> Result<Vec<dbus::strings::Path>, ThanatosError> {
    let proxy = dbus::blocking::Proxy::new(
        "org.freedesktop.realmd",
        "/org/freedesktop/realmd",
        DBUS_TIMEOUT,
        conn,
    );

    proxy
        .get("org.freedesktop.realmd.Provider", "Realms")
        .map_err(ThanatosError::DbusError)
}

fn check_domain_joined(conn: &dbus::blocking::Connection) -> Result<bool, ThanatosError> {
    let msg = dbus::message::Message::new_method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListActivatableNames",
    )
    .unwrap();

    let reply = conn
        .send_with_reply_and_block(msg, DBUS_TIMEOUT)
        .map_err(ThanatosError::DbusError)?;

    let services: Vec<String> = reply.get1().unwrap();

    Ok(services
        .iter()
        .find(|&svc| svc == "org.freedesktop.realmd")
        .is_some())
}

#[cfg(test)]
mod tests {
    use errors::ThanatosError;

    #[test]
    fn domain_test() {
        let domains = super::domains();

        match domains {
            Ok(_) => (),
            Err(ThanatosError::NotDomainJoined) => (),
            e => panic!("{:?}", e),
        }
    }
}
