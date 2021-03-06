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

use collaborator::{register_torc_controller, register_unmanaged_service};
use state::{SLA, StateManager, TaskState};
use std::thread;
use std::time::Duration;
use utils::{read_string, read_string_replace_variable, read_task};

struct DNSEntry {
    name: String,
    ip: String,
}

pub fn run_health_checker(state_manager: &StateManager) {
    println!("health check starting");
    state_manager.send_ping();

    let is_system_service = true;

    let config = state_manager.get_yaml();
    let wait_time = config["healthcheck"]["poll_interval_in_seconds"].as_i64().unwrap() as u64;

    let mut tasks = Vec::new();

    let system_services = config["healthcheck"]["system_services"].as_vec().unwrap();
    for system_service in system_services {
        let task = read_task(system_service, state_manager);
        match task.sla {
            SLA::None => tasks.push(task),
            SLA::SingletonEachNode => {
                let nodes = state_manager.request_list_nodes();
                for node in nodes {
                    let mut new_task = task.clone();
                    new_task.node_name = node.name.clone();
                    new_task.name = format!("{}-{}", new_task.name, node.name);
                    tasks.push(new_task)
                }
            }
            SLA::SingletonEachSlave => {
                let nodes = state_manager.request_list_nodes();
                for node in nodes {
                    if node.node_type != "slave" {
                        continue;
                    };
                    let mut new_task = task.clone();
                    new_task.node_name = node.name.clone();
                    new_task.name = format!("{}-{}", new_task.name, node.name);
                    tasks.push(new_task)
                }
            }
        }
    }

    let mut dns_entries = Vec::new();
    let dns_addons = config["dns-addons"].as_vec().unwrap();
    for dns_addon in dns_addons {
        dns_entries.push(DNSEntry {
            name: read_string(dns_addon, "name".to_string()),
            ip: read_string_replace_variable(dns_addon, "ip".to_string(), &state_manager),
        });
    }


    loop {
        thread::sleep(Duration::from_secs(wait_time));
        println!("checking health");

        for task in &tasks {
            match state_manager.request_task_state(task.name.to_string()) {
                TaskState::Running | TaskState::Requested | TaskState::Accepted | TaskState::Restart => {}
                TaskState::NotRunning => {
                    state_manager.send_start_task(&task.name,
                                                  &task.image,
                                                  &task.node_name,
                                                  &task.node_type,
                                                  &task.node_function,
                                                  &task.dependent_service,
                                                  &task.arguments,
                                                  &task.parameters,
                                                  &task.memory,
                                                  &task.cpu,
                                                  &task.volumes,
                                                  &task.privileged,
                                                  &task.sla,
                                                  &task.is_metered,
                                                  &is_system_service,
                                                  &task.is_job,
                                                  &task.network_type)
                }
            };
        }

        register_torc_controller(&state_manager.get_master_ip(),
                                 &state_manager.get_my_name(),
                                 &state_manager.get_my_ip());

        for dns_entry in &dns_entries {
            register_unmanaged_service(&state_manager.get_master_ip(),
                                       &dns_entry.name,
                                       &dns_entry.ip);

        }
    }
}
