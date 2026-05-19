use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use tracing::info;

use crate::manifest::Manifest;

/// Templates for instant project scaffolding.
pub struct Scaffolder;

impl Scaffolder {
    /// Create a new project from a built-in template.
    pub fn create(project_dir: &Path, template: &str, name: &str) -> Result<()> {
        std::fs::create_dir_all(project_dir)?;

        match template {
            "react" => Self::scaffold_react(project_dir, name)?,
            "next" => Self::scaffold_next(project_dir, name)?,
            "vite" => Self::scaffold_vite(project_dir, name)?,
            "node" | "basic" => Self::scaffold_node(project_dir, name)?,
            _ => anyhow::bail!(
                "Unknown template '{}'. Available: react, next, vite, node",
                template
            ),
        }

        info!("Scaffolded '{}' project: {}", template, name);
        Ok(())
    }

    /// List available templates.
    pub fn templates() -> Vec<(&'static str, &'static str)> {
        vec![
            ("react", "React 19 with modern JSX"),
            ("next", "Next.js 15 full-stack app"),
            ("vite", "Vite + React SPA"),
            ("node", "Basic Node.js project"),
        ]
    }

    fn scaffold_node(dir: &Path, name: &str) -> Result<()> {
        let mut manifest = Manifest::new(name);
        manifest.description = Some("A GhostFS project".to_string());
        manifest
            .scripts
            .insert("start".to_string(), "node index.js".to_string());
        manifest
            .scripts
            .insert("dev".to_string(), "node --watch index.js".to_string());
        manifest.save(&dir.join("ghost.json"))?;

        std::fs::write(
            dir.join("index.js"),
            "console.log('Hello from GhostFS! 👻');\n",
        )?;

        Ok(())
    }

    fn scaffold_react(dir: &Path, name: &str) -> Result<()> {
        let mut manifest = Manifest::new(name);
        manifest.description = Some("React app powered by GhostFS".to_string());
        manifest.dependencies = BTreeMap::from([
            ("react".to_string(), "^19.0.0".to_string()),
            ("react-dom".to_string(), "^19.0.0".to_string()),
        ]);
        manifest.dev_dependencies = BTreeMap::from([
            ("@vitejs/plugin-react".to_string(), "^4.0.0".to_string()),
            ("vite".to_string(), "^6.0.0".to_string()),
        ]);
        manifest.scripts = BTreeMap::from([
            ("dev".to_string(), "vite".to_string()),
            ("build".to_string(), "vite build".to_string()),
            ("preview".to_string(), "vite preview".to_string()),
        ]);
        manifest.save(&dir.join("ghost.json"))?;

        std::fs::create_dir_all(dir.join("src"))?;

        std::fs::write(
            dir.join("src").join("App.jsx"),
            r#"export default function App() {
  return (
    <div style={{ fontFamily: 'system-ui', padding: '2rem', textAlign: 'center' }}>
      <h1>👻 GhostFS + React</h1>
      <p>Zero node_modules. Instant setup.</p>
    </div>
  );
}
"#,
        )?;

        std::fs::write(
            dir.join("src").join("main.jsx"),
            r#"import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#,
        )?;

        std::fs::write(
            dir.join("index.html"),
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{}</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.jsx"></script>
</body>
</html>
"#,
                name
            ),
        )?;

        Ok(())
    }

    fn scaffold_next(dir: &Path, name: &str) -> Result<()> {
        let mut manifest = Manifest::new(name);
        manifest.description = Some("Next.js app powered by GhostFS".to_string());
        manifest.dependencies = BTreeMap::from([
            ("next".to_string(), "^15.0.0".to_string()),
            ("react".to_string(), "^19.0.0".to_string()),
            ("react-dom".to_string(), "^19.0.0".to_string()),
        ]);
        manifest.scripts = BTreeMap::from([
            ("dev".to_string(), "next dev".to_string()),
            ("build".to_string(), "next build".to_string()),
            ("start".to_string(), "next start".to_string()),
        ]);
        manifest.save(&dir.join("ghost.json"))?;

        std::fs::create_dir_all(dir.join("app"))?;

        std::fs::write(
            dir.join("app").join("page.js"),
            r#"export default function Home() {
  return (
    <main style={{ fontFamily: 'system-ui', padding: '2rem', textAlign: 'center' }}>
      <h1>👻 GhostFS + Next.js</h1>
      <p>Zero node_modules. Server components. Instant setup.</p>
    </main>
  );
}
"#,
        )?;

        std::fs::write(
            dir.join("app").join("layout.js"),
            format!(
                r#"export const metadata = {{
  title: '{}',
  description: 'Powered by GhostFS',
}};

export default function RootLayout({{ children }}) {{
  return (
    <html lang="en">
      <body>{{children}}</body>
    </html>
  );
}}
"#,
                name
            ),
        )?;

        Ok(())
    }

    fn scaffold_vite(dir: &Path, name: &str) -> Result<()> {
        let mut manifest = Manifest::new(name);
        manifest.description = Some("Vite app powered by GhostFS".to_string());
        manifest.dependencies = BTreeMap::from([
            ("react".to_string(), "^19.0.0".to_string()),
            ("react-dom".to_string(), "^19.0.0".to_string()),
        ]);
        manifest.dev_dependencies = BTreeMap::from([
            ("@vitejs/plugin-react".to_string(), "^4.0.0".to_string()),
            ("vite".to_string(), "^6.0.0".to_string()),
        ]);
        manifest.scripts = BTreeMap::from([
            ("dev".to_string(), "vite".to_string()),
            ("build".to_string(), "vite build".to_string()),
        ]);
        manifest.save(&dir.join("ghost.json"))?;

        std::fs::create_dir_all(dir.join("src"))?;

        std::fs::write(
            dir.join("src").join("main.jsx"),
            r#"import React from 'react';
import ReactDOM from 'react-dom/client';

function App() {
  return (
    <div style={{ fontFamily: 'system-ui', padding: '2rem', textAlign: 'center' }}>
      <h1>⚡ Vite + GhostFS</h1>
      <p>Blazing fast. Zero node_modules.</p>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
"#,
        )?;

        std::fs::write(
            dir.join("index.html"),
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{}</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.jsx"></script>
</body>
</html>
"#,
                name
            ),
        )?;

        std::fs::write(
            dir.join("vite.config.js"),
            r#"import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
});
"#,
        )?;

        Ok(())
    }
}
