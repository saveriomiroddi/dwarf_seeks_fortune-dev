#![allow(
    dead_code,
    unused_must_use,
    unused_imports,
    unused_variables,
    unused_parens,
    unused_mut
)]
#![forbid(unsafe_code)]

mod components;
mod game_data;
mod resources;
mod states;
mod systems;

use crate::resources::*;
use amethyst::prelude::{Config, SystemExt};
use amethyst::{
    assets::{PrefabLoaderSystemDesc, Processor},
    audio::Source,
    core::SystemDesc,
    utils::application_root_dir,
    Application,
};
use game_data::CustomGameDataBuilder;
use precompile::MyPrefabData;
use precompile::PrecompiledDefaultsBundle;
use precompile::PrecompiledRenderBundle;

fn main() {
    let result = make_game();
    if let Err(e) = result {
        println!("Error starting game: {:?}", e);
    }
}

fn make_game() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default()).start();
    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets/");
    let config_dir = assets_dir.join("config/");
    let display_config_path = config_dir.join("display.ron");
    let config_path = config_dir.join("debug_config.ron");
    let bindings_config_path = config_dir.join("bindings.ron");

    let mut app_builder = Application::build(assets_dir, states::LoadingState::default())?;

    let config = DebugConfig::load(&config_path)?;
    app_builder.world.insert(config);
    let game_data = CustomGameDataBuilder::default()
        .with_base_bundle(
            &mut app_builder.world,
            PrecompiledDefaultsBundle {
                bindings_config_path: &bindings_config_path,
            },
        )?
        .with_core(
            PrefabLoaderSystemDesc::<MyPrefabData>::default().build(&mut app_builder.world),
            "scene_loader",
            &[],
        )
        .with_core(Processor::<Source>::new(), "source_processor", &[])
        .with_core(
            systems::UiEventHandlerSystem::new(),
            "ui_event_handler",
            &[],
        )
        .with_core(systems::UiSystem::default(), "ui_system", &[])
        .with_core(
            systems::MovementSystem.pausable(CurrentState::Running),
            "movement_system",
            &["input_system"],
        )
        .with_core(
            systems::PlayerSystem.pausable(CurrentState::Running),
            "player_system",
            &["input_system"],
        )
        .with_core(systems::SpawnSystem::new(), "spawn_system", &[])
        .with_core(systems::DebugSystem, "debug_system", &["input_system"])
        .with_core(systems::CameraSystem, "camera_system", &[])
        .with_core(
            systems::RewindControlSystem,
            "rewind_control_system",
            &["player_system"],
        )
        .with_core(
            systems::RewindSystem.pausable(CurrentState::Rewinding),
            "rewind_system",
            &["rewind_control_system", "input_system"],
        )
        .with_core(systems::ResizeSystem, "resize_system", &[])
        .with_base_bundle(
            &mut app_builder.world,
            PrecompiledRenderBundle {
                display_config_path: &display_config_path,
            },
        )?;
    let mut game = app_builder.build(game_data)?;
    game.run();
    Ok(())
}