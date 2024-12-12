use futures::prelude::*; // 非同期処理のためのFutureとStreamのトレイトをインポート
use libp2p::swarm::SwarmEvent; // libp2pのSwarmEventをインポート
use libp2p::{ping, Multiaddr}; // libp2pのpingとMultiaddrをインポート
use std::error::Error; // 標準ライブラリのErrorトレイトをインポート
use std::time::Duration; // 標準ライブラリのDurationをインポート
use tracing_subscriber::EnvFilter; // tracing_subscriberのEnvFilterをインポート

#[derive(libp2p::swarm::NetworkBehaviour)] // NetworkBehaviourトレイトを自動実装するためのマクロ
struct Behaviour {
    mdns: libp2p::mdns::async_io::Behaviour, // mDNSの振る舞い
    ping: libp2p::ping::Behaviour, // pingの振る舞い
}

impl Behaviour {
    pub fn new(
        mdns: libp2p::mdns::async_io::Behaviour, // mDNSの振る舞いを受け取る
        ping: libp2p::ping::Behaviour, // pingの振る舞いを受け取る
    ) -> Self {
        Self { mdns, ping } // 構造体を初期化して返す
    }
}

#[async_std::main] // 非同期のメイン関数を定義するためのマクロ
async fn main() -> Result<(), Box<dyn Error>> { // メイン関数の定義、Result型を返す
    let mut swarm = libp2p::SwarmBuilder::with_new_identity() // 新しいSwarmを構築
        .with_async_std() // async-stdランタイムを使用
        .with_tcp(
            libp2p::tcp::Config::default(), // TCPのデフォルト設定
            libp2p::noise::Config::new, // Noiseプロトコルの設定
            || libp2p::yamux::Config::default(), // Yamuxのデフォルト設定
        )?
        .with_behaviour(|keypair| {
            Behaviour::new(
                libp2p::mdns::async_io::Behaviour::new(
                    libp2p::mdns::Config::default(), // mDNSのデフォルト設定
                    keypair.public().into(), // 公開鍵を使用
                )
                .unwrap(),
                libp2p::ping::Behaviour::new(
                    libp2p::ping::Config::new()
                        .with_timeout(Duration::from_secs(5)) // pingのタイムアウトを5秒に設定
                        .with_interval(Duration::from_secs(1)), // pingのインターバルを1秒に設定
                ),
            )
        })?
        .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(5))) // 接続のタイムアウトを5秒に設定
        .build();

    println!("peer id: {}", swarm.local_peer_id().to_string()); // ローカルのPeerIdを表示
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?; // 全てのインターフェースとランダムなOS割り当てポートでリッスン

    loop { // イベントループ
        let ev = swarm.select_next_some().await; // 次のイベントを待つ
        println!("{:#?}", ev); // イベントを表示
        if let libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(e))) = ev {
            for peer in e {
                swarm.dial(peer.1)?; // 発見したピアにダイヤル
            }
        }
    }
}
