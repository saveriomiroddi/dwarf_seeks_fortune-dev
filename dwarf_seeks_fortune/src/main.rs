#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

mod state_loading;
mod state_main_menu;
mod util_loading;

use amethyst::{
    assets::{PrefabLoaderSystemDesc, Processor},
    audio::{DjSystemDesc, Source},
    utils::application_root_dir,
    GameDataBuilder, LoggerConfig,
};

use dsf_core::systems;

use crate::state_loading::LoadingState;
use dsf_core::resources::Music;
use dsf_precompile::PrecompiledDefaultsBundle;
use dsf_precompile::PrecompiledRenderBundle;
use dsf_precompile::{start_game, MyPrefabData};

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(LoggerConfig::default()).start();
    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("../assets/");
    let config_dir = assets_dir.join("config/");
    let display_config_path = config_dir.join("display.ron");
    let bindings_config_path = config_dir.join("input.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(PrecompiledDefaultsBundle {
            bindings_config_path,
        })?
        .with_system_desc(
            PrefabLoaderSystemDesc::<MyPrefabData>::default(),
            "prefab_loader",
            &[],
        )
        .with(Processor::<Source>::new(), "source_processor", &[])
        .with(
            systems::FpsCounterUiSystem::default(),
            "fps_counter_ui_system",
            &[],
        )
        .with(systems::CameraSystem, "camera_system", &[])
        .with(
            systems::CameraControlSystem,
            "camera_control_system",
            &["camera_system"],
        )
        .with_system_desc(
            DjSystemDesc::new(|music: &mut Music| music.music.next()),
            "dj",
            &[],
        )
        .with(systems::DummySystem, "dummy_system", &[])
        .with_bundle(PrecompiledRenderBundle {
            display_config_path,
        })?;

    start_game(
        assets_dir,
        game_data,
        Some(Box::new(LoadingState::default())),
    );
    Ok(())
}
