pub mod analysis;

use analysis::run_analysis;
use serde_json::json;

use std::io::{self, BufRead, BufReader, Read};

fn main() {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    loop {
        buffer.clear();

        if reader.read_line(&mut buffer).unwrap_or(0) == 0 {
            eprintln!("EOF");
            break;
        }

        let line = buffer.trim();
        if line.starts_with("Content-Length: ") {
            let len = line["Content-Length: ".len()..]
                .trim()
                .parse::<usize>()
                .unwrap();

            buffer.clear();
            if reader.read_line(&mut buffer).unwrap_or(0) == 0 {
                eprintln!("Error: Expected blank line after Content-Length header.");
                break;
            }

            let mut payload = vec![0; len];
            let mut total_read = 0;

            while total_read < len {
                match reader.read(&mut payload[total_read..]) {
                    Ok(0) => {
                        eprintln!("Error: Unexpected EOF while reading payload.");
                        break;
                    }
                    Ok(n) => total_read += n,
                    Err(e) => {
                        eprintln!("Error reading payload: {}", e);
                        break;
                    }
                }
            }

            if total_read != len {
                eprintln!(
                    "Error: Expected {} bytes, but read {} bytes.",
                    len, total_read
                );
                break;
            }

            let message = String::from_utf8(payload).unwrap();
            match run_analysis(message) {
                Ok(result) => {
                    println!("Content-Length: {}\r\n\r\n{}", result.len(), result);
                }

                Err(e) => {
                    let response = json!({
                        "jsonrpc": "2.0",
                        "method": "textDocument/publishDiagnostics",
                        "params": {
                            "uri": "file://unknown",
                            "diagnostics": vec!(e)
                        }
                    });
                    let output = serde_json::to_string(&response).unwrap();
                    println!("Content-Length: {}\r\n\r\n{}", output.len(), output)
                }
            }
        }
    }
}
