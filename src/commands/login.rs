use std::sync::mpsc;
use std::thread;
use tiny_http::{Response, Server};
use url::Url;
use webbrowser;

pub fn run() {
    let auth_url = std::env::var("OSBN_AUTH_URL").unwrap_or_else(|_| {
        "https://api.ostadban.tech/user/auth?provider=google&redirect_uri=http://localhost:8000"
            .to_string()
    });

    let (tx, rx) = mpsc::channel();
    let server_thread = thread::spawn(move || {
        let server = Server::http("127.0.0.1:8000").unwrap();
        if let Ok(request) = server.recv() {
            let query = request.url().split('?').nth(1).unwrap_or("").to_string();
            let params: Vec<(String, String)> = Url::parse(&format!("http://localhost/?{}", query))
                .unwrap()
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            tx.send(params).unwrap();

            let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Login complete</title>
                <script type="text/javascript">
                    window.onload = function() {
                        window.open('', '_self').close();
                    };
                </script>
            </head>
            <body>
            </body>
            </html>
            "#;

            let header =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();
            let response = Response::from_string(html).with_header(header);
            let _ = request.respond(response);
        }
    });

    webbrowser::open(&auth_url).expect("Failed to open browser");
    println!("Browser opened for login...");

    let params = rx.recv().unwrap();
    println!("Received params from redirect: {:?}", params);

    server_thread.join().unwrap();
}
