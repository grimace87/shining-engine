
use crate::ColladaParser;

#[test]
fn models_are_processed() {
    let models_dir = {
        let mut dir = std::env::current_dir().unwrap();
        dir.pop();
        dir.pop();
        dir.push("resources");
        dir.push("test");
        dir.push("models");
        dir
    };
    ColladaParser::parse_directory(&models_dir).unwrap();
}
