use std::env;

use bevy::{diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}, prelude::*};
use rand::distributions::{Distribution, Uniform};
mod orbit_controls;
use orbit_controls::{OrbitCamera, OrbitCameraPlugin};

#[derive(Default)]
struct StartupOptions{
    box_count: i32,
}

struct FpsText;

fn init(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	startup_command: ResMut<StartupOptions>,
) {
	commands.spawn_bundle(LightBundle {
		transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
		..Default::default()
	});

	commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 12.5))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    })
	.insert(OrbitCamera::new(0.0, 0.0, 12.5, Vec3::ZERO));

	commands.spawn_bundle(UiCameraBundle::default());
	commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "FPS: unknown",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::WHITE,
            },
            Default::default(),
        ),
        ..Default::default()
    })
	.insert(FpsText);

	let box_mesh = meshes.add(Mesh::from(shape::Box::new(0.9, 0.9, 0.9)));
	//let box_material = materials.add(Color::rgb(1.0, 0.2, 0.3).into());

	let box_colors: [Color; 3] = [
		Color::rgb(1.0, 0.2, 0.3),
		Color::rgb(0.3, 1.0, 0.2),
		Color::rgb(0.2, 0.3, 1.0)
	];

	let mut box_materials: Vec<Handle<StandardMaterial>> = Vec::new();

	for color in box_colors.iter(){
		box_materials.push(materials.add((*color).into()));
	}

	let values = Uniform::new(0, 3);

	let amount: i32 = startup_command.box_count;

	for x in -(amount / 2)..(amount / 2) {
		for y in -(amount / 2)..(amount / 2) {
			for z in -(amount / 2)..(amount / 2) {
				let mut rng = rand::thread_rng();
				//let current_material = box_materials[values.sample(&mut rng)].clone_weak() as Handle<StandardMaterial>;
				let current_material = materials.add(box_colors[values.sample(&mut rng)].into());
				commands.spawn_bundle(PbrBundle {
					mesh: box_mesh.clone(),
					//material: box_material.clone(),
					material: current_material,
					transform: Transform::from_translation(Vec3::new(
						x as f32, y as f32, z as f32,
					)),
					..Default::default()
				});
			}
		}
	}
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[0].value = format!(" FPS: {:.0}", average);
            }
		}
		/* if let Some(draw_calls) = diagnostics.get(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::SHADER_MODULES) {
			if let Some(average) = draw_calls.average() {
				println!("{}", average);
			}
		} */
    }
}

fn parse_command_line_options(args: Vec<String>) -> StartupOptions {
    let mut options = StartupOptions {
        box_count: 6,
    };

    if args.len() > 1 {
        options.box_count = args[1].parse().expect("Please specify the number of boxes as an integer.");
    }

    return  options;
}

#[bevy_main]
fn main() {
	let args: Vec<String> = env::args().collect();
    let startup_options = parse_command_line_options(args);
	App::build()
		.insert_resource(WindowDescriptor {
			width: 800.0,
			height: 600.0,
			vsync: true,
			decorations: false,
			..Default::default()
		})
		.insert_resource(Msaa { samples: 4 })
		.insert_resource(startup_options)
		.add_plugins(DefaultPlugins)
		.add_plugin(OrbitCameraPlugin)
		.add_plugin(FrameTimeDiagnosticsPlugin::default())
		.add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
		.add_startup_system(init.system())
		.add_system(text_update_system.system())
		.run();
}