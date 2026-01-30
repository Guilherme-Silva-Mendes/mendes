//! Mendes compiler CLI

use clap::{Parser, Subcommand, ValueEnum};
use mendes_error::{DiagnosticRenderer, SourceCache};
use mendes_lexer::{Lexer, TokenKind};
use mendes_parser::parse;
use mendes_semantic::{analyze, SemanticContext};
use mendes_ir::lower_program;
use mendes_codegen::{CBackend, RustBackend, CodeGen};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Code generation backend
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum Backend {
    /// Generates Rust code (default)
    #[default]
    Rust,
    /// Generates C code
    C,
}

#[derive(Parser)]
#[command(name = "mendes")]
#[command(author = "Guilherme Mendes")]
#[command(version = "0.1.0")]
#[command(about = "Mendes language compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles a .ms file to a native executable
    Build {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Code generation backend
        #[arg(short, long, default_value = "rust")]
        backend: Backend,

        /// Release mode (optimized)
        #[arg(long)]
        release: bool,
    },

    /// Checks for errors without compiling
    Check {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Shows file tokens (debug)
    Lex {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Shows file AST (debug)
    Parse {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Shows file IR (debug)
    Ir {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Generates C code from file
    Emit {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Generates Rust code from file
    EmitRust {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Runs a .ms file (future: JIT or interpreter)
    Run {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { input, output, backend, release } => {
            println!("Compiling: {}", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);
                    let renderer = DiagnosticRenderer::new(&cache);

                    // Phase 1: Lexical analysis
                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }
                    println!("  [ok] Lexer: {} tokens", tokens.len());

                    // Phase 2: Parsing
                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("\nSyntax errors:\n");
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }
                    println!("  [ok] Parser: {} statements", program.statements.len());

                    // Phase 3: Semantic analysis
                    let mut ctx = SemanticContext::new();
                    let semantic_diags = analyze(&program, &mut ctx);

                    if semantic_diags.has_errors() {
                        eprintln!("\nSemantic errors:\n");
                        for diag in semantic_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }
                    println!("  [ok] Semantic: types verified");

                    // Phase 4: Generate IR
                    let ir_module = lower_program(&program);
                    println!("  [ok] IR: {} functions, {} routes", ir_module.functions.len(), ir_module.routes.len());

                    // Determine output name
                    let output_name = output
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| {
                            input.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "output".to_string())
                        });

                    match backend {
                        Backend::Rust => {
                            build_with_rust_backend(&ir_module, &output_name, release);
                        }
                        Backend::C => {
                            build_with_c_backend(&ir_module, &output_name);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Check { input } => {
            println!("Checking: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);
                    let renderer = DiagnosticRenderer::new(&cache);

                    // Phase 1: Lexical analysis
                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }
                    println!("  [ok] Lexer: {} tokens", tokens.len());

                    // Phase 2: Parsing
                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("\nSyntax errors:\n");
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }
                    println!("  [ok] Parser: {} statements", program.statements.len());

                    // Phase 3: Semantic analysis
                    let mut ctx = SemanticContext::new();
                    let semantic_diags = analyze(&program, &mut ctx);

                    if semantic_diags.has_errors() {
                        eprintln!("\nSemantic errors:\n");
                        for diag in semantic_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    let warning_count = semantic_diags.len();
                    if warning_count > 0 {
                        println!("  [warn] Semantic: {} warning(s)", warning_count);
                        for diag in semantic_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                    } else {
                        println!("  [ok] Semantic: types verified");
                    }

                    println!("\nNo errors found!");
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Lex { input } => {
            println!("Tokenizing: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);

                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let diagnostics = lexer.take_diagnostics();

                    // Show tokens
                    for token in &tokens {
                        let kind_str = format!("{:?}", token.kind);
                        let display = match &token.kind {
                            TokenKind::Newline => "↵".to_string(),
                            TokenKind::Indent => "→".to_string(),
                            TokenKind::Dedent => "←".to_string(),
                            TokenKind::Eof => "EOF".to_string(),
                            _ => format!("{}", token.kind),
                        };

                        println!(
                            "  {:4}:{:<3}  {:<20}  {}",
                            token.span.start.line,
                            token.span.start.column,
                            kind_str.chars().take(20).collect::<String>(),
                            display
                        );
                    }

                    println!("\nTotal: {} tokens", tokens.len());

                    // Show errors if any
                    if diagnostics.has_errors() {
                        println!("\nErrors found:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in diagnostics.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Parse { input } => {
            println!("Parsing: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);

                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("Syntax errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Show the AST
                    println!("AST ({} statements):\n", program.statements.len());
                    for (i, stmt) in program.statements.iter().enumerate() {
                        println!("{}. {}", i + 1, format_stmt(stmt, 0));
                    }

                    println!("\nParse completed successfully!");
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Ir { input } => {
            println!("Generating IR: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);

                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("Syntax errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Generate IR
                    let ir_module = lower_program(&program);

                    // Show the IR
                    println!("{}", ir_module);

                    // Statistics
                    println!("Statistics:");
                    println!("   Functions: {}", ir_module.functions.len());
                    println!("   HTTP Routes: {}", ir_module.routes.len());
                    println!("   Structs: {}", ir_module.structs.len());
                    println!("   Strings: {}", ir_module.string_table.len());

                    println!("\nIR generated successfully!");
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Emit { input, output } => {
            println!("Generating C code: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);

                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("Syntax errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Semantic analysis
                    let mut ctx = SemanticContext::new();
                    let semantic_diags = analyze(&program, &mut ctx);

                    if semantic_diags.has_errors() {
                        eprintln!("Semantic errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in semantic_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Generate IR
                    let ir_module = lower_program(&program);

                    // Generate C code
                    let backend = CBackend::new();
                    let c_code = backend.generate(&ir_module);

                    // Write output
                    if let Some(output_path) = output {
                        match fs::write(&output_path, &c_code) {
                            Ok(_) => {
                                println!("C code generated at: {}", output_path.display());
                                println!("\nTo compile:");
                                println!("  gcc -o program {}", output_path.display());
                                println!("  ./program");
                            }
                            Err(e) => {
                                eprintln!("Error writing file: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        // Output to stdout
                        println!("{}", c_code);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::EmitRust { input, output } => {
            println!("Generating Rust code: {}\n", input.display());

            match fs::read_to_string(&input) {
                Ok(source) => {
                    let mut cache = SourceCache::new();
                    let file_id = cache.add(input.display().to_string(), &source);

                    let mut lexer = Lexer::new(&source, file_id);
                    let tokens = lexer.tokenize();
                    let lex_diags = lexer.take_diagnostics();

                    if lex_diags.has_errors() {
                        eprintln!("Lexer errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in lex_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    let (program, parse_diags) = parse(tokens);

                    if parse_diags.has_errors() {
                        eprintln!("Syntax errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in parse_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Semantic analysis
                    let mut ctx = SemanticContext::new();
                    let semantic_diags = analyze(&program, &mut ctx);

                    if semantic_diags.has_errors() {
                        eprintln!("Semantic errors:\n");
                        let renderer = DiagnosticRenderer::new(&cache);
                        for diag in semantic_diags.iter() {
                            eprintln!("{}", renderer.render(diag));
                        }
                        std::process::exit(1);
                    }

                    // Generate IR
                    let ir_module = lower_program(&program);

                    // Generate Rust code
                    let backend = RustBackend::new();
                    let rust_code = backend.generate(&ir_module);

                    // Write output
                    if let Some(output_path) = output {
                        match fs::write(&output_path, &rust_code) {
                            Ok(_) => {
                                println!("Rust code generated at: {}", output_path.display());
                            }
                            Err(e) => {
                                eprintln!("Error writing file: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        // Output to stdout
                        println!("{}", rust_code);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Run { input } => {
            println!("Running: {}", input.display());
            println!("   [TODO] Execution not yet implemented");
        }
    }
}

/// Compiles using the Rust backend
fn build_with_rust_backend(ir_module: &mendes_ir::Module, output_name: &str, release: bool) {
    use std::io::Write;

    // Generate Rust code
    let backend = RustBackend::new();
    let rust_code = backend.generate(ir_module);
    println!("  [ok] Codegen: Rust code generated");

    // Create temporary directory for the Cargo project
    let temp_dir = std::env::temp_dir().join(format!("mendes_build_{}", std::process::id()));
    let src_dir = temp_dir.join("src");

    if let Err(e) = fs::create_dir_all(&src_dir) {
        eprintln!("Error creating temporary directory: {}", e);
        std::process::exit(1);
    }

    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
mendes-runtime = {{ path = "{}" }}
tokio = {{ version = "1.35", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
        output_name,
        // Use the absolute path of mendes-runtime
        std::env::current_dir()
            .unwrap_or_default()
            .join("crates")
            .join("mendes-runtime")
            .to_string_lossy()
            .replace('\\', "/")
    );

    let cargo_path = temp_dir.join("Cargo.toml");
    if let Err(e) = fs::write(&cargo_path, cargo_toml) {
        eprintln!("Error creating Cargo.toml: {}", e);
        std::process::exit(1);
    }

    // Write the source code
    let main_path = src_dir.join("main.rs");
    if let Err(e) = fs::write(&main_path, &rust_code) {
        eprintln!("Error writing main.rs: {}", e);
        std::process::exit(1);
    }

    println!("  [ok] Cargo project created at: {}", temp_dir.display());

    // Compile with cargo
    println!("\nCompiling with Cargo...\n");

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("build");

    if release {
        cargo_cmd.arg("--release");
    }

    cargo_cmd.current_dir(&temp_dir);

    match cargo_cmd.output() {
        Ok(output) => {
            // Show stderr (cargo warnings and errors)
            if !output.stderr.is_empty() {
                std::io::stderr().write_all(&output.stderr).ok();
            }

            if output.status.success() {
                // Copy the binary to the current directory
                let profile = if release { "release" } else { "debug" };
                let binary_name = if cfg!(windows) {
                    format!("{}.exe", output_name)
                } else {
                    output_name.to_string()
                };

                let binary_src = temp_dir.join("target").join(profile).join(&binary_name);
                let binary_dst = std::env::current_dir()
                    .unwrap_or_default()
                    .join(&binary_name);

                if let Err(e) = fs::copy(&binary_src, &binary_dst) {
                    eprintln!("Error copying binary: {}", e);
                    eprintln!("   Binary available at: {}", binary_src.display());
                } else {
                    println!("\nCompilation completed!");
                    println!("   Executable: {}", binary_dst.display());
                    println!("\n   To run:");
                    if cfg!(windows) {
                        println!("   .\\{}", binary_name);
                    } else {
                        println!("   ./{}", binary_name);
                    }
                }
            } else {
                eprintln!("\nCompilation failed");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error executing cargo: {}", e);
            eprintln!("   Make sure Rust is installed (rustup.rs)");
            std::process::exit(1);
        }
    }

    // Clean up temporary directory (optional, commented for debug)
    // fs::remove_dir_all(&temp_dir).ok();
}

/// Compiles using the C backend
fn build_with_c_backend(ir_module: &mendes_ir::Module, output_name: &str) {
    // Generate C code
    let backend = CBackend::new();
    let c_code = backend.generate(ir_module);
    println!("  [ok] Codegen: C code generated");

    // Create temporary .c file
    let temp_dir = std::env::temp_dir();
    let c_file = temp_dir.join(format!("{}.c", output_name));

    if let Err(e) = fs::write(&c_file, &c_code) {
        eprintln!("Error creating C file: {}", e);
        std::process::exit(1);
    }

    // Determine executable name
    let exe_name = if cfg!(windows) {
        format!("{}.exe", output_name)
    } else {
        output_name.to_string()
    };

    let exe_path = std::env::current_dir()
        .unwrap_or_default()
        .join(&exe_name);

    // Compile with gcc or clang
    println!("\nCompiling with gcc...\n");

    let compiler = if cfg!(windows) { "gcc" } else { "cc" };

    let output = Command::new(compiler)
        .arg("-o")
        .arg(&exe_path)
        .arg(&c_file)
        .arg("-lpthread")  // For thread support
        .output();

    match output {
        Ok(output) => {
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }

            if output.status.success() {
                println!("Compilation completed!");
                println!("   Executable: {}", exe_path.display());
                println!("\n   To run:");
                if cfg!(windows) {
                    println!("   .\\{}", exe_name);
                } else {
                    println!("   ./{}", exe_name);
                }
            } else {
                eprintln!("C compilation failed");
                eprintln!("   C code available at: {}", c_file.display());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error executing {}: {}", compiler, e);
            eprintln!("   C code available at: {}", c_file.display());
            eprintln!("   Compile manually:");
            eprintln!("   {} -o {} {}", compiler, exe_name, c_file.display());
            std::process::exit(1);
        }
    }
}

/// Formats a statement for display
fn format_stmt(stmt: &mendes_parser::Stmt, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match stmt {
        mendes_parser::Stmt::Import { path, alias, .. } => {
            let alias_str = alias.as_ref().map(|a| format!(" as {}", a)).unwrap_or_default();
            format!("{}Import \"{}\"{}", pad, path, alias_str)
        }
        mendes_parser::Stmt::FromImport { module, items, .. } => {
            let items_str = match items {
                mendes_parser::ImportItems::All => "*".to_string(),
                mendes_parser::ImportItems::Names(names) => {
                    names.iter()
                        .map(|item| {
                            if let Some(alias) = &item.alias {
                                format!("{} as {}", item.name, alias)
                            } else {
                                item.name.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            format!("{}From {} import {}", pad, module, items_str)
        }
        mendes_parser::Stmt::Let { name, ty, mutable, .. } => {
            let mut_str = if *mutable { "mut " } else { "" };
            let ty_str = ty.as_ref().map(|t| format!(": {}", format_type(t))).unwrap_or_default();
            format!("{}Let {}{}{}", pad, mut_str, name, ty_str)
        }
        mendes_parser::Stmt::Fn(f) => {
            let async_str = if f.is_async { " async" } else { "" };
            let ret_str = f.return_type.as_ref().map(|t| format!(" -> {}", format_type(t))).unwrap_or_default();
            let params: Vec<_> = f.params.iter().map(|p| format!("{}: {}", p.name, format_type(&p.ty))).collect();
            format!("{}Fn {}({}){}{}\n{}  body: {} statements",
                pad, f.name, params.join(", "), ret_str, async_str, pad, f.body.len())
        }
        mendes_parser::Stmt::Struct(s) => {
            let copy_str = if s.is_copy { " copy" } else { "" };
            let fields: Vec<_> = s.fields.iter().map(|f| format!("{}: {}", f.name, format_type(&f.ty))).collect();
            format!("{}Struct {}{} {{ {} }}", pad, s.name, copy_str, fields.join(", "))
        }
        mendes_parser::Stmt::Enum(e) => {
            let variants: Vec<_> = e.variants.iter().map(|v| v.name.clone()).collect();
            format!("{}Enum {} {{ {} }}", pad, e.name, variants.join(", "))
        }
        mendes_parser::Stmt::Api(api) => {
            let method = match api.method {
                mendes_parser::HttpMethod::Get => "GET",
                mendes_parser::HttpMethod::Post => "POST",
                mendes_parser::HttpMethod::Put => "PUT",
                mendes_parser::HttpMethod::Delete => "DELETE",
                mendes_parser::HttpMethod::Patch => "PATCH",
            };
            let async_str = if api.is_async { " async" } else { "" };
            let ret_str = api.return_type.as_ref().map(|t| format!(" -> {}", format_type(t))).unwrap_or_default();
            format!("{}Api {} {}{}{}\n{}  middlewares: {:?}\n{}  handler: {} statements",
                pad, method, api.path, ret_str, async_str, pad, api.middlewares, pad, api.handler.len())
        }
        mendes_parser::Stmt::WebSocket(ws) => {
            let handlers = vec![
                ws.on_connect.as_ref().map(|_| "on_connect"),
                ws.on_message.as_ref().map(|_| "on_message"),
                ws.on_disconnect.as_ref().map(|_| "on_disconnect"),
            ].into_iter().flatten().collect::<Vec<_>>().join(", ");
            format!("{}WebSocket {}\n{}  middlewares: {:?}\n{}  handlers: [{}]",
                pad, ws.path, pad, ws.middlewares, pad, handlers)
        }
        mendes_parser::Stmt::Server(s) => {
            format!("{}Server {{ host: \"{}\", port: {} }}", pad, s.host, s.port)
        }
        mendes_parser::Stmt::Middleware(m) => {
            format!("{}Middleware {}\n{}  body: {} statements", pad, m.name, pad, m.body.len())
        }
        mendes_parser::Stmt::Db(db) => {
            let db_type = match db.db_type {
                mendes_parser::DbType::Postgres => "postgres",
                mendes_parser::DbType::Mysql => "mysql",
                mendes_parser::DbType::Sqlite => "sqlite",
            };
            format!("{}Db {} {} {{ pool: {} }}", pad, db_type, db.name, db.pool_size)
        }
        mendes_parser::Stmt::If { then_block, else_block, .. } => {
            let else_str = else_block.as_ref().map(|b| format!(", else: {} stmts", b.len())).unwrap_or_default();
            format!("{}If (then: {} stmts{})", pad, then_block.len(), else_str)
        }
        mendes_parser::Stmt::For { var, body, .. } => {
            format!("{}For {} in ... ({} stmts)", pad, var, body.len())
        }
        mendes_parser::Stmt::While { body, .. } => {
            format!("{}While ... ({} stmts)", pad, body.len())
        }
        mendes_parser::Stmt::Return { value, .. } => {
            let val_str = if value.is_some() { " <expr>" } else { "" };
            format!("{}Return{}", pad, val_str)
        }
        mendes_parser::Stmt::Expr(_) => {
            format!("{}Expr <...>", pad)
        }
        mendes_parser::Stmt::Trait(t) => {
            let methods: Vec<_> = t.methods.iter().map(|m| m.name.clone()).collect();
            format!("{}Trait {} {{ methods: {} }}", pad, t.name, methods.join(", "))
        }
        mendes_parser::Stmt::ImplTrait(i) => {
            format!("{}Impl {} for {}", pad, i.trait_name, i.type_name)
        }
        mendes_parser::Stmt::TypeAlias { name, ty, .. } => {
            format!("{}Type {} = {}", pad, name, format_type(ty))
        }
        mendes_parser::Stmt::Break { .. } => {
            format!("{}Break", pad)
        }
        mendes_parser::Stmt::Continue { .. } => {
            format!("{}Continue", pad)
        }
    }
}

/// Formats a type for display
fn format_type(ty: &mendes_parser::Type) -> String {
    match ty {
        mendes_parser::Type::Int => "int".to_string(),
        mendes_parser::Type::Float => "float".to_string(),
        mendes_parser::Type::Bool => "bool".to_string(),
        mendes_parser::Type::String => "string".to_string(),
        mendes_parser::Type::Named(name) => name.clone(),
        mendes_parser::Type::Generic { name, args } => {
            let args_str: Vec<_> = args.iter().map(format_type).collect();
            format!("{}<{}>", name, args_str.join(", "))
        }
        mendes_parser::Type::Ref(inner) => format!("&{}", format_type(inner)),
        mendes_parser::Type::MutRef(inner) => format!("&mut {}", format_type(inner)),
        mendes_parser::Type::Array(inner) => format!("[{}]", format_type(inner)),
        mendes_parser::Type::Tuple(types) => {
            let types_str: Vec<_> = types.iter().map(format_type).collect();
            format!("({})", types_str.join(", "))
        }
        mendes_parser::Type::Function { params, return_type } => {
            let params_str: Vec<_> = params.iter().map(format_type).collect();
            format!("fn({}) -> {}", params_str.join(", "), format_type(return_type))
        }
    }
}
