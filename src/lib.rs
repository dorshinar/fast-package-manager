pub mod dependency_resolver;
pub mod downloader;
mod hardlink;
mod http;
pub mod install_package;
pub mod npm;
mod resolve_version_range;
mod symlink;

pub const STORE_FOLDER: &str = ".fpm";
pub const DEPS_FOLDER: &str = "node_modules";

// fn add_package_to_package_json(
//     package_name: &String,
//     version: &npm::Version,
// ) -> Result<(), Box<dyn Error>> {
//     let file = File::open(String::from("./package.json"))?;
//     let reader = BufReader::new(file);

//     // Read the JSON contents of the file as an instance of `User`.
//     let mut package_json: Value = serde_json::from_reader(reader)?;

//     match &mut package_json {
//         Value::Object(package_json_obj) => match package_json_obj.get_mut("dependencies") {
//             Some(deps) => match deps {
//                 Value::Object(deps_obj) => deps_obj.insert(
//                     package_name.to_owned(),
//                     Value::String(format!("^{version}").to_owned()),
//                 ),
//                 _ => None,
//             },
//             None => package_json_obj.insert(
//                 String::from("dependencies"),
//                 json!({ package_name: version }),
//             ),
//         },
//         _ => panic!("failed to read package.json"),
//     };

//     fs::write(
//         "./package.json",
//         serde_json::to_string_pretty(&package_json).unwrap(),
//     )?;

//     Ok(())
// }
