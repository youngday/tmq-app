use async_std::task;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use log4rs;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::Read, time::Duration};
use tmq::{publish, subscribe, Context};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Application {
    application: Data,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Data {
    build: String,
    container_name: String,
    environment2: Data2,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Data2 {
    one_env2: String,
    sec_env2: String,
}

#[async_std::main] //async-std = { version = "1.9", features = ["attributes"] }
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("./log.yaml", Default::default()).unwrap();
    let version: String = "0.2.0910".to_string();
    trace!("demo trace");
    debug!("demo debug");
    info!("demo info");
    warn!("demo warn");
    error!("demo error");

    info!("version:{:}", version);

    let filename = "config.yaml";
    match File::open(filename) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            let application_data: Application = serde_yaml::from_str(&content).unwrap();
            info!("{:?}", application_data.application.build);
            info!("{:?}", application_data.application.environment);
            //info!("{:?}", application_data.application.environment2);
            //info!("{:?}", application_data.application.environment2.one_env2);
        }
        Err(error) => {
            info!("There is an error {}: {}", filename, error);
        }
    }

    info!("Start your app.");

    let send_task = task::spawn(async {
        task::Builder::new()
            .name("send_task".to_string())
            .spawn(send())
            .unwrap()
            .await;
    });
    let recv_udp_task = task::spawn(async {
        task::Builder::new()
            .name("recv_udp_task".to_string())
            .spawn(received_udp())
            .unwrap()
            .await;
    });

    let recv_serial_task = task::spawn(async {
        task::Builder::new()
            .name("recv_serial_task".to_string())
            .spawn(recv_serial())
            .unwrap()
            .await;
    });

    let recv_http_task = task::spawn(async {
        task::Builder::new()
            .name("recv_http_task".to_string())
            .spawn(recv_http())
            .unwrap()
            .await;
    });

    send_task.await;
    recv_udp_task.await;
    recv_serial_task.await;
    recv_http_task.await;

    Ok(())
}

async fn send() {
    let bindip = "tcp://127.0.0.1:7899";
    let mut socket = publish(&Context::new()).bind(bindip).unwrap();
    let mut i = 0;
    loop {
        i += 1;
        let message = format!("Broadcast #{}", i);
        info!("Publish: {}", message);
        socket
            .send(vec![b"UDP2" as &[u8], message.as_bytes()])
            .await
            .unwrap();
        socket
            .send(vec![b"SERIAL2" as &[u8], message.as_bytes()])
            .await
            .unwrap();
        socket
            .send(vec![b"HTTP2" as &[u8], message.as_bytes()])
            .await
            .unwrap();
        socket
            .send(vec![b"AAA" as &[u8], message.as_bytes()])
            .await
            .unwrap();

        task::sleep(Duration::from_millis(2000)).await;
    }
}

async fn received_udp() {
    let bindip = "tcp://127.0.0.1:7898";
    let mut socket = subscribe(&Context::new())
        .connect(bindip)
        .unwrap()
        .subscribe(b"UDP1")
        .unwrap();

    while let Some(msg) = socket.next().await {
        info!(
            "Subscribe: {:?}",
            msg.unwrap()
                .iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}

async fn recv_serial() {
    let bindip = "tcp://127.0.0.1:7898";
    let mut socket = subscribe(&Context::new())
        .connect(bindip)
        .unwrap()
        .subscribe(b"SERIAL1")
        .unwrap();

    while let Some(msg) = socket.next().await {
        info!(
            "Subscribe: {:?}",
            msg.unwrap()
                .iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}

async fn recv_http() {
    let bindip = "tcp://127.0.0.1:7898";
    let mut socket = subscribe(&Context::new())
        .connect(bindip)
        .unwrap()
        .subscribe(b"HTTP1")
        .unwrap();

    while let Some(msg) = socket.next().await {
        info!(
            "Subscribe: {:?}",
            msg.unwrap()
                .iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
}
