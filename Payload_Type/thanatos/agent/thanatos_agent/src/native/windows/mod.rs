pub mod system {
    #[cfg(feature = "domaincheck")]
    pub use ffiwrappers::windows::domain;

    #[cfg(feature = "hostnamecheck")]
    pub use ffiwrappers::windows::hostname;

    #[cfg(feature = "usernamecheck")]
    pub use ffiwrappers::windows::username;
}
