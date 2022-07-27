use backroll_transport_udp::*;
use bevy::tasks::IoTaskPool;
use bevy_backroll::{backroll::*, *};
use bytemuck::{Pod, Zeroable};
use std::env;
use bevy::prelude::*;
use std::net::SocketAddr;
use std::ops::Deref;

#[macro_use]
extern crate bitflags;

bitflags! {
    #[derive(Default, Pod, Zeroable)]
    #[repr(C)]
    pub struct PlayerInputFrame: u32 {
        // bit shift the stuff in the input struct
        const UP = 1<<0;
        const DOWN = 1<<1;
        const LEFT = 1<<2;
        const RIGHT = 1<<3;
    }
}

#[derive(Debug)]
pub struct StartupNetworkConfig {
    pub client: usize,
    pub bind: SocketAddr,
    pub remote: SocketAddr,
}

#[derive(Clone, Component)]
pub struct Player {
    pub handle: PlayerHandle, // the network id
}

pub fn spawn_players(
    mut commands: Commands,
    config: Res<StartupNetworkConfig>,
    pool: Res<IoTaskPool>,
) {
    println!("spawning players");
    //peerid needs to go here
    let socket = UdpManager::bind(pool.deref().deref().clone(), config.bind).unwrap();
    let peer = socket.connect(UdpConnectionConfig::unbounded(config.remote));

    commands.insert_resource(socket);

    //println!("check 1");

    let mut builder = backroll::P2PSession::<BevyBackrollConfig<PlayerInputFrame>>::build();

    //println!("check 2");

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        // make sure to clone the player handles for reference stuff
        .insert(if config.client == 0 {
            // set up local player
            Player {
                handle: builder.add_player(backroll::Player::Local),
            }
        } else {
            // set up remote player
            Player {
                // make sure to clone the remote peer for reference stuff
                handle: builder.add_player(backroll::Player::Remote(peer.clone())),
            }
        });


    commands
        .spawn_bundle(SpriteBundle {
            //material: materials.player_material.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(if config.client == 1 {
            // set up local player
            Player {
                handle: builder.add_player(backroll::Player::Local),
            }
        } else {
            // set up remote player
            Player {
                handle: builder.add_player(backroll::Player::Remote(peer)),
            }
        });
    commands.start_backroll_session(builder.start(pool.deref().deref().clone()).unwrap());    //problem is here
}


pub fn sample_input(handle: In<PlayerHandle>, keyboard_input: Res<Input<KeyCode>>) -> PlayerInputFrame {
    let mut local_input = PlayerInputFrame::empty();

    // local input handling
    {
        if keyboard_input.pressed(KeyCode::Left) {
            local_input.insert(PlayerInputFrame::LEFT);
            println!("Left");
        } else if keyboard_input.pressed(KeyCode::Right) {
            local_input.insert(PlayerInputFrame::RIGHT);
            println!("Right");
        }

        if keyboard_input.pressed(KeyCode::Up) {
            local_input.insert(PlayerInputFrame::UP);
            println!("Up");
        } else if keyboard_input.pressed(KeyCode::Down) {
            local_input.insert(PlayerInputFrame::DOWN);
            println!("Down");
        }
    }

    local_input
}

fn start_app(player_num: usize) {
    let bind_addr: SocketAddr = if player_num == 0 {
        "127.0.0.1:4001".parse().unwrap()
    } else {
        "127.0.0.1:4002".parse().unwrap()
    };

    let remote_addr: SocketAddr = if player_num == 0 {
        "127.0.0.1:4002".parse().unwrap()
    } else {
        "127.0.0.1:4001".parse().unwrap()
    };

    let mut app = App::new();
    println!("world if: {:?}", app.world.id());

    // app.add_startup_system(network_config::setup_game);
    app.add_startup_stage("game_setup", SystemStage::single(spawn_players))
        .add_plugins(DefaultPlugins)
        .add_plugin(BackrollPlugin);

    app.register_rollback_input::<PlayerInputFrame, _>(
        sample_input.system(), //need .system()
    );

    
    app.register_rollback_component::<Player>();

    app.insert_resource(StartupNetworkConfig {
        client: player_num,
        bind: bind_addr,
        remote: remote_addr,
    });
    // .add_rollback_system(movement::player_movement);

    app.run();
}

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");
    let mut args = std::env::args();
    let base = args.next().unwrap();
    if let Some(player_num) = args.next() {
        println!("play num: {}", player_num);
        start_app(player_num.parse().unwrap());
    } else {
        println!("in else");
        let mut child_1 = std::process::Command::new(base.clone())
            .args(&["0"])
            .spawn()
            .unwrap();
        let mut child_2 = std::process::Command::new(base)
            .args(&["1"])
            .spawn()
            .unwrap();
        child_1.wait().unwrap();
        child_2.wait().unwrap();
    }
}
