use clap::{value_parser, Command};
use log::error;

use hecs::{Entity, World};
use shellwords::split;

use crate::{asset_loader::{clear_all, load_effect}, component::{effect::Effect, transform3d::Transform3D}, cvar::{print_cvars, set_cvar}, math::Vector3};

pub struct ConsoleCommandSystem {
    test_fx_id: Option<Entity>
}

impl ConsoleCommandSystem {
    pub fn new() -> ConsoleCommandSystem {
        ConsoleCommandSystem {
            test_fx_id: None,
        }
    }

    pub fn exec_commands<I>(self: &mut ConsoleCommandSystem, commands: I, world: &mut World) where I : Iterator::<Item = String> {
        for cmd in commands {
            let args = split(&cmd).unwrap();
    
            let cmd = Command::new("")
                .no_binary_name(true)
                .subcommand_required(true)
                .subcommand(Command::new("test-vfx")
                    .about("Spawn a test effect instance in the world, erasing the previous test effect if any. Also clears resource caches.")
                    .arg(clap::arg!(<PATH> "Path to the VFX to spawn"))
                    .arg(clap::arg!(<POS_X> "X Position to spawn the VFX at").value_parser(value_parser!(f32)))
                    .arg(clap::arg!(<POS_Y> "Y Position to spawn the VFX at").value_parser(value_parser!(f32)))
                    .arg(clap::arg!(<POS_Z> "Z Position to spawn the VFX at").value_parser(value_parser!(f32)))
                )
                .subcommand(Command::new("delete-entity")
                    .about("Delete entity by ID")
                    .arg(clap::arg!(<ID> "ID of the entity to delete").value_parser(value_parser!(u32)))
                )
                .subcommand(Command::new("clear-cache")
                    .about("Clear all resource caches")
                )
                .subcommand(Command::new("set")
                    .arg(clap::arg!(<NAME> "Name of the CVAR"))
                    .arg(clap::arg!(<VALUE> "Value to set"))
                    .about("Set CVAR by name")
                )
                .subcommand(Command::new("cvarlist")
                    .about("List all defined CVARs")
                );
    
            match cmd.try_get_matches_from(args) {
                Ok(m) => {
                    match m.subcommand() {
                        Some(("test-vfx", sub_args)) => {
                            clear_all();

                            let path = sub_args.get_one::<String>("PATH").unwrap();
                            let pos_x = sub_args.get_one::<f32>("POS_X").unwrap();
                            let pos_y = sub_args.get_one::<f32>("POS_Y").unwrap();
                            let pos_z = sub_args.get_one::<f32>("POS_Z").unwrap();
    
                            match load_effect(&path) {
                                Ok(effect) => {
                                    if let Some(prev_entity) = self.test_fx_id {
                                        world.despawn(prev_entity).unwrap();
                                    }

                                    let entity = world.spawn((
                                        Transform3D::default().with_position(Vector3::new(*pos_x, *pos_y, *pos_z)),
                                        Effect::new(&effect, true, false), 
                                    ));

                                    self.test_fx_id = Some(entity);
                                }
                                Err(_) => {
                                }
                            }
                        }
                        Some(("delete-entity", sub_args)) => {
                            let id = sub_args.get_one::<u32>("ID").unwrap();
                            let e = unsafe { world.find_entity_from_id(*id) };
                            world.despawn(e).unwrap();
                        }
                        Some(("clear-cache", _)) => {
                            clear_all();
                        }
                        Some(("set", sub_args)) => {
                            let name = sub_args.get_one::<String>("NAME").unwrap();
                            let value = sub_args.get_one::<String>("VALUE").unwrap();

                            set_cvar(&name, &value);
                        }
                        Some(("cvarlist", _)) => {
                            print_cvars();
                        }
                        _ => unreachable!()
                    }
                },
                Err(e) => {
                    error!("{}", e);
                },
            };
        }
    }
}