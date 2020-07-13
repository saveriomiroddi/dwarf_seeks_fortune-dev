use amethyst::prelude::WorldExt;
use amethyst::ui::UiPrefab;
use amethyst::State;
use amethyst::StateEvent;
use amethyst::{
    animation::{
        get_animation_set, AnimationCommand, AnimationControlSet, AnimationSet, EndControl,
    },
    assets::{AssetStorage, Handle, Loader, Prefab},
    core::{
        math::{Point2, Vector3},
        transform::Transform,
        Parent,
    },
    ecs::{prelude::World, Entities, Entity, Join, ReadStorage, WriteStorage},
    input::{get_key, is_close_requested, is_key_down, InputEvent, VirtualKeyCode},
    prelude::*,
    renderer::{
        formats::texture::ImageFormat, palette::Srgba, resources::Tint, sprite::SpriteRender,
        Camera, SpriteSheet, Texture, Transparent,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    winit::{Event, WindowEvent},
    StateData, Trans,
};
use precompile::AnimationId;

use crate::components::*;
use crate::entities::*;
use crate::game_data::CustomGameData;
use crate::levels::*;
use crate::resources::*;
use crate::states::editor::load::load;
use crate::states::editor::paint::paint_tiles;
use crate::states::editor::save::save;
use crate::states::{PausedState, PlayState};

pub struct EditorState;

impl<'a, 'b> EditorState {
    fn handle_action(
        &mut self,
        action: &str,
        world: &mut World,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        Trans::None
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for EditorState {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        let StateData { world, .. } = data;
        UiHandles::add_ui(&UiType::Fps, world);
        setup_debug_lines(world);
        let _ = init_cursor(world);
        create_camera(world);
        let mut editor_data = EditorData::default();
        if let Ok(level_edit) = load(world) {
            editor_data.level = level_edit;
        }
        let tile_defs = load_tile_definitions().expect("Tile definitions failed to load!");
        editor_data.brush.set_palette(&tile_defs);
        world.insert(editor_data);
        world.insert(tile_defs);
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        println!("EditorState on_stop");
        data.world.delete_all();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        match event {
            // Events related to the window and inputs.
            StateEvent::Window(event) => {
                if let Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::Resized(_),
                } = event
                {
                    *data.world.write_resource::<ResizeState>() = ResizeState::Resizing;
                };
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    save(data.world);
                    Trans::Pop
                } else {
                    Trans::None
                }
            }
            // Ui event. Button presses, mouse hover, etc...
            StateEvent::Ui(_) => Trans::None,
            StateEvent::Input(input_event) => match input_event {
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::Return,
                    scancode: _,
                } => {
                    paint_tiles(data.world);
                    Trans::None
                }
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::F5,
                    scancode: _,
                } => {
                    save(data.world);
                    let level_file = application_root_dir()
                        .expect("Root dir not found!")
                        .join("assets/")
                        .join("levels/")
                        .join("generated.ron");
                    Trans::Switch(Box::new(PlayState::new(level_file, true)))
                }
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::LBracket,
                    scancode: _,
                } => {
                    &(*data.world.write_resource::<EditorData>())
                        .brush
                        .select_previous();
                    Trans::None
                }
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::RBracket,
                    scancode: _,
                } => {
                    &(*data.world.write_resource::<EditorData>())
                        .brush
                        .select_next();
                    Trans::None
                }
                InputEvent::ActionPressed(action) => {
                    self.handle_action(&action, data.world);
                    Trans::None
                }
                _ => Trans::None,
            },
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        let StateData { world, .. } = data;
        // Execute a pass similar to a system
        world.exec(
            |(entities, animation_sets, mut control_sets): (
                Entities,
                ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
            )| {
                // For each entity that has AnimationSet
                for (entity, animation_set) in (&entities, &animation_sets).join() {
                    // Creates a new AnimationControlSet for the entity
                    let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                    // Adds the `Fly` animation to AnimationControlSet and loops infinitely
                    control_set.add_animation(
                        AnimationId::Fly,
                        &animation_set.get(&AnimationId::Fly).unwrap(),
                        EndControl::Loop(None),
                        1.0,
                        AnimationCommand::Start,
                    );
                }
            },
        );
        data.data.update(&world, true);
        Trans::None
    }
}

fn init_cursor(world: &mut World) -> Entity {
    let sprite_handle = world
        .read_resource::<Assets>()
        .get_still(&SpriteType::Selection);
    let asset_dimensions = get_asset_dimensions(&AssetType::Still(SpriteType::Selection, 0));
    let mut selection_transform = Transform::default();
    selection_transform.set_translation_z(1.0);
    world
        .create_entity()
        .with(SpriteRender {
            sprite_sheet: sprite_handle.clone(),
            sprite_number: 1,
        })
        .with(Transparent)
        .with(selection_transform)
        .with(SelectionTag)
        .build();
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.5, 0.5, 2.0);
    transform.set_scale(Vector3::new(
        1. / asset_dimensions.x as f32,
        1. / asset_dimensions.y as f32,
        1.0,
    ));
    world
        .create_entity()
        .with(SpriteRender {
            sprite_sheet: sprite_handle,
            sprite_number: 0,
        })
        .with(Transparent)
        .with(transform)
        .with(Cursor::default())
        .build()
}
