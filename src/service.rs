//! Windows Service Module

use crate::client::Client;
use futures::future::abortable;
use futures_lite::future::block_on;
///'
///Install:
///  New-Service -Name "BoreTunnel" -Description "Creates a bore tunnel." -BinaryPathName "D:\dev\Service_rs\target\release\bore_win_service.exe localhost 3389 tunnel.mobilewebapp.net 11250"
///
///
use windows_service::define_windows_service;

use std::env;
use std::{ffi::OsString, time::Duration, u16};
use tokio::runtime::Runtime;
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

struct Args<'l> {
    host: &'l str,
    host_port: u16,
    to: &'l str,
    to_port: u16,
    sec: Option<&'l str>,
}

define_windows_service!(ffi_service_main, my_service_main);

/// .
///
/// # Panics
///
/// Panics if .
pub fn start_service() {
    service_dispatcher::start("BoreTunnel", ffi_service_main).unwrap()
}

/// .
///
/// # Panics
///
/// Panics if .
pub fn my_service_main(arguments: Vec<OsString>) {
    let (bore_service, run_handle) = abortable(run_client_service(arguments));

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.

                let _ = run_handle.abort();
                ServiceControlHandlerResult::NoError
            }
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("BoreTunnel", event_handler).unwrap();

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .unwrap();

    let handler = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let _guard = rt.enter();

        block_on(async {
            let _ = bore_service.await.unwrap();
        });
    });

    let _ = handler.join().unwrap();

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .unwrap();
}

async fn run_client_service(_arguments: Vec<OsString>) {
    let args: Vec<_> = env::args().collect();

    let secret: Option<&str>;

    if args.get(5).is_none() {
        secret = None;
    } else {
        secret = Some(args[5].as_str());
    }

    let struct_args = Args {
        host: args[1].as_str(),
        host_port: args[2].parse::<u16>().unwrap(),
        to: args[3].as_str(),
        to_port: args[4].parse::<u16>().unwrap(),
        sec: secret,
    };

    let myclient = Client::new(
        struct_args.host,
        struct_args.host_port,
        struct_args.to,
        struct_args.to_port,
        struct_args.sec,
    )
    .await
    .unwrap();
    myclient.listen().await.unwrap();
}
