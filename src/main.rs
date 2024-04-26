use bevy::prelude::*;
mod plugins;
use plugins::CageCameraPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CageCameraPlugin)
        .run();
}
