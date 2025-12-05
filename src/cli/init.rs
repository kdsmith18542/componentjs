//! Project initialization command

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;

/// Initialize a new project
#[derive(Args, Debug)]
pub struct InitCommand {
    /// Project name / directory
    #[arg(default_value = ".")]
    pub name: String,

    /// Project template (react, vue, svelte, vanilla)
    #[arg(short, long, default_value = "vanilla")]
    pub template: String,

    /// Use TypeScript
    #[arg(long)]
    pub typescript: bool,
}

impl InitCommand {
    pub async fn execute(&self) -> Result<()> {
        let project_dir = Path::new(&self.name);
        
        eprintln!(
            "{} Initializing new {} project...\n",
            "→".blue(),
            self.template.cyan()
        );
        
        // Create project directory if needed
        if self.name != "." {
            fs::create_dir_all(project_dir)
                .context("Failed to create project directory")?;
        }
        
        // Generate component.toml
        let config_content = self.generate_config();
        fs::write(project_dir.join("component.toml"), config_content)
            .context("Failed to write component.toml")?;
        eprintln!("  {} Created {}", "✓".green(), "component.toml".cyan());
        
        // Generate source files based on template
        self.generate_template(project_dir)?;
        
        // Generate package.json for npm compatibility
        let package_json = self.generate_package_json();
        fs::write(project_dir.join("package.json"), package_json)
            .context("Failed to write package.json")?;
        eprintln!("  {} Created {}", "✓".green(), "package.json".cyan());
        
        // Generate index.html
        let index_html = self.generate_index_html();
        fs::write(project_dir.join("index.html"), index_html)
            .context("Failed to write index.html")?;
        eprintln!("  {} Created {}", "✓".green(), "index.html".cyan());
        
        eprintln!(
            "\n{} Project initialized successfully!\n",
            "✓".green().bold()
        );
        
        eprintln!("  Next steps:");
        if self.name != "." {
            eprintln!("    {} cd {}", "→".dimmed(), self.name.cyan());
        }
        eprintln!("    {} component dev", "→".dimmed());
        eprintln!();
        
        Ok(())
    }
    
    fn generate_config(&self) -> String {
        let ext = if self.typescript { "tsx" } else { "jsx" };
        let main_file = match self.template.as_str() {
            "vanilla" => if self.typescript { "src/main.ts" } else { "src/main.js" },
            _ => &format!("src/main.{}", ext),
        };
        
        format!(
r#"# Component Reborn Configuration
# https://github.com/componentjs/component

[project]
name = "{name}"
version = "0.1.0"

[entrypoints]
main = "{main_file}"

[output]
dir = "dist"
public_url = "/"

[features]
jsx = {jsx}
typescript = {typescript}
css_modules = true

[dev]
port = 3000
open = false
"#,
            name = if self.name == "." { "my-app" } else { &self.name },
            main_file = main_file,
            jsx = self.template != "vanilla",
            typescript = self.typescript,
        )
    }
    
    fn generate_template(&self, project_dir: &Path) -> Result<()> {
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        
        match self.template.as_str() {
            "react" => self.generate_react_template(&src_dir)?,
            "vue" => self.generate_vue_template(&src_dir)?,
            "svelte" => self.generate_svelte_template(&src_dir)?,
            _ => self.generate_vanilla_template(&src_dir)?,
        }
        
        // Generate common CSS
        let css_content = r#"/* Global styles */
:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;

  color-scheme: light dark;
  color: rgba(255, 255, 255, 0.87);
  background-color: #242424;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  display: flex;
  place-items: center;
  min-width: 320px;
  min-height: 100vh;
}

#app {
  max-width: 1280px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
}

h1 {
  font-size: 3.2em;
  line-height: 1.1;
}

button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  background-color: #1a1a1a;
  cursor: pointer;
  transition: border-color 0.25s;
}

button:hover {
  border-color: #646cff;
}

button:focus,
button:focus-visible {
  outline: 4px auto -webkit-focus-ring-color;
}

@media (prefers-color-scheme: light) {
  :root {
    color: #213547;
    background-color: #ffffff;
  }
  button {
    background-color: #f9f9f9;
  }
}
"#;
        fs::write(src_dir.join("style.css"), css_content)?;
        eprintln!("  {} Created {}", "✓".green(), "src/style.css".cyan());
        
        Ok(())
    }
    
    fn generate_vanilla_template(&self, src_dir: &Path) -> Result<()> {
        let ext = if self.typescript { "ts" } else { "js" };
        let content = if self.typescript {
            r#"import './style.css';

const app = document.querySelector<HTMLDivElement>('#app')!;

let count: number = 0;

function setupCounter(): void {
  app.innerHTML = `
    <h1>Component Reborn</h1>
    <p>A modern, batteries-included frontend build tool</p>
    <button id="counter" type="button">Count is ${count}</button>
  `;
  
  const button = document.querySelector<HTMLButtonElement>('#counter')!;
  button.addEventListener('click', () => {
    count++;
    button.textContent = `Count is ${count}`;
  });
}

setupCounter();
"#
        } else {
            r#"import './style.css';

const app = document.querySelector('#app');

let count = 0;

function setupCounter() {
  app.innerHTML = `
    <h1>Component Reborn</h1>
    <p>A modern, batteries-included frontend build tool</p>
    <button id="counter" type="button">Count is ${count}</button>
  `;
  
  const button = document.querySelector('#counter');
  button.addEventListener('click', () => {
    count++;
    button.textContent = `Count is ${count}`;
  });
}

setupCounter();
"#
        };
        
        fs::write(src_dir.join(format!("main.{}", ext)), content)?;
        eprintln!("  {} Created {}", "✓".green(), format!("src/main.{}", ext).cyan());
        
        Ok(())
    }
    
    fn generate_react_template(&self, src_dir: &Path) -> Result<()> {
        let ext = if self.typescript { "tsx" } else { "jsx" };
        
        let main_content = if self.typescript {
            r#"import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './style.css';

ReactDOM.createRoot(document.getElementById('app')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#
        } else {
            r#"import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './style.css';

ReactDOM.createRoot(document.getElementById('app')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#
        };
        
        let app_content = if self.typescript {
            r#"import React, { useState } from 'react';

function App(): JSX.Element {
  const [count, setCount] = useState<number>(0);

  return (
    <>
      <h1>Component Reborn</h1>
      <p>A modern, batteries-included frontend build tool</p>
      <button onClick={() => setCount((c) => c + 1)}>
        Count is {count}
      </button>
    </>
  );
}

export default App;
"#
        } else {
            r#"import React, { useState } from 'react';

function App() {
  const [count, setCount] = useState(0);

  return (
    <>
      <h1>Component Reborn</h1>
      <p>A modern, batteries-included frontend build tool</p>
      <button onClick={() => setCount((c) => c + 1)}>
        Count is {count}
      </button>
    </>
  );
}

export default App;
"#
        };
        
        fs::write(src_dir.join(format!("main.{}", ext)), main_content)?;
        eprintln!("  {} Created {}", "✓".green(), format!("src/main.{}", ext).cyan());
        
        fs::write(src_dir.join(format!("App.{}", ext)), app_content)?;
        eprintln!("  {} Created {}", "✓".green(), format!("src/App.{}", ext).cyan());
        
        Ok(())
    }
    
    fn generate_vue_template(&self, src_dir: &Path) -> Result<()> {
        let ext = if self.typescript { "ts" } else { "js" };
        
        let main_content = format!(
            r#"import {{ createApp }} from 'vue';
import App from './App.vue';
import './style.css';

createApp(App).mount('#app');
"#
        );
        
        let app_content = if self.typescript {
            r#"<script setup lang="ts">
import { ref } from 'vue';

const count = ref<number>(0);
</script>

<template>
  <h1>Component Reborn</h1>
  <p>A modern, batteries-included frontend build tool</p>
  <button @click="count++">Count is {{ count }}</button>
</template>
"#
        } else {
            r#"<script setup>
import { ref } from 'vue';

const count = ref(0);
</script>

<template>
  <h1>Component Reborn</h1>
  <p>A modern, batteries-included frontend build tool</p>
  <button @click="count++">Count is {{ count }}</button>
</template>
"#
        };
        
        fs::write(src_dir.join(format!("main.{}", ext)), main_content)?;
        eprintln!("  {} Created {}", "✓".green(), format!("src/main.{}", ext).cyan());
        
        fs::write(src_dir.join("App.vue"), app_content)?;
        eprintln!("  {} Created {}", "✓".green(), "src/App.vue".cyan());
        
        Ok(())
    }
    
    fn generate_svelte_template(&self, src_dir: &Path) -> Result<()> {
        let ext = if self.typescript { "ts" } else { "js" };
        
        let main_content = r#"import App from './App.svelte';
import './style.css';

const app = new App({
  target: document.getElementById('app'),
});

export default app;
"#;
        
        let app_content = if self.typescript {
            r#"<script lang="ts">
  let count: number = 0;
</script>

<main>
  <h1>Component Reborn</h1>
  <p>A modern, batteries-included frontend build tool</p>
  <button on:click={() => count++}>
    Count is {count}
  </button>
</main>
"#
        } else {
            r#"<script>
  let count = 0;
</script>

<main>
  <h1>Component Reborn</h1>
  <p>A modern, batteries-included frontend build tool</p>
  <button on:click={() => count++}>
    Count is {count}
  </button>
</main>
"#
        };
        
        fs::write(src_dir.join(format!("main.{}", ext)), main_content)?;
        eprintln!("  {} Created {}", "✓".green(), format!("src/main.{}", ext).cyan());
        
        fs::write(src_dir.join("App.svelte"), app_content)?;
        eprintln!("  {} Created {}", "✓".green(), "src/App.svelte".cyan());
        
        Ok(())
    }
    
    fn generate_package_json(&self) -> String {
        let deps = match self.template.as_str() {
            "react" => r#""react": "^18.2.0",
    "react-dom": "^18.2.0""#,
            "vue" => r#""vue": "^3.3.0""#,
            "svelte" => r#""svelte": "^4.0.0""#,
            _ => "",
        };
        
        let dev_deps = if self.typescript {
            match self.template.as_str() {
                "react" => r#""@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "typescript": "^5.0.0""#,
                _ => r#""typescript": "^5.0.0""#,
            }
        } else {
            ""
        };
        
        format!(
            r#"{{
  "name": "{}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "component dev",
    "build": "component build",
    "preview": "component preview"
  }}{}{}
}}
"#,
            if self.name == "." { "my-app" } else { &self.name },
            if deps.is_empty() { String::new() } else { format!(",\n  \"dependencies\": {{\n    {}\n  }}", deps) },
            if dev_deps.is_empty() { String::new() } else { format!(",\n  \"devDependencies\": {{\n    {}\n  }}", dev_deps) },
        )
    }
    
    fn generate_index_html(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.{}"></script>
  </body>
</html>
"#,
            if self.name == "." { "My App" } else { &self.name },
            if self.typescript {
                match self.template.as_str() {
                    "vanilla" => "ts",
                    _ => "tsx",
                }
            } else {
                match self.template.as_str() {
                    "vanilla" => "js",
                    _ => "jsx",
                }
            }
        )
    }
}
