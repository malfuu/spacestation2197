use bevy::prelude::*;
use toolbox_ui::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(ToolboxUiPlugin)
        .add_systems(Startup, build)
    ;

    app.run();
}

fn build() {

}
