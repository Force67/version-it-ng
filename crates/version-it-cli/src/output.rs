pub fn output_success(structured: bool, data: serde_json::Value) {
    if structured {
        println!("{}", serde_json::to_string(&data).unwrap());
    } else if let Some(version) = data.get("version") {
        println!("{}", version.as_str().unwrap());
    } else if let Some(message) = data.get("message") {
        println!("{}", message.as_str().unwrap());
    }
}

pub fn output_error(structured: bool, error: &str) -> ! {
    if structured {
        let data = serde_json::json!({
            "success": false,
            "error": error
        });
        println!("{}", serde_json::to_string(&data).unwrap());
        std::process::exit(1);
    } else {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}