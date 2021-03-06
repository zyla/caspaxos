use caspaxos::{simulate, Client};
use smol::Task;

fn set_client(mut client: Client) -> Task<()> {
    Task::spawn(async move {
        let responses = client.ping().await;
        println!("majority pinger got {} responses", responses);

        let set = client.set(b"k1".to_vec(), b"v1".to_vec()).await;
        println!("set response: {:?}", set);
    })
}

#[test]
fn flake_detector() {
    #[cfg(feature = "pretty_backtrace")]
    color_backtrace::install();

    let n_servers = 5;
    let n_clients = 15;

    // drop 1 in 10 messages
    let lossiness = Some(10);
    //let lossiness = None;

    let clients = vec![set_client as fn(Client) -> Task<_>; n_clients];

    for _ in 0..10 {
        simulate(lossiness, n_servers, clients.clone());
    }
}
