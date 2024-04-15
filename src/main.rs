mod client;
mod config;

#[macro_use]
mod utils;

use clap::Parser;
use std::sync::mpsc;
use std::time::Instant;

use client::*;
use config::*;

// -----
static mut VERBOSE: bool = false;

#[tokio::main]
async fn main() -> Result<(), utils::Errors> {
    let start_time = Instant::now();

    // TODO: Quick fix for oom issues in a high volume test. Do beter.
    // 0 = success, 1 = failed, 2 = error
    // ---
    // let (send, recv) = mpsc::channel();
    let (t_sender, t_recv) = mpsc::channel();
    let (s_sender, s_recv) = mpsc::channel();
    let (f_sender, f_recv) = mpsc::channel();
    let (e_sender, e_recv) = mpsc::channel();

    let config: Config = Config::parse();
    if !config.is_valid() {
        eprintln!(
            "Error: Configuration is invalid, see '--help', in:\n{:#?}",
            config
        );
        std::process::exit(1);
    }
    set_verbose!(config.verbose());

    let configs = config.to_vector()?;
    let expect: usize = configs.clone().into_iter().map(|c| c.iterations).sum();

    //let mut results = vec![];
    for c in configs {
        let t_send = t_sender.clone();
        let s_send = s_sender.clone();
        let f_send = f_sender.clone();
        let e_send = e_sender.clone();
        tokio::spawn(async move {
            let client = Client::new(&c.method, &c.endpoint(), c.headers, c.iterations);

            // TODO: Quick fix for oom issues in a high volume test. Do beter.
            // 0 = success, 1 = failed, 2 = error
            // ---
            //let _ = send.send(resp);
            let mut t: usize = 0;
            let mut s: usize = 0;
            let mut f: usize = 0;
            let mut e: usize = 0;

            if client.is_err() {
                // TODO: Quick fix for oom issues in a high volume test. Do beter.
                // 0 = success, 1 = failed, 2 = error
                // ---
                eprintln!(
                    "{:?}",
                    response_error_vector!(client.unwrap_err().to_string())
                );
                let _ = t_send.send(expect); // for exit / done

            //let _ = send.send(response_error_vector!(client.unwrap_err().to_string()));
            } else {
                let c = client.unwrap();
                let resp = c.run().await;

                for r in resp {
                    // TODO: Quick fix for oom issues in a high volume test. Do beter.
                    // 0 = success, 1 = failed, 2 = error
                    // ---
                    match r {
                        Ok(r) => {
                            let code = r.status().as_u16();
                            debug!(format!(
                                "code={:} method={} path={:}",
                                code, c.method, c.endpoint,
                            ));
                            if code >= 200 && code < 300 {
                                s += 1;
                            } else {
                                f += 1;
                            }
                        }
                        Err(err) => {
                            debug!(format!(
                                "method={} path={:} error='{:?}'",
                                c.method, c.endpoint, err
                            ));
                            e += 1;
                        }
                    }
                    t += 1;
                }

                // TODO: Quick fix for oom issues in a high volume test. Do beter.
                // 0 = success, 1 = failed, 2 = error
                // ---
                let _ = s_send.send(s);
                let _ = f_send.send(f);
                let _ = e_send.send(e);
                let _ = t_send.send(t);
            }
        });
    }

    // TODO: Quick fix for oom issues in a high volume test. Do beter.
    // 0 = success, 1 = failed, 2 = error
    // ---
    let mut t: usize = 0;
    let mut s: usize = 0;
    let mut f: usize = 0;
    let mut e: usize = 0;
    while t < expect {
        if let Ok(r) = s_recv.recv() {
            s += r;
        };
        if let Ok(r) = f_recv.recv() {
            f += r;
        };
        if let Ok(r) = e_recv.recv() {
            e += r;
        };
        if let Ok(r) = t_recv.recv() {
            t += r;
        };
    }

    println!("-------------------------");
    println!("  Requests sent: {:?}", t);
    println!("-------------------------");
    println!("        success: {:?}", s);
    println!("        failure: {:?}", f);
    println!("         errors: {:?}", e);
    println!("----------------------");

    // while results.len() < expect {
    //     let mut sent = match recv.recv() {
    //         Ok(sent) => sent,
    //         Err(err) => response_error_vector!(err.to_string()),
    //     };
    //     results.append(&mut sent);
    // }

    // [0]total, [1]pass, [2]fail, [3]error
    // let mut count: (usize, usize, usize, usize) = (0, 0, 0, 0);
    // let mut pass: Vec<reqwest::Response> = vec![];
    // let mut fail: Vec<reqwest::Response> = vec![];
    // let mut error: Vec<utils::Errors> = vec![];

    // for result in results.into_iter() {
    //     count.0 += 1;
    //     match result {
    //         Ok(response) => {
    //             let s = response.status();
    //             let code = s.as_u16();

    //             if code >= 200 && code < 300 {
    //                 count.1 += 1;
    //                 pass.push(response);
    //             } else {
    //                 count.2 += 1;
    //                 fail.push(response);
    //             }
    //         }
    //         Err(err) => {
    //             count.3 += 1;

    //             // Report errors on debug
    //             debug!(format!("{:?}", err));
    //             error.push(err);
    //         }
    //     }
    // }
    //
    // println!("-------------------------");
    // println!("  Requests sent: {:?}", count.0);
    // println!("-------------------------");
    // println!("        success: {:?}", count.1);
    // println!("        failure: {:?}", count.2);
    // println!("         errors: {:?}", count.3);
    // println!("----------------------");

    let duration = Instant::now() - start_time;
    println!("Run took: {:?}", duration);

    Ok(())
}
