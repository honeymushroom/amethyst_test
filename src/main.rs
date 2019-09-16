use amethyst::{
    assets::{
        PrefabLoaderSystem,
    },
    core::{TransformBundle},
    derive::PrefabData,
    input::{InputBundle, StringBindings},
    prelude::{
        Application, GameDataBuilder,
    },
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{
        application_root_dir,
        auto_fov::{AutoFovSystem},
    },
    Error,
    controls::{FlyMovementSystem},
};



mod graphics;

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config_dir = app_root.join("config");
    let assets_dir = app_root.join("assets");
    let display_config_path = config_dir.join("display.ron");
    let key_bindings_path = config_dir.join("input.ron");
    let input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?;

    let game_data = GameDataBuilder::new()
        // scene prefab stuff
        .with(PrefabLoaderSystem::<graphics::ScenePrefab>::default(), "prefab", &[])

        // controls
        .with_bundle(input_bundle)?
        .with(
            FlyMovementSystem::<StringBindings>::new(
                100.0,
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            ),
            "fly_movement",
            &["input_system"]
        )
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?

        // fov system
        .with(AutoFovSystem::default(), "auto_fov", &["prefab"]) // This makes the system adjust the camera right after it has been loaded (in the same frame), preventing any flickering
        .with(graphics::ShowFovSystem, "show_fov", &["auto_fov"])
        .with_bundle(UiBundle::<StringBindings>::new())?

        // rendering
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear(graphics::CLEAR_COLOR),
                )
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderUi::default()),
        )?
        ;

    let mut game = Application::build(assets_dir, graphics::Loading::new())?.build(game_data)?;
    game.run();

    Ok(())
}
