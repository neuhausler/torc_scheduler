// The MIT License (MIT)
//
// Copyright (c) 2016 AT&T
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use hyper::Client;
use hyper::status::StatusCode;
use rustc_serialize::json;
use std::io::Read;

const DEFAULT_PORT: i16 = 8085;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub fn shutdown_node(proxy_ip: &String, node_ip: &String) {
    send_command_to_node(proxy_ip.clone(),
                         DEFAULT_PORT,
                         format!("ipmitool -H {} -I lanplus -U root -P root power off",
                                 &node_ip));
}

pub fn startup_node(proxy_ip: &String, node_ip: &String) {
    send_command_to_node(proxy_ip.clone(),
                         DEFAULT_PORT,
                         format!("ipmitool -H {} -I lanplus -U root -P root power on",
                                 &node_ip));
}


#[derive(Clone, Debug, RustcEncodable)]
struct Command {
    cmd: String,
    env: String,
}

fn send_command_to_node(ip: String, port: i16, command: String) {
    let address = format!("http://{}:{}/sync", ip, port);
    let command = Command {
        cmd: command.clone(),
        env: "".to_string(),
    };

    let mut response = CLIENT.post(&address).body(&json::encode(&command).unwrap()).send().unwrap();

    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    println!("response from {}: {:?}", ip, body);

    match response.status {
        StatusCode::Accepted => {}
        _ => println!("error posting"),
    }
}