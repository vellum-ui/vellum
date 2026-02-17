// Custom Module Loader for AppJS
//
// Supports:
// - Local files (.js, .ts, .jsx, .tsx, .mjs, .mts, .json)
// - https:// URLs (remote ES modules)
// - jsr: specifiers (resolved via https://jsr.io)
// - npm: specifiers (resolved via https://esm.sh)
//
// TypeScript/JSX/TSX files are transpiled to JavaScript using deno_ast.
// Source maps are stored for better error reporting.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;

use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceMapOption;
use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use deno_core::error::ModuleLoaderError;
use deno_core::resolve_import;
use deno_error::JsErrorBox;

type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

pub struct AppJsModuleLoader {
    source_maps: SourceMapStore,
    http_client: reqwest::Client,
}

impl AppJsModuleLoader {
    pub fn new() -> Self {
        Self {
            source_maps: Rc::new(RefCell::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }
}

/// Resolve a jsr: specifier to an https URL via the JSR registry.
/// Format: jsr:@scope/package[@version][/path]
fn resolve_jsr_specifier(specifier: &str) -> Result<ModuleSpecifier, JsErrorBox> {
    let rest = specifier
        .strip_prefix("jsr:")
        .ok_or_else(|| JsErrorBox::generic("Not a jsr: specifier"))?;

    // Parse @scope/package[@version][/path]
    let rest = rest.trim_start_matches('/');

    if !rest.starts_with('@') {
        return Err(JsErrorBox::generic(
            "jsr: specifier must start with @scope/package",
        ));
    }

    // Split into package and path parts
    // @scope/package@version/path or @scope/package/path
    let parts: Vec<&str> = rest.splitn(3, '/').collect();
    if parts.len() < 2 {
        return Err(JsErrorBox::generic("jsr: specifier must be @scope/package"));
    }

    let scope = parts[0]; // @scope
    let (package, version, path) = if parts.len() == 2 {
        // @scope/package or @scope/package@version
        parse_package_version(parts[1], "")?
    } else {
        // @scope/package@version/path or @scope/package/path/more
        let remaining = parts[2];
        parse_package_version(parts[1], remaining)?
    };

    // Construct the jsr.io URL
    // https://jsr.io/@scope/package[@version][/path]
    let url_str = if version.is_empty() {
        if path.is_empty() {
            format!("https://jsr.io/{scope}/{package}/mod.ts")
        } else {
            format!("https://jsr.io/{scope}/{package}/{path}")
        }
    } else if path.is_empty() {
        format!("https://jsr.io/{scope}/{package}/{version}/mod.ts")
    } else {
        format!("https://jsr.io/{scope}/{package}/{version}/{path}")
    };

    ModuleSpecifier::parse(&url_str)
        .map_err(|e| JsErrorBox::generic(format!("Failed to parse jsr URL '{}': {}", url_str, e)))
}

/// Parse "package@version" into (package, version, path)
fn parse_package_version<'a>(
    pkg_str: &'a str,
    remaining_path: &'a str,
) -> Result<(&'a str, &'a str, &'a str), JsErrorBox> {
    if let Some(at_pos) = pkg_str.find('@') {
        let package = &pkg_str[..at_pos];
        let version = &pkg_str[at_pos + 1..];
        Ok((package, version, remaining_path))
    } else {
        Ok((pkg_str, "", remaining_path))
    }
}

/// Resolve an npm: specifier via esm.sh CDN.
/// Format: npm:package[@version][/path]
fn resolve_npm_specifier(specifier: &str) -> Result<ModuleSpecifier, JsErrorBox> {
    let rest = specifier
        .strip_prefix("npm:")
        .ok_or_else(|| JsErrorBox::generic("Not an npm: specifier"))?;

    // esm.sh serves npm packages as ES modules
    let url_str = format!("https://esm.sh/{rest}");
    ModuleSpecifier::parse(&url_str)
        .map_err(|e| JsErrorBox::generic(format!("Failed to parse npm URL '{}': {}", url_str, e)))
}

/// Determine MediaType from URL, using both path extension and Content-Type header.
fn media_type_from_specifier(specifier: &ModuleSpecifier) -> MediaType {
    let path = specifier.path();
    if let Some(ext) = path.rsplit('.').next() {
        match ext {
            "ts" | "mts" | "cts" => MediaType::TypeScript,
            "tsx" => MediaType::Tsx,
            "js" | "mjs" | "cjs" => MediaType::JavaScript,
            "jsx" => MediaType::Jsx,
            "json" => MediaType::Json,
            _ => {
                // For URLs without clear extension, default to TypeScript
                // (jsr/esm.sh often serve TS or modules without extension)
                if specifier.scheme() == "https" || specifier.scheme() == "http" {
                    MediaType::TypeScript
                } else {
                    MediaType::Unknown
                }
            }
        }
    } else if specifier.scheme() == "https" || specifier.scheme() == "http" {
        MediaType::TypeScript
    } else {
        MediaType::Unknown
    }
}

/// Determine MediaType from Content-Type header value.
fn media_type_from_content_type(content_type: &str, specifier: &ModuleSpecifier) -> MediaType {
    let ct = content_type.split(';').next().unwrap_or("").trim();
    match ct {
        "application/typescript" | "text/typescript" => MediaType::TypeScript,
        "application/javascript" | "text/javascript" | "application/x-javascript" => {
            MediaType::JavaScript
        }
        "application/json" | "text/json" => MediaType::Json,
        "text/tsx" => MediaType::Tsx,
        "text/jsx" => MediaType::Jsx,
        _ => media_type_from_specifier(specifier),
    }
}

/// Transpile TypeScript/JSX/TSX code to JavaScript using deno_ast.
fn transpile(
    specifier: &ModuleSpecifier,
    code: String,
    media_type: MediaType,
    source_maps: &SourceMapStore,
) -> Result<String, JsErrorBox> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: specifier.clone(),
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(JsErrorBox::from_err)?;

    let res = parsed
        .transpile(
            &deno_ast::TranspileOptions {
                imports_not_used_as_values: deno_ast::ImportsNotUsedAsValues::Remove,
                decorators: deno_ast::DecoratorsTranspileOption::Ecma,
                ..Default::default()
            },
            &deno_ast::TranspileModuleOptions { module_kind: None },
            &deno_ast::EmitOptions {
                source_map: SourceMapOption::Separate,
                inline_sources: true,
                ..Default::default()
            },
        )
        .map_err(JsErrorBox::from_err)?;

    let res = res.into_source();
    if let Some(source_map) = res.source_map {
        source_maps
            .borrow_mut()
            .insert(specifier.to_string(), source_map.into_bytes());
    }
    Ok(res.text)
}

/// Load a local file module (synchronous).
fn load_local(
    specifier: &ModuleSpecifier,
    source_maps: &SourceMapStore,
) -> Result<ModuleSource, ModuleLoaderError> {
    let path = specifier
        .to_file_path()
        .map_err(|_| JsErrorBox::generic("Failed to convert specifier to file path"))?;

    let media_type = MediaType::from_path(&path);
    let (module_type, should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
        MediaType::Jsx => (ModuleType::JavaScript, true),
        MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, true),
        MediaType::Json => (ModuleType::Json, false),
        _ => {
            return Err(JsErrorBox::generic(format!(
                "Unknown extension {:?}",
                path.extension()
            )));
        }
    };

    let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
    let code = if should_transpile {
        transpile(specifier, code, media_type, source_maps)?
    } else {
        code
    };

    Ok(ModuleSource::new(
        module_type,
        ModuleSourceCode::String(code.into()),
        specifier,
        None,
    ))
}

impl ModuleLoader for AppJsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        // Handle npm: specifiers
        if specifier.starts_with("npm:") {
            return resolve_npm_specifier(specifier);
        }

        // Handle jsr: specifiers
        if specifier.starts_with("jsr:") {
            return resolve_jsr_specifier(specifier);
        }

        // Handle https: and http: specifiers directly
        if specifier.starts_with("https://") || specifier.starts_with("http://") {
            return ModuleSpecifier::parse(specifier).map_err(|e| {
                ModuleLoaderError::from(JsErrorBox::generic(format!(
                    "Invalid URL '{}': {}",
                    specifier, e
                )))
            });
        }

        // For relative imports from an https module, resolve against the referrer
        if referrer.starts_with("https://") || referrer.starts_with("http://") {
            return resolve_import(specifier, referrer).map_err(JsErrorBox::from_err);
        }

        // Default: resolve as relative file path import
        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let scheme = module_specifier.scheme();

        match scheme {
            "file" => {
                // Synchronous local file load
                let source_maps = self.source_maps.clone();
                ModuleLoadResponse::Sync(load_local(module_specifier, &source_maps))
            }
            "https" | "http" => {
                // Async remote module fetch
                let specifier = module_specifier.clone();
                let client = self.http_client.clone();
                let source_maps = self.source_maps.clone();

                let fut = async move {
                    let response = client
                        .get(specifier.as_str())
                        .header("Accept", "application/typescript,application/javascript,text/typescript,text/javascript,*/*")
                        .send()
                        .await
                        .map_err(|e| {
                            JsErrorBox::generic(format!(
                                "Failed to fetch '{}': {}",
                                specifier, e
                            ))
                        })?;

                    // Follow redirects — reqwest does this by default, but capture the final URL
                    let final_url = response.url().clone();
                    let content_type = response
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();

                    if !response.status().is_success() {
                        return Err(JsErrorBox::generic(format!(
                            "HTTP {} fetching '{}'",
                            response.status(),
                            specifier
                        )));
                    }

                    let code = response.text().await.map_err(|e| {
                        JsErrorBox::generic(format!(
                            "Failed to read response from '{}': {}",
                            specifier, e
                        ))
                    })?;

                    // Determine media type from Content-Type header or URL extension
                    let final_specifier =
                        ModuleSpecifier::parse(final_url.as_str()).unwrap_or(specifier.clone());
                    let media_type = media_type_from_content_type(&content_type, &final_specifier);

                    let (module_type, should_transpile) = match media_type {
                        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                            (ModuleType::JavaScript, false)
                        }
                        MediaType::Jsx => (ModuleType::JavaScript, true),
                        MediaType::TypeScript
                        | MediaType::Mts
                        | MediaType::Cts
                        | MediaType::Dts
                        | MediaType::Dmts
                        | MediaType::Dcts
                        | MediaType::Tsx => (ModuleType::JavaScript, true),
                        MediaType::Json => (ModuleType::Json, false),
                        _ => {
                            // Default to JS for remote modules with unknown type
                            (ModuleType::JavaScript, false)
                        }
                    };

                    let code = if should_transpile {
                        transpile(&final_specifier, code, media_type, &source_maps)?
                    } else {
                        code
                    };

                    if final_url.as_str() != specifier.as_str() {
                        Ok(ModuleSource::new_with_redirect(
                            module_type,
                            ModuleSourceCode::String(code.into()),
                            &specifier,
                            &final_specifier,
                            None,
                        ))
                    } else {
                        Ok(ModuleSource::new(
                            module_type,
                            ModuleSourceCode::String(code.into()),
                            &specifier,
                            None,
                        ))
                    }
                };

                ModuleLoadResponse::Async(Pin::from(Box::new(fut)))
            }
            _ => ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
                "Unsupported module scheme: '{}' in '{}'",
                scheme, module_specifier
            )))),
        }
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(specifier)
            .map(|v| v.clone().into())
    }
}
