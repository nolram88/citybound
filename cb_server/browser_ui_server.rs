extern crate rouille;
use self::rouille::{Response, extension_to_mime};
use crate::init;
#[cfg(feature = "embed_assets")]
use rust_embed_flag::RustEmbed;

const MISSING_UI_HTML: &str = "<html><body><h1>Citybound server is running</h1><p>Browser UI bundle missing. Run `npm run build-browser` to build `cb_browser_ui/dist`.</p></body></html>";

#[cfg_attr(feature = "embed_assets", allow(dead_code))]
fn sanitize_asset_path(path: &str) -> Option<&str> {
    let sanitized = path.trim_start_matches('/');
    if sanitized.contains("..") || sanitized.is_empty() {
        return None;
    }
    Some(sanitized)
}

#[cfg(feature = "embed_assets")]
#[derive(RustEmbed)]
#[folder = "cb_browser_ui/dist/"]
struct EmbeddedAsset;

#[cfg(feature = "embed_assets")]
fn load_asset(path: &str) -> Option<Vec<u8>> {
    EmbeddedAsset::get(path)
}

#[cfg(not(feature = "embed_assets"))]
fn load_asset(path: &str) -> Option<Vec<u8>> {
    let sanitized = sanitize_asset_path(path)?;
    let root = std::env::var("CITYBOUND_ASSET_DIR").unwrap_or_else(|_| "cb_browser_ui/dist".to_owned());
    let path = ::std::path::Path::new(&root).join(sanitized);
    ::std::fs::read(path).ok()
}

fn render_root_html(
    template: Option<&[u8]>,
    version: &str,
    network_config: &init::NetworkConfig,
) -> String {
    let template = template
        .and_then(|bytes| ::std::str::from_utf8(bytes).ok())
        .unwrap_or(MISSING_UI_HTML)
        .to_owned();

    template
        .replace("CB_VERSION", version.trim())
        .replace(
            "CB_BATCH_MESSAGE_BYTES",
            &format!("{}", network_config.batch_msg_bytes),
        )
        .replace(
            "CB_ACCEPTABLE_TURN_DISTANCE",
            &format!("{}", network_config.ok_turn_dist),
        )
        .replace(
            "CB_SKIP_TURNS_PER_TURN_AHEAD",
            &format!("{}", network_config.skip_ratio),
        )
}

pub fn start_browser_ui_server(version: &'static str, network_config: init::NetworkConfig) {
    rouille::start_server(network_config.serve_host_port.clone(), move |request| {
        if request.raw_url() == "/" {
            println!("{:?} loaded page", request.remote_addr());

            let rendered = render_root_html(load_asset("index.html").as_deref(), version, &network_config);

            Response::html(rendered)
        } else if let Some(asset) = load_asset(&request.url()[1..]) {
            Response::from_data(
                if request.url().ends_with(".wasm") {
                    "application/wasm"
                } else {
                    extension_to_mime(request.url().split('.').last().unwrap_or(""))
                },
                asset,
            )
        } else {
            Response::html(format!("404 error. Not found: {}", request.url())).with_status_code(404)
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn sample_network_config() -> init::NetworkConfig {
        init::NetworkConfig {
            mode: "local".to_owned(),
            serve_host_port: "localhost:1234".to_owned(),
            bind_sim: "localhost:9999".to_owned(),
            batch_msg_bytes: 500000,
            ok_turn_dist: 2,
            skip_ratio: 5,
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos();
        std::env::temp_dir().join(format!("citybound-{}-{}", name, suffix))
    }

    fn free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
        let port = listener.local_addr().expect("port").port();
        drop(listener);
        port
    }

    fn http_get(port: u16, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            path
        );
        stream
            .write_all(request.as_bytes())
            .expect("write request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");
        response
    }

    fn response_body(response: &str) -> &str {
        response.split("\r\n\r\n").nth(1).unwrap_or("")
    }

    fn wait_for_server(port: u16) {
        for _ in 0..50 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("server did not start");
    }

    #[test]
    fn sanitizes_asset_paths() {
        assert_eq!(sanitize_asset_path("/index.html"), Some("index.html"));
        assert_eq!(sanitize_asset_path("index.html"), Some("index.html"));
        assert_eq!(sanitize_asset_path(""), None);
        assert_eq!(sanitize_asset_path("/"), None);
        assert_eq!(sanitize_asset_path("../secret"), None);
    }

    #[test]
    fn renders_fallback_html_when_template_missing() {
        let rendered = render_root_html(None, "v1.2.3", &sample_network_config());

        assert!(rendered.contains("Citybound server is running"));
        assert!(rendered.contains("npm run build-browser"));
    }

    #[test]
    fn renders_network_values_into_template() {
        let template = b"<p>CB_VERSION|CB_BATCH_MESSAGE_BYTES|CB_ACCEPTABLE_TURN_DISTANCE|CB_SKIP_TURNS_PER_TURN_AHEAD</p>";
        let rendered = render_root_html(Some(template), " v1.2.3 \n", &sample_network_config());

        assert_eq!(rendered, "<p>v1.2.3|500000|2|5</p>");
    }

    #[test]
    fn load_asset_reads_from_configured_asset_dir() {
        let _guard = env_lock().lock().unwrap();
        let dir = unique_test_dir("load-asset");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("index.html"), "<html>ok</html>").expect("write asset");
        std::env::set_var("CITYBOUND_ASSET_DIR", &dir);

        let asset = load_asset("index.html").expect("asset should load");
        assert_eq!(String::from_utf8(asset).unwrap(), "<html>ok</html>");
        assert!(load_asset("../secret").is_none());

        std::env::remove_var("CITYBOUND_ASSET_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_responds_with_rendered_index_template() {
        let _guard = env_lock().lock().unwrap();
        let dir = unique_test_dir("server-index");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(
            dir.join("index.html"),
            "<html>CB_VERSION|CB_BATCH_MESSAGE_BYTES|CB_ACCEPTABLE_TURN_DISTANCE|CB_SKIP_TURNS_PER_TURN_AHEAD</html>",
        )
        .expect("write index");
        std::env::set_var("CITYBOUND_ASSET_DIR", &dir);

        let mut cfg = sample_network_config();
        let port = free_port();
        cfg.serve_host_port = format!("127.0.0.1:{}", port);
        thread::spawn(move || start_browser_ui_server("v1.2.3", cfg));
        wait_for_server(port);

        let response = http_get(port, "/");
        let body = response_body(&response);
        assert!(body.contains("v1.2.3|500000|2|5"));

        std::env::remove_var("CITYBOUND_ASSET_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_returns_fallback_html_when_index_missing() {
        let _guard = env_lock().lock().unwrap();
        let dir = unique_test_dir("server-fallback");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::env::set_var("CITYBOUND_ASSET_DIR", &dir);

        let mut cfg = sample_network_config();
        let port = free_port();
        cfg.serve_host_port = format!("127.0.0.1:{}", port);
        thread::spawn(move || start_browser_ui_server("v1.2.3", cfg));
        wait_for_server(port);

        let response = http_get(port, "/");
        let body = response_body(&response);
        assert!(body.contains("Citybound server is running"));
        assert!(body.contains("npm run build-browser"));

        std::env::remove_var("CITYBOUND_ASSET_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
