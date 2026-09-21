mod route;

use route::Route;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: routesift <routes-file>");
        eprintln!("reads URLs or paths on stdin, one per line, and prints the matching route");
        process::exit(2);
    }

    let routes_path = &args[1];
    let routes = match load_routes(routes_path) {
        Ok(routes) => routes,
        Err(err) => {
            eprintln!("routesift: {}", err);
            process::exit(1);
        }
    };

    if routes.is_empty() {
        eprintln!("routesift: {} defines no routes", routes_path);
        process::exit(1);
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // stdin.lock().lines() pulls one line at a time off the underlying
    // buffered reader, so a multi-gigabyte access log never has to sit
    // in memory at once, only the current line does.
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                eprintln!("routesift: error reading stdin: {}", err);
                process::exit(1);
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let path = route::extract_path(&line);
        let result = routes.iter().find_map(|r| r.matches(path).map(|p| (r.name.as_str(), p)));

        let write_result = match result {
            Some((name, params)) if params.is_empty() => writeln!(out, "{}\t{}", name, line),
            Some((name, params)) => {
                let pairs: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                writeln!(out, "{}\t{}\t{}", name, pairs.join("&"), line)
            }
            None => writeln!(out, "NOMATCH\t{}", line),
        };

        if let Err(err) = write_result {
            eprintln!("routesift: error writing output: {}", err);
            process::exit(1);
        }
    }
}

fn load_routes(path: &str) -> Result<Vec<Route>, String> {
    // the routes file is config, not the stream we care about keeping
    // out of memory, so reading it whole is fine.
    let content = fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))?;

    let mut routes = Vec::new();
    for (i, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut fields = line.splitn(2, char::is_whitespace);
        let name = fields.next().unwrap_or("").trim();
        let pattern = fields.next().unwrap_or("").trim();
        if name.is_empty() || pattern.is_empty() {
            return Err(format!("{}:{}: expected '<name> <pattern>'", path, i + 1));
        }

        let route = Route::parse(name, pattern).map_err(|e| format!("{}:{}: {}", path, i + 1, e))?;
        routes.push(route);
    }
    Ok(routes)
}
