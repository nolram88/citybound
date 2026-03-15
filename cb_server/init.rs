extern crate backtrace;
extern crate clap;

use std::time::{Instant, Duration};

fn display_host_for_mode(mode: &str) -> &'static str {
    match mode {
        "local" => "localhost",
        "lan" => "<your LAN IP>",
        "internet" => "<your public IP>",
        _ => unreachable!(),
    }
}

fn display_url(network_config: &NetworkConfig) -> String {
    let port = network_config
        .serve_host_port
        .split(':')
        .last()
        .unwrap_or("");
    format!("http://{}:{}", display_host_for_mode(network_config.mode.as_str()), port)
}

pub fn print_start_message(version: &str, network_config: &NetworkConfig) {
    let my_host = display_url(network_config);

    println!("  {: ^41}  ", format!("Citybound {}", version.trim()));
    println!();
    println!("  {: ^41}  ", "This is the simulation server.");
    println!("  {: ^41}  ", "To connect and start playing, please open");
    println!("  {: ^41}  ", "this address in Chrome/Firefox/Safari:");
    println!("╭───────────────────────────────────────────╮");
    println!("│ {: ^41} │", my_host);
    println!("╰───────────────────────────────────────────╯");
}

#[derive(Clone)]
pub struct NetworkConfig {
    pub mode: String,
    pub serve_host_port: String,
    pub bind_sim: String,
    pub batch_msg_bytes: usize,
    pub ok_turn_dist: usize,
    pub skip_ratio: usize,
}

fn build_cli<'a, 'b>(version: &'a str) -> clap::App<'a, 'b> {
    use self::clap::{Arg, App};
    App::new("citybound")
        .version(version.trim())
        .author("ae play (Anselm Eickhoff)")
        .about("The city is us.")
        .arg(
            Arg::with_name("CITY_FOLDER")
                .help("Sets the folder containing the city savegame")
                .default_value("./city")
                .index(1),
        )
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .value_name("local/lan/internet")
                .display_order(0)
                .possible_values(&["local", "lan", "internet"])
                .default_value("local")
                .help("Where to expose the simulation. Sets defaults other settings."),
        )
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .value_name("host:port")
                .default_value_ifs(&[
                    ("mode", Some("local"), "localhost:1234"),
                    ("mode", Some("lan"), "0.0.0.0:1234"),
                    ("mode", Some("internet"), "0.0.0.0:1234"),
                ])
                .help("Address and port to serve the browser UI from"),
        )
        .arg(
            Arg::with_name("bind-sim")
                .long("bind-sim")
                .value_name("host:port")
                .default_value_ifs(&[
                    ("mode", Some("local"), "localhost:9999"),
                    ("mode", Some("lan"), "0.0.0.0:9999"),
                    ("mode", Some("internet"), "0.0.0.0:9999"),
                ])
                .help("Address and port to accept connections to the simulation from"),
        )
        .arg(
            Arg::with_name("batch-msg-b")
                .long("batch-msg-bytes")
                .value_name("n-bytes")
                .default_value("500000")
                .help("How many bytes of simulation messages to batch"),
        )
        .arg(
            Arg::with_name("ok-turn-dist")
                .long("ok-turn-dist")
                .value_name("n-turns")
                .default_value_ifs(&[
                    ("mode", Some("local"), "2"),
                    ("mode", Some("lan"), "10"),
                    ("mode", Some("internet"), "30"),
                ])
                .help("How many network turns client/server can be behind before skipping"),
        )
        .arg(
            Arg::with_name("skip-ratio")
                .long("skip-ratio")
                .value_name("n-turns")
                .default_value("5")
                .help("How many network turns to skip if server/client are ahead"),
        )
}

fn matches_to_config(matches: &clap::ArgMatches<'_>) -> (NetworkConfig, String) {
    (
        NetworkConfig {
            serve_host_port: matches.value_of("bind").unwrap().to_owned(),
            bind_sim: matches.value_of("bind-sim").unwrap().to_owned(),
            mode: matches.value_of("mode").unwrap().to_owned(),
            batch_msg_bytes: matches.value_of("batch-msg-b").unwrap().parse().unwrap(),
            ok_turn_dist: matches.value_of("ok-turn-dist").unwrap().parse().unwrap(),
            skip_ratio: matches.value_of("skip-ratio").unwrap().parse().unwrap(),
        },
        matches.value_of("CITY_FOLDER").unwrap().to_owned(),
    )
}

pub fn parse_cmd_line_args_from<I, T>(
    version: &str,
    args: I,
) -> Result<(NetworkConfig, String), clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<::std::ffi::OsString> + Clone,
{
    let matches = build_cli(version).get_matches_from_safe(args)?;
    Ok(matches_to_config(&matches))
}

pub fn match_cmd_line_args(version: &str) -> (NetworkConfig, String) {
    parse_cmd_line_args_from(version, ::std::env::args_os()).unwrap_or_else(|e| e.exit())
}

pub fn ensure_crossplatform_proper_thread<F: Fn() -> () + Send + 'static>(callback: F) {
    // Makes sure that:
    // a) on Windows we use a dummy thread with manually set stack size
    // b) on Mac/Linux we use the main thread, because we have to create the UI there

    if cfg!(windows) {
        let dummy_thread = ::std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(callback)
            .unwrap();
        dummy_thread.join().unwrap();
    } else {
        callback();
    }
}

use std::panic::{set_hook, PanicHookInfo, Location};
use self::backtrace::Backtrace;
use std::fs::File;
use std::io::Write;

pub fn set_error_hook() {
    let callback: Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static> =
        Box::new(move |panic_info| {
        let title = "SIMULATION BROKE :(";

        let message = match panic_info.payload().downcast_ref::<String>() {
            Some(string) => string.clone(),
            None => match panic_info.payload().downcast_ref::<&'static str>() {
                Some(static_str) => (*static_str).to_string(),
                None => "Weird error type".to_string(),
            },
        };

        let backtrace = Backtrace::new();
        let location = format!(
            "at {}, line {}",
            panic_info
                .location()
                .map(Location::file)
                .unwrap_or("unknown"),
            panic_info.location().map(Location::line).unwrap_or(0)
        );

        let body = format!(
            "WHAT HAPPENED:\n{}\n\nWHERE IT HAPPENED:\n{}\n\nWHERE EXACTLY:\n{:?}",
            message, location, backtrace
        );

        let report_guide = "HOW TO REPORT \
                            BUGS:\nhttps://github.com/citybound/citybound/blob/master/\
                            CONTRIBUTING.md#reporting-bugs";

        let mut error_file_path = ::std::env::temp_dir();
        error_file_path.push("cb_last_error.txt");

        println!(
            "{}\n\n{}\n\nERROR ALSO SAVED AT {:?}\nTHIS CRASH PROBABLY CORRUPTED YOUR SAVEGAME :(",
            title, body, error_file_path
        );

        {
            if let Ok(mut file) = File::create(&error_file_path) {
                let file_content = format!("{}\n\n{}\n\n{}", title, report_guide, body);
                let file_content = file_content.replace("\n", "\r\n");

                file.write_all(file_content.as_bytes())
                    .expect("Error writing error file, lol");
            };
        }
    });

    set_hook(callback);
}

pub struct FrameCounter {
    last_frame: Instant,
    elapsed_ms_collected: Vec<f32>,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            last_frame: Instant::now(),
            elapsed_ms_collected: Vec::new(),
        }
    }

    pub fn start_frame(&mut self) {
        let elapsed_ms = self.last_frame.elapsed().as_secs() as f32 * 1000.0
            + self.last_frame.elapsed().subsec_nanos() as f32 / 10.0E5;

        self.elapsed_ms_collected.push(elapsed_ms);

        if self.elapsed_ms_collected.len() > 10 {
            self.elapsed_ms_collected.remove(0);
        }

        self.last_frame = Instant::now();
    }

    pub fn sleep_if_faster_than(&self, fps: usize) {
        let ideal_frame_duration = Duration::from_millis((1000.0 / (fps as f32)) as u64);

        if let Some(pos_difference) = ideal_frame_duration.checked_sub(self.last_frame.elapsed()) {
            ::std::thread::sleep(pos_difference);
        }
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    fn panic_hook_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn parses_default_local_args() {
        let (cfg, city_folder) = parse_cmd_line_args_from("0.0.0", vec!["citybound"]).unwrap();

        assert_eq!(cfg.mode, "local");
        assert_eq!(cfg.serve_host_port, "localhost:1234");
        assert_eq!(cfg.bind_sim, "localhost:9999");
        assert_eq!(cfg.batch_msg_bytes, 500000);
        assert_eq!(cfg.ok_turn_dist, 2);
        assert_eq!(cfg.skip_ratio, 5);
        assert_eq!(city_folder, "./city");
    }

    #[test]
    fn parses_mode_internet_defaults() {
        let (cfg, _) =
            parse_cmd_line_args_from("0.0.0", vec!["citybound", "--mode", "internet"]).unwrap();

        assert_eq!(cfg.mode, "internet");
        assert_eq!(cfg.serve_host_port, "0.0.0.0:1234");
        assert_eq!(cfg.bind_sim, "0.0.0.0:9999");
        assert_eq!(cfg.ok_turn_dist, 30);
    }

    #[test]
    fn parses_explicit_overrides() {
        let (cfg, city_folder) = parse_cmd_line_args_from(
            "0.0.0",
            vec![
                "citybound",
                "my_city",
                "--mode",
                "lan",
                "--bind",
                "127.0.0.1:8080",
                "--bind-sim",
                "127.0.0.1:9000",
                "--batch-msg-bytes",
                "12345",
                "--ok-turn-dist",
                "99",
                "--skip-ratio",
                "7",
            ],
        )
        .unwrap();

        assert_eq!(cfg.mode, "lan");
        assert_eq!(cfg.serve_host_port, "127.0.0.1:8080");
        assert_eq!(cfg.bind_sim, "127.0.0.1:9000");
        assert_eq!(cfg.batch_msg_bytes, 12345);
        assert_eq!(cfg.ok_turn_dist, 99);
        assert_eq!(cfg.skip_ratio, 7);
        assert_eq!(city_folder, "my_city");
    }

    #[test]
    fn formats_display_url_for_all_modes() {
        let mut cfg = NetworkConfig {
            mode: "local".to_owned(),
            serve_host_port: "0.0.0.0:1234".to_owned(),
            bind_sim: "0.0.0.0:9999".to_owned(),
            batch_msg_bytes: 1,
            ok_turn_dist: 1,
            skip_ratio: 1,
        };

        assert_eq!(display_url(&cfg), "http://localhost:1234");

        cfg.mode = "lan".to_owned();
        assert_eq!(display_url(&cfg), "http://<your LAN IP>:1234");

        cfg.mode = "internet".to_owned();
        assert_eq!(display_url(&cfg), "http://<your public IP>:1234");
    }

    #[test]
    fn parse_rejects_invalid_mode() {
        let result = parse_cmd_line_args_from("0.0.0", vec!["citybound", "--mode", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn ensure_crossplatform_proper_thread_runs_callback() {
        let ran = Arc::new(AtomicBool::new(false));
        let ran_clone = Arc::clone(&ran);

        ensure_crossplatform_proper_thread(move || {
            ran_clone.store(true, Ordering::SeqCst);
        });

        assert!(ran.load(Ordering::SeqCst));
    }

    #[test]
    fn frame_counter_keeps_recent_samples_only() {
        let mut frame_counter = FrameCounter::new();

        for _ in 0..15 {
            frame_counter.start_frame();
            std::thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(frame_counter.elapsed_ms_collected.len(), 10);
        frame_counter.sleep_if_faster_than(120);
    }

    #[test]
    fn error_hook_writes_report_for_str_payload() {
        let _guard = panic_hook_lock().lock().unwrap();
        let path = std::env::temp_dir().join("cb_last_error.txt");
        let previous_hook = std::panic::take_hook();

        set_error_hook();
        let _ = std::panic::catch_unwind(|| panic!("boom"));

        std::panic::set_hook(previous_hook);

        let report = std::fs::read_to_string(&path).expect("expected error report");
        assert!(report.contains("SIMULATION BROKE :("));
        assert!(report.contains("WHAT HAPPENED:"));
        assert!(report.contains("boom"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn error_hook_writes_report_for_string_payload() {
        let _guard = panic_hook_lock().lock().unwrap();
        let path = std::env::temp_dir().join("cb_last_error.txt");
        let previous_hook = std::panic::take_hook();

        set_error_hook();
        let _ = std::panic::catch_unwind(|| std::panic::panic_any(String::from("boom-string")));

        std::panic::set_hook(previous_hook);

        let report = std::fs::read_to_string(&path).expect("expected error report");
        assert!(report.contains("boom-string"));

        let _ = std::fs::remove_file(path);
    }
}
