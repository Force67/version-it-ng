use regex;
use toml;

impl super::Config {
    /// Updates package files with the new version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to set in package files.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub fn update_package_files(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(package_files) = &self.package_files {
            for package_file in package_files {
                self.update_single_package_file(package_file, version)?;
            }
        }
        Ok(())
    }

    fn update_single_package_file(&self, package_file: &super::PackageFile, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = self.base_path.join(&package_file.path);
        if !full_path.exists() {
            // Skip files that don't exist
            return Ok(());
        }
        let content = std::fs::read_to_string(&full_path)?;
        let updated_content = match package_file.manager.as_str() {
            "npm" | "yarn" | "pnpm" => self.update_json_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            "cargo" => self.update_toml_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            "python" => self.update_python_file(&content, version, package_file.field.as_deref().unwrap_or("__version__"))?,
            "maven" => self.update_xml_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            "cmake" => self.update_cmake_file(&content, version, package_file.field.as_deref().unwrap_or("PROJECT_VERSION"))?,
            _ => return Err(format!("Unsupported package manager: {}", package_file.manager).into()),
        };
        std::fs::write(&full_path, updated_content)?;
        Ok(())
    }

    fn update_json_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut json: serde_json::Value = serde_json::from_str(content)?;
        if let Some(obj) = json.as_object_mut() {
            obj.insert(field.to_string(), serde_json::Value::String(version.to_string()));
        }
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn update_toml_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut toml_value: toml::Value = toml::from_str(content)?;

        // Handle dotted field paths like "package.version"
        let field_parts: Vec<&str> = field.split('.').collect();
        let mut current_value = &mut toml_value;

        for (i, part) in field_parts.iter().enumerate() {
            if i == field_parts.len() - 1 {
                // Last part - set the value
                if let Some(table) = current_value.as_table_mut() {
                    table.insert(part.to_string(), toml::Value::String(version.to_string()));
                }
            } else {
                // Navigate to the nested table
                if let Some(table) = current_value.as_table_mut() {
                    if !table.contains_key(*part) {
                        table.insert(part.to_string(), toml::Value::Table(Default::default()));
                    }
                    current_value = table.get_mut(*part).unwrap();
                } else {
                    return Err(format!("Cannot navigate to field '{}': not a table", part).into());
                }
            }
        }

        Ok(toml::to_string(&toml_value)?)
    }

    fn update_python_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_lines = Vec::new();
        let assignment_pattern = format!("{} = ", field);

        for line in lines {
            if line.trim().starts_with(&assignment_pattern) {
                // Simple string assignment replacement
                if let Some(quote_start) = line.find('"').or_else(|| line.find('\'')) {
                    if let Some(quote_end) = line[quote_start + 1..].find(line.chars().nth(quote_start).unwrap()).map(|i| i + quote_start + 1) {
                        let before = &line[..quote_start + 1];
                        let after = &line[quote_end..];
                        updated_lines.push(format!("{}{}{}", before, version, after));
                        continue;
                    }
                }
            }
            updated_lines.push(line.to_string());
        }

        Ok(updated_lines.join("\n"))
    }

    fn update_xml_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simple XML version update - this is a basic implementation
        // For more complex XML structures, a proper XML parser would be better
        let version_tag = format!("<{}>{}</{}>", field, version, field);
        let pattern = format!("<{}>[^<]*</{}>", regex::escape(field), regex::escape(field));

        let re = regex::Regex::new(&pattern)?;
        Ok(re.replace_all(content, version_tag).to_string())
    }

    fn update_cmake_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_lines = Vec::new();
        let set_pattern = format!("set({}", field);

        for line in lines {
            // Handle set(PROJECT_VERSION "x.y.z") pattern
            if line.trim().starts_with(&set_pattern) {
                // CMake set() command replacement
                if let Some(quote_start) = line.find('"') {
                    if let Some(quote_end) = line[quote_start + 1..].find('"').map(|i| i + quote_start + 1) {
                        let before = &line[..quote_start + 1];
                        let after = &line[quote_end..];
                        updated_lines.push(format!("{}{}{}", before, version, after));
                        continue;
                    }
                }
            }
            // Handle project(name VERSION x.y.z ...) pattern
            else if line.trim().starts_with("project(") && line.contains("VERSION") {
                // CMake project() command with VERSION
                let version_pattern = regex::Regex::new(r"VERSION\s+[\d.]+")?;
                if version_pattern.is_match(line) {
                    let updated_line = version_pattern.replace(line, &format!("VERSION {}", version));
                    updated_lines.push(updated_line.to_string());
                    continue;
                }
            }
            updated_lines.push(line.to_string());
        }

        Ok(updated_lines.join("\n"))
    }
}