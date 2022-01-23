use async_std::task;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use log4rs;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::Read, time::Duration};
use tmq::{publish, subscribe, Context};

use once_cell::sync::Lazy;
use std::collections::HashMap;

static HASHMAP: Lazy<HashMap<i32, String>> = Lazy::new(|| {
    let app_data: Application;
    let filename = "config.yaml";
    let mut m = HashMap::new();
    match File::open(filename) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            app_data = serde_yaml::from_str(&content).unwrap();
            info!("{:?}", app_data.app.build);
            info!("{:?}", app_data.app.environment);

            let _send_ip = app_data.app.net_cfg.send_ip;
            let _recv_ip = app_data.app.net_cfg.recv_ip;
            let _udp_pub_topic = app_data.app.net_cfg.udp_pub_topic;
            let _serial_pub_topic = app_data.app.net_cfg.serial_pub_topic;
            let _http_pub_topic = app_data.app.net_cfg.http_pub_topic;
            let _udp_sub_topic = app_data.app.net_cfg.udp_sub_topic;
            let _serial_sub_topic = app_data.app.net_cfg.serial_sub_topic;
            let _http_sub_topic = app_data.app.net_cfg.http_sub_topic;

            info!("_send_ip:{:?}", _send_ip);
            info!("_recv_ip:{:?}", _recv_ip);
            info!("_udp_pub_topic:{:?}", _udp_pub_topic);
            info!("_serial_pub_topic:{:?}", _serial_pub_topic);
            info!("_http_pub_topic:{:?}", _http_pub_topic);
            info!("_udp_sub_topic:{:?}", _udp_sub_topic);
            info!("_serial_sub_topic:{:?}", _serial_sub_topic);
            info!("_http_sub_topic:{:?}", _http_sub_topic);

            m.insert(0, _send_ip.to_string());
            m.insert(1, _recv_ip.to_string());
            m.insert(2, _udp_pub_topic.to_string());
            m.insert(3, _serial_pub_topic.to_string());
            m.insert(4, _http_pub_topic.to_string());
            m.insert(5, _udp_sub_topic.to_string());
            m.insert(6, _serial_sub_topic.to_string());
            m.insert(7, _http_sub_topic.to_string());
        }
        Err(error) => {
            info!("There is an error {}: {}", filename, error);
        }
    }
    m
});

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Application {
    app: Data,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Data {
    build: String,
    container_name: String,
    environment2: Data2,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<Vec<String>>,
    net_cfg: NetCfg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Data2 {
    one_env2: String,
    sec_env2: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NetCfg {
    send_ip: String,
    recv_ip: String,
    udp_pub_topic: String,
    serial_pub_topic: String,
    http_pub_topic: String,
    udp_sub_topic: String,
    serial_sub_topic: String,
    http_sub_topic: String,
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

    info!("_send_ip:{:?}", HASHMAP.get(&0));
    info!("_recv_ip:{:?}", HASHMAP.get(&1));
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
    let _bindip = HASHMAP.get(&1).unwrap().to_string(); // _recv_ip;
    let _udp_pub_topic = HASHMAP.get(&2).unwrap().to_string(); // _udp_pub_topic;
    let _serial_pub_topic = HASHMAP.get(&3).unwrap().to_string(); // _serial_pub_topic;
    let _http_pub_topic = HASHMAP.get(&4).unwrap().to_string(); // _http_pub_topic;

    let mut socket = publish(&Context::new()).bind(&_bindip).unwrap();
    let mut i = 0;
    loop {
        i += 1;
        let message = format!("Broadcast #{}", i);
        info!("Publish: {}", message);
        socket.send(vec![&_udp_pub_topic, &message]).await.unwrap();
        socket
            .send(vec![&_serial_pub_topic, &message])
            .await
            .unwrap();
        socket.send(vec![&_http_pub_topic, &message]).await.unwrap();
        socket.send(vec!["AAA", &message]).await.unwrap();

        task::sleep(Duration::from_millis(2000)).await;
    }
}

async fn received_udp() {
    let _bindip = HASHMAP.get(&0).unwrap().to_string(); // _send_ip;
    let _udp_sub_topic = HASHMAP.get(&5).unwrap().to_string(); // _udp_sub_topic;
    let mut socket = subscribe(&Context::new())
        .connect(&_bindip)
        .unwrap()
        .subscribe(_udp_sub_topic.as_bytes())
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
    let _bindip = HASHMAP.get(&0).unwrap().to_string(); //_send_ip;
    let _serial_sub_topic = HASHMAP.get(&5).unwrap().to_string(); // _serial_sub_topic;
    let mut socket = subscribe(&Context::new())
        .connect(&_bindip)
        .unwrap()
        .subscribe(_serial_sub_topic.as_bytes())
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
    let _bindip = HASHMAP.get(&0).unwrap().to_string(); // _send_ip;
    let _http_sub_topic = HASHMAP.get(&5).unwrap().to_string(); // _http_sub_topic;
    let mut socket = subscribe(&Context::new())
        .connect(&_bindip)
        .unwrap()
        .subscribe(_http_sub_topic.as_bytes())
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
