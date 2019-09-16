use amethyst::{
    controls::{FlyControlTag},
    assets::{
        Completion, Handle, Prefab, PrefabData, PrefabLoader, ProgressCounter,
        RonFormat,
    },
    core::{Transform},
    derive::PrefabData,
    ecs::{Entity, ReadExpect, ReadStorage, System, WriteStorage},
    input::{is_close_requested, is_key_down},
    prelude::{
        Builder, GameData, SimpleState, SimpleTrans, StateData,
        StateEvent, Trans,
    },
    renderer::{
        camera::{Camera, CameraPrefab},
        formats::GraphicsPrefab,
        light::LightPrefab,
        rendy::mesh::{Normal, Position, Tangent, TexCoord},
    },
    ui::{UiCreator, UiFinder, UiText},
    utils::{
        auto_fov::{AutoFov},
        tag::{Tag, TagFinder},
    },
    window::ScreenDimensions,
    winit::VirtualKeyCode,
    Error,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

pub const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Default, Deserialize, PrefabData, Serialize)]
#[serde(default)]
pub struct ScenePrefab {
    graphics: Option<GraphicsPrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    auto_fov: Option<AutoFov>, // `AutoFov` implements `PrefabData` trait
    show_fov_tag: Option<Tag<ShowFov>>,
    control_tag: Option<Tag<FlyControlTag>>,
}

#[derive(Clone, Default)]
pub struct ShowFov;

pub struct Loading {
    progress: ProgressCounter,
    scene: Option<Handle<Prefab<ScenePrefab>>>,
}

impl Loading {
    pub fn new() -> Self {
        Loading {
            progress: ProgressCounter::new(),
            scene: None,
        }
    }
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/loading.ron", &mut self.progress);
            creator.create("ui/fov.ron", &mut self.progress);
        });

        let handle = data.world.exec(|loader: PrefabLoader<'_, ScenePrefab>| {
            loader.load("../my_scene.ron", RonFormat, &mut self.progress)
        });
        self.scene = Some(handle);
    }

    fn update(&mut self, _: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Loading => Trans::None,
            Completion::Failed => {
                error!("Failed to load the scene");
                Trans::Quit
            }
            Completion::Complete => {
                info!("Loading finished. Moving to the main state.");
                Trans::Switch(Box::new(Example {
                    scene: self.scene.take().unwrap(),
                }))
            }
        }
    }
}

pub struct Example {
    scene: Handle<Prefab<ScenePrefab>>,
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.create_entity().with(self.scene.clone()).build();
        data.world
            .exec(|finder: UiFinder| finder.find("loading"))
            .map_or_else(
                || error!("Unable to find Ui Text `loading`"),
                |e| {
                    data.world
                        .delete_entity(e)
                        .unwrap_or_else(|err| error!("{}", err))
                },
            );
    }

    /*
    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(ref event) = event {
            if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
    */
}

pub struct ShowFovSystem;

impl<'a> System<'a> for ShowFovSystem {
    type SystemData = (
        TagFinder<'a, ShowFov>,
        UiFinder<'a>,
        WriteStorage<'a, UiText>,
        ReadStorage<'a, Camera>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (tag_finder, ui_finder, mut ui_texts, cameras, screen): Self::SystemData) {
        let screen_aspect = screen.aspect_ratio();
        if let Some(t) = ui_finder
            .find("screen_aspect")
            .and_then(|e| ui_texts.get_mut(e))
        {
            t.text = format!("Screen Aspect Ratio: {:.2}", screen_aspect);
        }

        if let Some(entity) = tag_finder.find() {
            if let Some(camera) = cameras.get(entity) {
                let fovy = get_fovy(camera);
                let camera_aspect = get_aspect(camera);
                if let Some(t) = ui_finder
                    .find("camera_aspect")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    t.text = format!("Camera Aspect Ratio: {:.2}", camera_aspect);
                }
                if let Some(t) = ui_finder
                    .find("camera_fov")
                    .and_then(|e| ui_texts.get_mut(e))
                {
                    t.text = format!("Camera Fov: ({:.2}, {:.2})", fovy * camera_aspect, fovy);
                }
            }
        }
    }
}

fn get_fovy(camera: &Camera) -> f32 {
    (-1.0 / camera.as_matrix()[(1, 1)]).atan() * 2.0
}

fn get_aspect(camera: &Camera) -> f32 {
    (camera.as_matrix()[(1, 1)] / camera.as_matrix()[(0, 0)]).abs()
}
