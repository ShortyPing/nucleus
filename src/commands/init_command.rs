use crate::templates::Templates;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use dialoguer::Select;
use crate::frameworks::next;

enum Framework {
    Next,
    None
}

impl Framework {
    fn variants() -> &'static [&'static str] {
        &["Next.js", "None"]
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Next,
            1 => Self::None,
            _ => Self::None
        }
    }
}

fn render(template: &str, vars: &HashMap<&str, &String>) -> String {
    let mut output = template.to_string();
    for (key, value) in vars {
        let placeholder = ["{{", key, "}}"].concat();
        output = output.replace(&placeholder, &value);
    }
    output
}

pub fn handle_init_command(name: String) -> anyhow::Result<()> {

    let index = Select::new()
        .with_prompt("Which frontend framework do you want to use?")
        .items(Framework::variants())
        .default(0)
        .interact()?;

    let framework = Framework::from_index(index);


    let dir_name = name.trim().to_lowercase().replace(" ", "-");


    let target = Path::new(&dir_name);
    let mut vars = HashMap::new();
    vars.insert("name", &name);

    for path in Templates::iter() {
        let template_path = path.as_ref();
        let target_path = target.join(template_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = Templates::get(&template_path).unwrap();
        let text = std::str::from_utf8(file.data.as_ref())?;

        let rendered = render(text, &vars);

        fs::write(target_path, rendered)?;
    }

    match framework {
        Framework::Next => next::scaffold(&target)?,
        Framework::None => {},
    }

    Ok(())
}