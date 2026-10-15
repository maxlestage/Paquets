//! `paquets make:*`: generators in the style of Laravel's Artisan and Rails, which
//! write idiomatic code and wire it into `src/main.rs`.
//!
//! ```text
//! paquets make:model Post title:string body:text published:bool --api --admin
//! paquets make:api Comment body:text post_id:i64
//! paquets make:page about
//! paquets make:migration add_slug_to_posts
//! paquets make:test checkout
//! paquets make:auth
//! paquets make:admin Post Comment
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The usage of the generators, for `paquets help`.
pub const USAGE: &str = "\
  paquets make:model <Name> [field:type…] [--api] [--admin]
                                   a model in src/models/ (types: string, text, i32,
                                   i64, f64, bool; `?` for optional, `:unique`)
  paquets make:api <Name> [field:type…]   a model and its REST API, with a test
  paquets make:page <name>         a page in src/pages/, routed at /<name>
  paquets make:migration <name>    a versioned migration in src/migrations/
  paquets make:test <name>         a feature test in src/feature_tests/
  paquets make:auth                sign up, log in, password resets, 2FA
  paquets make:admin [Name…]       the admin panel, with these models";

/// Where the generators write: the project folder (the current one).
pub struct Project {
    root: PathBuf,
    /// What was written, for the summary.
    log: Vec<String>,
}

impl Project {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Project {
            root: root.into(),
            log: Vec::new(),
        }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn read(&self, relative: &str) -> Option<String> {
        fs::read_to_string(self.path(relative)).ok()
    }

    fn write(&mut self, relative: &str, content: &str, created: bool) -> Result<(), String> {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
        self.log.push(format!(
            "  {} {relative}",
            if created { "created" } else { "updated" }
        ));
        Ok(())
    }

    /// Writes a new file; an error if it exists.
    fn create(&mut self, relative: &str, content: &str) -> Result<(), String> {
        if self.path(relative).exists() {
            return Err(format!("{relative} already exists"));
        }
        self.write(relative, content, true)
    }

    /// Formats the code like `cargo fmt`, when rustfmt is installed (the generated
    /// code is already formatted; this keeps edited lines tidy, such as long chains).
    pub fn format(&self) {
        let manifest = self.read("Cargo.toml").unwrap_or_default();
        let edition = ["2024", "2021", "2018"]
            .into_iter()
            .find(|e| manifest.contains(&format!("edition = \"{e}\"")))
            .unwrap_or("2021");
        let _ = std::process::Command::new("rustfmt")
            .args(["--edition", edition])
            .arg(self.path("src/main.rs"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    fn main_rs(&self) -> Result<String, String> {
        if !self.path("Cargo.toml").is_file() {
            return Err(
                "no Cargo.toml here: run this command in a project folder (see `paquets new`)"
                    .into(),
            );
        }
        self.read("src/main.rs")
            .ok_or_else(|| "no src/main.rs in this project".to_owned())
    }
}

pub fn run(command: &str, args: &[String]) -> Result<(), String> {
    let mut project = Project::new(".");
    let notes = generate(&mut project, command, args)?;
    project.format();
    println!("📦 {command}");
    for line in &project.log {
        println!("{line}");
    }
    for note in notes {
        println!("\n{note}");
    }
    Ok(())
}

/// Runs a generator; returns notes for the user.
pub fn generate(
    project: &mut Project,
    command: &str,
    args: &[String],
) -> Result<Vec<String>, String> {
    let flags: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| a.starts_with("--"))
        .collect();
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with("--"))
        .collect();
    let known: &[&str] = match command {
        "make:model" => &["--api", "--admin"],
        "make:api" => &["--admin"],
        _ => &[],
    };
    if let Some(flag) = flags.iter().find(|f| !known.contains(f)) {
        return Err(format!("unknown option `{flag}` for {command}"));
    }
    match command {
        "make:model" | "make:api" => {
            let name = words
                .first()
                .ok_or(format!("usage: paquets {command} <Name> [field:type…]"))?;
            let fields = words[1..]
                .iter()
                .map(|f| Field::parse(f))
                .collect::<Result<Vec<_>, _>>()?;
            let api = command == "make:api" || flags.contains(&"--api");
            model(project, name, &fields, api, flags.contains(&"--admin"))
        }
        "make:page" => page(
            project,
            words.first().ok_or("usage: paquets make:page <name>")?,
        ),
        "make:migration" => migration(
            project,
            words
                .first()
                .ok_or("usage: paquets make:migration <name>")?,
            &timestamp(),
        ),
        "make:test" => feature_test(
            project,
            words.first().ok_or("usage: paquets make:test <name>")?,
        ),
        "make:auth" => auth(project),
        "make:admin" => admin(project, &words),
        other => Err(format!(
            "unknown generator `{other}`\n\nGenerators:\n{USAGE}"
        )),
    }
}

// ----- Names -----------------------------------------------------------------------

/// `BlogPost`, `blog-post`, `blog post` → `blog_post`.
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    let mut previous_lower = false;
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if previous_lower {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            previous_lower = false;
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
            previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
            previous_lower = false;
        }
    }
    out.trim_end_matches('_').to_owned()
}

/// `blog_post` → `BlogPost`.
pub fn pascal(name: &str) -> String {
    snake(name)
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |c| {
                c.to_ascii_uppercase().to_string() + chars.as_str()
            })
        })
        .collect()
}

/// `post` → `posts`, `category` → `categories`, `box` → `boxes`.
pub fn plural(word: &str) -> String {
    if word.ends_with('y')
        && !word.ends_with("ay")
        && !word.ends_with("ey")
        && !word.ends_with("oy")
    {
        format!("{}ies", &word[..word.len() - 1])
    } else if ["s", "x", "z", "ch", "sh"]
        .iter()
        .any(|end| word.ends_with(end))
    {
        format!("{word}es")
    } else {
        format!("{word}s")
    }
}

/// `blog_post` → `Blog post`.
fn sentence(snake_name: &str) -> String {
    let words = snake_name.replace('_', " ");
    let mut chars = words.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_ascii_uppercase().to_string() + chars.as_str()
    })
}

fn identifier(name: &str, what: &str) -> Result<String, String> {
    let snake = snake(name);
    let valid = snake.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && !matches!(
            snake.as_str(),
            "mod"
                | "fn"
                | "type"
                | "struct"
                | "impl"
                | "use"
                | "self"
                | "super"
                | "crate"
                | "main"
                | "app"
                | "test"
                | "tests"
                | "match"
                | "loop"
                | "move"
                | "ref"
                | "where"
                | "async"
                | "await"
                | "dyn"
                | "enum"
                | "trait"
                | "static"
                | "const"
                | "let"
                | "if"
                | "else"
                | "for"
                | "while"
                | "return"
                | "true"
                | "false"
                | "pub"
                | "in"
                | "as"
                | "box"
        );
    if valid {
        Ok(snake)
    } else {
        Err(format!("`{name}` is not a valid {what} name"))
    }
}

// ----- Fields ----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    name: String,
    rust: &'static str,
    optional: bool,
    unique: bool,
}

impl Field {
    /// `title:string`, `views:i64?`, `email:string:unique`.
    pub fn parse(spec: &str) -> Result<Field, String> {
        let mut parts = spec.split(':');
        let name = parts.next().unwrap_or_default();
        let kind = parts.next().unwrap_or("string");
        let mut unique = false;
        for modifier in parts {
            match modifier {
                "unique" => unique = true,
                other => return Err(format!("unknown modifier `{other}` in `{spec}` (unique)")),
            }
        }
        let name = identifier(name, "field")?;
        if name == "id" {
            return Err("`id` is added to every model: leave it out".into());
        }
        let (kind, optional) = match kind.strip_suffix('?') {
            Some(kind) => (kind, true),
            None => (kind, false),
        };
        let rust = match kind {
            "string" | "str" | "text" => "String",
            "int" | "integer" | "i64" | "bigint" => "i64",
            "i32" => "i32",
            "float" | "f64" | "double" | "number" | "decimal" => "f64",
            "bool" | "boolean" => "bool",
            other => {
                return Err(format!(
                    "unknown type `{other}` in `{spec}` (string, text, i32, i64, f64, bool)"
                ));
            }
        };
        Ok(Field {
            name,
            rust,
            optional,
            unique,
        })
    }

    fn rust_type(&self) -> String {
        if self.optional {
            format!("Option<{}>", self.rust)
        } else {
            self.rust.to_owned()
        }
    }

    /// An example value, in JSON.
    fn sample(&self) -> String {
        match self.rust {
            "String" => format!("\"{} 1\"", sentence(&self.name)),
            "i64" | "i32" => "1".into(),
            "f64" => "1.5".into(),
            _ => "true".into(),
        }
    }
}

// ----- Editing src/main.rs ---------------------------------------------------------------

/// Adds `line` to the run of consecutive lines starting with `prefix`, in sorted
/// order (as rustfmt keeps `mod` declarations); without such a run, puts it before
/// the line starting with `before` (followed by a blank line), or at the end.
fn insert_sorted(text: &str, prefix: &str, line: &str, before: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    if lines.iter().any(|l| l == line) {
        return text.to_owned();
    }
    // Declarations only (`mod x;`, `pub use a::B;`), not `mod tests {`.
    let run: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with(prefix) && l.ends_with(';'))
        .map(|(i, _)| i)
        .collect();
    if let (Some(&first), Some(&last)) = (run.first(), run.last()) {
        let at = (first..=last)
            .find(|&i| lines[i].as_str() > line)
            .unwrap_or(last + 1);
        lines.insert(at, line.to_owned());
    } else if let Some(at) = lines.iter().position(|l| l.starts_with(before)) {
        lines.insert(at, String::new());
        lines.insert(at, line.to_owned());
    } else {
        if lines.last().is_some_and(|l| !l.is_empty()) {
            lines.push(String::new());
        }
        lines.push(line.to_owned());
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Declares the module `name` in `src/main.rs`.
fn declare_module(main: &str, name: &str) -> String {
    insert_sorted(main, "mod ", &format!("mod {name};"), "use ")
}

/// Adds `statement` to the function that builds the app (`fn app() -> App`, before
/// it returns `app`; otherwise after `App::new()`). `None` when there is no such place.
fn register(main: &str, statement: &str) -> Option<String> {
    let mut lines: Vec<String> = main.lines().map(str::to_owned).collect();
    let line = format!("    {statement}");
    if lines.contains(&line) {
        return Some(main.to_owned());
    }
    let start = lines.iter().position(|l| l.starts_with("fn app()"));
    let at = start
        .and_then(|s| (s..lines.len()).find(|&i| lines[i] == "    app"))
        .or_else(|| {
            lines
                .iter()
                .position(|l| l.trim_start().starts_with("let mut app = App::new()"))
                .map(|i| i + 1)
        })?;
    lines.insert(at, line);
    let mut out = lines.join("\n");
    out.push('\n');
    Some(out)
}

fn has_app_function(main: &str) -> bool {
    main.lines().any(|l| l.starts_with("fn app() -> App"))
}

/// Registers `statement`, or tells the user where to write it.
fn wire(main: String, statement: &str, notes: &mut Vec<String>) -> String {
    match register(&main, statement) {
        Some(main) => main,
        None => {
            notes.push(format!(
                "Add this line where your app is built (after `App::new()`):\n    {statement}"
            ));
            main
        }
    }
}

// ----- Generators ------------------------------------------------------------------

fn model(
    project: &mut Project,
    name: &str,
    fields: &[Field],
    api: bool,
    with_admin: bool,
) -> Result<Vec<String>, String> {
    let main = project.main_rs()?;
    let module = identifier(name, "model")?;
    let type_name = pascal(&module);
    let table = plural(&module);
    let mut notes = Vec::new();

    let mut code = format!(
        "//! The `{type_name}` model: a table of the database (`{table}`), also sent and\n\
         //! received as JSON.\n\nuse paquets::prelude::*;\n\n/// {}.\n#[model(table = \"{table}\")]\npub struct {type_name} {{\n    pub id: i64,\n",
        sentence(&module)
    );
    for field in fields {
        if field.unique {
            code.push_str("    #[memoire(unique)]\n");
        }
        code.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type()));
    }
    code.push_str("}\n");
    let path = format!("/api/{}", table.replace('_', "-"));
    if api && has_app_function(&main) {
        let body = fields
            .iter()
            .map(|f| format!("\"{}\": {}", f.name, f.sample()))
            .collect::<Vec<_>>()
            .join(", ");
        let request = format!("            .json(&json!({{ {body} }}))");
        let request = if request.len() <= 100 {
            format!(
                "        let res = client\n            .post(\"{path}\")\n{request}\n            .await;\n"
            )
        } else {
            let mut lines = String::from("            .json(&json!({\n");
            for f in fields {
                lines.push_str(&format!(
                    "                \"{}\": {},\n",
                    f.name,
                    f.sample()
                ));
            }
            lines.push_str("            }))\n");
            format!(
                "        let res = client\n            .post(\"{path}\")\n{lines}            .await;\n"
            )
        };
        code.push_str(&format!(
            "\n#[cfg(test)]\nmod tests {{\n    use paquets::prelude::*;\n\n    #[tokio::test]\n    async fn the_{table}_api() {{\n        let client = crate::app().test().await;\n{request}        assert_eq!(res.status(), 201);\n\n        let res = client.get(\"{path}\").await;\n        assert_eq!(res.status(), 200);\n        assert_eq!(res.header(\"x-total-count\"), Some(\"1\"));\n    }}\n}}\n"
        ));
    }
    project.create(&format!("src/models/{module}.rs"), &code)?;

    let index = project
        .read("src/models.rs")
        .unwrap_or_else(|| "//! The models: the tables of the database.\n".to_owned());
    let index = insert_sorted(&index, "mod ", &format!("mod {module};"), "pub use ");
    let index = insert_sorted(
        &index,
        "pub use ",
        &format!("pub use {module}::{type_name};"),
        "\u{0}",
    );
    let created = !project.path("src/models.rs").exists();
    project.write("src/models.rs", &index, created)?;

    let mut main = declare_module(&main, "models");
    main = if api {
        wire(
            main,
            &format!("app.api::<models::{type_name}>(\"{path}\");"),
            &mut notes,
        )
    } else {
        wire(
            main,
            &format!("app.model::<models::{type_name}>();"),
            &mut notes,
        )
    };
    if with_admin {
        main = add_to_admin(main, &type_name, &mut notes);
    }
    project.write("src/main.rs", &main, false)?;
    if api {
        notes.push(format!(
            "REST API: GET/POST {path}, GET/PUT/PATCH/DELETE {path}/:id"
        ));
    }
    Ok(notes)
}

fn add_to_admin(main: String, type_name: &str, notes: &mut Vec<String>) -> String {
    let registration = format!(".model::<models::{type_name}>()");
    // The `app.admin(…)` statement, up to its `;`.
    let statement = main
        .find("app.admin(")
        .map(|at| &main[at..at + main[at..].find(';').unwrap_or(main.len() - at)]);
    if statement.is_some_and(|s| s.contains(&registration)) {
        return main;
    }
    if let Some(at) = main.find("Admin::new()") {
        let mut main = main;
        main.insert_str(at + "Admin::new()".len(), &registration);
        return main;
    }
    wire(
        main,
        &format!("app.admin(Admin::new(){registration});"),
        notes,
    )
}

fn page(project: &mut Project, name: &str) -> Result<Vec<String>, String> {
    let main = project.main_rs()?;
    let module = identifier(name, "page")?;
    let title = sentence(&module);
    let route = format!("/{}", module.replace('_', "-"));
    let mut notes = Vec::new();
    let mut code = format!(
        "//! The `{route}` page.\n\nuse paquets::prelude::*;\n\n/// `GET {route}`, rendered on the server with its SEO tags.\npub async fn {module}(_req: Request) -> Page {{\n    Page::new(\"{title}\")\n        .description(\"{title}.\")\n        .body(\n            html::main()\n                .class(\"stack\")\n                .child(h1().text(\"{title}\"))\n                .child(p().text(\"Edit src/pages/{module}.rs.\")),\n        )\n}}\n"
    );
    if has_app_function(&main) {
        code.push_str(&format!(
            "\n#[cfg(test)]\nmod tests {{\n    #[tokio::test]\n    async fn the_page_is_served() {{\n        let client = crate::app().test().await;\n        let res = client.get(\"{route}\").await;\n        assert_eq!(res.status(), 200);\n        assert!(res.text().contains(\"<h1>{title}</h1>\"));\n    }}\n}}\n"
        ));
    }
    project.create(&format!("src/pages/{module}.rs"), &code)?;
    let index = project
        .read("src/pages.rs")
        .unwrap_or_else(|| "//! The pages, rendered on the server.\n".to_owned());
    let created = !project.path("src/pages.rs").exists();
    let index = insert_sorted(&index, "pub mod ", &format!("pub mod {module};"), "\u{0}");
    project.write("src/pages.rs", &index, created)?;
    let main = declare_module(&main, "pages");
    let main = wire(
        main,
        &format!("app.get(\"{route}\", pages::{module}::{module});"),
        &mut notes,
    );
    project.write("src/main.rs", &main, false)?;
    notes.push(format!("Page: http://localhost:3000{route}"));
    Ok(notes)
}

/// The UTC time as `YYYYMMDDHHMMSS`.
fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    // Howard Hinnant's civil-from-days.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}{:02}{:02}{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

fn migration(project: &mut Project, name: &str, version: &str) -> Result<Vec<String>, String> {
    let main = project.main_rs()?;
    let name = identifier(name, "migration")?;
    let module = format!("m{version}_{name}");
    let mut notes = Vec::new();
    project.create(
        &format!("src/migrations/{module}.rs"),
        &format!(
            "//! Migration {version}: {}.\n\nuse paquets::prelude::*;\n\n/// The changes of this migration, applied once, in a transaction.\npub fn migration() -> Migration {{\n    // Add the changes, for example:\n    //     .create_table::<crate::models::Post>()\n    //     .sql(\"ALTER TABLE posts ADD COLUMN slug TEXT\")\n    Migration::new({version}, \"{name}\")\n}}\n",
            sentence(&name).to_lowercase()
        ),
    )?;
    let index = project.read("src/migrations.rs").unwrap_or_else(|| {
        "//! The versioned migrations, run at startup and by `paquets db:migrate`.\n\nuse paquets::prelude::*;\n\n/// Every migration, in order.\nconst MIGRATIONS: &[fn() -> Migration] = &[\n];\n\n/// The migrator of the app.\npub fn all() -> Migrator {\n    MIGRATIONS\n        .iter()\n        .fold(Migrator::new(), |migrator, migration| {\n            migrator.add(migration())\n        })\n}\n".to_owned()
    });
    let created = !project.path("src/migrations.rs").exists();
    let index = insert_sorted(
        &index,
        "mod ",
        &format!("mod {module};"),
        "/// Every migration",
    );
    let index = add_migration(&index, &format!("{module}::migration"));
    project.write("src/migrations.rs", &index, created)?;
    let main = declare_module(&main, "migrations");
    let main = wire(main, "app.migrate(migrations::all());", &mut notes);
    project.write("src/main.rs", &main, false)?;
    notes.push(
        "Run it with `paquets db:migrate` (or start the app: it runs pending migrations).".into(),
    );
    Ok(notes)
}

/// Adds `entry` to the `MIGRATIONS` list of `src/migrations.rs`, one per line.
fn add_migration(index: &str, entry: &str) -> String {
    let Some(start) = index.find("MIGRATIONS: &[fn() -> Migration] = &[") else {
        return index.to_owned();
    };
    let open = start + "MIGRATIONS: &[fn() -> Migration] = &[".len();
    let Some(len) = index[open..].find("];") else {
        return index.to_owned();
    };
    let mut entries: Vec<String> = index[open..open + len]
        .split(',')
        .map(|e| e.trim().to_owned())
        .filter(|e| !e.is_empty())
        .collect();
    if !entries.iter().any(|e| e == entry) {
        entries.push(entry.to_owned());
    }
    let list: String = entries.iter().map(|e| format!("\n    {e},")).collect();
    format!("{}{list}\n{}", &index[..open], &index[open + len..])
}

fn feature_test(project: &mut Project, name: &str) -> Result<Vec<String>, String> {
    let main = project.main_rs()?;
    if !has_app_function(&main) {
        return Err(
            "make:test needs a `fn app() -> App` in src/main.rs (as `paquets new` writes it)"
                .into(),
        );
    }
    let module = identifier(name, "test")?;
    project.create(
        &format!("src/feature_tests/{module}.rs"),
        &format!(
            "//! Feature test: {}.\n\n#[tokio::test]\nasync fn {module}() {{\n    let client = crate::app().test().await;\n    let res = client.get(\"/\").await;\n    assert_eq!(res.status(), 200);\n}}\n",
            sentence(&module).to_lowercase()
        ),
    )?;
    let index = project.read("src/feature_tests.rs").unwrap_or_else(|| {
        "//! Feature tests: the whole app, through the in-memory test client (`cargo test`).\n#![cfg(test)]\n".to_owned()
    });
    let created = !project.path("src/feature_tests.rs").exists();
    let index = insert_sorted(&index, "mod ", &format!("mod {module};"), "\u{0}");
    project.write("src/feature_tests.rs", &index, created)?;
    let main = declare_module(&main, "feature_tests");
    project.write("src/main.rs", &main, false)?;
    Ok(vec!["Run it with `cargo test`.".into()])
}

fn auth(project: &mut Project) -> Result<Vec<String>, String> {
    let main = project.main_rs()?;
    if main.contains("app.auth(") {
        return Err("src/main.rs already calls app.auth(…)".into());
    }
    let mut notes = Vec::new();
    let main = wire(main, "app.auth(paquets::auth::Auth::new());", &mut notes);
    project.write("src/main.rs", &main, false)?;
    // Argon2 is slow when compiled without optimizations: optimize it, even in debug.
    let manifest = project.read("Cargo.toml").unwrap_or_default();
    if !manifest.contains("[profile.dev.package.argon2]") {
        let manifest = format!(
            "{}\n# Password hashing (paquets::auth) at full speed in debug builds too.\n[profile.dev.package.argon2]\nopt-level = 3\n",
            manifest.trim_end()
        );
        project.write("Cargo.toml", &manifest, false)?;
    }
    notes.push(
        "Pages: /register, /login, /forgot-password, /account (two-factor codes).\n\
         Protect a route: `handler.with(paquets::auth::login_required)`; read the user with `req.user()`.\n\
         In production, set APP_KEY (32+ random bytes: `openssl rand -base64 48`) and APP_URL."
            .into(),
    );
    Ok(notes)
}

fn admin(project: &mut Project, models: &[&str]) -> Result<Vec<String>, String> {
    let mut main = project.main_rs()?;
    let mut notes = Vec::new();
    if models.is_empty() {
        if main.contains("Admin::new()") {
            return Err("src/main.rs already has an admin panel".into());
        }
        main = wire(main, "app.admin(Admin::new());", &mut notes);
    }
    for model in models {
        let type_name = pascal(&identifier(model, "model")?);
        if !Path::new(&project.path(&format!("src/models/{}.rs", snake(model)))).is_file() {
            notes.push(format!(
                "note: src/models/{}.rs does not exist: create it with `paquets make:model {type_name}`",
                snake(model)
            ));
        }
        main = add_to_admin(main, &type_name, &mut notes);
    }
    project.write("src/main.rs", &main, false)?;
    notes.push("Admin panel: http://localhost:3000/admin".into());
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(snake("BlogPost"), "blog_post");
        assert_eq!(snake("blog-post"), "blog_post");
        assert_eq!(snake("post"), "post");
        assert_eq!(pascal("blog_post"), "BlogPost");
        assert_eq!(plural("post"), "posts");
        assert_eq!(plural("category"), "categories");
        assert_eq!(plural("day"), "days");
        assert_eq!(plural("box"), "boxes");
        assert!(identifier("2fast", "model").is_err());
        assert!(identifier("type", "model").is_err());
    }

    #[test]
    fn fields() {
        let f = Field::parse("views:i64?").unwrap();
        assert_eq!(f.rust_type(), "Option<i64>");
        let f = Field::parse("email:string:unique").unwrap();
        assert!(f.unique);
        assert_eq!(Field::parse("title").unwrap().rust, "String");
        assert!(Field::parse("id:i64").is_err());
        assert!(Field::parse("x:date").is_err());
    }

    #[test]
    fn modules_are_declared_in_order() {
        let main = "//! App\n\nuse paquets::prelude::*;\n\nfn main() {}\n";
        let main = declare_module(main, "pages");
        assert!(
            main.contains("//! App\n\nmod pages;\n\nuse paquets"),
            "{main}"
        );
        let main = declare_module(&main, "models");
        assert!(main.contains("mod models;\nmod pages;\n\nuse"), "{main}");
        assert_eq!(declare_module(&main, "models"), main);
    }

    #[test]
    fn statements_go_where_the_app_is_built() {
        let main = "fn app() -> App {\n    let mut app = App::new();\n    app.get(\"/\", home);\n    app\n}\n";
        let main = register(main, "app.model::<models::Post>();").unwrap();
        assert!(main.contains("home);\n    app.model::<models::Post>();\n    app\n}"));
        let other = "fn main() {\n    let mut app = App::new();\n    app.run()\n}\n";
        assert!(
            register(other, "x();")
                .unwrap()
                .contains("App::new();\n    x();\n")
        );
        assert!(register("fn main() {}\n", "x();").is_none());
    }

    #[test]
    fn admin_registrations_are_chained_and_wrapped() {
        let main = "fn app() -> App {\n    let mut app = App::new();\n    app\n}\n".to_owned();
        let mut notes = Vec::new();
        let main = add_to_admin(main, "Post", &mut notes);
        assert!(main.contains("    app.admin(Admin::new().model::<models::Post>());\n"));
        let main = add_to_admin(main, "Comment", &mut notes);
        assert!(main.contains("Admin::new().model::<models::Comment>().model::<models::Post>()"));
        assert!(notes.is_empty());
    }

    #[test]
    fn migrations_are_listed_in_order() {
        let index = "const MIGRATIONS: &[fn() -> Migration] = &[m1_a::migration];\n";
        let index = add_migration(index, "m2_b::migration");
        assert_eq!(
            index,
            "const MIGRATIONS: &[fn() -> Migration] = &[\n    m1_a::migration,\n    m2_b::migration,\n];\n"
        );
        assert_eq!(add_migration(&index, "m2_b::migration"), index);
    }

    #[test]
    fn timestamps_have_fourteen_digits() {
        let t = timestamp();
        assert_eq!(t.len(), 14);
        assert!(t.starts_with("20"));
    }
}
